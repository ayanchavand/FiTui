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
        
        custom_themes.insert(
            "dracula".to_string(),
            ThemeConfig {
                accent: "#bd93f9".to_string(),      // purple
                accent_soft: "#6272a4".to_string(), // comment/gray-blue
                credit: "#50fa7b".to_string(),      // green
                debit: "#ff5555".to_string(),       // red
                muted: "#6272a4".to_string(),       // comment
                subtle: "#44475a".to_string(),      // selection/darker gray
                background: "#282a36".to_string(),  // bg
                surface: "#343746".to_string(),     // current line/surface
                row_alt: "#2d2f3b".to_string(),     // midpoint
                foreground: "#f8f8f2".to_string(),  // fg
            },
        );

        custom_themes.insert(
            "nord".to_string(),
            ThemeConfig {
                accent: "#88c0d0".to_string(),      // frost blue
                accent_soft: "#81a1c1".to_string(), // medium frost blue
                credit: "#a3be8c".to_string(),      // green
                debit: "#bf616a".to_string(),       // red
                muted: "#4c566a".to_string(),       // polar night (nord3)
                subtle: "#3b4252".to_string(),      // polar night (nord1)
                background: "#2e3440".to_string(),  // polar night (nord0)
                surface: "#434c5e".to_string(),     // polar night (nord2)
                row_alt: "#353c4a".to_string(),     // midpoint
                foreground: "#d8dee9".to_string(),  // snow storm (nord4)
            },
        );

        custom_themes.insert(
            "gruvbox".to_string(),
            ThemeConfig {
                accent: "#fabd2f".to_string(),      // yellow
                accent_soft: "#d79921".to_string(), // darker yellow
                credit: "#b8bb26".to_string(),      // green
                debit: "#fb4934".to_string(),       // red
                muted: "#928374".to_string(),       // gray
                subtle: "#504945".to_string(),      // dark gray
                background: "#282828".to_string(),  // bg0
                surface: "#3c3836".to_string(),     // bg1
                row_alt: "#32302f".to_string(),     // midpoint
                foreground: "#ebdbb2".to_string(),  // fg0
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
    let mut config: Config = serde_yaml::from_str(&text).expect("Invalid YAML format");

    // Auto-migrate older configs that don't have theme options visible
    if !text.contains("theme:") {
        let default = Config::default();
        config.theme = default.theme;
        config.custom_themes = default.custom_themes;

        let yaml =
            serde_yaml::to_string(&config).expect("Failed to serialize migrated config");
        let _ = fs::write(&path, yaml); // Ignore write error on read-only environments
    }

    config
}
