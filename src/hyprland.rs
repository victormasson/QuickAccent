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
/// the user's config files are left untouched (QuickAccent re-applies it on
/// every start and after each `configreloaded`).
///
/// Hyprland has two config parsers. The legacy one accepts `hyprctl keyword`;
/// the Lua one (Omarchy 4: `~/.config/hypr/hyprland.lua`) rejects it with
/// "keyword can't work with non-legacy parsers. Use eval." — while `hyprctl`
/// still exits 0, so the exit code proves nothing. Returns whether the
/// compositor *accepted* the command; callers must confirm the effect with
/// `get_option`.
pub fn set_option(option: &str, value: &str) -> bool {
    if uses_lua_config() {
        return set_option_lua(option, value) || set_option_legacy(option, value);
    }
    set_option_legacy(option, value) || set_option_lua(option, value)
}

fn set_option_legacy(option: &str, value: &str) -> bool {
    hyprctl(&["keyword", option, value])
        .map(|r| is_ok_response(&r))
        .unwrap_or(false)
}

fn set_option_lua(option: &str, value: &str) -> bool {
    let Some(snippet) = lua_config_snippet(option, value) else {
        return false;
    };
    hyprctl(&["eval", &snippet])
        .map(|r| !is_error_response(&r))
        .unwrap_or(false)
}

/// Omarchy 4 and any Hyprland ≥ 0.55 with a Lua config.
pub fn uses_lua_config() -> bool {
    dirs::config_dir()
        .map(|d| d.join("hypr/hyprland.lua").is_file())
        .unwrap_or(false)
}

/// `hyprctl keyword` answers a bare `ok` on success.
fn is_ok_response(response: &str) -> bool {
    response.trim() == "ok"
}

/// Error replies from the compositor come back on stdout with exit 0.
fn is_error_response(response: &str) -> bool {
    let r = response.trim().to_ascii_lowercase();
    r.starts_with("error")
        || r.contains("can't work")
        || r.contains("invalid")
        || r.contains("unknown")
        || r.contains("failed")
}

/// `input:kb_options` + `a,b` → `hl.config({ input = { kb_options = "a,b" } })`
/// (nested sections nest tables). Only string values are needed here.
pub fn lua_config_snippet(option: &str, value: &str) -> Option<String> {
    let mut parts: Vec<&str> = option.split(':').collect();
    let key = parts.pop()?;
    if key.is_empty() || parts.iter().any(|p| p.is_empty()) {
        return None;
    }
    let mut inner = format!("{key} = \"{}\"", lua_escape(value));
    for section in parts.iter().rev() {
        inner = format!("{section} = {{ {inner} }}");
    }
    Some(format!("hl.config({{ {inner} }})"))
}

fn lua_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Persistent form of the option for the user's own config — Omarchy keeps
/// user overrides in `~/.config/hypr/input.lua`.
pub fn persistent_hint(option: &str, value: &str) -> String {
    match lua_config_snippet(option, value) {
        Some(snippet) if uses_lua_config() => format!(
            "add to ~/.config/hypr/input.lua (Omarchy) or hyprland.lua, then `hyprctl reload`:\n  {snippet}"
        ),
        _ => format!("add to hyprland.conf, then `hyprctl reload`:\n  {option} = {value}"),
    }
}

/// Window class of the accent overlay (see `app.rs`
/// `application_id = "quickaccent"`).
pub const OVERLAY_CLASS: &str = "quickaccent";

/// Keep Hyprland's keyboard focus off the accent overlay.
///
/// The overlay is focus-proofed with X11 `override_redirect` (`app.rs`),
/// which does nothing for a native-Wayland toplevel: Hyprland focuses the
/// overlay, so the committed accent is typed into it instead of the app the
/// user is working in, and nothing lands. A `no_focus` window rule leaves
/// focus on that app. Applied at runtime (config files untouched) and, like
/// `kb_options`, re-applied after each `configreloaded` — `hyprctl reload`
/// drops runtime rules, which also keeps this from accumulating.
///
/// Unlike an option there is no read-back for window rules, so success is the
/// compositor accepting the command; on refusal we print the persistent form.
pub fn ensure_overlay_no_focus() {
    if !is_running() {
        return;
    }
    // Try the running config's own parser first, fall back to the other.
    // The branches look identical to clippy but the `||` order is the point:
    // short-circuit means "prefer this parser".
    #[allow(clippy::if_same_then_else)]
    let applied = if uses_lua_config() {
        set_overlay_no_focus_lua() || set_overlay_no_focus_legacy()
    } else {
        set_overlay_no_focus_legacy() || set_overlay_no_focus_lua()
    };
    if !applied {
        eprintln!(
            "[QuickAccent] could not keep Hyprland's focus off the accent overlay; \
             picker accents may be swallowed. Add to ~/.config/hypr/windows.lua \
             (Omarchy) or hyprland.lua, then `hyprctl reload`:\n  {}",
            overlay_no_focus_lua()
        );
    }
}

/// `hl.window_rule` is Hyprland's native Lua binding (Omarchy's `o.window`
/// helper wraps it), so it works on any Lua-config Hyprland.
fn overlay_no_focus_lua() -> String {
    format!("hl.window_rule({{ match = {{ class = \"{OVERLAY_CLASS}\" }}, no_focus = true }})")
}

fn set_overlay_no_focus_lua() -> bool {
    hyprctl(&["eval", &overlay_no_focus_lua()])
        .map(|r| !is_error_response(&r))
        .unwrap_or(false)
}

fn set_overlay_no_focus_legacy() -> bool {
    let rule = format!("nofocus, class:^({OVERLAY_CLASS})$");
    hyprctl(&["keyword", "windowrulev2", &rule])
        .map(|r| is_ok_response(&r))
        .unwrap_or(false)
}

/// Hyprland re-reads its config on `hyprctl reload` — and Omarchy triggers
/// that on every theme change — which drops runtime-set options. Watch the
/// event socket and call `on_reload` after each `configreloaded`.
pub fn watch_config_reloads(on_reload: impl Fn() + Send + 'static) {
    let Some(sig) = std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE") else {
        return;
    };
    let Some(runtime) = std::env::var_os("XDG_RUNTIME_DIR") else {
        return;
    };
    let socket = std::path::Path::new(&runtime)
        .join("hypr")
        .join(sig)
        .join(".socket2.sock");
    std::thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        loop {
            let Ok(stream) = std::os::unix::net::UnixStream::connect(&socket) else {
                std::thread::sleep(std::time::Duration::from_secs(10));
                continue;
            };
            let reader = BufReader::new(stream);
            let mut last = std::time::Instant::now() - std::time::Duration::from_secs(5);
            for line in reader.lines().map_while(Result::ok) {
                if is_config_reloaded_event(&line)
                    && last.elapsed() > std::time::Duration::from_millis(500)
                {
                    last = std::time::Instant::now();
                    on_reload();
                }
            }
            // Socket closed (compositor restarting?) — back off and retry.
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
    });
}

/// Event lines look like `configreloaded>>` (no payload).
pub fn is_config_reloaded_event(line: &str) -> bool {
    line.split(">>").next().map(str::trim) == Some("configreloaded")
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
    fn lua_snippet_builds_nested_tables_and_escapes() {
        assert_eq!(
            lua_config_snippet("input:kb_options", "caps:escape,quickaccent:accents").as_deref(),
            Some(r#"hl.config({ input = { kb_options = "caps:escape,quickaccent:accents" } })"#)
        );
        assert_eq!(
            lua_config_snippet("input:touchpad:natural_scroll", "true").as_deref(),
            Some(r#"hl.config({ input = { touchpad = { natural_scroll = "true" } } })"#)
        );
        assert_eq!(
            lua_config_snippet("misc:x", r#"a"b\c"#).as_deref(),
            Some(r#"hl.config({ misc = { x = "a\"b\\c" } })"#)
        );
        assert_eq!(lua_config_snippet("", "v"), None);
        assert_eq!(lua_config_snippet("input:", "v"), None);
    }

    #[test]
    fn classifies_hyprctl_responses() {
        assert!(is_ok_response("ok\n"));
        assert!(!is_ok_response("keyword can't work with non-legacy parsers. Use eval."));
        assert!(is_error_response("keyword can't work with non-legacy parsers. Use eval."));
        assert!(is_error_response("Error: invalid option"));
        assert!(!is_error_response("ok"));
        assert!(!is_error_response(""));
    }

    #[test]
    fn recognises_configreloaded_events() {
        assert!(is_config_reloaded_event("configreloaded>>"));
        assert!(is_config_reloaded_event("configreloaded>>\r"));
        assert!(!is_config_reloaded_event("workspace>>1"));
        assert!(!is_config_reloaded_event("activewindow>>foot,configreloaded"));
    }

    #[test]
    fn overlay_no_focus_rule_targets_the_overlay_class() {
        assert_eq!(
            overlay_no_focus_lua(),
            r#"hl.window_rule({ match = { class = "quickaccent" }, no_focus = true })"#
        );
        assert_eq!(OVERLAY_CLASS, "quickaccent");
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
