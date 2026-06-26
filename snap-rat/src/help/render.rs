use ratatui::{
    Frame,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Wrap},
};

use crate::app::App;
use crate::layout::centered_popup;

pub(crate) fn render_help(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let popup = centered_popup(68, 40, area);
    frame.render_widget(Clear, popup);
    app.help_area = Some(popup);

    let block = Block::default()
        .title(" Help - Keybindings ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::symmetric(2, 1));

    fn row<'a>(key: &'a str, desc: &'a str) -> Line<'a> {
        Line::from(vec![
            Span::styled(
                format!("{key:<22}"),
                Style::default().fg(Color::Yellow).bold(),
            ),
            Span::raw(desc),
        ])
    }
    fn section(title: &str) -> Line<'_> {
        Line::from(Span::styled(
            format!("{} {title} ", crate::symbols::dash_sep()),
            Style::default().fg(Color::DarkGray),
        ))
    }

    let up_down = if crate::symbols::is_unicode() {
        "↑ / k  ↓ / j".to_string()
    } else {
        "^ / k  v / j".to_string()
    };
    let open = format!(
        "{} / l / {}",
        crate::symbols::arrow(),
        crate::symbols::enter_key()
    );
    let close = if crate::symbols::is_unicode() {
        "← / h / Esc".to_string()
    } else {
        "< / h / Esc".to_string()
    };
    let yes = if crate::symbols::is_unicode() {
        "← / h".to_string()
    } else {
        "< / h".to_string()
    };
    let no = format!("{} / l", crate::symbols::arrow());
    let confirm = format!("{} / Enter", crate::symbols::enter_key());
    let click_again = format!(
        "Select snap  (click again {} manage)",
        crate::symbols::arrow()
    );
    let back_to_browse = if crate::symbols::is_unicode() {
        "← / Esc / c".to_string()
    } else {
        "< / Esc / c".to_string()
    };
    let esc_twice = if crate::symbols::is_unicode() {
        "Esc (×2 fast)".to_string()
    } else {
        "Esc (x2 fast)".to_string()
    };

    let lines = vec![
        section("Browse"),
        row(&up_down, "Navigate snap list"),
        row(&open, "Open manage panel"),
        row(&close, "Close manage / cancel search"),
        row("/", "Focus search bar"),
        row("i", "Toggle installed-only filter"),
        row("o", "Cycle sort order  ([o]rder: in title)"),
        row("r", "Refresh snap list"),
        row("c", "Switch to [c]hanges tab"),
        row("s", "Switch to [s]naps tab (from Changes)"),
        row("p", "Toggle changes sidebar"),
        row("Click", &click_again),
        Line::raw(""),
        section("Manage - Actions pane"),
        row(&up_down, "Navigate actions"),
        row(&open, "Select (1st click) then run (2nd)"),
        row(&close, "Close manage panel"),
        row("Tab", "Switch to Connections pane"),
        row("Click item", "Select (1st click) then run (2nd)"),
        row("Click left pane", "Close manage panel"),
        Line::raw(""),
        section("Manage - Connections pane"),
        row(&up_down, "Navigate connections"),
        row(&open, "Select then connect / disconnect"),
        row("Tab", "Switch to Actions pane"),
        row("Esc", "Return to Actions pane"),
        row("Click item", "Select (1st click) then toggle (2nd)"),
        Line::raw(""),
        section("Confirm dialog"),
        row(&yes, "Move to Yes"),
        row(&no, "Move to No  (default)"),
        row(&confirm, "Confirm highlighted button"),
        row("y", "Confirm yes immediately"),
        row("n / Esc", "Cancel"),
        row("Click button", "Highlight (click again to confirm)"),
        row("Click outside", "Cancel"),
        Line::raw(""),
        section("Changes view"),
        row(&up_down, "Navigate changes list"),
        row("Tab", "Switch between list / task detail"),
        row("a", "Abort selected change"),
        row("r", "Refresh immediately (auto: every 3 s)"),
        row(&back_to_browse, "Back to browse"),
        Line::raw(""),
        section("General"),
        row("? / F1", "Toggle this help dialog"),
        row(&esc_twice, "Quit from browse"),
        row("q", "Quit from anywhere"),
        row("Ctrl-C", "Force quit"),
        Line::raw(""),
        Line::from(Span::styled(
            "  Press ?, F1, q, or Esc - or click outside - to close",
            Style::default().fg(Color::DarkGray).italic(),
        )),
    ];

    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        popup,
    );
}
