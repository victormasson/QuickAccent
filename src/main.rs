mod app;
mod config;
mod grab;
mod injection;
mod mappings;
mod state_machine;

use std::sync::{Arc, Mutex};

fn main() -> iced::Result {
    env_logger::init();

    let config = config::load_config();
    mappings::init(&config.languages);

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let grab_rx = Arc::new(Mutex::new(Some(rx)));

    grab::run_grab_thread(tx);

    let grab_rx_clone = grab_rx.clone();
    iced::daemon("QuickAccent", app::App::update, app::App::view)
        .subscription(app::App::subscription)
        .theme(app::App::theme)
        .run_with(move || app::App::new(grab_rx_clone.clone()))
}
