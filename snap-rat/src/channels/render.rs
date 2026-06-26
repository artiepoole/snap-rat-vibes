use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph},
};

use crate::app::App;
use crate::layout::confinement_label;

/// Renders the channel picker list into `area` (already inset by the caller).
pub(crate) fn render_channel_picker_in(frame: &mut Frame, app: &mut App, area: Rect) {
    frame.render_widget(Clear, area);
    app.channel_picker_area = Some(area);

    let title = app
        .pending_channel_action
        .as_ref()
        .map(|action| action.label())
        .unwrap_or_else(|| "Pick channel".to_string());

    let items: Vec<ListItem> = app
        .available_channels
        .iter()
        .map(|(channel, info)| {
            if channel.is_empty() {
                return ListItem::new(Line::from(Span::styled(
                    format!("Custom channel{}", crate::symbols::ellipsis()),
                    Style::default().fg(Color::Cyan),
                )));
            }

            let mut spans = vec![Span::styled(
                channel.clone(),
                Style::default().fg(Color::White).bold(),
            )];

            if let Some(version) = &info.version {
                spans.push(Span::styled(
                    format!("  {version}"),
                    Style::default().fg(Color::Yellow),
                ));
            }

            if let Some(confinement) = &info.confinement {
                spans.push(Span::styled(
                    format!("  {}", confinement_label(confinement)),
                    Style::default().fg(Color::DarkGray).italic(),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(" {title} "))
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Yellow))
                .padding(Padding::horizontal(1)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(crate::symbols::play());

    frame.render_stateful_widget(list, area, &mut app.channel_picker_state);
}

/// Renders the custom channel text input into `area` (already inset by the caller).
pub(crate) fn render_channel_input_in(frame: &mut Frame, app: &mut App, area: Rect) {
    frame.render_widget(Clear, area);
    app.channel_input_area = Some(area);

    let action_label = app
        .pending_channel_action
        .as_ref()
        .map(|a| a.label())
        .unwrap_or_else(|| "Channel".to_string());

    let block = Block::default()
        .title(format!(" {action_label} "))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Yellow))
        .padding(Padding::horizontal(1));

    let text = format!(
        "{}{}",
        app.channel_input,
        if crate::symbols::is_unicode() {
            "█"
        } else {
            "_"
        }
    );
    frame.render_widget(Paragraph::new(text).block(block), area);
}
