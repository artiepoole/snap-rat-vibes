use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
};
use snapd_rs::{ChangeStatus, SnapConfinement};

pub fn rect_contains(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}

/// Maps a mouse row to a list-item index for a widget rendered with a single-row
/// border on each side (the standard ratatui `Block` with `Borders::ALL`).
pub fn list_row_to_index(area: Rect, row: u16, offset: usize) -> Option<usize> {
    let inner_y = area.y + 1;
    let inner_end = area.y + area.height.saturating_sub(1);
    if row >= inner_y && row < inner_end {
        Some((row - inner_y) as usize + offset)
    } else {
        None
    }
}

/// Maps a mouse row to a list-item index for a list rendered directly into an
/// inner area (border/padding already removed before passing to the widget).
pub fn inner_list_row_to_index(inner: Rect, row: u16, offset: usize) -> Option<usize> {
    if row >= inner.y && row < inner.y + inner.height {
        Some((row - inner.y) as usize + offset)
    } else {
        None
    }
}

pub fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let visible = max_chars.saturating_sub(1).max(1);
    let mut truncated = text.chars().take(visible).collect::<String>();
    truncated.push('…');
    truncated
}

pub fn format_size(bytes: u64) -> String {
    match bytes {
        0 => "0 B".to_string(),
        b if b < 1024 => format!("{b} B"),
        b if b < 1024 * 1024 => format!("{:.1} KB", b as f64 / 1024.0),
        b if b < 1024 * 1024 * 1024 => {
            format!("{:.1} MB", b as f64 / (1024.0 * 1024.0))
        }
        b => format!("{:.2} GB", b as f64 / (1024.0 * 1024.0 * 1024.0)),
    }
}

/// Format task progress as human-readable.
/// When both values are >= 1 KB we treat them as byte counts; otherwise show
/// plain integers (e.g. "1/3" for a multi-step task).
pub fn format_progress(done: i64, total: i64) -> String {
    if total >= 1024 || done >= 1024 {
        let done_s = format_size(done.max(0) as u64);
        let total_s = format_size(total.max(0) as u64);
        format!("{done_s} / {total_s}")
    } else {
        format!("{done}/{total}")
    }
}

pub fn progress_bar(done: i64, total: i64, width: usize) -> String {
    let total = total.max(0);
    let done = done.clamp(0, total.max(done));
    let filled = if total > 0 {
        ((done as usize) * width + (total as usize / 2)) / total as usize
    } else {
        0
    }
    .min(width);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(width - filled))
}

pub fn change_status_label(status: &ChangeStatus) -> &'static str {
    match status {
        ChangeStatus::Do => "Do",
        ChangeStatus::Doing => "Doing",
        ChangeStatus::Done => "Done",
        ChangeStatus::Abort => "Abort",
        ChangeStatus::Aborting => "Aborting",
        ChangeStatus::Error => "Error",
        ChangeStatus::Hold => "Hold",
        ChangeStatus::Wait => "Wait",
        ChangeStatus::Undone => "Undone",
        ChangeStatus::Undoing => "Undoing",
        _ => "Unknown",
    }
}

pub fn change_status_color(status: &ChangeStatus) -> Color {
    match status {
        ChangeStatus::Doing | ChangeStatus::Wait => Color::Yellow,
        ChangeStatus::Done => Color::Green,
        ChangeStatus::Error | ChangeStatus::Abort | ChangeStatus::Aborting => Color::Red,
        ChangeStatus::Hold => Color::DarkGray,
        _ => Color::White,
    }
}

pub fn confinement_label(confinement: &SnapConfinement) -> &'static str {
    match confinement {
        SnapConfinement::Strict => "strict",
        SnapConfinement::Classic => "classic",
        SnapConfinement::Devmode => "devmode",
        _ => "unknown",
    }
}

pub fn centered_popup(percent_x: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height + 2),
            Constraint::Fill(1),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

pub fn centered_popup_percent(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
