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
    if let Err(e) = rt.block_on(export()) {
        eprintln!("[QuickAccent] D-Bus: {e}");
    }
}

async fn export() -> zbus::Result<()> {
    let _conn = Builder::session()?
        .name("io.github.victormasson.QuickAccent")?
        .serve_at("/io/github/victormasson/QuickAccent", Service)?
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
