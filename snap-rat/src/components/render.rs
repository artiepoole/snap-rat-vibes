use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::ListItem,
};
use snapd_rs::ComponentInfo;

pub(crate) fn component_list_item(component: &ComponentInfo) -> ListItem<'static> {
    let installed = component.install_date.is_some();
    let marker = if installed {
        Span::styled(crate::symbols::dot_on(), Style::default().fg(Color::Green))
    } else {
        Span::styled(
            crate::symbols::dot_off(),
            Style::default().fg(Color::DarkGray),
        )
    };

    let mut spans = vec![
        marker,
        Span::styled(
            component.name.clone(),
            Style::default()
                .fg(if installed {
                    Color::White
                } else {
                    Color::DarkGray
                })
                .bold(),
        ),
    ];

    if let Some(ref version) = component.version {
        spans.push(Span::styled(
            format!("  {version}"),
            Style::default().fg(Color::DarkGray),
        ));
    }

    if let Some(ref type_) = component.type_ {
        spans.push(Span::styled(
            format!("  [{type_}]"),
            Style::default().fg(Color::DarkGray),
        ));
    }

    // Action hint at the end.
    if installed {
        spans.push(Span::styled(
            format!("  Remove {}", crate::symbols::arrow()),
            Style::default().fg(Color::Red),
        ));
    } else {
        spans.push(Span::styled(
            format!("  Install {}", crate::symbols::arrow()),
            Style::default().fg(Color::Green),
        ));
    }

    ListItem::new(Line::from(spans))
}
