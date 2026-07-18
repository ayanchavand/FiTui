use serde::{Deserialize, Serialize};
use directories::ProjectDirs;
use std::{fs, path::PathBuf};

use crate::theme::ThemeConfig;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub tags: Vec<String>,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default = "default_theme_name")]
    pub theme: String,
    #[serde(default)]
    pub custom_themes: HashMap<String, ThemeConfig>,
}

fn default_currency() -> String {
    "$".to_string()
}

fn default_theme_name() -> String {
    "default".to_string()
}

impl Default for Config {
    fn default() -> Self {
        let mut custom_themes = HashMap::new();
        // Add a sample custom theme so users see how it's formatted
        custom_themes.insert(
            "sunset".to_string(),
            ThemeConfig {
                accent: "#ffb86c".to_string(),      // orange
                accent_soft: "#ff79c6".to_string(), // pink
                credit: "#50fa7b".to_string(),      // green
                debit: "#ff5555".to_string(),       // red
                muted: "#6272a4".to_string(),       // gray
                subtle: "#44475a".to_string(),      // dark gray
                background: "#21222c".to_string(),  // dark purple bg
                surface: "#282a36".to_string(),     // purple surface
                row_alt: "#242530".to_string(),     // row alt
                foreground: "#f8f8f2".to_string(),  // white/cream
            },
        );

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
            theme: default_theme_name(),
            custom_themes,
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
