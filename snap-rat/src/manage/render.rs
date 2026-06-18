use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap},
};
use snapd_rs::Change;

use crate::app::{App, AppMode, ManageAction, RightPane};
use crate::channels::{render_channel_input_in, render_channel_picker_in};
use crate::components::component_list_item;
use crate::connections::connection_list_item;
use crate::layout::{
    change_status_color, change_status_label, format_progress, format_size, progress_bar,
};
use crate::services::service_list_item;

pub(crate) fn render_manage(frame: &mut Frame, app: &mut App, area: Rect) {
    frame.render_widget(Clear, area);

    let snap = match app.selected_snap() {
        Some(s) => s,
        None => return,
    };

    let progress_height = app
        .active_change
        .as_ref()
        .map(|change| (change.tasks.len() as u16).saturating_add(4).min(10))
        .unwrap_or(0);

    // Always render: header (6 rows) + body (actions list or sub-pane) + optional progress.
    let mut constraints = vec![Constraint::Length(6), Constraint::Min(0)];
    if progress_height > 0 {
        constraints.push(Constraint::Length(progress_height));
    }
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // --- Header (always visible) ---
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

    // --- Body: sub-pane, channel overlay, or actions list ---
    if app.active_right_pane != RightPane::None {
        let popup_area = actions_popup_area(frame, app, layout[1]);

        let pane_title = match app.active_right_pane {
            RightPane::None => unreachable!(),
            RightPane::Connections => " Connections  [Esc] back ",
            RightPane::Components => " Components  [Esc] back ",
            RightPane::Services => " Services  [Esc] back ",
        };

        let pane_block = Block::default()
            .title(pane_title)
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Cyan))
            .padding(Padding::horizontal(1));
        let inner = pane_block.inner(popup_area);
        frame.render_widget(Clear, popup_area);
        frame.render_widget(pane_block, popup_area);

        app.manage_actions_area = None;
        let connection_items = app.connection_items();
        match app.active_right_pane {
            RightPane::None => unreachable!(),
            RightPane::Connections => {
                app.connections_inner_area = Some(inner);
                app.components_inner_area = None;
                app.services_inner_area = None;
                render_connections_content(frame, app, inner, &snap, &connection_items);
            }
            RightPane::Components => {
                app.connections_inner_area = None;
                app.components_inner_area = Some(inner);
                app.services_inner_area = None;
                render_components_content(frame, app, inner, &snap);
            }
            RightPane::Services => {
                app.connections_inner_area = None;
                app.components_inner_area = None;
                app.services_inner_area = Some(inner);
                render_services_content(frame, app, inner, &snap);
            }
        }
    } else if app.mode == AppMode::ChannelPicker {
        let popup_area = actions_popup_area(frame, app, layout[1]);
        app.manage_actions_area = None;
        app.connections_inner_area = None;
        app.components_inner_area = None;
        app.services_inner_area = None;
        render_channel_picker_in(frame, app, popup_area);
    } else if app.mode == AppMode::ChannelInput {
        let popup_area = actions_popup_area(frame, app, layout[1]);
        app.manage_actions_area = None;
        app.connections_inner_area = None;
        app.components_inner_area = None;
        app.services_inner_area = None;
        render_channel_input_in(frame, app, popup_area);
    } else {
        app.connections_inner_area = None;
        app.components_inner_area = None;
        app.services_inner_area = None;

        let action_block = Block::default()
            .title(" Actions ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan));

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
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(crate::symbols::play());
        frame.render_stateful_widget(action_list, layout[1], &mut app.manage_state);
        app.manage_actions_area = Some(layout[1]);
    }

    if let Some(change) = &app.active_change {
        render_change_progress(frame, change, layout[2]);
    }
}

/// Draws the dimmed "Actions" backdrop and returns the popup area inset by 1 cell.
fn actions_popup_area(frame: &mut Frame, _app: &mut App, slot: Rect) -> Rect {
    let backdrop = Block::default()
        .title(" Actions ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(backdrop, slot);
    Rect {
        x: slot.x + 1,
        y: slot.y + 1,
        width: slot.width.saturating_sub(2),
        height: slot.height.saturating_sub(2),
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
            Paragraph::new(format!("Loading{}", crate::symbols::ellipsis()))
                .style(Style::default().fg(Color::DarkGray).italic()),
            area,
        );
    } else if connection_items.is_empty() {
        frame.render_widget(
            Paragraph::new("No connections").style(Style::default().fg(Color::DarkGray).italic()),
            area,
        );
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);
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
                crate::symbols::play()
            } else {
                crate::symbols::play_hollow()
            });
        frame.render_stateful_widget(list, chunks[0], &mut app.connections_state);
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(crate::symbols::dot_on(), Style::default().fg(Color::Green)),
                Span::styled("connected  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    crate::symbols::dot_off(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled("disconnected", Style::default().fg(Color::DarkGray)),
            ])),
            chunks[1],
        );
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
            Paragraph::new(format!("Loading{}", crate::symbols::ellipsis()))
                .style(Style::default().fg(Color::DarkGray).italic()),
            area,
        );
    } else if app.snap_components.is_empty() {
        frame.render_widget(
            Paragraph::new("No components").style(Style::default().fg(Color::DarkGray).italic()),
            area,
        );
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);
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
                crate::symbols::play()
            } else {
                crate::symbols::play_hollow()
            });
        frame.render_stateful_widget(list, chunks[0], &mut app.components_state);
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(crate::symbols::dot_on(), Style::default().fg(Color::Green)),
                Span::styled("installed  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    crate::symbols::dot_off(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled("not installed", Style::default().fg(Color::DarkGray)),
            ])),
            chunks[1],
        );
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
            Paragraph::new(format!("Loading{}", crate::symbols::ellipsis()))
                .style(Style::default().fg(Color::DarkGray).italic()),
            area,
        );
    } else if app.snap_services.is_empty() {
        frame.render_widget(
            Paragraph::new("No services").style(Style::default().fg(Color::DarkGray).italic()),
            area,
        );
    } else if app.service_actions_open {
        render_service_action_menu(frame, app, area);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);

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
                crate::symbols::play()
            } else {
                crate::symbols::play_hollow()
            });
        frame.render_stateful_widget(list, chunks[0], &mut app.services_state);

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(crate::symbols::dot_on(), Style::default().fg(Color::Green)),
                Span::styled("running  ", Style::default().fg(Color::DarkGray)),
                Span::styled(crate::symbols::dot_on(), Style::default().fg(Color::Yellow)),
                Span::styled("no-boot  ", Style::default().fg(Color::DarkGray)),
                Span::styled(crate::symbols::dot_err(), Style::default().fg(Color::Red)),
                Span::styled("failed  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    crate::symbols::dot_off(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled("disabled", Style::default().fg(Color::DarkGray)),
            ])),
            chunks[1],
        );
    }
}

fn render_service_action_menu(frame: &mut Frame, app: &mut App, area: Rect) {
    use crate::app::ServiceAction;

    // Find the selected service for the header.
    let service_name = app
        .services_state
        .selected()
        .and_then(|i| app.snap_services.get(i))
        .map(|s| s.name.clone())
        .unwrap_or_default();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(area);

    // Header: service name + back hint.
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    if crate::symbols::is_unicode() {
                        "⚙ "
                    } else {
                        "* "
                    },
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(service_name, Style::default().fg(Color::Cyan).bold()),
            ]),
            Line::from(Span::styled(
                "[Esc] back",
                Style::default().fg(Color::DarkGray),
            )),
        ]),
        chunks[0],
    );

    let actions = app.service_actions.clone();
    let action_items: Vec<ListItem> = actions
        .iter()
        .map(|a| {
            let (symbol, color) = match a {
                ServiceAction::Start => (crate::symbols::play(), Color::Green),
                ServiceAction::Enable => (crate::symbols::play(), Color::Green),
                ServiceAction::Stop => (
                    if crate::symbols::is_unicode() {
                        "■ "
                    } else {
                        "[] "
                    },
                    Color::Red,
                ),
                ServiceAction::Disable => (
                    if crate::symbols::is_unicode() {
                        "■ "
                    } else {
                        "[] "
                    },
                    Color::Red,
                ),
                ServiceAction::Restart => (
                    if crate::symbols::is_unicode() {
                        "↺ "
                    } else {
                        "R "
                    },
                    Color::Cyan,
                ),
            };
            ListItem::new(Line::from(vec![
                Span::styled(symbol, Style::default().fg(color)),
                Span::styled(a.label(), Style::default().fg(Color::White)),
            ]))
        })
        .collect();

    let action_list = List::new(action_items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(crate::symbols::play());
    frame.render_stateful_widget(action_list, chunks[1], &mut app.service_actions_state);
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
