use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Wrap},
};

use crate::app::{App, ConfirmPending, ManageAction};
use crate::layout::centered_popup;

pub(crate) fn render_classic_confirm(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let popup = centered_popup(60, 9, area);
    frame.render_widget(Clear, popup);
    app.classic_confirm_area = Some(popup);

    let snap_name = app
        .classic_pending
        .as_ref()
        .map(|(n, _)| n.as_str())
        .or_else(|| {
            app.classic_local_path
                .as_deref()
                .and_then(|p| std::path::Path::new(p).file_name())
                .and_then(|n| n.to_str())
        })
        .unwrap_or("this snap");

    let block = Block::default()
        .title(" ⚠  Classic Confinement ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow))
        .padding(Padding::uniform(1));

    let message = Text::from(vec![
        Line::from(vec![
            Span::styled(snap_name, Style::default().fg(Color::Cyan).bold()),
            Span::raw(" uses "),
            Span::styled(
                "classic confinement",
                Style::default().fg(Color::Yellow).bold(),
            ),
            Span::raw(" and has full access to your system."),
        ]),
        Line::raw(""),
        Line::raw("Install anyway?"),
    ]);

    let inner = block.inner(popup);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);
    let btn_row = chunks[1];

    let yes_label = "  [ ✔ Install ]  ";
    let no_label = "  [ ✘ Cancel ]  ";
    let yes_w = yes_label.chars().count() as u16;
    let no_w = no_label.chars().count() as u16;
    app.confirm_yes_area = Some(Rect::new(btn_row.x, btn_row.y, yes_w, 1));
    app.confirm_no_area = Some(Rect::new(btn_row.x + yes_w + 1, btn_row.y, no_w, 1));

    let yes_style = if app.confirm_hovered == Some(true) {
        Style::default().fg(Color::Black).bg(Color::Green).bold()
    } else {
        Style::default().fg(Color::Green).bold()
    };
    let no_style = if app.confirm_hovered == Some(false) {
        Style::default().fg(Color::Black).bg(Color::Red).bold()
    } else {
        Style::default().fg(Color::Red).bold()
    };

    frame.render_widget(block, popup);
    frame.render_widget(
        Paragraph::new(message).wrap(Wrap { trim: false }),
        chunks[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(yes_label, yes_style),
            Span::raw(" "),
            Span::styled(no_label, no_style),
        ])),
        btn_row,
    );
}

pub(crate) fn render_confirm(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let popup = centered_popup(62, 7, area);
    frame.render_widget(Clear, popup);

    let border_color = if matches!(
        app.confirm_pending,
        Some(ConfirmPending::Action(ManageAction::Uninstall))
            | Some(ConfirmPending::Action(ManageAction::UninstallPurge))
            | Some(ConfirmPending::Action(ManageAction::Revert))
            | Some(ConfirmPending::Disconnect)
    ) {
        Color::Red
    } else {
        Color::Yellow
    };

    let block = Block::default()
        .title(" ⚠  Confirm ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .padding(Padding::uniform(1));

    let message = app.confirm_message.as_deref().unwrap_or("Are you sure?");

    // Split inner area: message rows on top, fixed 1-row button bar at bottom.
    let inner = block.inner(popup);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);
    let btn_row = chunks[1];

    let yes_label = "  [ ✔ Confirm ]  ";
    let no_label = "  [ ✘ Cancel ]  ";
    let yes_w = yes_label.chars().count() as u16;
    let no_w = no_label.chars().count() as u16;
    app.confirm_yes_area = Some(Rect::new(btn_row.x, btn_row.y, yes_w, 1));
    app.confirm_no_area = Some(Rect::new(btn_row.x + yes_w + 1, btn_row.y, no_w, 1));

    let yes_style = if app.confirm_hovered == Some(true) {
        Style::default().fg(Color::Black).bg(Color::Green).bold()
    } else {
        Style::default().fg(Color::Green).bold()
    };
    let no_style = if app.confirm_hovered == Some(false) {
        Style::default().fg(Color::Black).bg(Color::Red).bold()
    } else {
        Style::default().fg(Color::Red).bold()
    };

    // Render the block border first, then the content inside separately.
    frame.render_widget(block, popup);
    frame.render_widget(
        Paragraph::new(message)
            .style(Style::default().fg(Color::White).bold())
            .wrap(Wrap { trim: false }),
        chunks[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(yes_label, yes_style),
            Span::raw(" "),
            Span::styled(no_label, no_style),
        ])),
        btn_row,
    );
}
