use async_std::task;
use chrono::Datelike;
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use std::fs::{self, File};
use std::io::Write;
mod config;

fn get_all_tasks(path: &str, pattern: &Option<String>) -> Vec<String> {
    let mut reg = r"\s* \[ \] (.*)";
    if let Some(regex) = pattern {
        reg = &regex;
    }
    let pattern_matcher = Regex::new(reg).unwrap();
    let mut task_list = Vec::new();
    println!("path:  {path}");
    for line in fs::read_to_string(path).unwrap().lines() {
        if let Some(task) = pattern_matcher.captures(line) {
            task_list.push(task[1].to_string());
        };
    }
    task_list
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
    println!("path: {path}");
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
    println!("write_path: {file}");
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
    let tasks = get_all_tasks(&watch_path, matcher);
    let _ = write_all_tasks_to_file(list_path, tasks);
    let first_list_task = get_first_task_from_list(list_path);
    let _ = write_task_to_file(write_path, first_list_task);
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(watch_path.as_ref(), RecursiveMode::NonRecursive)?;

    while let Some(res) = rx.next().await {
        println!("next...");
        if is_new_day(day) {
            println!("new day");
            break;
        }
        match res {
            Ok(event) => {
                println!("Change: {event:?}");
                let tasks = get_all_tasks(&watch_path, matcher);
                println!("task: {:?}", tasks);
                let _ = write_all_tasks_to_file(list_path, tasks);
                let first_task = get_first_task_from_list(list_path);
                println!("first_task: {}", first_task);
                let file_result = write_task_to_file(write_path, first_task);
                match file_result {
                    Err(e) => eprintln!("{}", e),
                    Ok(_) => continue,
                }
                // println!("file result: {:#?}", file_result);
            }
            Err(error) => println!("Error: {error:?}"),
        }
    }

    Ok(())
}

// TODO: This is very specific to my own preferences, this should be made to handle general cases
#[async_std::main]
async fn main() -> notify::Result<()> {
    let conf = config::read_config_from_file()
        .inspect_err(|e| eprintln!("Found an error when parsing config: {}", e))
        .unwrap();
    let mut date_time = chrono::offset::Local::now();
    let format_opts = conf.daily_note.format_style.unwrap();
    task::block_on(async {
        loop {
            let mut note_path = String::new();
            for format_opt in format_opts.iter() {
                if format_opt.starts_with('$') {
                    let path = &format_opt[1..];
                    note_path.push_str(path);
                } else {
                    let date_time_format = date_time.format(format_opt);
                    note_path.push_str(&date_time_format.to_string());
                }
                note_path.push('\\');
            }
            note_path.pop().unwrap();
            note_path = note_path.replace('/', "\\");

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
