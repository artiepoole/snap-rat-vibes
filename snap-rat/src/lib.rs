use std::time::{Duration, Instant};

use crossterm::event::{self, Event};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Clear,
};

mod app;
mod browse;
mod changes;
mod channels;
mod components;
mod confirm;
mod connections;
mod detail;
mod help;
mod keyboard;
mod layout;
mod manage;
mod mouse;
pub mod resume;
mod services;
mod slots;
pub mod symbols;
mod types;
use app::{App, AppMode};

pub async fn run() -> anyhow::Result<()> {
    let resume_state = resume::parse_resume_arg();
    let no_unicode = std::env::args().any(|a| a == "--no-unicode");
    symbols::init(if no_unicode { false } else { symbols::detect() });
    let terminal = ratatui::init();
    crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)?;
    let result = run_loop(terminal, resume_state).await;
    crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture)?;
    ratatui::restore();
    result
}

async fn run_loop(
    mut terminal: DefaultTerminal,
    resume_state: Option<resume::ResumeState>,
) -> anyhow::Result<()> {
    let mut app = App::new();
    app.load_installed().await;

    if let Some(state) = resume_state {
        app.apply_resume(state).await;
    }

    // Clear the physical screen without touching ratatui's buffer state.
    // terminal.clear() resets both diff buffers which confuses ratatui-image's
    // sixel renderer on VTE (Tilix). A direct backend clear is enough to erase
    // any stale content from the re-exec path; ratatui's first draw handles the rest.
    use crossterm::execute;
    execute!(
        std::io::stdout(),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        crossterm::cursor::MoveTo(0, 0),
    )?;

    let mut last_esc: Option<Instant> = None;

    loop {
        terminal.draw(|frame| ui(frame, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if keyboard::handle(&mut app, key, &mut last_esc).await => {
                    break;
                }
                Event::Mouse(mouse) => {
                    mouse::handle(&mut app, mouse).await;
                }
                _ => {}
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
        let left_pane = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(list_area);
        browse::render_tabs(frame, app, left_pane[0]);
        changes::render_changes_screen(frame, app, left_pane[1], detail_area);

        if let Some(sidebar_area) = sidebar_area {
            changes::render_changes_sidebar(frame, app, sidebar_area);
        } else if app.show_changes_sidebar {
            let sidebar_area = sidebar_overlay_rect(outer[0]);
            frame.render_widget(Clear, sidebar_area);
            changes::render_changes_sidebar(frame, app, sidebar_area);
        }
        return;
    }

    let (list_area, main_area, sidebar_area) = split_main_columns(outer[0], show_wide_sidebar);
    app.left_pane_area = Some(list_area);
    let list_pane = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(list_area);

    browse::render_tabs(frame, app, list_pane[0]);
    browse::render_search(frame, app, list_pane[1]);
    browse::render_list(frame, app, list_pane[2]);

    match app.mode {
        AppMode::Browse => detail::render_detail(frame, app, main_area),
        AppMode::Manage
        | AppMode::ChannelPicker
        | AppMode::ChannelInput
        | AppMode::ClassicConfirm
        | AppMode::Confirm
        | AppMode::SlotPicker => manage::render_manage(frame, app, main_area),
        AppMode::Changes => unreachable!(),
    }

    detail::render_status_bar(frame, app, outer[1]);

    if let Some(sidebar_area) = sidebar_area {
        changes::render_changes_sidebar(frame, app, sidebar_area);
    } else if app.show_changes_sidebar {
        let sidebar_area = sidebar_overlay_rect(outer[0]);
        frame.render_widget(Clear, sidebar_area);
        changes::render_changes_sidebar(frame, app, sidebar_area);
    }

    if app.mode == AppMode::ClassicConfirm {
        confirm::render_classic_confirm(frame, app);
    }
    if app.mode == AppMode::SlotPicker {
        slots::render_slot_picker(frame, app);
    }
    if app.mode == AppMode::Confirm {
        confirm::render_confirm(frame, app);
    }
    // Help overlay renders on top of everything.
    if app.show_help {
        help::render_help(frame, app);
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
