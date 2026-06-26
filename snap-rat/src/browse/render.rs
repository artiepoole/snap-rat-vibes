use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Padding, Paragraph},
};

use crate::app::{App, AppMode};
use crate::types::DisplaySnap;

pub(crate) fn render_tabs(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_changes = app.mode == AppMode::Changes;

    let tab_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    app.snaps_tab_area = Some(tab_layout[0]);
    app.changes_tab_area = Some(tab_layout[1]);

    let snaps_style = if !is_changes {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let changes_style = if is_changes {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                if !is_changes {
                    crate::symbols::play()
                } else {
                    crate::symbols::play_hollow()
                },
                snaps_style,
            ),
            Span::styled("[s]naps", snaps_style),
        ])),
        tab_layout[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                if is_changes {
                    crate::symbols::play()
                } else {
                    crate::symbols::play_hollow()
                },
                changes_style,
            ),
            Span::styled("[c]hanges", changes_style),
        ])),
        tab_layout[1],
    );
}

pub(crate) fn render_search(frame: &mut Frame, app: &mut App, area: Rect) {
    app.search_area = Some(area);
    let border_style = if app.search_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Search ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .padding(Padding::horizontal(1));

    let query_display = if app.search_focused {
        format!(
            "{}{}",
            app.search_query,
            if crate::symbols::is_unicode() {
                "█"
            } else {
                "_"
            }
        )
    } else if app.search_query.is_empty() {
        format!("Press / to search the store{}", crate::symbols::ellipsis())
    } else {
        app.search_query.clone()
    };

    let style = if app.search_query.is_empty() && !app.search_focused {
        Style::default().fg(Color::DarkGray).italic()
    } else {
        Style::default().fg(Color::White)
    };

    let paragraph = Paragraph::new(query_display).style(style).block(block);
    frame.render_widget(paragraph, area);
}

pub(crate) fn render_list(frame: &mut Frame, app: &mut App, area: Rect) {
    app.snap_list_area = Some(area);
    let list_active = app.mode == AppMode::Browse && !app.search_focused;
    let sort_label = app.sort_mode.label();
    let order_hint = format!("[o]rder: {sort_label}");
    let title = if app.showing_results {
        if app.show_installed_only {
            format!(
                " Installed from \"{}\" ({}) {} ",
                app.search_query,
                app.display_snaps().len(),
                order_hint
            )
        } else {
            format!(
                " Results for \"{}\" ({}) {} ",
                app.search_query,
                app.store_results.len(),
                order_hint
            )
        }
    } else {
        format!(" Installed Snaps ({}) {} ", app.installed.len(), order_hint)
    };

    let border_style = if list_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let items: Vec<ListItem> = app.display_snaps().iter().map(snap_list_item).collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(if list_active {
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        })
        .highlight_symbol(if list_active {
            crate::symbols::play()
        } else {
            crate::symbols::play_hollow()
        });

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn snap_list_item(snap: &DisplaySnap) -> ListItem<'static> {
    let installed_marker = if snap.installed {
        Span::styled(crate::symbols::dot_on(), Style::default().fg(Color::Green))
    } else {
        Span::styled(
            crate::symbols::dot_off(),
            Style::default().fg(Color::DarkGray),
        )
    };

    let name_style = if snap.is_local_file {
        Style::default().fg(Color::Rgb(255, 165, 0))
    } else {
        Style::default().fg(Color::White).bold()
    };

    let name = Span::styled(snap.name.clone(), name_style);

    let version = if let Some(v) = &snap.version {
        Span::styled(
            format!(" {v}"),
            Style::default().fg(Color::DarkGray).italic(),
        )
    } else {
        Span::raw("")
    };

    ListItem::new(Line::from(vec![installed_marker, name, version]))
}
