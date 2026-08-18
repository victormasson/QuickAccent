//! Hyprland integration (Omarchy and other Hyprland desktops).
//!
//! Hyprland keeps its own copy of the xkb configuration and exposes window
//! geometry over `hyprctl`, so the GNOME paths (gsettings + a shell
//! extension) do not apply. Everything here degrades to None/false when
//! Hyprland is not running, so the GNOME path stays untouched.

use crate::shell_ext::Rect;

/// Hyprland sets this for every client in the session.
pub fn is_running() -> bool {
    std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some()
}

fn hyprctl(args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("hyprctl").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Read a config option, e.g. `input:kb_options`.
pub fn get_option(option: &str) -> Option<String> {
    parse_option_str(&hyprctl(&["getoption", option, "-j"])?)
}

/// Set a config option at runtime. Applies immediately — no re-login, and
/// the user's hyprland.conf is left untouched (QuickAccent re-applies it on
/// every start).
pub fn set_option(option: &str, value: &str) -> bool {
    hyprctl(&["keyword", option, value]).is_some()
}

/// Frame rectangle of the focused window.
pub fn active_window_rect() -> Option<Rect> {
    parse_active_window(&hyprctl(&["activewindow", "-j"])?)
}

/// `{"option":"input:kb_options","int":0,"str":"caps:escape","set":true}`.
/// Hyprland reports unset strings as `[[EMPTY]]`.
fn parse_option_str(json: &str) -> Option<String> {
    let v = json_string_field(json, "str")?;
    (v != "[[EMPTY]]").then_some(v)
}

/// `{"address":"0x…","at":[11,11],"size":[1898,1058],…}`
fn parse_active_window(json: &str) -> Option<Rect> {
    let at = json_int_pair(json, "at")?;
    let size = json_int_pair(json, "size")?;
    (size.0 > 0 && size.1 > 0).then_some(Rect {
        x: at.0 as f32,
        y: at.1 as f32,
        width: size.0 as f32,
        height: size.1 as f32,
    })
}

/// Minimal extraction of `"key": "value"` — avoids a JSON dependency for
/// the two shapes hyprctl gives us.
fn json_string_field(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let rest = json.split(&needle).nth(1)?;
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let body = rest.strip_prefix('"')?;
    let mut out = String::new();
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => out.push(chars.next()?),
            _ => out.push(c),
        }
    }
    None
}

/// Extract `"key": [a, b]` as a pair of integers.
fn json_int_pair(json: &str, key: &str) -> Option<(i64, i64)> {
    let needle = format!("\"{key}\"");
    let rest = json.split(&needle).nth(1)?;
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let body = rest.strip_prefix('[')?;
    let body = body.split(']').next()?;
    let mut parts = body.split(',').map(|p| p.trim().parse::<i64>().ok());
    Some((parts.next()??, parts.next()??))
}

/// Append our option to a comma-separated xkb option list, without
/// duplicating it and without dropping the user's own options.
pub fn merge_option_list(current: &str, ours: &str) -> String {
    let mut kept: Vec<&str> = current
        .split(',')
        .map(str::trim)
        .filter(|o| !o.is_empty() && *o != ours)
        .collect();
    kept.push(ours);
    kept.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_getoption_string() {
        assert_eq!(
            parse_option_str(r#"{"option":"input:kb_options","int":0,"str":"caps:escape","set":true}"#),
            Some("caps:escape".into())
        );
        // Unset options
        assert_eq!(
            parse_option_str(r#"{"option":"input:kb_options","str":"[[EMPTY]]","set":false}"#),
            None
        );
        assert_eq!(parse_option_str(r#"{"option":"x","str":"","set":false}"#), Some(String::new()));
        assert_eq!(parse_option_str("not json"), None);
    }

    #[test]
    fn parses_active_window_geometry() {
        let json = r#"{"address":"0x5f0","mapped":true,"at":[11,53],"size":[1898,1024],"workspace":{"id":1}}"#;
        assert_eq!(
            parse_active_window(json),
            Some(Rect { x: 11.0, y: 53.0, width: 1898.0, height: 1024.0 })
        );
        // Negative coordinates (monitor left of the primary) are valid.
        assert_eq!(
            parse_active_window(r#"{"at":[-1920,0],"size":[1920,1080]}"#),
            Some(Rect { x: -1920.0, y: 0.0, width: 1920.0, height: 1080.0 })
        );
        // Degenerate or missing geometry must not place the overlay.
        assert_eq!(parse_active_window(r#"{"at":[0,0],"size":[0,0]}"#), None);
        assert_eq!(parse_active_window(r#"{"at":[0,0]}"#), None);
        assert_eq!(parse_active_window("{}"), None);
    }

    #[test]
    fn merges_option_lists_without_duplicates() {
        assert_eq!(merge_option_list("", "quickaccent:accents"), "quickaccent:accents");
        assert_eq!(
            merge_option_list("caps:escape", "quickaccent:accents"),
            "caps:escape,quickaccent:accents"
        );
        // Already present → stays once, and keeps its neighbours.
        assert_eq!(
            merge_option_list("caps:escape,quickaccent:accents,grp:alts_toggle", "quickaccent:accents"),
            "caps:escape,grp:alts_toggle,quickaccent:accents"
        );
        assert_eq!(merge_option_list("  ,  ", "quickaccent:accents"), "quickaccent:accents");
    }
}
