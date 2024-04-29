use serde::{Deserialize, Serialize};
use std::{
    env::var_os,
    fs,
    path::{Path, PathBuf},
};

#[derive(Deserialize, Serialize, Debug)]
pub enum FormatStyles {
    glaze,
}

// TODO: change format options to vector
#[derive(Deserialize, Serialize, Debug)]
pub struct DailyNote {
    pub regex: Option<String>,
    pub task_list_path: String,
    pub format_style: Option<Vec<String>>,
}

// TODO: Implement Pomodoro timer
// pub struct Pomodoro {}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub state_path: String,
    pub daily_note: DailyNote,
}

impl DailyNote {
    fn new(home_dir: String) -> Self {
        Self {
            regex: None,
            task_list_path: format!("{home_dir}\\.glaze-wm\\components\\task_list.log"),
            format_style: None,
        }
    }
}

impl Config {
    fn new(config_path: PathBuf, home_dir: String) -> Self {
        let config = Self {
            state_path: config_path.to_str().unwrap().to_string(),
            daily_note: DailyNote::new(home_dir),
        };
        let config_string: String = serde_yaml::to_string(&config).unwrap();
        let res = fs::write(config_path, config_string)
            .inspect_err(|e| eprintln!("error creating config: {}", e));
        if res.is_err() {
            // File system error... just quit
            std::process::exit(-1);
        }
        config
    }
}

pub fn read_config_from_file() -> anyhow::Result<Config> {
    let key = "USERPROFILE";

    if let Some(user_profile) = var_os(key) {
        let home_dir = user_profile.into_string().unwrap();
        let path = Path::new(&home_dir).join(".glaze-wm\\components\\config.yaml");

        if !path.exists() {
            println!("attempting to create new config at {:?}", path);
            Config::new(path, home_dir);
            println!("created config successfully");
            println!("The program will now quit, as it will fail without a valid config");
            println!("Please refer to https://github.com/oonamo/glaze-components?tab=readme-ov-file#usage if you have an issues");
            std::process::exit(0);
        }

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

    unimplemented!("unhandled exception to missing user_profile");
}
