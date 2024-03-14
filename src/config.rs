use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

pub struct ConfigDir(PathBuf);

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub user: Option<String>,
}

impl From<&ConfigDir> for Config {
    fn from(value: &ConfigDir) -> Self {
        let file = value.read_file(PathBuf::from("config.json"));
        match file {
            Some(file) => {
                let config: Config = serde_json::from_str(&file).unwrap();
                config
            }
            None => Config { user: None },
        }
    }
}

impl Config {
    pub fn save(&self, config_dir: &ConfigDir) {
        let content = serde_json::to_string(&self).unwrap();
        config_dir.write_file(PathBuf::from("config.json"), content);
    }
}

impl ConfigDir {
    pub fn new() -> ConfigDir {
        ConfigDir(home::home_dir().unwrap().join(".plugin-cli"))
    }

    fn read_file(&self, path: PathBuf) -> Option<String> {
        let path = self.0.join(path);
        match fs::read_to_string(path) {
            Ok(content) => Some(content),
            Err(_) => None,
        }
    }

    fn write_file(&self, path: PathBuf, content: String) {
        let path = self.0.join(path);
        let parent_dir = path.parent().unwrap();
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir).unwrap();
        }
        fs::write(path, content).unwrap();
    }
}

impl Default for ConfigDir {
    fn default() -> Self {
        Self::new()
    }
}
