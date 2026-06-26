use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Padding, Paragraph, Wrap},
};
use snapd_rs_artie::{Change, ChangeStatus};

use crate::app::App;
use crate::layout::{
    change_status_color, change_status_label, format_progress, progress_bar, truncate_text,
};

pub(crate) fn render_changes_screen(
    frame: &mut Frame,
    app: &mut App,
    list_area: Rect,
    detail_area: Rect,
) {
    app.changes_list_area = Some(list_area);
    app.changes_detail_area = Some(detail_area);
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
                Span::raw(" - "),
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
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(crate::symbols::play());
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
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
        )
        .highlight_symbol(crate::symbols::play());
    frame.render_stateful_widget(tasks, detail_layout[1], &mut app.changes_detail_state);

    if let Some(err) = &change.err {
        frame.render_widget(
            Paragraph::new(err.clone()).style(Style::default().fg(Color::Red)),
            detail_layout[2],
        );
    }
}

pub(crate) fn render_changes_sidebar(frame: &mut Frame, app: &App, area: Rect) {
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
        " Changes ".to_string()
    } else {
        format!(" {} Active Changes ", crate::symbols::dot())
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
            crate::symbols::dot_on()
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
