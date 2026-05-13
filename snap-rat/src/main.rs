use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap},
};
use ratatui_image::{Resize, StatefulImage, picker::Picker, protocol::StatefulProtocol};
use snapd_rs::{Change, ChangeStatus, SnapConfinement, api::snaps::Snap, api::store::StoreSnap};

mod app;
use app::{App, AppMode, ConnectionItem, ManageAction};

/// Re-exec the current process under `sudo` if we are not already root.
///
/// Uses `exec()` to replace the process entirely so that sudo's password
/// prompt appears on the terminal before ratatui initialises. pkexec is
/// intentionally avoided: it creates a new session without a controlling
/// terminal, which breaks ratatui regardless of whether a display is present.
///
/// This function only returns if we are already root (uid 0).
fn maybe_elevate() {
    if unsafe { libc::getuid() } == 0 {
        return;
    }

    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("snap-rat: cannot determine executable path: {e}");
            std::process::exit(1);
        }
    };

    use std::os::unix::process::CommandExt;
    let err = std::process::Command::new("sudo")
        .arg(&exe)
        .args(std::env::args_os().skip(1))
        .exec();

    eprintln!("snap-rat: failed to exec sudo: {err}");
    std::process::exit(1);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    maybe_elevate();
    let terminal = ratatui::init();
    let result = run(terminal).await;
    ratatui::restore();
    result
}

async fn run(mut terminal: DefaultTerminal) -> anyhow::Result<()> {
    let mut app = App::new();
    app.load_installed().await;
    let mut last_esc: Option<Instant> = None;

    loop {
        terminal.draw(|frame| ui(frame, &mut app))?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            if key.kind != KeyEventKind::Press {
                app.tick().await;
                continue;
            }
            // Ctrl-C always quits
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                break;
            }

            // Double-tap Esc quits (within 400 ms)
            if key.code == KeyCode::Esc && !app.search_focused && app.mode == AppMode::Browse {
                if last_esc.is_some_and(|t| t.elapsed() < Duration::from_millis(400)) {
                    break;
                }
                last_esc = Some(Instant::now());
            } else {
                last_esc = None;
            }

            match app.mode {
                AppMode::Browse => match key.code {
                    KeyCode::Char('q') if !app.search_focused => break,
                    KeyCode::Tab => app.toggle_focus(),
                    KeyCode::Char('/') if !app.search_focused => {
                        app.search_focused = true;
                    }
                    KeyCode::Enter | KeyCode::Right if app.search_focused => {
                        app.search_focused = false;
                        app.perform_search().await;
                    }
                    KeyCode::Enter | KeyCode::Right if !app.search_focused => {
                        app.open_manage();
                        if let Some(snap) = app.selected_snap()
                            && snap.installed
                        {
                            let name = snap.name.clone();
                            app.load_snap_interfaces(&name).await;
                        }
                    }
                    KeyCode::Esc | KeyCode::Left if app.search_focused => {
                        app.search_focused = false;
                    }
                    KeyCode::Char(c) if app.search_focused => {
                        app.search_query.push(c);
                    }
                    KeyCode::Char('c') if !app.search_focused => {
                        app.mode = AppMode::Changes;
                        app.load_changes().await;
                    }
                    KeyCode::Char('p') if !app.search_focused => app.toggle_changes_sidebar(),
                    KeyCode::Backspace if app.search_focused => {
                        app.search_query.pop();
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.prev(),
                    KeyCode::PageDown => app.page_down(),
                    KeyCode::PageUp => app.page_up(),
                    KeyCode::Char('i') if !app.search_focused => app.toggle_installed_filter(),
                    KeyCode::Char('r') if !app.search_focused => app.reload().await,
                    KeyCode::Char('s') if !app.search_focused => app.cycle_sort(),
                    _ => {}
                },
                AppMode::Manage => match key.code {
                    KeyCode::Esc if app.connections_mode => app.close_connections_mode(),
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Left => app.close_manage(),
                    KeyCode::Tab => app.toggle_connections_mode(),
                    KeyCode::Enter | KeyCode::Right if app.connections_mode => {
                        app.activate_selected_connection().await;
                    }
                    KeyCode::Enter => app.execute_selected_action().await,
                    KeyCode::Down | KeyCode::Char('j') if app.connections_mode => {
                        app.connections_next()
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.manage_next(),
                    KeyCode::Up | KeyCode::Char('k') if app.connections_mode => {
                        app.connections_prev()
                    }
                    KeyCode::Up | KeyCode::Char('k') => app.manage_prev(),
                    KeyCode::PageDown if app.connections_mode => app.connections_page_down(),
                    KeyCode::PageDown => {
                        for _ in 0..10 {
                            app.manage_next();
                        }
                    }
                    KeyCode::PageUp if app.connections_mode => app.connections_page_up(),
                    KeyCode::PageUp => {
                        for _ in 0..10 {
                            app.manage_prev();
                        }
                    }
                    KeyCode::Char('p') => app.toggle_changes_sidebar(),
                    KeyCode::Char('r') => {
                        app.close_manage();
                        app.reload().await;
                    }
                    _ => {}
                },
                AppMode::ChannelPicker => match key.code {
                    KeyCode::Esc | KeyCode::Left => app.close_channel_picker(),
                    KeyCode::Enter | KeyCode::Right => app.confirm_channel_pick().await,
                    KeyCode::Down | KeyCode::Char('j') => app.channel_picker_next(),
                    KeyCode::Up | KeyCode::Char('k') => app.channel_picker_prev(),
                    KeyCode::Char('n') => app.open_custom_channel_input(),
                    _ => {}
                },
                AppMode::ChannelInput => match key.code {
                    KeyCode::Esc | KeyCode::Left => app.close_channel_input(),
                    KeyCode::Enter | KeyCode::Right => app.execute_channel_action().await,
                    KeyCode::Char(c) => app.channel_input.push(c),
                    KeyCode::Backspace => {
                        app.channel_input.pop();
                    }
                    _ => {}
                },
                AppMode::ClassicConfirm => match key.code {
                    KeyCode::Esc | KeyCode::Left | KeyCode::Char('n') | KeyCode::Char('q') => {
                        app.cancel_classic()
                    }
                    KeyCode::Enter | KeyCode::Right | KeyCode::Char('y') => {
                        app.confirm_classic().await
                    }
                    _ => {}
                },
                AppMode::SlotPicker => match key.code {
                    KeyCode::Esc | KeyCode::Left => app.close_slot_picker(),
                    KeyCode::Enter | KeyCode::Right => app.confirm_slot_pick().await,
                    KeyCode::Down | KeyCode::Char('j') => app.slot_picker_next(),
                    KeyCode::Up | KeyCode::Char('k') => app.slot_picker_prev(),
                    KeyCode::PageDown => app.slot_picker_next(),
                    KeyCode::PageUp => app.slot_picker_prev(),
                    _ => {}
                },
                AppMode::Changes => match key.code {
                    KeyCode::Char('c') | KeyCode::Esc | KeyCode::Left => app.mode = AppMode::Browse,
                    KeyCode::Tab => app.changes_focus_detail = !app.changes_focus_detail,
                    KeyCode::Down | KeyCode::Char('j') => {
                        if app.changes_focus_detail {
                            app.changes_detail_next();
                        } else {
                            app.changes_next();
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.changes_focus_detail {
                            app.changes_detail_prev();
                        } else {
                            app.changes_prev();
                        }
                    }
                    KeyCode::PageDown => {
                        for _ in 0..10 {
                            if app.changes_focus_detail {
                                app.changes_detail_next();
                            } else {
                                app.changes_next();
                            }
                        }
                    }
                    KeyCode::PageUp => {
                        for _ in 0..10 {
                            if app.changes_focus_detail {
                                app.changes_detail_prev();
                            } else {
                                app.changes_prev();
                            }
                        }
                    }
                    KeyCode::Char('a') => app.abort_selected_change().await,
                    KeyCode::Char('p') => app.toggle_changes_sidebar(),
                    KeyCode::Char('r') => app.load_changes().await,
                    _ => {}
                },
            }
        }

        app.tick().await;
    }

    Ok(())
}

fn ui(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    let show_wide_sidebar = app.show_changes_sidebar && outer[0].width >= 120;

    if app.mode == AppMode::Changes {
        let (list_area, detail_area, sidebar_area) =
            split_main_columns(outer[0], show_wide_sidebar);
        render_changes_screen(frame, app, list_area, detail_area);
        render_status_bar(frame, app, outer[1]);

        if let Some(sidebar_area) = sidebar_area {
            render_changes_sidebar(frame, app, sidebar_area);
        } else if app.show_changes_sidebar {
            let sidebar_area = sidebar_overlay_rect(outer[0]);
            frame.render_widget(Clear, sidebar_area);
            render_changes_sidebar(frame, app, sidebar_area);
        }
        return;
    }

    let (list_area, main_area, sidebar_area) = split_main_columns(outer[0], show_wide_sidebar);
    let list_pane = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(list_area);

    render_search(frame, app, list_pane[0]);
    render_list(frame, app, list_pane[1]);

    match app.mode {
        AppMode::Browse => render_detail(frame, app, main_area),
        AppMode::Manage
        | AppMode::ChannelPicker
        | AppMode::ChannelInput
        | AppMode::ClassicConfirm
        | AppMode::SlotPicker => render_manage(frame, app, main_area),
        AppMode::Changes => unreachable!(),
    }

    render_status_bar(frame, app, outer[1]);

    if let Some(sidebar_area) = sidebar_area {
        render_changes_sidebar(frame, app, sidebar_area);
    } else if app.show_changes_sidebar {
        let sidebar_area = sidebar_overlay_rect(outer[0]);
        frame.render_widget(Clear, sidebar_area);
        render_changes_sidebar(frame, app, sidebar_area);
    }

    if app.mode == AppMode::ChannelPicker {
        render_channel_picker(frame, app);
    }
    if app.mode == AppMode::ChannelInput {
        render_channel_input(frame, app);
    }
    if app.mode == AppMode::ClassicConfirm {
        render_classic_confirm(frame, app);
    }
    if app.mode == AppMode::SlotPicker {
        render_slot_picker(frame, app);
    }
}

fn split_main_columns(area: Rect, show_wide_sidebar: bool) -> (Rect, Rect, Option<Rect>) {
    let columns = if show_wide_sidebar {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(45),
                Constraint::Percentage(25),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area)
    };

    if show_wide_sidebar {
        (columns[0], columns[1], Some(columns[2]))
    } else {
        (columns[0], columns[1], None)
    }
}

fn sidebar_overlay_rect(area: Rect) -> Rect {
    let sidebar_offset = area.width.saturating_mul(30) / 100;
    Rect {
        x: area.x + sidebar_offset,
        y: area.y,
        width: area.width.saturating_sub(sidebar_offset),
        height: area.height,
    }
}

fn render_changes_screen(frame: &mut Frame, app: &mut App, list_area: Rect, detail_area: Rect) {
    let list_block = Block::default()
        .title(" Changes ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if app.changes_focus_detail {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Cyan)
        });

    let items: Vec<ListItem> = app
        .changes_list
        .iter()
        .map(|change| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("[{}] ", change_status_label(&change.status)),
                    Style::default().fg(change_status_color(&change.status)),
                ),
                Span::styled(
                    change.kind.clone(),
                    Style::default().fg(Color::White).bold(),
                ),
                Span::raw(" — "),
                Span::raw(change.summary.clone()),
                Span::styled(
                    format!(" ({})", change.spawn_time.as_deref().unwrap_or("unknown")),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(list_block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, list_area, &mut app.changes_list_state);

    let detail_block = Block::default()
        .title(" Change Details ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if app.changes_focus_detail {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .padding(Padding::uniform(1));
    let detail_inner = detail_block.inner(detail_area);
    frame.render_widget(detail_block, detail_area);

    let Some(change) = app.selected_change().cloned() else {
        frame.render_widget(
            Paragraph::new("No changes loaded")
                .style(Style::default().fg(Color::DarkGray).italic()),
            detail_inner,
        );
        return;
    };

    let mut detail_constraints = vec![Constraint::Length(6), Constraint::Min(0)];
    if change.err.is_some() {
        detail_constraints.push(Constraint::Length(2));
    }
    let detail_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(detail_constraints)
        .split(detail_inner);

    let header = vec![
        Line::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::DarkGray)),
            Span::raw(change.id.clone()),
        ]),
        Line::from(vec![
            Span::styled("Kind: ", Style::default().fg(Color::DarkGray)),
            Span::raw(change.kind.clone()),
            Span::raw("   "),
            Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                change_status_label(&change.status),
                Style::default().fg(change_status_color(&change.status)),
            ),
        ]),
        Line::from(vec![
            Span::styled("Summary: ", Style::default().fg(Color::DarkGray)),
            Span::raw(change.summary.clone()),
        ]),
        Line::from(vec![
            Span::styled("Spawned: ", Style::default().fg(Color::DarkGray)),
            Span::raw(
                change
                    .spawn_time
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
            ),
        ]),
        Line::from(vec![
            Span::styled("Ready: ", Style::default().fg(Color::DarkGray)),
            Span::raw(
                change
                    .ready_time
                    .clone()
                    .unwrap_or_else(|| "pending".to_string()),
            ),
        ]),
    ];
    frame.render_widget(Paragraph::new(header), detail_layout[0]);

    let task_items: Vec<ListItem> = if change.tasks.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "No tasks",
            Style::default().fg(Color::DarkGray).italic(),
        )))]
    } else {
        change
            .tasks
            .iter()
            .map(|task| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!(
                            "{} {} ",
                            progress_bar(task.progress.done, task.progress.total, 6),
                            format_progress(task.progress.done, task.progress.total),
                        ),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(
                        format!("{} ", change_status_label(&task.status)),
                        Style::default().fg(change_status_color(&task.status)),
                    ),
                    Span::raw(task.summary.clone()),
                ]))
            })
            .collect()
    };

    let tasks = List::new(task_items)
        .block(Block::default().title(" Tasks ").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(tasks, detail_layout[1], &mut app.changes_detail_state);

    if let Some(err) = &change.err {
        frame.render_widget(
            Paragraph::new(err.clone()).style(Style::default().fg(Color::Red)),
            detail_layout[2],
        );
    }
}

fn render_search(frame: &mut Frame, app: &App, area: Rect) {
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
        format!("{}█", app.search_query)
    } else if app.search_query.is_empty() {
        "Press / to search the store…".to_string()
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

fn render_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let list_focused = !app.search_focused;

    let sort_label = app.sort_mode.label();
    let title = if app.showing_results {
        if app.show_installed_only {
            format!(
                " Installed from \"{}\" ({}) [{}] ",
                app.search_query,
                app.display_snaps().len(),
                sort_label
            )
        } else {
            format!(
                " Results for \"{}\" ({}) [{}] ",
                app.search_query,
                app.store_results.len(),
                sort_label
            )
        }
    } else {
        format!(
            " Installed Snaps ({}) [{}] ",
            app.installed.len(),
            sort_label
        )
    };

    let border_style = if list_focused {
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
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn snap_list_item(snap: &DisplaySnap) -> ListItem<'static> {
    let installed_marker = if snap.installed {
        Span::styled("● ", Style::default().fg(Color::Green))
    } else {
        Span::styled("○ ", Style::default().fg(Color::DarkGray))
    };

    let name = Span::styled(snap.name.clone(), Style::default().fg(Color::White).bold());

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

fn render_detail(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Snap Details ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::uniform(1));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(snap) = app.selected_snap() else {
        let placeholder = Paragraph::new("Select a snap to see details")
            .style(Style::default().fg(Color::DarkGray).italic());
        frame.render_widget(placeholder, inner);
        return;
    };

    // Layout: header + body
    let detail_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .split(inner);

    // Header: name + version + publisher
    let mut header_lines = vec![
        Line::from(vec![Span::styled(
            snap.title.clone().unwrap_or_else(|| snap.name.clone()),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("Name:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(snap.name.clone(), Style::default().fg(Color::Cyan)),
        ]),
    ];

    if let Some(v) = &snap.version {
        header_lines.push(Line::from(vec![
            Span::styled("Version: ", Style::default().fg(Color::DarkGray)),
            Span::raw(v.clone()),
        ]));
    }

    if let Some(pub_) = &snap.publisher {
        header_lines.push(Line::from(vec![
            Span::styled("By:      ", Style::default().fg(Color::DarkGray)),
            Span::raw(pub_.clone()),
        ]));
    }

    if let Some(size) = snap.size {
        header_lines.push(Line::from(vec![
            Span::styled("Size:    ", Style::default().fg(Color::DarkGray)),
            Span::raw(format_size(size)),
        ]));
    }

    frame.render_widget(Paragraph::new(header_lines), detail_layout[0]);

    // Body: summary + description + metadata
    let mut body_lines: Vec<Line> = vec![];

    if let Some(summary) = &snap.summary {
        body_lines.push(Line::from(Span::styled(
            summary.clone(),
            Style::default().fg(Color::Yellow),
        )));
        body_lines.push(Line::raw(""));
    }

    if let Some(desc) = &snap.description {
        for line in desc.lines().take(20) {
            body_lines.push(Line::raw(line.to_string()));
        }
        body_lines.push(Line::raw(""));
    }

    // Metadata badges
    let mut badges: Vec<Span> = vec![];

    if snap.installed {
        badges.push(Span::styled(
            " installed ",
            Style::default().bg(Color::Green).fg(Color::Black),
        ));
        badges.push(Span::raw(" "));
    }

    if let Some(conf) = &snap.confinement {
        let (label, color) = match conf {
            SnapConfinement::Strict => ("strict", Color::Blue),
            SnapConfinement::Classic => ("classic", Color::Magenta),
            SnapConfinement::Devmode => ("devmode", Color::Red),
            _ => ("unknown", Color::DarkGray),
        };
        badges.push(Span::styled(
            format!(" {label} "),
            Style::default().bg(color).fg(Color::White),
        ));
        badges.push(Span::raw(" "));
    }

    if let Some(channel) = &snap.channel {
        badges.push(Span::styled(
            format!(" {channel} "),
            Style::default().bg(Color::DarkGray).fg(Color::White),
        ));
    }

    if !badges.is_empty() {
        body_lines.push(Line::from(badges));
    }

    let body = Paragraph::new(Text::from(body_lines)).wrap(Wrap { trim: false });
    frame.render_widget(body, detail_layout[1]);

    if let Some(icon_url) = &snap.icon_url
        && let Some(picker) = &app.icon_picker
        && let Some(Some(image)) = app.icon_cache.get(icon_url)
    {
        render_snap_icon(frame, picker, image, inner);
    }
}

fn render_snap_icon(frame: &mut Frame, picker: &Picker, image: &image::DynamicImage, area: Rect) {
    let icon_width = area.width.min(16);
    let icon_height = area.height.min(8);
    if icon_width < 8 || icon_height < 4 {
        return;
    }

    let icon_area = Rect {
        x: area.x + area.width.saturating_sub(icon_width),
        y: area.y,
        width: icon_width,
        height: icon_height,
    };
    let mut protocol = picker.new_resize_protocol(image.clone());
    let image_widget = StatefulImage::<StatefulProtocol>::default().resize(Resize::Fit(None));
    frame.render_stateful_widget(image_widget, icon_area, &mut protocol);
}

fn render_changes_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let mut changes: Vec<&Change> = Vec::new();
    if let Some(active_change) = &app.active_change {
        changes.push(active_change);
    }
    for change in &app.sidebar_changes {
        if changes.iter().any(|existing| existing.id == change.id) {
            continue;
        }
        changes.push(change);
    }

    let title = if changes.is_empty() {
        " Changes "
    } else {
        " ● Active Changes "
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if changes.is_empty() {
        frame.render_widget(
            Paragraph::new("No active changes")
                .style(Style::default().fg(Color::DarkGray).italic()),
            inner,
        );
        return;
    }

    let summary_width = inner.width.saturating_sub(4) as usize;
    let mut lines = Vec::new();
    for (index, change) in changes.iter().enumerate() {
        let status_color = change_status_color(&change.status);
        let prefix = if app
            .active_change
            .as_ref()
            .map(|active| active.id == change.id)
            .unwrap_or(false)
        {
            "● "
        } else {
            ""
        };
        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("[{}] ", change_status_label(&change.status)),
                Style::default().fg(status_color),
            ),
            Span::styled(
                change.kind.clone(),
                Style::default().fg(Color::White).bold(),
            ),
        ]));
        lines.push(Line::from(Span::raw(truncate_text(
            &change.summary,
            summary_width.max(12),
        ))));

        if !change.tasks.is_empty() {
            let done = change
                .tasks
                .iter()
                .filter(|task| task.status == ChangeStatus::Done)
                .count() as i64;
            let total = change.tasks.len() as i64;
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{} ", progress_bar(done, total, 5)),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!("{done}/{total}"),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }

        if index + 1 < changes.len() {
            lines.push(Line::raw(""));
        }
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    let visible = max_chars.saturating_sub(1).max(1);
    let mut truncated = text.chars().take(visible).collect::<String>();
    truncated.push('…');
    truncated
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let style = Style::default().bg(Color::DarkGray).fg(Color::White);

    let help = match app.mode {
        AppMode::Browse if app.search_focused => " Enter  confirm   Esc  cancel ",
        AppMode::Browse => {
            " /  search   ↑↓jk  navigate   Enter  manage   c  changes   p  changes sidebar   i  installed only   r  refresh   s  sort   q/Esc×2  quit "
        }
        AppMode::Manage if app.connections_mode => {
            " ↑↓jk  navigate   Enter  connect/disconnect   Tab  actions   p  changes sidebar   Esc  back "
        }
        AppMode::Manage => {
            " ↑↓jk  navigate   Enter  select   Tab  connections   p  changes sidebar   Esc  back "
        }
        AppMode::ChannelPicker => {
            " ↑↓jk  navigate   Enter  select   n  custom channel   Esc  back "
        }
        AppMode::ChannelInput => " Type channel name   Enter  confirm   Esc  cancel ",
        AppMode::ClassicConfirm => " y/Enter  install (classic)   n/Esc  cancel ",
        AppMode::SlotPicker => " ↑↓jk  navigate   Enter  connect   Esc  cancel ",
        AppMode::Changes => {
            " ↑↓  navigate   Tab  switch pane   a  abort   p  changes sidebar   r  refresh   c/Esc  back "
        }
    };

    let indicator = if app.loading {
        Span::styled(
            " Loading… ",
            Style::default().bg(Color::Yellow).fg(Color::Black),
        )
    } else if let Some(err) = &app.error {
        Span::styled(
            format!(" ✗ {err} "),
            Style::default().bg(Color::Red).fg(Color::White),
        )
    } else if app.active_change_id.is_some() {
        let message = app
            .status_message
            .as_deref()
            .or_else(|| {
                app.active_change
                    .as_ref()
                    .map(|change| change.summary.as_str())
            })
            .unwrap_or("Working…");
        Span::styled(
            format!(" ⟳ {message} "),
            Style::default().bg(Color::Blue).fg(Color::White),
        )
    } else if let Some(msg) = &app.status_message {
        Span::styled(
            format!(" ✓ {msg} "),
            Style::default().bg(Color::Green).fg(Color::Black),
        )
    } else {
        Span::raw("")
    };

    let indicator_width = indicator.content.chars().count() as u16;
    let help_width = area.width.saturating_sub(indicator_width) as usize;
    let help_truncated = truncate_text(help, help_width);

    let bar = Paragraph::new(Line::from(vec![
        Span::styled(help_truncated, style),
        indicator,
    ]));
    frame.render_widget(bar, area);
}

fn render_manage(frame: &mut Frame, app: &mut App, area: Rect) {
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

    let actions_focused = !app.connections_mode;
    let action_block = Block::default()
        .title(" Actions [Tab] connections ")
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
                ManageAction::Uninstall => Style::default().fg(Color::Red),
                ManageAction::Install
                | ManageAction::Refresh
                | ManageAction::SwitchChannel
                | ManageAction::InstallFromChannel => Style::default().fg(Color::Green),
                ManageAction::OpenStorePage | ManageAction::OpenContactPage => {
                    Style::default().fg(Color::Cyan)
                }
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
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(action_list, panes[0], &mut app.manage_state);

    let connection_block = Block::default()
        .title(" Connections [Tab] actions ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if app.connections_mode {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .padding(Padding::horizontal(1));
    let connection_inner = connection_block.inner(panes[1]);
    frame.render_widget(connection_block, panes[1]);

    if !snap.installed {
        frame.render_widget(
            Paragraph::new("Install this snap to inspect connections")
                .style(Style::default().fg(Color::DarkGray).italic()),
            connection_inner,
        );
    } else if app.interfaces_loading {
        frame.render_widget(
            Paragraph::new("Loading…").style(Style::default().fg(Color::DarkGray).italic()),
            connection_inner,
        );
    } else if connection_items.is_empty() {
        frame.render_widget(
            Paragraph::new("No connections").style(Style::default().fg(Color::DarkGray).italic()),
            connection_inner,
        );
    } else {
        let items: Vec<ListItem> = connection_items.iter().map(connection_list_item).collect();
        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(list, connection_inner, &mut app.connections_state);
    }

    if let Some(change) = &app.active_change {
        render_change_progress(frame, change, layout[2]);
    }
}

fn connection_list_item(item: &ConnectionItem) -> ListItem<'static> {
    let marker = if item.connected {
        Span::styled("● ", Style::default().fg(Color::Green))
    } else {
        Span::styled("○ ", Style::default().fg(Color::DarkGray))
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
            format!(" · {name}"),
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
            format!("  → {peer}"),
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

fn render_change_progress(frame: &mut Frame, change: &Change, area: Rect) {
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

fn render_channel_picker(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let popup = centered_popup_percent(70, 60, area);
    frame.render_widget(Clear, popup);

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

fn render_slot_picker(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let popup = centered_popup_percent(65, 55, area);
    frame.render_widget(Clear, popup);

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
                .title(format!(" Connect '{interface_name}' to… "))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
                .padding(Padding::horizontal(1)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, popup, &mut app.slot_picker_state);
}

fn render_channel_input(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let popup = centered_popup(50, 3, area);
    frame.render_widget(Clear, popup);

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

fn format_size(bytes: u64) -> String {
    match bytes {
        0 => "0 B".to_string(),
        b if b < 1024 => format!("{b} B"),
        b if b < 1024 * 1024 => format!("{:.1} KB", b as f64 / 1024.0),
        b if b < 1024 * 1024 * 1024 => {
            format!("{:.1} MB", b as f64 / (1024.0 * 1024.0))
        }
        b => format!("{:.2} GB", b as f64 / (1024.0 * 1024.0 * 1024.0)),
    }
}

/// Format task progress as human-readable.
/// When both values are >= 1 KB we treat them as byte counts; otherwise show
/// plain integers (e.g. "1/3" for a multi-step task).
fn format_progress(done: i64, total: i64) -> String {
    if total >= 1024 || done >= 1024 {
        let done_s = format_size(done.max(0) as u64);
        let total_s = format_size(total.max(0) as u64);
        format!("{done_s} / {total_s}")
    } else {
        format!("{done}/{total}")
    }
}

fn render_classic_confirm(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let popup = centered_popup(60, 9, area);
    frame.render_widget(Clear, popup);

    let snap_name = app
        .classic_pending
        .as_ref()
        .map(|(n, _)| n.as_str())
        .unwrap_or("this snap");

    let block = Block::default()
        .title(" ⚠  Classic Confinement ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow))
        .padding(Padding::uniform(1));

    let text = Text::from(vec![
        Line::from(vec![
            Span::styled(snap_name, Style::default().fg(Color::Cyan).bold()),
            Span::raw(" uses "),
            Span::styled(
                "classic confinement",
                Style::default().fg(Color::Yellow).bold(),
            ),
            Span::raw("."),
        ]),
        Line::raw(""),
        Line::raw("Classic snaps run without any sandboxing and have"),
        Line::raw("full access to your system, like a traditional app."),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  y / Enter  ", Style::default().fg(Color::Green).bold()),
            Span::raw("Install anyway"),
            Span::styled("     n / Esc  ", Style::default().fg(Color::Red).bold()),
            Span::raw("Cancel"),
        ]),
    ]);

    frame.render_widget(Paragraph::new(text).block(block), popup);
}

fn progress_bar(done: i64, total: i64, width: usize) -> String {
    let total = total.max(0);
    let done = done.clamp(0, total.max(done));
    let filled = if total > 0 {
        ((done as usize) * width + (total as usize / 2)) / total as usize
    } else {
        0
    }
    .min(width);

    format!("[{}{}]", "█".repeat(filled), "░".repeat(width - filled))
}

fn change_status_label(status: &ChangeStatus) -> &'static str {
    match status {
        ChangeStatus::Do => "Do",
        ChangeStatus::Doing => "Doing",
        ChangeStatus::Done => "Done",
        ChangeStatus::Abort => "Abort",
        ChangeStatus::Aborting => "Aborting",
        ChangeStatus::Error => "Error",
        ChangeStatus::Hold => "Hold",
        ChangeStatus::Wait => "Wait",
        ChangeStatus::Undone => "Undone",
        ChangeStatus::Undoing => "Undoing",
        _ => "Unknown",
    }
}

fn change_status_color(status: &ChangeStatus) -> Color {
    match status {
        ChangeStatus::Doing | ChangeStatus::Wait => Color::Yellow,
        ChangeStatus::Done => Color::Green,
        ChangeStatus::Error | ChangeStatus::Abort | ChangeStatus::Aborting => Color::Red,
        ChangeStatus::Hold => Color::DarkGray,
        _ => Color::White,
    }
}

fn confinement_label(confinement: &SnapConfinement) -> &'static str {
    match confinement {
        SnapConfinement::Strict => "strict",
        SnapConfinement::Classic => "classic",
        SnapConfinement::Devmode => "devmode",
        _ => "unknown",
    }
}

fn centered_popup(percent_x: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height + 2),
            Constraint::Fill(1),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn centered_popup_percent(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

/// Unified view of a snap (installed or from store).
#[derive(Debug, Clone)]
pub struct DisplaySnap {
    pub name: String,
    pub title: Option<String>,
    pub version: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub confinement: Option<SnapConfinement>,
    pub channel: Option<String>,
    pub contact: Option<String>,
    pub icon_url: Option<String>,
    pub size: Option<u64>,
    pub installed: bool,
}

impl From<&Snap> for DisplaySnap {
    fn from(s: &Snap) -> Self {
        Self {
            name: s.name.clone(),
            title: s.title.clone(),
            version: s.version.clone(),
            summary: s.summary.clone(),
            description: s.description.clone(),
            publisher: s
                .publisher
                .as_ref()
                .and_then(|p| p.display_name.clone().or_else(|| p.username.clone())),
            confinement: s.confinement.clone(),
            channel: s.tracking_channel.clone().or_else(|| s.channel.clone()),
            contact: s.contact.clone(),
            icon_url: s.icon.clone(),
            size: s.installed_size,
            installed: true,
        }
    }
}

impl From<&StoreSnap> for DisplaySnap {
    fn from(s: &StoreSnap) -> Self {
        Self {
            name: s.name.clone(),
            title: s.title.clone(),
            version: s.version.clone(),
            summary: s.summary.clone(),
            description: s.description.clone(),
            publisher: s
                .publisher
                .as_ref()
                .and_then(|p| p.display_name.clone().or_else(|| p.username.clone())),
            confinement: None,
            channel: s.channel.clone(),
            contact: None,
            icon_url: s.icon.clone(),
            size: s.download_size.map(|value| value.max(0) as u64),
            installed: false,
        }
    }
}
