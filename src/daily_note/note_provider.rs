use crate::config;
use crate::debug;
use crate::time;
use async_std::task;
use chrono::{DateTime, Datelike};
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use std::fs::{self, File};
use std::io::Write;

fn get_all_tasks(path: &str, pattern: &Option<String>) -> Vec<String> {
    let reg = r"\s* \[ \] (.*)";
    let re: Regex;
    if let Some(regex) = pattern {
        re = Regex::new(regex).unwrap_or(Regex::new(reg).unwrap());
    } else {
        re = Regex::new(reg).unwrap();
    }
    // TODO: Check if path exists before unwraping
    let mut task_list = Vec::new();
    for line in fs::read_to_string(path).unwrap().lines() {
        if let Some(task) = re.captures(line) {
            task_list.push(task[1].to_string());
        };
    }
    debug!(task_list)
}

fn write_all_tasks_to_file(path: &str, contents: Vec<String>) -> std::io::Result<()> {
    let mut f = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    for task in contents {
        let result = writeln!(&mut f, "{}", task);
        if result.is_err() {
            todo!("handle io error")
        }
    }
    Ok(())
}

fn get_first_task_from_list(path: &str) -> String {
    let tasks = std::fs::read_to_string(path).unwrap();
    let mut lines = tasks.lines();
    match lines.next() {
        Some(task) => task.to_string(),
        _ => String::from("done!"),
    }
}

fn is_new_day(cur_day: u32) -> bool {
    cur_day != chrono::offset::Local::now().day()
}

fn write_task_to_file(file: &str, contents: String) -> std::io::Result<()> {
    // Clear file before writing it to not have any multiline strings bother the
    // component
    let mut f = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(format!("{file}/daily_note.log"))?;
    let result = writeln!(&mut f, "\n{}", contents);
    if result.is_err() {
        todo!("handle io error")
    }
    Ok(())
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);
    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(
        move |res| {
            task::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

async fn watch(
    watch_path: String,
    write_path: &str,
    list_path: &str,
    matcher: &Option<String>,
    day: u32,
) -> notify::Result<()> {
    let tasks = time!(get_all_tasks, &watch_path, matcher);
    let _ = time!(write_all_tasks_to_file, list_path, tasks);
    let first_list_task = time!(get_first_task_from_list, list_path);
    let _ = time!(write_task_to_file, write_path, first_list_task);
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(watch_path.as_ref(), RecursiveMode::NonRecursive)?;

    while let Some(res) = rx.next().await {
        if is_new_day(day) {
            break;
        }
        match res {
            Ok(event) => {
                debug!(event);
                let tasks = get_all_tasks(&watch_path, matcher);
                let _ = write_all_tasks_to_file(list_path, tasks);
                let first_task = get_first_task_from_list(list_path);
                debug!(&first_task);
                let file_result = write_task_to_file(write_path, first_task);
                match file_result {
                    Err(e) => eprintln!("{}", e),
                    Ok(_) => continue,
                }
            }
            Err(error) => eprintln!("Error: {error:?}"),
        }
    }

    Ok(())
}

// HACK: Cursed Bounds
fn convert_format_opts_to_path<T: chrono::TimeZone>(
    date_time: &DateTime<T>,
    format_opts: &[String],
) -> String
where
    <T as chrono::TimeZone>::Offset: chrono::TimeZone + std::fmt::Display,
{
    let mut note_path = String::new();
    for format_opt in format_opts.iter() {
        if let Some(stripped) = format_opt.strip_prefix('$') {
            let path = stripped;
            note_path.push_str(path);
        } else {
            let date_time_format = date_time.format(format_opt);
            note_path.push_str(&date_time_format.to_string());
        }
        note_path.push('\\');
    }
    note_path.pop().unwrap();
    note_path = note_path.replace('/', "\\");
    note_path
}

pub fn start_daily_note_watch() -> notify::Result<()> {
    let conf = config::read_config_from_file()
        .inspect_err(|e| eprintln!("Found an error when parsing config: {}", e))
        .unwrap();

    let mut date_time = chrono::offset::Local::now();
    let format_opts = conf.daily_note.format_style.unwrap();

    task::block_on(async {
        loop {
            let note_path = convert_format_opts_to_path(&date_time, &format_opts);
            if let Err(e) = watch(
                note_path,
                &conf.state_path,
                &conf.daily_note.task_list_path,
                &conf.daily_note.regex,
                date_time.day(),
            )
            .await
            {
                eprintln!("err: {}", e);
                break;
            }
            date_time = chrono::offset::Local::now();
        }
        Ok(())
    })
}

#[allow(deprecated)]
#[cfg(test)]
mod tests {
    use super::{convert_format_opts_to_path, get_all_tasks};
    use chrono::prelude::*;
    use chrono::{NaiveDate, NaiveDateTime};

    #[test]
    fn get_task_from_file() {
        let path = std::env::current_dir()
            .unwrap()
            .join("test_assets/2024-05-01.md");
        let tasks = get_all_tasks(path.to_str().unwrap(), &None);
        assert_eq!(tasks[0], "abc".to_string());
    }
    #[test]
    fn user_format_strings_convert() {
        struct TestCase {
            pub format_opts: Vec<String>,
            pub asserttion: String,
        }
        impl TestCase {
            pub fn new(opts: Vec<&str>, assert_with: &str) -> Self {
                let mut vec = vec![];
                for opt in opts {
                    vec.push(opt.to_owned().to_string());
                }
                Self {
                    format_opts: vec,
                    asserttion: assert_with.to_string(),
                }
            }
        }
        let test_cases = [
            TestCase::new(
                vec![
                    "$C:/Users/your_name/your_notes_path/notes",
                    "%b %G",
                    "%G-%m-%d.md",
                ],
                "C:\\Users\\your_name\\your_notes_path\\notes\\May 2024\\2024-05-01.md",
            ),
            TestCase::new(
                vec!["$C:/Users/some/path", "%G", "$daily_notes/"],
                "C:\\Users\\some\\path\\2024\\daily_notes\\",
            ),
            TestCase::new(
                vec!["$C:\\Users\\some\\path", "%G", "$daily_notes\\"],
                "C:\\Users\\some\\path\\2024\\daily_notes\\",
            ),
        ];
        let naive = NaiveDate::from_ymd(2024, 5, 1);
        let t = NaiveTime::from_hms_milli(12, 34, 56, 789);
        let naive_date = NaiveDateTime::new(naive, t);
        let date_time = Utc.from_utc_datetime(&naive_date);
        // let date_time = Utc::from_utc_datetime(NaiveDateTime::from_ymd(2024, 5, 1));
        for test in test_cases {
            let path = convert_format_opts_to_path(&date_time, &test.format_opts);
            assert_eq!(test.asserttion, path);
        }
    }
    #[test]
    fn invalid_regex_fallback() {
        let path = std::env::current_dir()
            .unwrap()
            .join("test_assets/2024-05-01.md");
        let results = get_all_tasks(path.to_str().unwrap(), &Some(".*\\".to_string()));
        assert_eq!(results[0], "abc".to_string());
    }
}
