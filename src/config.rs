use serde::{Deserialize, Serialize};
use directories::ProjectDirs;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub tags: Vec<String>,
    #[serde(default = "default_currency")]
    pub currency: String,
}

fn default_currency() -> String {
    "$".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tags: vec![
                "food".into(),
                "travel".into(),
                "shopping".into(),
                "bills".into(),
                "salary".into(),
                "other".into(),
            ],
            currency: default_currency(),
        }
    }
}

fn config_path() -> PathBuf {
    let proj_dirs =
        ProjectDirs::from("com", "ayan", "fitui").expect("Could not find config directory");

    let config_dir = proj_dirs.config_dir();
    fs::create_dir_all(config_dir).expect("Failed to create config directory");

    config_dir.join("config.yaml")
}

pub fn load_config() -> Config {
    let path = config_path();

    // Auto-create default config if missing
    if !path.exists() {
        let default = Config::default();

        let yaml =
            serde_yaml::to_string(&default).expect("Failed to serialize default config");

        fs::write(&path, yaml).expect("Failed to write default config.yaml");

        println!("Created default config at: {:?}", path);

        return default;
    }

    let text = fs::read_to_string(&path).expect("Failed to read config.yaml");
    serde_yaml::from_str(&text).expect("Invalid YAML format")
}
