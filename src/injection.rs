use enigo::{Enigo, Keyboard, Settings, Key, Direction};
use std::thread;
use std::time::Duration;

pub fn inject_char(ch: char) {
    thread::spawn(move || {
        let delay = Duration::from_millis(20);
        thread::sleep(delay);

        let mut enigo = match Enigo::new(&Settings::default()) {
            Ok(e) => e,
            Err(e) => {
                log::error!("Failed to create Enigo instance: {}", e);
                return;
            }
        };

        // Backspace to remove the original character
        if let Err(e) = enigo.key(Key::Backspace, Direction::Click) {
            log::error!("Failed to send backspace: {}", e);
            return;
        }
        thread::sleep(delay);

        // Type the selected accent character
        if let Err(e) = enigo.text(&ch.to_string()) {
            log::error!("Failed to inject character '{}': {}", ch, e);
        }
    });
}

/// Replay a space character after a false start (letter released too quickly).
pub fn inject_space() {
    thread::spawn(move || {
        let delay = Duration::from_millis(20);
        thread::sleep(delay);

        let mut enigo = match Enigo::new(&Settings::default()) {
            Ok(e) => e,
            Err(e) => {
                log::error!("Failed to create Enigo instance: {}", e);
                return;
            }
        };

        if let Err(e) = enigo.key(Key::Space, Direction::Click) {
            log::error!("Failed to inject space: {}", e);
        }
    });
}
