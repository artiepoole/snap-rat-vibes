use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph},
};

use crate::app::App;
use crate::layout::{centered_popup, centered_popup_percent, confinement_label};

pub(crate) fn render_channel_picker(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let popup = centered_popup_percent(70, 60, area);
    frame.render_widget(Clear, popup);
    app.channel_picker_area = Some(popup);

    let title = app
        .pending_channel_action
        .as_ref()
        .map(|action| action.label())
        .unwrap_or("Pick channel");

    let items: Vec<ListItem> = app
        .available_channels
        .iter()
        .map(|(channel, info)| {
            if channel.is_empty() {
                return ListItem::new(Line::from(Span::styled(
                    "Custom channel…",
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
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow))
                .padding(Padding::horizontal(1)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, popup, &mut app.channel_picker_state);
}

pub(crate) fn render_channel_input(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let popup = centered_popup(50, 3, area);
    frame.render_widget(Clear, popup);
    app.channel_input_area = Some(popup);

    let action_label = app
        .pending_channel_action
        .as_ref()
        .map(|a| a.label())
        .unwrap_or("Channel");

    let block = Block::default()
        .title(format!(" {action_label} "))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow))
        .padding(Padding::horizontal(1));

    let text = format!("{}█", app.channel_input);
    frame.render_widget(Paragraph::new(text).block(block), popup);
}
