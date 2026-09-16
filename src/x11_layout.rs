//! Map compositor-logical window geometry onto the XWayland screen.
//!
//! The overlay is an X11 window (see `main.rs`), so it is placed in XWayland
//! pixel coordinates. Hyprland reports window geometry (`hyprctl
//! activewindow`) in its own logical layout, but lays XWayland monitors out
//! differently: packed left to right from x=0 in monitor order, ignoring the
//! logical positions, and at physical size when
//! `xwayland:force_zero_scaling` is on. A monitor above or below the first
//! one, out of order, or scaled therefore has different coordinates in the two
//! spaces, and an overlay placed with logical numbers lands elsewhere or off
//! the X screen entirely (X has no negative coordinates). So: find the
//! logical monitor under the focused window, find the same output in
//! XWayland's RandR layout, and rescale the rect from one to the other.

use crate::shell_ext::Rect;

/// A monitor rectangle, in either the compositor's logical space or the
/// XWayland pixel space, keyed by the output name (`DP-1`, `eDP-1`, …) that
/// both sides use.
#[derive(Debug, Clone, PartialEq)]
pub struct Monitor {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Where to open the overlay: the focused window's rect in X11 pixels, and
/// how many X pixels one logical pixel spans on that monitor (1 unless
/// `force_zero_scaling` is on) — the overlay is drawn at that scale so it
/// keeps its size on screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub rect: Rect,
    pub scale: f32,
}

/// Focused-window placement for the overlay. On Hyprland the logical rect is
/// mapped through the XWayland layout; when that fails (no RandR, unknown
/// output names) or on GNOME — where mutter lays XWayland out like the logical
/// layout — the rect is used as is.
pub fn focused_window_placement() -> Option<Placement> {
    let rect = crate::shell_ext::focused_window_rect()?;
    if crate::hyprland::is_running() {
        let logical = hyprland_monitors();
        let x11 = x11_monitors();
        match place(rect, &logical, &x11) {
            Some(p) => {
                log::debug!("[QuickAccent] overlay anchor {rect:?} -> x11 {p:?}");
                return Some(p);
            }
            None => log::warn!(
                "[QuickAccent] could not map {rect:?} onto the XWayland layout \
                 (hyprland: {logical:?}, x11: {x11:?}); using logical coordinates"
            ),
        }
    }
    Some(Placement { rect, scale: 1.0 })
}

/// Map `rect` (logical) onto the X11 layout. The monitor is the logical one
/// containing the rect's centre, or the nearest; its X11 twin is matched by
/// name, falling back to list order when the names differ (Hyprland creates
/// XWayland outputs in monitor order).
pub fn place(rect: Rect, logical: &[Monitor], x11: &[Monitor]) -> Option<Placement> {
    let (cx, cy) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    let (index, lm) = logical
        .iter()
        .enumerate()
        .min_by(|a, b| distance_sq(a.1, cx, cy).total_cmp(&distance_sq(b.1, cx, cy)))?;
    let xm = x11
        .iter()
        .find(|m| m.name == lm.name)
        .or_else(|| (x11.len() == logical.len()).then(|| x11.get(index)).flatten())?;
    if lm.width <= 0.0 || lm.height <= 0.0 {
        return None;
    }
    let sx = xm.width / lm.width;
    let sy = xm.height / lm.height;
    Some(Placement {
        rect: Rect {
            x: xm.x + (rect.x - lm.x) * sx,
            y: xm.y + (rect.y - lm.y) * sy,
            width: rect.width * sx,
            height: rect.height * sy,
        },
        scale: sx,
    })
}

/// Squared distance from a point to a monitor rect (0 inside).
fn distance_sq(m: &Monitor, px: f32, py: f32) -> f32 {
    let dx = (m.x - px).max(0.0).max(px - (m.x + m.width));
    let dy = (m.y - py).max(0.0).max(py - (m.y + m.height));
    dx * dx + dy * dy
}

/// Hyprland's logical layout from `hyprctl monitors -j`.
pub fn hyprland_monitors() -> Vec<Monitor> {
    std::process::Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| parse_hyprland_monitors(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_default()
}

/// `width`/`height` are the mode's pixels; the logical size is that divided
/// by `scale`, with the sides swapped for the 90°/270° transforms (odd
/// values). `x`/`y` are already logical.
pub fn parse_hyprland_monitors(json: &str) -> Vec<Monitor> {
    let Ok(serde_json::Value::Array(items)) = serde_json::from_str::<serde_json::Value>(json) else {
        return Vec::new();
    };
    items
        .iter()
        .filter(|m| m["disabled"].as_bool() != Some(true))
        .filter_map(|m| {
            let name = m["name"].as_str()?.to_string();
            let scale = m["scale"].as_f64().filter(|s| *s > 0.0).unwrap_or(1.0) as f32;
            let (mut w, mut h) = (m["width"].as_f64()? as f32, m["height"].as_f64()? as f32);
            if m["transform"].as_i64().unwrap_or(0) % 2 == 1 {
                std::mem::swap(&mut w, &mut h);
            }
            Some(Monitor {
                name,
                x: m["x"].as_f64()? as f32,
                y: m["y"].as_f64()? as f32,
                width: w / scale,
                height: h / scale,
            })
        })
        .collect()
}

/// XWayland's layout from RandR (`$DISPLAY`), the same view `xrandr` shows.
pub fn x11_monitors() -> Vec<Monitor> {
    fn query() -> Result<Vec<Monitor>, Box<dyn std::error::Error>> {
        use x11rb::connection::Connection;
        use x11rb::protocol::randr::ConnectionExt as _;
        use x11rb::protocol::xproto::ConnectionExt as _;
        let (conn, screen) = x11rb::connect(None)?;
        let root = conn.setup().roots[screen].root;
        let reply = conn.randr_get_monitors(root, true)?.reply()?;
        let mut monitors = Vec::with_capacity(reply.monitors.len());
        for m in reply.monitors {
            let name = conn.get_atom_name(m.name)?.reply()?.name;
            monitors.push(Monitor {
                name: String::from_utf8_lossy(&name).into_owned(),
                x: f32::from(m.x),
                y: f32::from(m.y),
                width: f32::from(m.width),
                height: f32::from(m.height),
            });
        }
        Ok(monitors)
    }
    query().unwrap_or_else(|e| {
        log::warn!("[QuickAccent] RandR monitor query failed: {e}");
        Vec::new()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mon(name: &str, x: f32, y: f32, w: f32, h: f32) -> Monitor {
        Monitor { name: name.into(), x, y, width: w, height: h }
    }

    const RECT: Rect = Rect { x: 100.0, y: -980.0, width: 800.0, height: 600.0 };

    #[test]
    fn maps_a_monitor_above_the_primary_onto_its_packed_x11_slot() {
        // Logical: DP-2 sits above DP-1. XWayland: packed side by side.
        let logical = [mon("DP-1", 0.0, 0.0, 1920.0, 1080.0), mon("DP-2", 0.0, -1080.0, 1920.0, 1080.0)];
        let x11 = [mon("DP-1", 0.0, 0.0, 1920.0, 1080.0), mon("DP-2", 1920.0, 0.0, 1920.0, 1080.0)];
        let p = place(RECT, &logical, &x11).unwrap();
        assert_eq!(p.rect, Rect { x: 2020.0, y: 100.0, width: 800.0, height: 600.0 });
        assert_eq!(p.scale, 1.0);
    }

    #[test]
    fn rescales_for_force_zero_scaling() {
        // A 2× monitor: logical 1920×1080, XWayland 3840×2160 at the packed offset.
        let logical = [mon("eDP-1", 0.0, 0.0, 1920.0, 1080.0), mon("DP-1", 1920.0, 0.0, 1920.0, 1080.0)];
        let x11 = [mon("eDP-1", 0.0, 0.0, 3840.0, 2160.0), mon("DP-1", 3840.0, 0.0, 1920.0, 1080.0)];
        let rect = Rect { x: 10.0, y: 20.0, width: 500.0, height: 300.0 };
        let p = place(rect, &logical, &x11).unwrap();
        assert_eq!(p.rect, Rect { x: 20.0, y: 40.0, width: 1000.0, height: 600.0 });
        assert_eq!(p.scale, 2.0);
        // Same rect on the unscaled monitor is a pure translation.
        let rect = Rect { x: 1930.0, y: 20.0, width: 500.0, height: 300.0 };
        let p = place(rect, &logical, &x11).unwrap();
        assert_eq!(p.rect, Rect { x: 3850.0, y: 20.0, width: 500.0, height: 300.0 });
        assert_eq!(p.scale, 1.0);
    }

    #[test]
    fn identity_layout_is_a_no_op() {
        let layout = [mon("DP-1", 0.0, 0.0, 1920.0, 1080.0), mon("DP-2", 1920.0, 0.0, 1920.0, 1080.0)];
        let rect = Rect { x: 2000.0, y: 100.0, width: 1000.0, height: 800.0 };
        assert_eq!(place(rect, &layout, &layout).unwrap(), Placement { rect, scale: 1.0 });
    }

    #[test]
    fn falls_back_to_list_order_when_names_differ() {
        let logical = [mon("DP-1", 0.0, 0.0, 1920.0, 1080.0), mon("DP-2", 0.0, -1080.0, 1920.0, 1080.0)];
        let x11 = [mon("XWAYLAND0", 0.0, 0.0, 1920.0, 1080.0), mon("XWAYLAND1", 1920.0, 0.0, 1920.0, 1080.0)];
        assert_eq!(place(RECT, &logical, &x11).unwrap().rect.x, 2020.0);
        // Different counts: no guess.
        assert_eq!(place(RECT, &logical, &x11[..1]), None);
        assert_eq!(place(RECT, &[], &x11), None);
    }

    #[test]
    fn picks_the_nearest_monitor_for_a_rect_hanging_off_screen() {
        let logical = [mon("DP-1", 0.0, 0.0, 1920.0, 1080.0), mon("DP-2", 1920.0, 0.0, 1920.0, 1080.0)];
        let rect = Rect { x: 3700.0, y: 900.0, width: 800.0, height: 600.0 }; // centre past DP-2's edge
        let p = place(rect, &logical, &logical).unwrap();
        assert_eq!(p.rect, rect);
    }

    /// Needs an X server: `cargo test -- --ignored randr_query`.
    #[test]
    #[ignore]
    fn randr_query_lists_monitors() {
        let monitors = x11_monitors();
        eprintln!("{monitors:#?}");
        assert!(!monitors.is_empty());
        assert!(monitors.iter().all(|m| !m.name.is_empty() && m.width > 0.0 && m.height > 0.0));
    }

    #[test]
    fn parses_hyprctl_monitors_json() {
        let json = r#"[
          {"id":0,"name":"DP-1","width":2560,"height":1440,"x":0,"y":0,"scale":1.0,"transform":0,"disabled":false},
          {"id":1,"name":"DP-2","width":3840,"height":2160,"x":2560,"y":0,"scale":2.0,"transform":0,"disabled":false},
          {"id":2,"name":"HDMI-A-1","width":1920,"height":1080,"x":-1080,"y":0,"scale":1.0,"transform":1,"disabled":false},
          {"id":3,"name":"DP-3","width":1920,"height":1080,"x":0,"y":0,"scale":1.0,"transform":0,"disabled":true}
        ]"#;
        assert_eq!(
            parse_hyprland_monitors(json),
            vec![
                mon("DP-1", 0.0, 0.0, 2560.0, 1440.0),
                mon("DP-2", 2560.0, 0.0, 1920.0, 1080.0),
                mon("HDMI-A-1", -1080.0, 0.0, 1080.0, 1920.0),
            ]
        );
        assert!(parse_hyprland_monitors("not json").is_empty());
        assert!(parse_hyprland_monitors("{}").is_empty());
    }
}
