use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding},
};

use crate::app::App;
use crate::layout::centered_popup_percent;

pub(crate) fn render_slot_picker(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let popup = centered_popup_percent(65, 55, area);
    frame.render_widget(Clear, popup);
    app.slot_picker_area = Some(popup);

    let interface_name = app
        .slot_picker_plug
        .as_ref()
        .map(|p| p.interface_name.as_str())
        .unwrap_or("interface");

    let items: Vec<ListItem> = app
        .slot_picker_items
        .iter()
        .map(|slot| {
            let is_system = matches!(slot.snap.as_str(), "system" | "core" | "snapd" | "");
            let snap_label = if is_system {
                Span::styled("system", Style::default().fg(Color::Yellow))
            } else {
                Span::styled(slot.snap.clone(), Style::default().fg(Color::Cyan))
            };
            let mut spans = vec![snap_label];
            if slot.slot != interface_name && slot.slot != slot.snap {
                spans.push(Span::styled(
                    format!(":{}", slot.slot),
                    Style::default().fg(Color::DarkGray),
                ));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(
                    " Connect '{interface_name}' to{} ",
                    crate::symbols::ellipsis()
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Cyan))
                .padding(Padding::horizontal(1)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(crate::symbols::play());

    frame.render_stateful_widget(list, popup, &mut app.slot_picker_state);
}
