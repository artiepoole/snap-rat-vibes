use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::ListItem,
};

use crate::app::ConnectionItem;

pub(crate) fn connection_list_item(item: &ConnectionItem) -> ListItem<'static> {
    let marker = if item.connected {
        Span::styled(crate::symbols::dot_on(), Style::default().fg(Color::Green))
    } else {
        Span::styled(
            crate::symbols::dot_off(),
            Style::default().fg(Color::DarkGray),
        )
    };
    let name = if item.is_plug {
        item.plug_name.clone()
    } else {
        item.slot_name.clone()
    };

    let mut spans = vec![
        marker,
        Span::styled(
            item.interface_name.clone(),
            Style::default().fg(Color::White).bold(),
        ),
    ];
    if name != item.interface_name {
        spans.push(Span::styled(
            format!(" - {name}"),
            Style::default().fg(Color::DarkGray),
        ));
    }
    if item.connected {
        let peer = if item.is_plug {
            format!("{}:{}", item.slot_snap, item.slot_name)
        } else {
            format!("{}:{}", item.plug_snap, item.plug_name)
        };
        spans.push(Span::styled(
            format!("  {} {peer}", crate::symbols::arrow()),
            Style::default().fg(Color::DarkGray),
        ));
    } else {
        spans.push(Span::styled(
            "  [disconnected]",
            Style::default().fg(Color::DarkGray),
        ));
    }

    ListItem::new(Line::from(spans))
}
