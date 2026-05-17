use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationKey {
    Space,
    LeftRightArrow,
    Both,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_languages")]
    pub languages: Vec<String>,
    #[serde(default = "default_input_time_ms")]
    pub input_time_ms: u64,
    #[serde(default = "default_activation_key")]
    pub activation_key: String,
}

fn default_languages() -> Vec<String> {
    vec!["French".to_string()]
}

fn default_input_time_ms() -> u64 {
    200
}

fn default_activation_key() -> String {
    "Both".to_string()
}

impl Config {
    pub fn activation_key_parsed(&self) -> ActivationKey {
        match self.activation_key.as_str() {
            "Space" => ActivationKey::Space,
            "LeftRightArrow" => ActivationKey::LeftRightArrow,
            _ => ActivationKey::Both,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            languages: default_languages(),
            input_time_ms: default_input_time_ms(),
            activation_key: default_activation_key(),
        }
    }
}

pub fn config_path() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(".config").join("quickaccent").join("config.toml")
    } else {
        PathBuf::from("config.toml")
    }
}

pub fn load_config() -> Config {
    let path = config_path();
    eprintln!("[QuickAccent] Looking for config at: {}", path.display());

    match std::fs::read_to_string(&path) {
        Ok(contents) => match toml::from_str::<Config>(&contents) {
            Ok(config) => {
                eprintln!("[QuickAccent] Loaded config: languages = {:?}, input_time_ms = {}, activation_key = {}",
                    config.languages, config.input_time_ms, config.activation_key);
                config
            }
            Err(e) => {
                eprintln!("[QuickAccent] Failed to parse config: {}. Using defaults.", e);
                Config::default()
            }
        },
        Err(_) => {
            eprintln!("[QuickAccent] No config file found. Creating default at {}", path.display());
            let config = Config::default();
            // Try to create default config file
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let default_toml = r#"# QuickAccent Configuration
# Available languages:
#   Catalan, CrimeanTatar, Croatian, Czech, Danish, Dutch, Esperanto,
#   Estonian, Finnish, French, German, Greek, Hungarian, IPA, Iceland,
#   Irish, Italian, Kurdish, Lithuanian, Maltese, Maori, Norwegian,
#   Pinyin, Polish, Portuguese, ProtoIndoEuropean, Romanian,
#   Romanization, ScottishGaelic, Serbian, Slovak, Slovenian,
#   Spanish, Swedish, Turkish, Vietnamese, Welsh

languages = ["French"]

# Minimum time (ms) the letter must be held before accent is committed.
# If released sooner, it's treated as a false start and the trigger key
# (space/arrow) is replayed. Default: 200
# input_time_ms = 200

# Which key(s) trigger the accent overlay: "Space", "LeftRightArrow", or "Both"
# Default: "Both"
# activation_key = "Both"
"#;
            std::fs::write(&path, default_toml).ok();
            config
        }
    }
}
