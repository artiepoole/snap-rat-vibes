use std::sync::OnceLock;

static USE_UNICODE: OnceLock<bool> = OnceLock::new();

pub fn init(unicode: bool) {
    let _ = USE_UNICODE.set(unicode);
}

pub fn detect() -> bool {
    if std::env::var_os("SNAP_RAT_ASCII").is_some() {
        return false;
    }
    if std::env::var("TERM").as_deref() == Ok("dumb") {
        return false;
    }
    for var in ["LC_ALL", "LC_CTYPE", "LANG"] {
        if let Ok(val) = std::env::var(var) {
            let up = val.to_ascii_uppercase();
            if up.contains("UTF-8") || up.contains("UTF8") {
                return true;
            }
        }
    }
    let term = std::env::var("TERM").unwrap_or_default();
    let prog = std::env::var("TERM_PROGRAM").unwrap_or_default();
    term.contains("256color")
        || term.starts_with("xterm")
        || matches!(
            prog.as_str(),
            "iTerm.app" | "WezTerm" | "vscode" | "Hyper" | "ghostty"
        )
}

pub fn is_unicode() -> bool {
    *USE_UNICODE.get_or_init(detect)
}

pub fn dot() -> &'static str {
    if is_unicode() { "●" } else { "*" }
}

pub fn dot_on() -> &'static str {
    if is_unicode() { "● " } else { "* " }
}

pub fn dot_off() -> &'static str {
    if is_unicode() { "○ " } else { "o " }
}

pub fn dot_err() -> &'static str {
    if is_unicode() { "✗ " } else { "X " }
}

pub fn error_sym() -> &'static str {
    if is_unicode() { "✗" } else { "X" }
}

pub fn ok_sym() -> &'static str {
    if is_unicode() { "✓" } else { "+" }
}

pub fn check() -> &'static str {
    if is_unicode() { "✔" } else { "+" }
}

pub fn play() -> &'static str {
    if is_unicode() { "▶ " } else { "> " }
}

pub fn play_hollow() -> &'static str {
    if is_unicode() { "▷ " } else { "> " }
}

pub fn arrow() -> &'static str {
    if is_unicode() { "→" } else { ">" }
}

pub fn ellipsis() -> &'static str {
    if is_unicode() { "…" } else { "..." }
}

pub fn dash_sep() -> &'static str {
    if is_unicode() { "──" } else { "--" }
}

pub fn enter_key() -> &'static str {
    if is_unicode() { "↵" } else { "Enter" }
}

pub fn nav_arrows() -> &'static str {
    if is_unicode() { "↑↓←→" } else { "^v<>" }
}

pub fn nav_hint() -> &'static str {
    if is_unicode() {
        "↑↓←→  navigate   ↵  confirm"
    } else {
        "^v<>  navigate   Enter  confirm"
    }
}
