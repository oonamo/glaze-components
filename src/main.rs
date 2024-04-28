use async_std::task;
use chrono::{Datelike, Month};
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use std::fs::{self, File};
use std::io::Write;
mod config;

// TODO:
// User defined enums/abbrvs,
// or even better,
// allow user to specify the date format
pub fn month_to_string(month: Month) -> String {
    match month {
        Month::January => "Jan".to_string(),
        Month::February => "Feb".to_string(),
        Month::March => "Mar".to_string(),
        Month::April => "Apr".to_string(),
        Month::May => "May".to_string(),
        Month::June => "Jun".to_string(),
        Month::July => "Jul".to_string(),
        Month::August => "Aug".to_string(),
        Month::September => "Sep".to_string(),
        Month::October => "Oct".to_string(),
        Month::November => "Nov".to_string(),
        Month::December => "Dec".to_string(),
    }
}

pub fn read_from_file(path: String) -> anyhow::Result<()> {
    for line in fs::read_to_string(path)?.lines() {
        // println!("{}", line);
    }
    // println!("reached ok");
    Ok(())
}

pub fn get_all_tasks(path: &str, pattern: &Option<String>) -> Vec<String> {
    let mut reg = r"\s* \[ \] (.*)";
    if let Some(regex) = pattern {
        reg = &regex;
    }
    let pattern_matcher = Regex::new(reg).unwrap();
    let mut task_list = Vec::new();
    for line in fs::read_to_string(path).unwrap().lines() {
        if let Some(task) = pattern_matcher.captures(line) {
            task_list.push(task[1].to_string());
            // println!("found: {:?}", task);
        };
    }
    task_list
}

pub fn write_all_tasks_to_file(path: &str, contents: Vec<String>) -> std::io::Result<()> {
    let mut f = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    // let result = writeln!(&mut f, "\n{}", contents);
    for task in contents {
        let result = writeln!(&mut f, "{}", task);
        if result.is_err() {
            todo!("handle io error")
        }
    }
    Ok(())
}

pub fn get_first_task_from_list(path: &str) -> String {
    let tasks = std::fs::read_to_string(path).unwrap();
    let mut lines = tasks.lines();
    match lines.next() {
        Some(task) => task.to_string(),
        _ => String::from("done!"),
    }
}

pub fn is_new_day(cur_day: u32) -> bool {
    cur_day != chrono::offset::Local::now().day()
}

pub fn write_task_to_file(file: &str, contents: String) -> std::io::Result<()> {
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
                println!("file result: {:#?}", file_result);
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
    let date_time = chrono::offset::Local::now();
    let mut dir_string = date_time.format("%b %G");
    let mut file_string = date_time.format("%G-%m-%d");
    task::block_on(async {
        loop {
            let mut note_path = String::from(&conf.daily_note.base_dir);

            if conf.daily_note.dir_format.is_some() {
                note_path.push_str(&dir_string.to_string());
                note_path.push('\\');
            }
            if conf.daily_note.day_format.is_some() {
                note_path.push_str(&file_string.to_string());
            }
            note_path.push_str(&conf.daily_note.file_extension);

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
            let new_date = chrono::offset::Local::now();
            if let Some(c) = &conf.daily_note.dir_format {
                dir_string = new_date.format(c);
            }
            if let Some(c) = &conf.daily_note.day_format {
                file_string = new_date.format(c);
            }
        }
        Ok(())
    })
}
