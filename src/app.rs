use iced::futures::SinkExt;
use iced::widget::{checkbox, column, container, pick_list, row, scrollable, slider, text};

use crate::config::{ActivationKey, ThemeChoice};
use iced::window;
use iced::{Color, Element, Length, Subscription, Task, Theme};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::state_machine::GrabEvent;

static GRAB_RX: OnceLock<Arc<Mutex<Option<UnboundedReceiver<GrabEvent>>>>> = OnceLock::new();

/// Requests from native UI (the status-bar menu) into the iced app.
#[derive(Debug, Clone, Copy)]
pub enum UiEvent {
    OpenSettings,
    Quit,
}

struct UiChannel {
    tx: UnboundedSender<UiEvent>,
    rx: Mutex<Option<UnboundedReceiver<UiEvent>>>,
}

fn ui_channel() -> &'static UiChannel {
    static CHANNEL: OnceLock<UiChannel> = OnceLock::new();
    CHANNEL.get_or_init(|| {
        let (tx, rx) = unbounded_channel();
        UiChannel {
            tx,
            rx: Mutex::new(Some(rx)),
        }
    })
}

/// Hand a native UI event to the app. Safe from any thread, before or after
/// the app is running.
pub fn request(event: UiEvent) {
    let _ = ui_channel().tx.send(event);
}

const SETTINGS_COLUMNS: usize = 3;

const TEXT_SIZE: f32 = 28.0;
const CELL_PADDING: f32 = 14.0;
const CELL_SPACING: f32 = 4.0;
// Outer padding (20px) plus a little spare room for rounding.
const PADDING: f32 = 28.0;
const WINDOW_HEIGHT: f32 = 70.0;

fn window_width_for(variants: &[String]) -> f32 {
    use iced::advanced::{
        graphics::text::Paragraph,
        text::{Paragraph as _, Text},
    };

    PADDING
        + variants
            .iter()
            .map(|variant| {
                let content = variant_label(variant);
                let paragraph = Paragraph::with_text(Text {
                    content: &content,
                    bounds: iced::Size::INFINITY,
                    size: TEXT_SIZE.into(),
                    line_height: Default::default(),
                    font: iced::Font::DEFAULT,
                    horizontal_alignment: iced::alignment::Horizontal::Left,
                    vertical_alignment: iced::alignment::Vertical::Top,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::None,
                });
                paragraph.min_bounds().width.ceil() + 2.0 * CELL_PADDING + CELL_SPACING
            })
            .sum::<f32>()
}

fn variant_label(ch: &str) -> String {
    // Display-only bases make combining marks visible. LRM prevents iced's
    // shrink-width labels from clipping RTL glyphs at the far edge.
    let base = if matches!(
        ch.chars().next(),
        Some(
            '\u{0300}'..='\u{036f}'
            | '\u{05b0}'..='\u{05bd}'
            | '\u{05bf}'
            | '\u{05c1}'
            | '\u{05c2}'
            | '\u{05c7}',
        )
    ) {
        "◌"
    } else {
        ""
    };
    format!("\u{200e}{base}{ch}")
}

/// Frame rect of the window being typed in, in the window system's own
/// coordinates (X11 pixels on Linux, screen points on macOS), refreshed before
/// opening the overlay — the overlay opens centered on it (i.e. on the
/// monitor in use). `scale` is how many of those units one overlay logical
/// pixel spans (>1 on a scaled XWayland monitor); the window is sized in
/// those units and drawn at that scale so it looks the same everywhere.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Anchor {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub scale: f32,
}

/// None = center on the primary monitor.
static OVERLAY_ANCHOR: Mutex<Option<Anchor>> = Mutex::new(None);

pub fn set_overlay_placement(anchor: Option<Anchor>) {
    *OVERLAY_ANCHOR.lock().unwrap() = anchor;
}

/// Unscaled anchor (x, y, w, h) — macOS.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn set_overlay_anchor(anchor: Option<(f32, f32, f32, f32)>) {
    set_overlay_placement(
        anchor.map(|(x, y, width, height)| Anchor { x, y, width, height, scale: 1.0 }),
    );
}

fn overlay_anchor() -> Option<Anchor> {
    *OVERLAY_ANCHOR.lock().unwrap()
}

/// Window size in window-system units for a `width` logical overlay.
fn overlay_size(width: f32, scale: f32) -> iced::Size {
    iced::Size::new(width * scale, WINDOW_HEIGHT * scale)
}

fn overlay_position(width: f32) -> window::Position {
    match overlay_anchor() {
        Some(a) => {
            let size = overlay_size(width, a.scale);
            window::Position::Specific(iced::Point::new(
                a.x + (a.width - size.width) / 2.0,
                a.y + (a.height - size.height) / 2.0,
            ))
        }
        None => window::Position::Centered,
    }
}

fn overlay_settings(width: f32) -> window::Settings {
    let scale = overlay_anchor().map_or(1.0, |a| a.scale);
    #[allow(unused_mut)]
    let mut settings = window::Settings {
        size: overlay_size(width, scale),
        decorations: false,
        transparent: true,
        level: window::Level::AlwaysOnTop,
        position: overlay_position(width),
        ..Default::default()
    };
    #[cfg(target_os = "linux")]
    {
        settings.platform_specific.override_redirect = true;
        settings.platform_specific.application_id = "quickaccent".into();
    }
    settings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_glyphs_stay_inside_the_visible_label() {
        use iced::advanced::{
            graphics::text::Paragraph,
            text::{Paragraph as _, Text},
        };
        for variant in [
            "﷼",
            "؋",
            "°C",
            "°F",
            "V\u{0307}",
            "…",
            "\u{0301}",
            "SS",
            "א",
            "אַ",
            "ײַ",
            "דזש",
            "\u{05b7}",
        ] {
            let content = variant_label(variant);
            let paragraph = Paragraph::with_text(Text {
                content: &content,
                bounds: iced::Size::new(1000.0, 42.0),
                size: 28.0.into(),
                line_height: Default::default(),
                font: iced::Font::DEFAULT,
                horizontal_alignment: iced::alignment::Horizontal::Left,
                vertical_alignment: iced::alignment::Vertical::Top,
                shaping: text::Shaping::Advanced,
                wrapping: Default::default(),
            });
            let visible_width = paragraph.min_bounds().width;
            let glyphs: Vec<_> = paragraph
                .buffer()
                .layout_runs()
                .flat_map(|run| run.glyphs)
                .collect();
            assert!(!glyphs.is_empty());
            assert!(
                glyphs
                    .iter()
                    .all(|g| g.x >= 0.0 && g.x + g.w <= visible_width + 0.1),
                "{content:?}: glyphs outside visible width {visible_width}"
            );
            let row = vec![variant.to_string(); 4];
            let required_width = 20.0 + 4.0 * (visible_width + 28.0) + 3.0 * 4.0;
            assert!(
                window_width_for(&row) >= required_width,
                "{variant:?}: shaped row is wider than the overlay window"
            );
        }
    }

    #[test]
    fn window_width_grows_with_variant_count() {
        assert!(window_width_for(&["é".into()]) < window_width_for(&["é".into(), "é".into()]));
        assert!(window_width_for(&["C".into()]) < window_width_for(&["°C".into()]));
        assert_eq!(window_width_for(&[]), PADDING);
    }

    #[test]
    fn overlay_follows_window_in_global_logical_coordinates() {
        // Screens left of or above the primary screen have negative origins.
        for (anchor, expected) in [
            ((2000.0, 100.0, 1000.0, 800.0), (2400.0, 465.0)),
            ((-1600.0, -900.0, 1200.0, 800.0), (-1100.0, -535.0)),
        ] {
            set_overlay_anchor(Some(anchor));
            let window::Position::Specific(point) = overlay_position(200.0) else {
                panic!("expected focused-window position");
            };
            assert_eq!((point.x, point.y), expected);
        }
        // An unavailable focused window must clear the previous anchor.
        set_overlay_anchor(None);
        assert!(matches!(
            overlay_position(200.0),
            window::Position::Centered
        ));
    }

    #[test]
    fn scaled_anchor_sizes_and_centers_the_overlay_in_window_units() {
        // A 2× XWayland monitor: the window is twice as big in X pixels and
        // still centered on the anchor.
        set_overlay_placement(Some(Anchor { x: 3840.0, y: 0.0, width: 1000.0, height: 800.0, scale: 2.0 }));
        let window::Position::Specific(point) = overlay_position(200.0) else {
            panic!("expected focused-window position");
        };
        assert_eq!((point.x, point.y), (4140.0, 330.0));
        assert_eq!(overlay_settings(200.0).size, iced::Size::new(400.0, WINDOW_HEIGHT * 2.0));
        set_overlay_placement(None);
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ShowOverlay(Vec<String>, usize),
    UpdateSelection(usize),
    HideOverlay,
    InjectChar,
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    OpenSettings,
    /// Right mouse button pressed in a window of ours (the picker has no
    /// panel icon on Hyprland, so this is its way to Settings).
    RightClicked(window::Id),
    Quit,
    ToggleLanguage(String, bool),
    SetTheme(ThemeChoice),
    SetHoldDelay(f32),
    SetInputTime(f32),
    SetActivationKey(ActivationKey),
    SetOverlayOpacity(f32),
    SetOverlayRadius(f32),
    SetChipRadius(f32),
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    Noop,
}

pub struct App {
    variants: Vec<String>,
    selected_index: usize,
    overlay_window: Option<window::Id>,
    /// Scale the open overlay window was sized with (see `Anchor::scale`).
    overlay_scale: f32,
    settings_window: Option<window::Id>,
    /// Enabled languages / symbol sets, in config order.
    languages: Vec<String>,
    /// Appearance from config; `System` follows macOS when a window opens.
    theme_choice: ThemeChoice,
    /// Resolved appearance for the open windows (macOS glass adapts to what is
    /// behind it; our text has to follow).
    dark: bool,
    hold_delay_ms: u64,
    input_time_ms: u64,
    activation_key: ActivationKey,
    overlay_opacity: f32,
    overlay_radius: f32,
    chip_radius: f32,
}

fn resolve_dark(choice: ThemeChoice) -> bool {
    crate::theme::is_dark(choice)
}

fn settings_slider<'a>(
    label: &str,
    value: String,
    sl: Element<'a, Message>,
) -> Element<'a, Message> {
    column(vec![
        row(vec![
            text(label.to_string()).size(13).width(Length::Fill).into(),
            text(value).size(13).into(),
        ])
        .into(),
        sl,
    ])
    .spacing(4)
    .into()
}

impl App {
    pub fn new(grab_rx: Arc<Mutex<Option<UnboundedReceiver<GrabEvent>>>>) -> (Self, Task<Message>) {
        GRAB_RX.set(grab_rx).ok();
        // Development aid: QUICKACCENT_DEMO=overlay|settings opens that window
        // at startup without needing the keyboard grab (or its permissions).
        let demo = std::env::var("QUICKACCENT_DEMO");
        if let Ok(which) = &demo {
            eprintln!("[QuickAccent] demo mode: {which}");
        }
        let boot = match demo.as_deref() {
            Ok("overlay") => Task::done(Message::ShowOverlay(
                ["é", "è", "ê", "ë", "ē", "ė"].map(String::from).to_vec(),
                1,
            )),
            Ok("settings") => Task::done(Message::OpenSettings),
            _ => Task::none(),
        };
        let cfg = crate::config::read_config().unwrap_or_default();
        let theme_choice = cfg.theme_parsed();
        let activation_key = cfg.activation_key_parsed();
        (
            App {
                variants: Vec::new(),
                selected_index: 0,
                overlay_window: None,
                overlay_scale: 1.0,
                settings_window: None,
                languages: cfg.languages,
                theme_choice,
                dark: resolve_dark(theme_choice),
                hold_delay_ms: cfg.hold_delay_ms,
                input_time_ms: cfg.input_time_ms,
                activation_key,
                overlay_opacity: cfg.overlay_opacity as f32,
                overlay_radius: cfg.overlay_radius as f32,
                chip_radius: cfg.chip_radius as f32,
            },
            boot,
        )
    }

    /// Keep the settings window's native chrome (title bar) on the resolved
    /// theme; iced only paints the content area.
    fn sync_settings_appearance(&self) -> Task<Message> {
        #[cfg(target_os = "macos")]
        if let Some(id) = self.settings_window {
            let dark = self.dark;
            return window::run_with_handle(id, move |handle| {
                use iced::window::raw_window_handle::RawWindowHandle;
                if let RawWindowHandle::AppKit(appkit) = handle.as_raw() {
                    crate::macos::apply_window_appearance(appkit.ns_view.as_ptr(), dark);
                }
            })
            .map(|_| Message::Noop);
        }
        Task::none()
    }

    /// Re-read appearance from config (it may have been edited by hand) and
    /// resolve it for the windows about to open.
    fn refresh_from_config(&mut self) {
        let cfg = crate::config::read_config().unwrap_or_default();
        self.theme_choice = cfg.theme_parsed();
        self.dark = resolve_dark(self.theme_choice);
        self.hold_delay_ms = cfg.hold_delay_ms;
        self.input_time_ms = cfg.input_time_ms;
        self.activation_key = cfg.activation_key_parsed();
        self.overlay_opacity = cfg.overlay_opacity as f32;
        self.overlay_radius = cfg.overlay_radius as f32;
        self.chip_radius = cfg.chip_radius as f32;
        self.languages = cfg.languages;
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowOverlay(variants, index) => {
                self.selected_index = index;
                let width = window_width_for(&variants);
                self.variants = variants;

                if let Some(id) = self.overlay_window {
                    // Window already open, just resize and update
                    return window::resize(id, overlay_size(width, self.overlay_scale));
                }

                #[cfg(target_os = "macos")]
                set_overlay_anchor(crate::macos::focused_window_rect());
                self.refresh_from_config();

                let settings = overlay_settings(width);
                self.overlay_scale = overlay_anchor().map_or(1.0, |a| a.scale);
                log::debug!(
                    "opening overlay window at {:?}, scale {}",
                    settings.position,
                    self.overlay_scale
                );
                let (id, open_task) = window::open(settings);
                log::debug!("overlay window id {id:?}");
                self.overlay_window = Some(id);
                open_task.map(Message::WindowOpened)
            }
            Message::UpdateSelection(index) => {
                self.selected_index = index;
                Task::none()
            }
            // On InjectChar the injection already happened (Linux: grab
            // thread via uinput; macOS: grab dispatch) — just close.
            Message::HideOverlay | Message::InjectChar => {
                self.variants.clear();
                if let Some(id) = self.overlay_window.take() {
                    return window::close(id);
                }
                Task::none()
            }
            Message::WindowOpened(id) => {
                log::debug!("window opened {id:?}");
                #[cfg(target_os = "macos")]
                if self.overlay_window == Some(id) {
                    // Runs on the event-loop thread before the first frame is
                    // shown, so the glass is there from the start.
                    let dark = self.dark;
                    let radius = self.overlay_radius as f64;
                    let glass = crate::theme::uses_glass();
                    return window::run_with_handle(id, move |handle| {
                        use iced::window::raw_window_handle::RawWindowHandle;
                        if let RawWindowHandle::AppKit(appkit) = handle.as_raw() {
                            if glass {
                                crate::macos::attach_glass_backdrop(
                                    appkit.ns_view.as_ptr(),
                                    dark,
                                    radius,
                                );
                            } else {
                                crate::macos::apply_window_appearance(appkit.ns_view.as_ptr(), dark);
                            }
                        }
                    })
                    .map(|_| Message::Noop);
                }
                if self.settings_window == Some(id) {
                    #[cfg(target_os = "macos")]
                    {
                        crate::macos::activate_app();
                        return Task::batch([
                            self.sync_settings_appearance(),
                            window::gain_focus(id),
                        ]);
                    }
                    #[cfg(not(target_os = "macos"))]
                    return window::gain_focus(id);
                }
                Task::none()
            }
            Message::WindowClosed(id) => {
                if self.settings_window == Some(id) {
                    self.settings_window = None;
                }
                if self.overlay_window == Some(id) {
                    self.overlay_window = None;
                    self.variants.clear();
                }
                Task::none()
            }
            Message::RightClicked(id) => {
                if self.overlay_window == Some(id) {
                    return Task::done(Message::OpenSettings);
                }
                Task::none()
            }
            Message::OpenSettings => {
                if let Some(id) = self.settings_window {
                    #[cfg(target_os = "macos")]
                    crate::macos::activate_app();
                    return window::gain_focus(id);
                }
                self.refresh_from_config();
                #[allow(unused_mut)]
                let mut settings = window::Settings {
                    size: iced::Size::new(560.0, 720.0),
                    min_size: Some(iced::Size::new(420.0, 360.0)),
                    position: window::Position::Centered,
                    level: window::Level::Normal,
                    exit_on_close_request: true,
                    ..Default::default()
                };
                #[cfg(target_os = "linux")]
                {
                    // Own class: Hyprland floats it (`hyprland.rs`) and the
                    // overlay's no_focus rule stays off it.
                    settings.platform_specific.application_id =
                        crate::hyprland::SETTINGS_CLASS.into();
                }
                let (id, open_task) = window::open(settings);
                self.settings_window = Some(id);
                open_task.map(Message::WindowOpened)
            }
            Message::Quit => iced::exit(),
            Message::ToggleLanguage(name, enabled) => {
                log::debug!("toggle {name} -> {enabled}");
                if enabled {
                    if !self.languages.contains(&name) {
                        self.languages.push(name);
                    }
                } else {
                    self.languages.retain(|l| *l != name);
                }
                // Apply now; the config watcher will reload once more when the
                // file lands, which is harmless.
                crate::mappings::reload(&self.languages);
                if let Err(e) = crate::config::set_languages(&self.languages) {
                    eprintln!("[QuickAccent] Failed to save languages: {e}");
                }
                Task::none()
            }
            Message::SetTheme(choice) => {
                self.theme_choice = choice;
                self.dark = resolve_dark(choice);
                if let Err(e) = crate::config::set_theme(choice) {
                    eprintln!("[QuickAccent] Failed to save theme: {e}");
                }
                self.sync_settings_appearance()
            }
            Message::SetHoldDelay(ms) => {
                self.hold_delay_ms = ms.round().clamp(50.0, 800.0) as u64;
                crate::grab::set_live(
                    self.input_time_ms,
                    self.hold_delay_ms,
                    self.activation_key,
                );
                if let Err(e) = crate::config::set_u64("hold_delay_ms", self.hold_delay_ms) {
                    eprintln!("[QuickAccent] Failed to save hold_delay_ms: {e}");
                }
                Task::none()
            }
            Message::SetInputTime(ms) => {
                self.input_time_ms = ms.round().clamp(50.0, 800.0) as u64;
                crate::grab::set_live(
                    self.input_time_ms,
                    self.hold_delay_ms,
                    self.activation_key,
                );
                if let Err(e) = crate::config::set_u64("input_time_ms", self.input_time_ms) {
                    eprintln!("[QuickAccent] Failed to save input_time_ms: {e}");
                }
                Task::none()
            }
            Message::SetActivationKey(key) => {
                self.activation_key = key;
                crate::grab::set_live(
                    self.input_time_ms,
                    self.hold_delay_ms,
                    self.activation_key,
                );
                if let Err(e) = crate::config::set_activation_key(key) {
                    eprintln!("[QuickAccent] Failed to save activation_key: {e}");
                }
                Task::none()
            }
            Message::SetOverlayOpacity(v) => {
                self.overlay_opacity = v.clamp(0.35, 1.0);
                if let Err(e) =
                    crate::config::set_f64("overlay_opacity", self.overlay_opacity as f64)
                {
                    eprintln!("[QuickAccent] Failed to save overlay_opacity: {e}");
                }
                Task::none()
            }
            Message::SetOverlayRadius(v) => {
                self.overlay_radius = v.clamp(0.0, 28.0);
                if let Err(e) =
                    crate::config::set_f64("overlay_radius", self.overlay_radius as f64)
                {
                    eprintln!("[QuickAccent] Failed to save overlay_radius: {e}");
                }
                Task::none()
            }
            Message::SetChipRadius(v) => {
                self.chip_radius = v.clamp(0.0, 16.0);
                if let Err(e) = crate::config::set_f64("chip_radius", self.chip_radius as f64) {
                    eprintln!("[QuickAccent] Failed to save chip_radius: {e}");
                }
                Task::none()
            }
            Message::Noop => Task::none(),
        }
    }

    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if self.settings_window == Some(window_id) {
            return self.settings_view();
        }
        if self.overlay_window != Some(window_id) || self.variants.is_empty() {
            return container(text(""))
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }

        let colors =
            crate::theme::overlay_colors(self.theme_choice, self.dark, self.overlay_opacity);
        let chip = colors.chip;
        let chip_text = colors.chip_text;
        let selected = colors.selected;
        let selected_text = colors.selected_text;
        let panel = colors.panel;
        let chip_radius = self.chip_radius;
        let overlay_radius = self.overlay_radius;

        let cells: Vec<Element<Message>> =
            self.variants
                .iter()
                .enumerate()
                .map(|(i, ch)| {
                    let is_selected = i == self.selected_index;
                    let label = text(variant_label(ch))
                        .size(TEXT_SIZE)
                        .font(iced::Font::DEFAULT)
                        .shaping(text::Shaping::Advanced)
                        .wrapping(text::Wrapping::None);

                    let cell = container(label).padding([8.0, CELL_PADDING]).style(
                        move |_theme: &Theme| container::Style {
                            background: Some(iced::Background::Color(if is_selected {
                                selected
                            } else {
                                chip
                            })),
                            border: iced::Border {
                                radius: chip_radius.into(),
                                ..Default::default()
                            },
                            text_color: Some(if is_selected {
                                selected_text
                            } else {
                                chip_text
                            }),
                            ..Default::default()
                        },
                    );

                    cell.into()
                })
                .collect();

        container(
            row(cells)
                .spacing(CELL_SPACING)
                .align_y(iced::Alignment::Center),
        )
        .padding(10)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: panel.map(iced::Background::Color),
            border: iced::Border {
                radius: overlay_radius.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }

    fn settings_view(&self) -> Element<'_, Message> {
        let section = |title: &str, names: &'static [&'static str]| -> Element<'_, Message> {
            let rows = names.chunks(SETTINGS_COLUMNS).map(|chunk| {
                let mut cells: Vec<Element<Message>> = chunk
                    .iter()
                    .map(|name| {
                        let enabled = self.languages.iter().any(|l| l == name);
                        checkbox(*name, enabled)
                            .on_toggle(move |on| Message::ToggleLanguage(name.to_string(), on))
                            .width(Length::Fill)
                            .into()
                    })
                    .collect();
                // Keep the grid aligned on a short last row.
                while cells.len() < SETTINGS_COLUMNS {
                    cells.push(iced::widget::Space::with_width(Length::Fill).into());
                }
                row(cells).spacing(8).into()
            });
            column(
                std::iter::once(text(title.to_string()).size(15).into())
                    .chain(rows)
                    .collect::<Vec<Element<Message>>>(),
            )
            .spacing(8)
            .into()
        };

        let appearance: Element<'_, Message> = column(vec![
            text("Appearance").size(15).into(),
            pick_list(ThemeChoice::ALL, Some(self.theme_choice), Message::SetTheme)
                .width(Length::Fill)
                .into(),
            settings_slider(
                "Overlay opacity",
                format!("{:.0}%", self.overlay_opacity * 100.0),
                slider(
                    35.0..=100.0,
                    self.overlay_opacity * 100.0,
                    |v| Message::SetOverlayOpacity(v / 100.0),
                )
                .into(),
            ),
            settings_slider(
                "Corner radius",
                format!("{:.0} px", self.overlay_radius),
                slider(0.0..=28.0, self.overlay_radius, Message::SetOverlayRadius).into(),
            ),
            settings_slider(
                "Chip radius",
                format!("{:.0} px", self.chip_radius),
                slider(0.0..=16.0, self.chip_radius, Message::SetChipRadius).into(),
            ),
        ])
        .spacing(10)
        .into();

        let behaviour: Element<'_, Message> = column(vec![
            text("Behaviour").size(15).into(),
            settings_slider(
                "Hold delay",
                format!("{} ms", self.hold_delay_ms),
                slider(50.0..=800.0, self.hold_delay_ms as f32, Message::SetHoldDelay).into(),
            ),
            text("How long to hold a letter before Space shows the picker.")
                .size(11)
                .into(),
            settings_slider(
                "Input time",
                format!("{} ms", self.input_time_ms),
                slider(50.0..=800.0, self.input_time_ms as f32, Message::SetInputTime).into(),
            ),
            text("Minimum hold before a picked accent is committed.")
                .size(11)
                .into(),
            text("Activation key").size(13).into(),
            pick_list(
                ActivationKey::ALL,
                Some(self.activation_key),
                Message::SetActivationKey,
            )
            .width(Length::Fill)
            .into(),
        ])
        .spacing(8)
        .into();

        let body = column(vec![
            text("QuickAccent").size(22).into(),
            text("Hold a letter, press Space, pick a variant, release the letter.")
                .size(13)
                .into(),
            appearance,
            behaviour,
            section("Languages", crate::mappings::LANGUAGES),
            section("Symbol sets", crate::mappings::SYMBOL_SETS),
            text(format!(
                "Changes apply immediately and are saved to {}",
                crate::config::config_path().display()
            ))
            .size(11)
            .into(),
        ])
        .spacing(18)
        .padding(24);

        container(scrollable(body))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|theme: &Theme| container::Style {
                background: Some(iced::Background::Color(theme.palette().background)),
                ..Default::default()
            })
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            Subscription::run(grab_subscription),
            Subscription::run(ui_subscription),
            window::close_events().map(Message::WindowClosed),
            iced::event::listen_with(|event, _status, window| match event {
                iced::Event::Mouse(iced::mouse::Event::ButtonPressed(
                    iced::mouse::Button::Right,
                )) => Some(Message::RightClicked(window)),
                _ => None,
            }),
        ])
    }

    /// Per-window UI scale: the overlay is sized in window-system units
    /// (`Anchor::scale` of them per logical pixel); other windows are 1:1.
    pub fn scale_factor(&self, window_id: window::Id) -> f64 {
        if self.overlay_window == Some(window_id) {
            f64::from(self.overlay_scale)
        } else {
            1.0
        }
    }

    pub fn theme(&self, _window_id: window::Id) -> Theme {
        crate::theme::iced_theme(self.theme_choice, self.dark)
    }

    /// Window backgrounds. The picker is see-through so rounded corners and
    /// (on macOS) glass show; the settings window paints its own background.
    pub fn style(&self, theme: &Theme) -> iced::daemon::Appearance {
        iced::daemon::Appearance {
            background_color: Color::TRANSPARENT,
            text_color: theme.palette().text,
        }
    }
}

fn ui_subscription() -> impl iced::futures::Stream<Item = Message> {
    iced::stream::channel(8, |mut output| async move {
        let mut rx = ui_channel()
            .rx
            .lock()
            .unwrap()
            .take()
            .expect("ui channel already taken");
        while let Some(event) = rx.recv().await {
            let msg = match event {
                UiEvent::OpenSettings => Message::OpenSettings,
                UiEvent::Quit => Message::Quit,
            };
            output.send(msg).await.ok();
        }
        std::future::pending::<()>().await;
    })
}

fn grab_subscription() -> impl iced::futures::Stream<Item = Message> {
    iced::stream::channel(50, |mut output| async move {
        let rx_holder = GRAB_RX.get().expect("GRAB_RX not initialized");
        let mut rx = rx_holder
            .lock()
            .unwrap()
            .take()
            .expect("grab_rx already taken");

        // recv() returns None when the grab side is gone (grab disabled or
        // its thread died) — stop instead of busy-looping on a closed channel.
        while let Some(event) = rx.recv().await {
            let msg = match event {
                GrabEvent::ShowOverlay { variants, index } => Message::ShowOverlay(variants, index),
                GrabEvent::UpdateSelection(index) => Message::UpdateSelection(index),
                GrabEvent::HideOverlay => Message::HideOverlay,
                GrabEvent::InjectChar(_) => Message::InjectChar,
                GrabEvent::FalseStart => Message::HideOverlay,
            };
            output.send(msg).await.ok();
        }
        std::future::pending::<()>().await;
    })
}
