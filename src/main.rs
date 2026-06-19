mod app;
mod config;
mod grab;
mod injection;
mod mappings;
mod state_machine;
#[cfg(target_os = "macos")]
mod macos;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use notify::{recommended_watcher, RecursiveMode, Watcher};

fn main() -> iced::Result {
    env_logger::init();

    let config = config::load_config();
    mappings::init(&config.languages);

    start_config_watcher();

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let grab_rx = Arc::new(Mutex::new(Some(rx)));

    grab::run_grab_thread(tx, &config);

    #[cfg(target_os = "macos")]
    crate::macos::setup_status_item();

    let grab_rx_clone = grab_rx.clone();
    iced::daemon("QuickAccent", app::App::update, app::App::view)
        .subscription(app::App::subscription)
        .theme(app::App::theme)
        .run_with(move || app::App::new(grab_rx_clone.clone()))
}

fn start_config_watcher() {
    let config_dir = match config::config_path().parent() {
        Some(p) => p.to_path_buf(),
        None => return,
    };
    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = match recommended_watcher(tx) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("[QuickAccent] Failed to create config watcher: {}", e);
                return;
            }
        };
        if let Err(e) = watcher.watch(&config_dir, RecursiveMode::NonRecursive) {
            eprintln!("[QuickAccent] Failed to watch config dir: {}", e);
            return;
        }
        let mut last_reload = Instant::now() - Duration::from_secs(1);
        let debounce = Duration::from_millis(400);
        for res in rx {
            match res {
                Ok(event) => {
                    let relevant = event.paths.iter().any(|p| {
                        p.file_name().map_or(false, |n| n == "config.toml")
                    });
                    if !relevant {
                        continue;
                    }
                    let now = Instant::now();
                    if now.duration_since(last_reload) < debounce {
                        continue;
                    }
                    last_reload = now;
                    if let Some(cfg) = config::read_config() {
                        mappings::reload(&cfg.languages);
                        eprintln!("[QuickAccent] Reloaded config: languages = {:?}", cfg.languages);
                    }
                }
                Err(e) => eprintln!("[QuickAccent] Config watcher error: {}", e),
            }
        }
    });
}
