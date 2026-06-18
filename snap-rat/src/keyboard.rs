use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::{App, AppMode};

/// Handle a key event. Returns `true` if the application should quit.
pub(crate) async fn handle(app: &mut App, key: KeyEvent, last_esc: &mut Option<Instant>) -> bool {
    if key.kind != KeyEventKind::Press {
        app.tick().await;
        return false;
    }
    // Ctrl-C always quits
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return true;
    }

    // q always quits (except when typing in search or channel input)
    if key.code == KeyCode::Char('q')
        && !app.search_focused
        && !matches!(app.mode, AppMode::ChannelInput)
    {
        return true;
    }

    // ? or F1 toggles help from anywhere (except text input modes)
    if (key.code == KeyCode::Char('?') || key.code == KeyCode::F(1))
        && !app.search_focused
        && !matches!(app.mode, AppMode::ChannelInput)
    {
        app.show_help = !app.show_help;
        return false;
    }

    // s/c switch tabs from any non-popup context (not when typing or in overlays)
    let is_popup = app.search_focused
        || matches!(
            app.mode,
            AppMode::ChannelPicker
                | AppMode::ChannelInput
                | AppMode::SlotPicker
                | AppMode::Confirm
                | AppMode::ClassicConfirm
        );
    if !is_popup {
        if key.code == KeyCode::Char('c') && app.mode != AppMode::Changes {
            // Switch view without closing manage pane; Esc/Left still closes it properly.
            app.mode = AppMode::Changes;
            app.load_changes().await;
            return false;
        }
        if key.code == KeyCode::Char('s') && app.mode != AppMode::Browse {
            // Just switch view — don't call close_manage(), preserving snap selection.
            app.mode = AppMode::Browse;
            return false;
        }
    }

    // Esc / ? / F1 dismiss help if open
    if app.show_help {
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('?') | KeyCode::F(1)) {
            app.show_help = false;
        }
        return false;
    }

    // Double-tap Esc quits (within 400 ms)
    if key.code == KeyCode::Esc && !app.search_focused && app.mode == AppMode::Browse {
        if last_esc.is_some_and(|t| t.elapsed() < Duration::from_millis(400)) {
            return true;
        }
        *last_esc = Some(Instant::now());
    } else {
        *last_esc = None;
    }

    match app.mode {
        AppMode::Browse => match key.code {
            KeyCode::Tab => app.toggle_focus(),
            KeyCode::Char('/') if !app.search_focused => {
                app.search_focused = true;
            }
            KeyCode::Enter | KeyCode::Right if app.search_focused => {
                app.search_focused = false;
                app.perform_search().await;
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') if !app.search_focused => {
                app.open_manage();
            }
            KeyCode::Esc | KeyCode::Left if app.search_focused => {
                app.search_focused = false;
            }
            KeyCode::Char(c) if app.search_focused => {
                app.search_query.push(c);
            }
            KeyCode::Char('p') if !app.search_focused => app.toggle_changes_sidebar(),
            KeyCode::Backspace if app.search_focused => {
                app.search_query.pop();
            }
            KeyCode::Delete if app.search_focused => {
                app.search_query.clear();
            }
            KeyCode::Down | KeyCode::Char('j') if !app.search_focused => app.next(),
            KeyCode::Up | KeyCode::Char('k') if !app.search_focused => app.prev(),
            KeyCode::PageDown => app.page_down(),
            KeyCode::PageUp => app.page_up(),
            KeyCode::Char('i') if !app.search_focused => app.toggle_installed_filter(),
            KeyCode::Char('r') if !app.search_focused => app.reload().await,
            KeyCode::Char('o') if !app.search_focused => app.cycle_sort(),
            _ => {}
        },
        AppMode::Manage => match key.code {
            KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') if app.right_pane_focused => {
                app.close_right_pane_focus()
            }
            KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => app.close_manage(),
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') if app.right_pane_focused => {
                app.activate_right_pane_item().await;
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                app.manage_activated = false;
                app.execute_selected_action().await;
            }
            KeyCode::Down | KeyCode::Char('j') if app.right_pane_focused => app.right_pane_next(),
            KeyCode::Down | KeyCode::Char('j') => {
                app.manage_activated = false;
                app.manage_next()
            }
            KeyCode::Up | KeyCode::Char('k') if app.right_pane_focused => app.right_pane_prev(),
            KeyCode::Up | KeyCode::Char('k') => {
                app.manage_activated = false;
                app.manage_prev()
            }
            KeyCode::PageDown if app.right_pane_focused => app.right_pane_page_down(),
            KeyCode::PageDown => {
                for _ in 0..10 {
                    app.manage_next();
                }
            }
            KeyCode::PageUp if app.right_pane_focused => app.right_pane_page_up(),
            KeyCode::PageUp => {
                for _ in 0..10 {
                    app.manage_prev();
                }
            }
            KeyCode::Char('p') => app.toggle_changes_sidebar(),
            KeyCode::Char('r')
                if app.right_pane_focused
                    && app.active_right_pane == crate::app::RightPane::Services =>
            {
                app.restart_selected_service().await;
            }
            KeyCode::Char('r') => {
                app.close_manage();
                app.reload().await;
            }
            _ => {}
        },
        AppMode::ChannelPicker => match key.code {
            KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => app.close_channel_picker(),
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                app.confirm_channel_pick().await
            }
            KeyCode::Down | KeyCode::Char('j') => app.channel_picker_next(),
            KeyCode::Up | KeyCode::Char('k') => app.channel_picker_prev(),
            KeyCode::Char('n') => app.open_custom_channel_input(),
            _ => {}
        },
        AppMode::ChannelInput => match key.code {
            KeyCode::Esc => app.close_channel_input(),
            KeyCode::Enter => app.execute_channel_action().await,
            KeyCode::Char(c) => app.channel_input.push(c),
            KeyCode::Backspace => {
                app.channel_input.pop();
            }
            _ => {}
        },
        AppMode::ClassicConfirm => match key.code {
            KeyCode::Esc | KeyCode::Char('n') => {
                app.cancel_classic();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                app.confirm_hovered = Some(true); // move focus to Install
            }
            KeyCode::Right | KeyCode::Char('l') => {
                app.confirm_hovered = Some(false); // move focus to Cancel
            }
            KeyCode::Char('y') => {
                app.confirm_classic().await;
            }
            KeyCode::Enter => match app.confirm_hovered {
                Some(true) => app.confirm_classic().await,
                _ => app.cancel_classic(),
            },
            _ => {}
        },
        AppMode::SlotPicker => match key.code {
            KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => app.close_slot_picker(),
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => app.confirm_slot_pick().await,
            KeyCode::Down | KeyCode::Char('j') => app.slot_picker_next(),
            KeyCode::Up | KeyCode::Char('k') => app.slot_picker_prev(),
            KeyCode::PageDown => app.slot_picker_next(),
            KeyCode::PageUp => app.slot_picker_prev(),
            _ => {}
        },
        AppMode::Changes => match key.code {
            KeyCode::Char('s') | KeyCode::Esc | KeyCode::Left => app.mode = AppMode::Browse,
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
        AppMode::Confirm => match key.code {
            KeyCode::Esc | KeyCode::Char('n') => {
                app.cancel_confirm();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                app.confirm_hovered = Some(true); // move to Yes
            }
            KeyCode::Right | KeyCode::Char('l') => {
                app.confirm_hovered = Some(false); // move to No
            }
            KeyCode::Char('y') => {
                app.execute_confirm().await;
            }
            KeyCode::Enter => match app.confirm_hovered {
                Some(true) => app.execute_confirm().await,
                _ => app.cancel_confirm(),
            },
            _ => {}
        },
    }

    false
}
