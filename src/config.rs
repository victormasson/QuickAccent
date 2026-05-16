use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_languages")]
    pub languages: Vec<String>,
}

fn default_languages() -> Vec<String> {
    vec!["French".to_string()]
}

impl Default for Config {
    fn default() -> Self {
        Config {
            languages: default_languages(),
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
                eprintln!("[QuickAccent] Loaded config: languages = {:?}", config.languages);
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
"#;
            std::fs::write(&path, default_toml).ok();
            config
        }
    }
}
