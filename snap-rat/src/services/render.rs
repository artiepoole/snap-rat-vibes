use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::ListItem,
};
use snapd_rs_artie::AppInfo;

pub(crate) fn service_list_item(service: &AppInfo) -> ListItem<'static> {
    let active = service.active == Some(true);
    // Only treat as explicitly enabled if snapd says so.
    let explicitly_enabled = service.enabled == Some(true);

    // Four states with distinct symbols and colours.
    let (marker, name_color, status_span) = match (active, explicitly_enabled) {
        // Running and enabled — healthy.
        (true, true) => (
            Span::styled(crate::symbols::dot_on(), Style::default().fg(Color::Green)),
            Color::White,
            Span::styled("  running", Style::default().fg(Color::Green)),
        ),
        // Running but disabled — will stop on next reboot.
        (true, false) => (
            Span::styled(crate::symbols::dot_on(), Style::default().fg(Color::Yellow)),
            Color::White,
            Span::styled("  running (disabled)", Style::default().fg(Color::Yellow)),
        ),
        // Stopped but enabled — likely failed or crashed.
        (false, true) => (
            Span::styled(crate::symbols::dot_err(), Style::default().fg(Color::Red)),
            Color::Red,
            Span::styled("  stopped", Style::default().fg(Color::Red)),
        ),
        // Stopped and disabled — intentionally off.
        (false, false) => (
            Span::styled(
                crate::symbols::dot_off(),
                Style::default().fg(Color::DarkGray),
            ),
            Color::DarkGray,
            Span::styled("  stopped (disabled)", Style::default().fg(Color::DarkGray)),
        ),
    };

    ListItem::new(Line::from(vec![
        marker,
        Span::styled(service.name.clone(), Style::default().fg(name_color).bold()),
        status_span,
    ]))
}
