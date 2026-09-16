use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationKey {
    Space,
    LeftRightArrow,
    Both,
}

impl ActivationKey {
    pub const ALL: [ActivationKey; 3] = [
        ActivationKey::Space,
        ActivationKey::LeftRightArrow,
        ActivationKey::Both,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ActivationKey::Space => "Space",
            ActivationKey::LeftRightArrow => "LeftRightArrow",
            ActivationKey::Both => "Both",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ActivationKey::Space => "Space",
            ActivationKey::LeftRightArrow => "Left / Right arrow",
            ActivationKey::Both => "Space and arrows",
        }
    }
}

impl std::fmt::Display for ActivationKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Appearance of the picker and the settings window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeChoice {
    System,
    Light,
    Dark,
    Dracula,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    RosePine,
    RosePineMoon,
    RosePineDawn,
}

impl ThemeChoice {
    /// Everything but `System`: the palettes `System` can resolve to.
    pub const PALETTES: [ThemeChoice; 10] = [
        ThemeChoice::Light,
        ThemeChoice::Dark,
        ThemeChoice::Dracula,
        ThemeChoice::CatppuccinLatte,
        ThemeChoice::CatppuccinFrappe,
        ThemeChoice::CatppuccinMacchiato,
        ThemeChoice::CatppuccinMocha,
        ThemeChoice::RosePine,
        ThemeChoice::RosePineMoon,
        ThemeChoice::RosePineDawn,
    ];

    pub const ALL: [ThemeChoice; 11] = [
        ThemeChoice::System,
        ThemeChoice::Light,
        ThemeChoice::Dark,
        ThemeChoice::Dracula,
        ThemeChoice::CatppuccinLatte,
        ThemeChoice::CatppuccinFrappe,
        ThemeChoice::CatppuccinMacchiato,
        ThemeChoice::CatppuccinMocha,
        ThemeChoice::RosePine,
        ThemeChoice::RosePineMoon,
        ThemeChoice::RosePineDawn,
    ];

    /// The value written to `config.toml`.
    pub fn as_str(self) -> &'static str {
        match self {
            ThemeChoice::System => "system",
            ThemeChoice::Light => "light",
            ThemeChoice::Dark => "dark",
            ThemeChoice::Dracula => "dracula",
            ThemeChoice::CatppuccinLatte => "catppuccin-latte",
            ThemeChoice::CatppuccinFrappe => "catppuccin-frappe",
            ThemeChoice::CatppuccinMacchiato => "catppuccin-macchiato",
            ThemeChoice::CatppuccinMocha => "catppuccin-mocha",
            ThemeChoice::RosePine => "rose-pine",
            ThemeChoice::RosePineMoon => "rose-pine-moon",
            ThemeChoice::RosePineDawn => "rose-pine-dawn",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ThemeChoice::System => "System",
            ThemeChoice::Light => "Light",
            ThemeChoice::Dark => "Dark",
            ThemeChoice::Dracula => "Dracula",
            ThemeChoice::CatppuccinLatte => "Catppuccin Latte",
            ThemeChoice::CatppuccinFrappe => "Catppuccin Frappé",
            ThemeChoice::CatppuccinMacchiato => "Catppuccin Macchiato",
            ThemeChoice::CatppuccinMocha => "Catppuccin Mocha",
            ThemeChoice::RosePine => "Rosé Pine",
            ThemeChoice::RosePineMoon => "Rosé Pine Moon",
            ThemeChoice::RosePineDawn => "Rosé Pine Dawn",
        }
    }
}

impl std::fmt::Display for ThemeChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_languages")]
    pub languages: Vec<String>,
    #[serde(default = "default_input_time_ms")]
    pub input_time_ms: u64,
    #[serde(default = "default_hold_delay_ms")]
    pub hold_delay_ms: u64,
    #[serde(default = "default_activation_key")]
    pub activation_key: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Palettes used while `theme = "system"`, for a light / dark desktop.
    #[serde(default = "default_theme_light")]
    pub theme_light: String,
    #[serde(default = "default_theme_dark")]
    pub theme_dark: String,
    #[serde(default = "default_overlay_opacity")]
    pub overlay_opacity: f64,
    #[serde(default = "default_overlay_radius")]
    pub overlay_radius: f64,
    #[serde(default = "default_chip_radius")]
    pub chip_radius: f64,
}

fn default_languages() -> Vec<String> {
    vec!["French".to_string()]
}

fn default_theme() -> String {
    "system".to_string()
}

fn default_theme_light() -> String {
    "light".to_string()
}

fn default_theme_dark() -> String {
    "dark".to_string()
}

fn default_input_time_ms() -> u64 {
    200
}

fn default_hold_delay_ms() -> u64 {
    250
}

fn default_activation_key() -> String {
    "Both".to_string()
}

fn default_overlay_opacity() -> f64 {
    0.88
}

fn default_overlay_radius() -> f64 {
    16.0
}

fn default_chip_radius() -> f64 {
    8.0
}

impl Config {
    pub fn activation_key_parsed(&self) -> ActivationKey {
        match self.activation_key.as_str() {
            "Space" => ActivationKey::Space,
            "LeftRightArrow" => ActivationKey::LeftRightArrow,
            _ => ActivationKey::Both,
        }
    }

    pub fn theme_parsed(&self) -> ThemeChoice {
        parse_theme(&self.theme).unwrap_or(ThemeChoice::System)
    }

    /// Palette for a light desktop while following the system; `system`
    /// or garbage falls back to plain Light.
    pub fn theme_light_parsed(&self) -> ThemeChoice {
        parse_theme(&self.theme_light)
            .filter(|c| *c != ThemeChoice::System)
            .unwrap_or(ThemeChoice::Light)
    }

    /// Palette for a dark desktop while following the system.
    pub fn theme_dark_parsed(&self) -> ThemeChoice {
        parse_theme(&self.theme_dark)
            .filter(|c| *c != ThemeChoice::System)
            .unwrap_or(ThemeChoice::Dark)
    }
}

fn parse_theme(value: &str) -> Option<ThemeChoice> {
    let value = value.to_ascii_lowercase();
    ThemeChoice::ALL.into_iter().find(|c| c.as_str() == value)
}

impl Default for Config {
    fn default() -> Self {
        Config {
            languages: default_languages(),
            input_time_ms: default_input_time_ms(),
            hold_delay_ms: default_hold_delay_ms(),
            activation_key: default_activation_key(),
            theme: default_theme(),
            theme_light: default_theme_light(),
            theme_dark: default_theme_dark(),
            overlay_opacity: default_overlay_opacity(),
            overlay_radius: default_overlay_radius(),
            chip_radius: default_chip_radius(),
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
                eprintln!("[QuickAccent] Loaded config: languages = {:?}, input_time_ms = {}, hold_delay_ms = {}, activation_key = {}",
                    config.languages, config.input_time_ms, config.hold_delay_ms, config.activation_key);
                config
            }
            Err(e) => {
                eprintln!(
                    "[QuickAccent] Failed to parse config: {}. Using defaults.",
                    e
                );
                Config::default()
            }
        },
        Err(_) => {
            eprintln!(
                "[QuickAccent] No config file found. Creating default at {}",
                path.display()
            );
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
# Additional sets: Special, Currency
# Extensions: Typography, Arrows, Math, CurrencyExtended
# See docs/CHARACTERS.md for symbol bindings.
# Hebrew/Yiddish use phonetic Latin keys; see docs/CHARACTERS.md.

languages = ["French"]

# Minimum time (ms) the letter must be held before accent is committed.
# If released sooner, it's treated as a false start and the trigger key
# (space/arrow) is replayed. Default: 200
# input_time_ms = 200

# Minimum hold time (ms) before a trigger (Space) will show the accent banner.
# Quick taps below this are treated as normal typing. Default: 250
# (PowerToys uses 200)
# hold_delay_ms = 250

# Which key(s) trigger the accent overlay: "Space", "LeftRightArrow", or "Both"
# Default: "Both"
# activation_key = "Both"

# Appearance of the picker and the settings window:
#   system, light, dark, dracula,
#   catppuccin-latte, catppuccin-frappe, catppuccin-macchiato, catppuccin-mocha,
#   rose-pine, rose-pine-moon, rose-pine-dawn
# Default: "system"
# theme = "system"
# With theme = "system", which palettes to use on a light / dark desktop
# (any value above except "system"):
# theme_light = "light"
# theme_dark = "dark"

# Picker overlay (GNOME-style rounded translucent panel), both Linux and macOS.
# overlay_opacity = 0.88    # 0.35–1.0
# overlay_radius = 16       # corner radius in px
# chip_radius = 8           # variant-chip corner radius in px
"#;
            std::fs::write(&path, default_toml).ok();
            config
        }
    }
}

pub fn read_config() -> Option<Config> {
    let path = config_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|contents| toml::from_str::<Config>(&contents).ok())
}

/// Parse config from a TOML string (tests + future tooling).
pub fn parse_config_str(contents: &str) -> Result<Config, toml::de::Error> {
    toml::from_str(contents)
}

/// Persist a new `languages` list, touching nothing else in the file so the
/// user's comments and other settings survive. The config watcher picks the
/// change up like a manual edit.
pub fn set_languages(languages: &[String]) -> std::io::Result<()> {
    let value = format!(
        "[{}]",
        languages
            .iter()
            .map(|l| format!("{l:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    set_value("languages", &value)
}

/// Persist the appearance choice; see [`set_languages`].
pub fn set_theme(theme: ThemeChoice) -> std::io::Result<()> {
    set_value("theme", &format!("{:?}", theme.as_str()))
}

/// Persist the palette used on a light desktop while following the system.
pub fn set_theme_light(theme: ThemeChoice) -> std::io::Result<()> {
    set_value("theme_light", &format!("{:?}", theme.as_str()))
}

/// Persist the palette used on a dark desktop while following the system.
pub fn set_theme_dark(theme: ThemeChoice) -> std::io::Result<()> {
    set_value("theme_dark", &format!("{:?}", theme.as_str()))
}

pub fn set_activation_key(key: ActivationKey) -> std::io::Result<()> {
    set_value("activation_key", &format!("{:?}", key.as_str()))
}

pub fn set_u64(key: &str, value: u64) -> std::io::Result<()> {
    set_value(key, &value.to_string())
}

pub fn set_f64(key: &str, value: f64) -> std::io::Result<()> {
    set_value(key, &format!("{value:.2}"))
}

fn set_value(key: &str, rendered_value: &str) -> std::io::Result<()> {
    let path = config_path();
    let current = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => {
            load_config(); // writes the commented default template
            std::fs::read_to_string(&path).unwrap_or_default()
        }
    };
    std::fs::write(&path, replace_assignment(&current, key, rendered_value))
}

/// Replace the `key = ...` assignment (a single-line value or a multi-line
/// array) in a TOML document, or append one if missing.
fn replace_assignment(toml: &str, key: &str, rendered_value: &str) -> String {
    let rendered = format!("{key} = {rendered_value}");
    let mut out = String::with_capacity(toml.len() + rendered.len());
    let mut lines = toml.lines();
    let mut replaced = false;
    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();
        let is_key = !replaced
            && trimmed.starts_with(key)
            && trimmed[key.len()..].trim_start().starts_with('=');
        if is_key {
            // Skip the rest of a multi-line array.
            let mut rest = &line[line.find('=').unwrap() + 1..];
            if rest.trim_start().starts_with('[') {
                while !rest.contains(']') {
                    match lines.next() {
                        Some(l) => rest = l,
                        None => break,
                    }
                }
            }
            out.push_str(&rendered);
            out.push('\n');
            replaced = true;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !replaced {
        if !out.is_empty() && !out.ends_with("\n\n") {
            out.push('\n');
        }
        out.push_str(&rendered);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let c = Config::default();
        assert_eq!(c.languages, vec!["French".to_string()]);
        assert_eq!(c.input_time_ms, 200);
        assert_eq!(c.hold_delay_ms, 250);
        assert_eq!(c.activation_key, "Both");
        assert_eq!(c.activation_key_parsed(), ActivationKey::Both);
        assert_eq!(c.overlay_opacity, 0.88);
        assert_eq!(c.overlay_radius, 16.0);
        assert_eq!(c.chip_radius, 8.0);
    }

    #[test]
    fn missing_fields_use_defaults() {
        let c = parse_config_str("# no keys\n").unwrap();
        assert_eq!(c.languages, vec!["French".to_string()]);
        assert_eq!(c.input_time_ms, 200);
        assert_eq!(c.hold_delay_ms, 250);
        assert_eq!(c.activation_key_parsed(), ActivationKey::Both);
    }

    #[test]
    fn full_toml_parse() {
        let c = parse_config_str(
            r#"
            languages = ["German", "Spanish"]
            input_time_ms = 100
            hold_delay_ms = 300
            activation_key = "Space"
            "#,
        )
        .unwrap();
        assert_eq!(
            c.languages,
            vec!["German".to_string(), "Spanish".to_string()]
        );
        assert_eq!(c.input_time_ms, 100);
        assert_eq!(c.hold_delay_ms, 300);
        assert_eq!(c.activation_key_parsed(), ActivationKey::Space);
        assert_eq!(c.overlay_opacity, 0.88);
    }

    #[test]
    fn overlay_style_parses() {
        let c = parse_config_str(
            "overlay_opacity = 0.5\noverlay_radius = 20\nchip_radius = 4\n",
        )
        .unwrap();
        assert_eq!(c.overlay_opacity, 0.5);
        assert_eq!(c.overlay_radius, 20.0);
        assert_eq!(c.chip_radius, 4.0);
    }

    #[test]
    fn activation_key_parsing() {
        let mut c = Config::default();
        c.activation_key = "Space".into();
        assert_eq!(c.activation_key_parsed(), ActivationKey::Space);
        c.activation_key = "LeftRightArrow".into();
        assert_eq!(c.activation_key_parsed(), ActivationKey::LeftRightArrow);
        c.activation_key = "Both".into();
        assert_eq!(c.activation_key_parsed(), ActivationKey::Both);
        c.activation_key = "whatever".into();
        assert_eq!(c.activation_key_parsed(), ActivationKey::Both);
    }

    #[test]
    fn invalid_toml_errors() {
        assert!(parse_config_str("languages = [").is_err());
    }

    #[test]
    fn replace_assignment_keeps_everything_else() {
        let doc = "# QuickAccent Configuration\n# comment\n\nlanguages = [\"French\"]\n\n# hold\n# hold_delay_ms = 250\ninput_time_ms = 100\n";
        let out = replace_assignment(doc, "languages", "[\"German\", \"Spanish\"]");
        assert_eq!(
            out,
            "# QuickAccent Configuration\n# comment\n\nlanguages = [\"German\", \"Spanish\"]\n\n# hold\n# hold_delay_ms = 250\ninput_time_ms = 100\n"
        );
        let parsed = parse_config_str(&out).unwrap();
        assert_eq!(parsed.languages, ["German", "Spanish"]);
        assert_eq!(parsed.input_time_ms, 100);
    }

    #[test]
    fn replace_assignment_handles_multiline_missing_and_lookalikes() {
        let multi = "languages = [\n  \"French\",\n  \"German\",\n]\nhold_delay_ms = 300\n";
        assert_eq!(
            replace_assignment(multi, "languages", "[\"Welsh\"]"),
            "languages = [\"Welsh\"]\nhold_delay_ms = 300\n"
        );
        // Commented-out or similarly named keys are left alone; a missing key is appended.
        let none = "# languages = [\"French\"]\nlanguages_extra = 1\n";
        assert_eq!(
            replace_assignment(none, "languages", "[\"Welsh\"]"),
            "# languages = [\"French\"]\nlanguages_extra = 1\n\nlanguages = [\"Welsh\"]\n"
        );
        assert_eq!(
            replace_assignment("", "languages", "[]"),
            "languages = []\n"
        );
    }

    #[test]
    fn theme_round_trips_through_the_file() {
        let doc = "languages = [\"French\"]\n# theme = \"system\"\n";
        let out = replace_assignment(doc, "theme", "\"dark\"");
        assert_eq!(
            out,
            "languages = [\"French\"]\n# theme = \"system\"\n\ntheme = \"dark\"\n"
        );
        assert_eq!(
            parse_config_str(&out).unwrap().theme_parsed(),
            ThemeChoice::Dark
        );
        let again = replace_assignment(&out, "theme", "\"light\"");
        assert_eq!(
            parse_config_str(&again).unwrap().theme_parsed(),
            ThemeChoice::Light
        );
        assert_eq!(Config::default().theme_parsed(), ThemeChoice::System);
        // System-mode palettes: defaults, named values, and "system" itself
        // is not allowed there.
        let d = Config::default();
        assert_eq!(d.theme_light_parsed(), ThemeChoice::Light);
        assert_eq!(d.theme_dark_parsed(), ThemeChoice::Dark);
        let c = parse_config_str(
            "theme_light = \"Rose-Pine-Dawn\"\ntheme_dark = \"catppuccin-mocha\"\n",
        )
        .unwrap();
        assert_eq!(c.theme_light_parsed(), ThemeChoice::RosePineDawn);
        assert_eq!(c.theme_dark_parsed(), ThemeChoice::CatppuccinMocha);
        let c = parse_config_str("theme_light = \"system\"\ntheme_dark = \"nope\"\n").unwrap();
        assert_eq!(c.theme_light_parsed(), ThemeChoice::Light);
        assert_eq!(c.theme_dark_parsed(), ThemeChoice::Dark);
        assert!(!ThemeChoice::PALETTES.contains(&ThemeChoice::System));
        assert_eq!(ThemeChoice::PALETTES.len() + 1, ThemeChoice::ALL.len());
        assert_eq!(
            parse_config_str("theme = \"weird\"\n")
                .unwrap()
                .theme_parsed(),
            ThemeChoice::System
        );
        for choice in ThemeChoice::ALL {
            assert_eq!(
                parse_config_str(&format!("theme = {:?}\n", choice.as_str()))
                    .unwrap()
                    .theme_parsed(),
                choice
            );
        }
    }
}
