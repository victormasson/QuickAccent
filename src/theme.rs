use iced::{color, Color, Theme};

use crate::config::ThemeChoice;

/// Official Rosé Pine palettes (https://rosepinetheme.com) — iced has no built-in.
fn rose_pine_palette(
    background: Color,
    text: Color,
    primary: Color,
    success: Color,
    danger: Color,
) -> Theme {
    Theme::custom(
        "Rose Pine".into(),
        iced::theme::Palette {
            background,
            text,
            primary,
            success,
            danger,
        },
    )
}

/// The palette to draw with: `System` follows the desktop, using the
/// configured light or dark palette; anything else is itself.
pub fn effective(
    choice: ThemeChoice,
    light: ThemeChoice,
    dark: ThemeChoice,
    system_is_dark: bool,
) -> ThemeChoice {
    match choice {
        ThemeChoice::System if system_is_dark => dark,
        ThemeChoice::System => light,
        other => other,
    }
}

pub fn iced_theme(choice: ThemeChoice, dark: bool) -> Theme {
    match choice {
        ThemeChoice::System | ThemeChoice::Light | ThemeChoice::Dark => {
            if dark {
                Theme::Dark
            } else {
                Theme::Light
            }
        }
        ThemeChoice::Dracula => Theme::Dracula,
        ThemeChoice::CatppuccinLatte => Theme::CatppuccinLatte,
        ThemeChoice::CatppuccinFrappe => Theme::CatppuccinFrappe,
        ThemeChoice::CatppuccinMacchiato => Theme::CatppuccinMacchiato,
        ThemeChoice::CatppuccinMocha => Theme::CatppuccinMocha,
        ThemeChoice::RosePine => rose_pine_palette(
            color!(0x191724),
            color!(0xe0def4),
            color!(0xc4a7e7),
            color!(0x9ccfd8),
            color!(0xeb6f92),
        ),
        ThemeChoice::RosePineMoon => rose_pine_palette(
            color!(0x232136),
            color!(0xe0def4),
            color!(0xc4a7e7),
            color!(0x9ccfd8),
            color!(0xeb6f92),
        ),
        ThemeChoice::RosePineDawn => rose_pine_palette(
            color!(0xfaf4ed),
            color!(0x575279),
            color!(0x907aa9),
            color!(0x56949f),
            color!(0xb4637a),
        ),
    }
}

/// Blur behind the translucent picker (macOS only).
pub fn uses_glass() -> bool {
    cfg!(target_os = "macos")
}

pub fn is_dark(choice: ThemeChoice) -> bool {
    match choice {
        ThemeChoice::Light | ThemeChoice::CatppuccinLatte | ThemeChoice::RosePineDawn => false,
        ThemeChoice::Dark
        | ThemeChoice::Dracula
        | ThemeChoice::CatppuccinFrappe
        | ThemeChoice::CatppuccinMacchiato
        | ThemeChoice::CatppuccinMocha
        | ThemeChoice::RosePine
        | ThemeChoice::RosePineMoon => true,
        #[cfg(target_os = "macos")]
        ThemeChoice::System => crate::macos::is_dark_appearance(),
        #[cfg(target_os = "linux")]
        ThemeChoice::System => linux_system_is_dark(),
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        ThemeChoice::System => false,
    }
}

#[cfg(target_os = "linux")]
fn linux_system_is_dark() -> bool {
    crate::xkb_custom::gsettings_get("org.gnome.desktop.interface", "color-scheme")
        .map(|s| s.contains("prefer-dark"))
        .unwrap_or(false)
}

pub struct OverlayColors {
    pub panel: Option<Color>,
    pub chip: Color,
    pub chip_text: Color,
    pub selected: Color,
    pub selected_text: Color,
}

pub fn overlay_colors(choice: ThemeChoice, dark: bool, opacity: f32) -> OverlayColors {
    let opacity = opacity.clamp(0.35, 1.0);
    let p = iced_theme(choice, dark).palette();
    let chip = if dark {
        shift(p.background, 0.08)
    } else {
        shift(p.background, -0.06)
    };
    OverlayColors {
        panel: Some(Color {
            a: opacity,
            ..p.background
        }),
        chip,
        chip_text: p.text,
        selected: p.primary,
        selected_text: color!(0xFFFFFF),
    }
}

fn shift(c: Color, delta: f32) -> Color {
    Color {
        r: (c.r + delta).clamp(0.0, 1.0),
        g: (c.g + delta).clamp(0.0, 1.0),
        b: (c.b + delta).clamp(0.0, 1.0),
        a: c.a,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_themes_map_to_iced() {
        assert!(matches!(
            iced_theme(ThemeChoice::Dracula, true),
            Theme::Dracula
        ));
        assert!(matches!(
            iced_theme(ThemeChoice::CatppuccinMocha, true),
            Theme::CatppuccinMocha
        ));
        assert!(matches!(
            iced_theme(ThemeChoice::CatppuccinLatte, false),
            Theme::CatppuccinLatte
        ));
        assert!(matches!(
            iced_theme(ThemeChoice::Light, false),
            Theme::Light
        ));
        assert!(matches!(iced_theme(ThemeChoice::Dark, true), Theme::Dark));
    }

    #[test]
    fn system_resolves_to_the_configured_palettes() {
        let (l, d) = (ThemeChoice::RosePineDawn, ThemeChoice::CatppuccinMocha);
        assert_eq!(effective(ThemeChoice::System, l, d, false), l);
        assert_eq!(effective(ThemeChoice::System, l, d, true), d);
        // A named choice ignores the system palettes.
        assert_eq!(effective(ThemeChoice::Dracula, l, d, false), ThemeChoice::Dracula);
        assert_eq!(effective(ThemeChoice::Light, l, d, true), ThemeChoice::Light);
    }

    #[test]
    fn glass_is_macos_only() {
        assert_eq!(uses_glass(), cfg!(target_os = "macos"));
    }

    #[test]
    fn dark_classification() {
        assert!(!is_dark(ThemeChoice::Light));
        assert!(!is_dark(ThemeChoice::CatppuccinLatte));
        assert!(!is_dark(ThemeChoice::RosePineDawn));
        assert!(is_dark(ThemeChoice::Dark));
        assert!(is_dark(ThemeChoice::Dracula));
        assert!(is_dark(ThemeChoice::RosePine));
        assert!(is_dark(ThemeChoice::CatppuccinMocha));
    }
}
