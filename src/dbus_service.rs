//! Session-bus API the GNOME Shell panel menu, `quickaccent --settings` and
//! a second launch of the app call (Settings… / Quit).

use zbus::connection::Builder;
use zbus::interface;

struct Service;

#[interface(name = "io.github.victormasson.QuickAccent")]
impl Service {
    fn open_settings(&self) {
        crate::app::request(crate::app::UiEvent::OpenSettings);
    }

    fn quit(&self) {
        crate::app::request(crate::app::UiEvent::Quit);
    }
}

/// Export the well-known name on a background thread with its own Tokio
/// runtime (zbus's tokio backend will not run on a bare OS thread).
pub fn start() {
    std::thread::Builder::new()
        .name("quickaccent-dbus".into())
        .spawn(serve)
        .ok();
}

fn serve() {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("[QuickAccent] D-Bus runtime: {e}");
            return;
        }
    };
    rt.block_on(async {
        // The name may still be held by an instance the service is taking
        // over from (it was just asked to quit) — keep trying rather than
        // leaving the daemon unreachable for the rest of its life.
        let mut attempt = 0u32;
        loop {
            match export().await {
                Ok(()) => return,
                Err(e) => {
                    if attempt == 0 {
                        eprintln!("[QuickAccent] D-Bus: {e} — retrying");
                    }
                    attempt += 1;
                    tokio::time::sleep(std::time::Duration::from_secs(2.min(attempt as u64 * 2)))
                        .await;
                }
            }
        }
    });
}

async fn export() -> zbus::Result<()> {
    let _conn = Builder::session()?
        // Nobody may take the name from a live daemon (zbus otherwise asks
        // for ReplaceExisting/AllowReplacement, so a second copy would have
        // silently stolen it and left the daemon unreachable).
        .allow_name_replacements(false)
        .replace_existing_names(false)
        .name(NAME)?
        .serve_at(PATH, Service)?
        .build()
        .await?;
    std::future::pending::<()>().await;
    Ok(())
}

const NAME: &str = "io.github.victormasson.QuickAccent";
const PATH: &str = "/io/github/victormasson/QuickAccent";

/// Call `OpenSettings` or `Quit` on the running daemon. Errors when no
/// daemon owns the name.
pub fn call_remote(method: &str) -> zbus::Result<()> {
    let conn = zbus::blocking::Connection::session()?;
    conn.call_method(Some(NAME), PATH, Some(NAME), method, &())?;
    Ok(())
}
