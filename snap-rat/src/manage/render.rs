use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap},
};
use snapd_rs::Change;

use crate::app::{App, ManageAction, RightPane};
use crate::components::component_list_item;
use crate::connections::connection_list_item;
use crate::layout::{
    change_status_color, change_status_label, format_progress, format_size, progress_bar,
};
use crate::services::service_list_item;

pub(crate) fn render_manage(frame: &mut Frame, app: &mut App, area: Rect) {
    // Clear the entire area first to prevent stale terminal cells showing through
    // when the layout changes (e.g. progress bar appearing/disappearing).
    frame.render_widget(Clear, area);

    let snap = match app.selected_snap() {
        Some(s) => s,
        None => return,
    };
    let connection_items = app.connection_items();

    let progress_height = app
        .active_change
        .as_ref()
        .map(|change| (change.tasks.len() as u16).saturating_add(4).min(10))
        .unwrap_or(0);
    let mut constraints = vec![Constraint::Length(6), Constraint::Min(0)];
    if progress_height > 0 {
        constraints.push(Constraint::Length(progress_height));
    }
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let header_block = Block::default()
        .title(" Manage ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow))
        .padding(Padding::horizontal(1));

    let version_line = if let Some(v) = &snap.version {
        Line::from(vec![
            Span::styled("version  ", Style::default().fg(Color::DarkGray)),
            Span::raw(v.clone()),
        ])
    } else {
        Line::raw("")
    };

    let mut header_lines = vec![
        Line::from(Span::styled(
            snap.title.as_deref().unwrap_or(&snap.name).to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("snap     ", Style::default().fg(Color::DarkGray)),
            Span::styled(snap.name.clone(), Style::default().fg(Color::Cyan)),
        ]),
        version_line,
    ];
    if let Some(size) = snap.size {
        header_lines.push(Line::from(vec![
            Span::styled("size     ", Style::default().fg(Color::DarkGray)),
            Span::raw(format_size(size)),
        ]));
    }

    frame.render_widget(Paragraph::new(header_lines).block(header_block), layout[0]);

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[1]);

    let actions_focused = !app.right_pane_focused;
    let action_block = Block::default()
        .title(" Actions ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if actions_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let action_items: Vec<ListItem> = app
        .manage_actions
        .iter()
        .map(|a| {
            let style = match a {
                ManageAction::Uninstall | ManageAction::UninstallPurge => {
                    Style::default().fg(Color::Red)
                }
                ManageAction::Install
                | ManageAction::Refresh
                | ManageAction::SwitchChannel
                | ManageAction::InstallFromChannel => Style::default().fg(Color::Green),
                ManageAction::OpenStorePage | ManageAction::OpenContactPage => {
                    Style::default().fg(Color::Cyan)
                }
                ManageAction::OpenConnections
                | ManageAction::OpenComponents
                | ManageAction::OpenServices => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::White),
            };
            ListItem::new(Line::from(Span::styled(a.label(), style)))
        })
        .collect();

    let action_list = List::new(action_items)
        .block(action_block)
        .highlight_style(if actions_focused {
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default() // ghost: arrow position visible, no background
        })
        .highlight_symbol(if actions_focused { "▶ " } else { "▷ " });
    frame.render_stateful_widget(action_list, panes[0], &mut app.manage_state);
    app.manage_actions_area = Some(panes[0]);

    // Right pane: title and focus state
    let right_pane_title = match app.active_right_pane {
        RightPane::None => " Details ",
        RightPane::Connections => " Connections ",
        RightPane::Components => " Components ",
        RightPane::Services => " Services ",
    };

    let right_pane_block = Block::default()
        .title(right_pane_title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if app.right_pane_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .padding(Padding::horizontal(1));

    let right_inner = right_pane_block.inner(panes[1]);
    frame.render_widget(right_pane_block, panes[1]);

    match app.active_right_pane {
        RightPane::None => {
            app.connections_inner_area = None;
            app.components_inner_area = None;
            app.services_inner_area = None;
            frame.render_widget(
                ratatui::widgets::Paragraph::new(
                    "Select Connections →, Components →,\nor Services → from the actions list",
                )
                .style(
                    ratatui::style::Style::default()
                        .fg(Color::DarkGray)
                        .italic(),
                ),
                right_inner,
            );
        }
        RightPane::Connections => {
            app.connections_inner_area = Some(right_inner);
            app.components_inner_area = None;
            app.services_inner_area = None;
            render_connections_content(frame, app, right_inner, &snap, &connection_items);
        }
        RightPane::Components => {
            app.connections_inner_area = None;
            app.components_inner_area = Some(right_inner);
            app.services_inner_area = None;
            render_components_content(frame, app, right_inner, &snap);
        }
        RightPane::Services => {
            app.connections_inner_area = None;
            app.components_inner_area = None;
            app.services_inner_area = Some(right_inner);
            render_services_content(frame, app, right_inner, &snap);
        }
    }

    if let Some(change) = &app.active_change {
        render_change_progress(frame, change, layout[2]);
    }
}

fn render_connections_content(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    snap: &crate::types::DisplaySnap,
    connection_items: &[crate::app::ConnectionItem],
) {
    if !snap.installed {
        frame.render_widget(
            Paragraph::new("Install this snap to inspect connections")
                .style(Style::default().fg(Color::DarkGray).italic()),
            area,
        );
    } else if app.interfaces_loading {
        frame.render_widget(
            Paragraph::new("Loading…").style(Style::default().fg(Color::DarkGray).italic()),
            area,
        );
    } else if connection_items.is_empty() {
        frame.render_widget(
            Paragraph::new("No connections").style(Style::default().fg(Color::DarkGray).italic()),
            area,
        );
    } else {
        let items: Vec<ListItem> = connection_items.iter().map(connection_list_item).collect();
        let list = List::new(items)
            .highlight_style(if app.right_pane_focused {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            })
            .highlight_symbol(if app.right_pane_focused {
                "▶ "
            } else {
                "▷ "
            });
        frame.render_stateful_widget(list, area, &mut app.connections_state);
    }
}

fn render_components_content(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    snap: &crate::types::DisplaySnap,
) {
    if !snap.installed {
        frame.render_widget(
            Paragraph::new("Install this snap to manage components")
                .style(Style::default().fg(Color::DarkGray).italic()),
            area,
        );
    } else if app.components_loading {
        frame.render_widget(
            Paragraph::new("Loading…").style(Style::default().fg(Color::DarkGray).italic()),
            area,
        );
    } else if app.snap_components.is_empty() {
        frame.render_widget(
            Paragraph::new("No components").style(Style::default().fg(Color::DarkGray).italic()),
            area,
        );
    } else {
        let components = app.snap_components.clone();
        let items: Vec<ListItem> = components.iter().map(component_list_item).collect();
        let list = List::new(items)
            .highlight_style(if app.right_pane_focused {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            })
            .highlight_symbol(if app.right_pane_focused {
                "▶ "
            } else {
                "▷ "
            });
        frame.render_stateful_widget(list, area, &mut app.components_state);
    }
}

fn render_services_content(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    snap: &crate::types::DisplaySnap,
) {
    if !snap.installed {
        frame.render_widget(
            Paragraph::new("Install this snap to manage services")
                .style(Style::default().fg(Color::DarkGray).italic()),
            area,
        );
    } else if app.services_loading {
        frame.render_widget(
            Paragraph::new("Loading…").style(Style::default().fg(Color::DarkGray).italic()),
            area,
        );
    } else if app.snap_services.is_empty() {
        frame.render_widget(
            Paragraph::new("No services").style(Style::default().fg(Color::DarkGray).italic()),
            area,
        );
    } else {
        let services = app.snap_services.clone();
        let items: Vec<ListItem> = services.iter().map(service_list_item).collect();
        let list = List::new(items)
            .highlight_style(if app.right_pane_focused {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            })
            .highlight_symbol(if app.right_pane_focused {
                "▶ "
            } else {
                "▷ "
            });
        frame.render_stateful_widget(list, area, &mut app.services_state);
    }
}

pub(crate) fn render_change_progress(frame: &mut Frame, change: &Change, area: Rect) {
    let block = Block::default()
        .title(" Progress ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = vec![Line::from(vec![
        Span::styled(change.kind.clone(), Style::default().fg(Color::Cyan).bold()),
        Span::raw("  "),
        Span::raw(change.summary.clone()),
    ])];

    let task_limit = inner.height.saturating_sub(1) as usize;
    for task in change.tasks.iter().take(task_limit) {
        lines.push(Line::from(vec![
            Span::styled(
                format!(
                    "{} {} ",
                    progress_bar(task.progress.done, task.progress.total, 8),
                    format_progress(task.progress.done, task.progress.total),
                ),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!("{} ", change_status_label(&task.status)),
                Style::default().fg(change_status_color(&task.status)),
            ),
            Span::raw(task.summary.clone()),
        ]));
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}
