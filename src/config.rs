use serde::{Deserialize, Serialize};
use std::{env::var_os, fs, path::Path};

#[derive(Deserialize, Serialize, Debug)]
pub enum FileOptions {
    ShowUpdates(Option<String>),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DailyNote {
    pub regex: Option<String>,
    pub dir_format: Option<String>,
    pub day_format: Option<String>,
    pub file_extension: String,
    pub base_dir: String,
    pub task_list_path: String,
}

// TODO: Implement Pomodoro timer
pub struct Pomodoro {}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub state_path: String,
    pub daily_note: DailyNote,
}

pub fn read_config_from_file() -> anyhow::Result<Config> {
    let key = "USERPROFILE";

    if let Some(user_profile) = var_os(key) {
        let home_dir = user_profile.into_string().unwrap();
        let path = Path::new(&home_dir).join(".glaze-wm\\components\\config.yaml");

        // TODO: handle case where config file does not exist
        let config_str = fs::read_to_string(path)?;
        let mut config: Config = serde_yaml::from_str(&config_str)?;
        let last_base_dir_char = config
            .daily_note
            .base_dir
            .chars()
            .last()
            .expect("cannot be empty string");
        if last_base_dir_char != '/' || last_base_dir_char != '\\' {
            config.daily_note.base_dir.push('\\')
        }
        return Ok(config);
    }

    unimplemented!("create error when USERPROFILE var is not found");
}
