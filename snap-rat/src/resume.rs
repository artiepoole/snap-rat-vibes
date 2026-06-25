use serde::{Deserialize, Serialize};

/// Minimal app state serialized before re-execing as root.
/// Restored on startup so the user lands back where they were.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeState {
    /// Name of the snap that was selected when elevation was triggered.
    pub selected_snap: Option<String>,
    /// Current search query text.
    pub search_query: String,
    /// Whether the "installed only" tab was active.
    pub show_installed_only: bool,
    /// The action that was being attempted — re-executed automatically on resume.
    pub pending: Option<ResumeAction>,
}

/// Actions that can be replayed after re-exec with elevated privileges.
/// Each variant carries everything needed to call the snapd API directly,
/// without showing a confirm dialog again (the user already confirmed).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResumeAction {
    Install {
        snap_name: String,
        channel: Option<String>,
    },
    InstallClassic {
        snap_name: String,
        channel: Option<String>,
    },
    Refresh {
        snap_name: String,
        channel: Option<String>,
    },
    SwitchChannel {
        snap_name: String,
        channel: String,
    },
    Revert {
        snap_name: String,
    },
    Enable {
        snap_name: String,
    },
    Disable {
        snap_name: String,
    },
    Uninstall {
        snap_name: String,
    },
    UninstallPurge {
        snap_name: String,
    },
}

impl ResumeAction {
    pub fn snap_name(&self) -> &str {
        match self {
            ResumeAction::Install { snap_name, .. }
            | ResumeAction::InstallClassic { snap_name, .. }
            | ResumeAction::Refresh { snap_name, .. }
            | ResumeAction::SwitchChannel { snap_name, .. }
            | ResumeAction::Revert { snap_name }
            | ResumeAction::Enable { snap_name }
            | ResumeAction::Disable { snap_name }
            | ResumeAction::Uninstall { snap_name }
            | ResumeAction::UninstallPurge { snap_name } => snap_name,
        }
    }
}

/// Returns true if a snapd error means the operation was denied due to
/// insufficient privileges.
pub fn is_elevation_needed(e: &snapd_rs_artie::Error) -> bool {
    if matches!(e, snapd_rs_artie::Error::Io(_) | snapd_rs_artie::Error::Connection(_)) {
        return true;
    }
    e.is_kind("login-required")
        || e.is_kind("access-denied")
        || e.is_kind("auth-cancelled")
        || e.is_kind("forbidden")
        || e.is_kind("unauthorized")
}

/// Parse a `--resume <path>` argument from the command line, deserialize
/// the resume state from that file, and delete the file.
pub fn parse_resume_arg() -> Option<ResumeState> {
    let args: Vec<String> = std::env::args().collect();
    let idx = args.iter().position(|a| a == "--resume")?;
    let path = args.get(idx + 1)?;
    let json = std::fs::read_to_string(path).ok()?;
    let _ = std::fs::remove_file(path);
    serde_json::from_str(&json).ok()
}

/// Serialize `resume`, write it to a temp file, restore the terminal, then
/// `exec()` the current binary under `pkexec` (or `sudo` as fallback) with
/// `--resume <path>`. Never returns.
pub fn reexec_elevated(resume: &ResumeState) -> ! {
    let json = serde_json::to_string(resume).expect("serialize resume state");
    let tmp_path = format!("/tmp/snap-rat-resume-{}.json", std::process::id());
    std::fs::write(&tmp_path, &json).expect("write resume state file");

    // Restore terminal before handing control to pkexec/sudo.
    // disable_raw_mode first so the terminal is in a sane state.
    let _ = crossterm::terminal::disable_raw_mode();
    {
        use std::io::Write as _;
        let mut stdout = std::io::stdout();
        // Leave alternate screen, clear the primary buffer, home the cursor.
        let _ = crossterm::execute!(
            stdout,
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0),
        );
        // Must flush before exec() — buffered writes are lost on process replacement.
        let _ = stdout.flush();
    }

    let exe = std::env::current_exe().expect("current_exe");
    use std::os::unix::process::CommandExt as _;

    // pkexec opens a native GUI polkit dialog and never touches the terminal,
    // so it works cleanly with ratatui. Only use it when a display is available
    // — without one, pkexec fails with "authentication error missing cookie"
    // because there is no polkit agent running.
    let has_display =
        std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some();
    if has_display && std::path::Path::new("/usr/bin/pkexec").exists() {
        let err = std::process::Command::new("/usr/bin/pkexec")
            .arg(&exe)
            .arg("--resume")
            .arg(&tmp_path)
            .exec();
        eprintln!("snap-rat: pkexec exec failed: {err}");
    }

    // Fallback: sudo prompts on the (now-restored) terminal.
    let err = std::process::Command::new("sudo")
        .arg(&exe)
        .arg("--resume")
        .arg(&tmp_path)
        .exec();
    eprintln!("snap-rat: sudo exec failed: {err}");
    std::process::exit(1);
}
