use iced::futures::SinkExt;
use iced::widget::{container, row, text};
use iced::window;
use iced::{color, Element, Length, Subscription, Task, Theme};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::state_machine::GrabEvent;

static GRAB_RX: std::sync::OnceLock<Arc<Mutex<Option<UnboundedReceiver<GrabEvent>>>>> =
    std::sync::OnceLock::new();

// Per-cell: ~28px font + 2*14px horizontal padding + 4px spacing ≈ 60px per cell
// Plus 2*10px container padding + 2*12px for border radius margin
const CELL_WIDTH: f32 = 56.0;
const PADDING: f32 = 28.0;
const WINDOW_HEIGHT: f32 = 70.0;

fn window_width_for(count: usize) -> f32 {
    PADDING + (count as f32 * CELL_WIDTH)
}

#[derive(Debug, Clone)]
pub enum Message {
    ShowOverlay(Vec<String>, usize),
    UpdateSelection(usize),
    HideOverlay,
    InjectChar(String),
    WindowOpened(window::Id),
}

pub struct App {
    variants: Vec<String>,
    selected_index: usize,
    overlay_window: Option<window::Id>,
}

impl App {
    pub fn new(grab_rx: Arc<Mutex<Option<UnboundedReceiver<GrabEvent>>>>) -> (Self, Task<Message>) {
        GRAB_RX.set(grab_rx).ok();
        (
            App {
                variants: Vec::new(),
                selected_index: 0,
                overlay_window: None,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowOverlay(variants, index) => {
                self.selected_index = index;
                let width = window_width_for(variants.len());
                self.variants = variants;

                if let Some(id) = self.overlay_window {
                    // Window already open, just resize and update
                    return window::resize(id, iced::Size::new(width, WINDOW_HEIGHT));
                }

                let settings = window::Settings {
                    size: iced::Size::new(width, WINDOW_HEIGHT),
                    decorations: false,
                    transparent: true,
                    level: window::Level::AlwaysOnTop,
                    position: window::Position::Centered,
                    ..Default::default()
                };
                let (id, open_task) = window::open(settings);
                self.overlay_window = Some(id);
                open_task.map(Message::WindowOpened)
            }
            Message::UpdateSelection(index) => {
                self.selected_index = index;
                Task::none()
            }
            Message::HideOverlay | Message::InjectChar(_) => {
                self.variants.clear();
                if let Some(id) = self.overlay_window.take() {
                    return window::close(id);
                }
                Task::none()
            }
            Message::WindowOpened(_id) => Task::none(),
        }
    }

    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if self.overlay_window != Some(window_id) || self.variants.is_empty() {
            return container(text(""))
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }

        let cells: Vec<Element<Message>> = self
            .variants
            .iter()
            .enumerate()
            .map(|(i, ch)| {
                let is_selected = i == self.selected_index;
                let label = text(ch.clone()).size(28);

                let cell = container(label)
                    .padding([8, 14])
                    .style(move |_theme: &Theme| {
                        if is_selected {
                            container::Style {
                                background: Some(iced::Background::Color(color!(0x4A90D9))),
                                border: iced::Border {
                                    radius: 6.0.into(),
                                    ..Default::default()
                                },
                                text_color: Some(color!(0xFFFFFF)),
                                ..Default::default()
                            }
                        } else {
                            container::Style {
                                background: Some(iced::Background::Color(color!(0x3C3C3C))),
                                border: iced::Border {
                                    radius: 6.0.into(),
                                    ..Default::default()
                                },
                                text_color: Some(color!(0xCCCCCC)),
                                ..Default::default()
                            }
                        }
                    });

                cell.into()
            })
            .collect();

        container(row(cells).spacing(4).align_y(iced::Alignment::Center))
            .padding(10)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(color!(0x2D2D2D, 0.95))),
                border: iced::Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run(grab_subscription)
    }

    pub fn theme(&self, _window: window::Id) -> Theme {
        Theme::CatppuccinMocha
    }
}

fn grab_subscription() -> impl iced::futures::Stream<Item = Message> {
    iced::stream::channel(50, |mut output| async move {
        let rx_holder = GRAB_RX.get().expect("GRAB_RX not initialized");
        let mut rx = rx_holder
            .lock()
            .unwrap()
            .take()
            .expect("grab_rx already taken");

        loop {
            if let Some(event) = rx.recv().await {
                let msg = match event {
                    GrabEvent::ShowOverlay { variants, index } => {
                        Message::ShowOverlay(variants, index)
                    }
                    GrabEvent::UpdateSelection(index) => Message::UpdateSelection(index),
                    GrabEvent::HideOverlay => Message::HideOverlay,
                    GrabEvent::InjectChar(ch) => Message::InjectChar(ch),
                    GrabEvent::FalseStart => Message::HideOverlay,
                };
                output.send(msg).await.ok();
            }
        }
    })
}
