use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::app::{App, AppMode};
use crate::layout::{inner_list_row_to_index, list_row_to_index, rect_contains};

/// Handle a mouse event.
pub(crate) async fn handle(app: &mut App, mouse: MouseEvent) {
    let col = mouse.column;
    let row = mouse.row;
    match mouse.kind {
        MouseEventKind::ScrollDown => match app.mode {
            AppMode::Browse => app.next(),
            AppMode::Manage if app.right_pane_focused => app.right_pane_next(),
            AppMode::Manage => app.manage_next(),
            AppMode::ChannelPicker => app.channel_picker_next(),
            AppMode::SlotPicker => app.slot_picker_next(),
            AppMode::Changes => {
                let in_detail = app
                    .changes_detail_area
                    .is_some_and(|a| rect_contains(a, col, row));
                if in_detail || app.changes_focus_detail {
                    app.changes_detail_next();
                } else {
                    app.changes_next();
                }
            }
            _ => {}
        },
        MouseEventKind::ScrollUp => match app.mode {
            AppMode::Browse => app.prev(),
            AppMode::Manage if app.right_pane_focused => app.right_pane_prev(),
            AppMode::Manage => app.manage_prev(),
            AppMode::ChannelPicker => app.channel_picker_prev(),
            AppMode::SlotPicker => app.slot_picker_prev(),
            AppMode::Changes => {
                let in_detail = app
                    .changes_detail_area
                    .is_some_and(|a| rect_contains(a, col, row));
                if in_detail || app.changes_focus_detail {
                    app.changes_detail_prev();
                } else {
                    app.changes_prev();
                }
            }
            _ => {}
        },
        MouseEventKind::Down(MouseButton::Left) => {
            // Help dialog: click outside to close.
            if app.show_help {
                if !app.help_area.is_some_and(|a| rect_contains(a, col, row)) {
                    app.show_help = false;
                }
                return;
            }
            // Confirm dialog intercepts all clicks first.
            if app.mode == AppMode::Confirm {
                let on_yes = app
                    .confirm_yes_area
                    .is_some_and(|a| rect_contains(a, col, row));
                let on_no = app
                    .confirm_no_area
                    .is_some_and(|a| rect_contains(a, col, row));
                if on_yes {
                    if app.confirm_hovered == Some(true) {
                        app.execute_confirm().await;
                    } else {
                        app.confirm_hovered = Some(true);
                    }
                } else if on_no {
                    if app.confirm_hovered == Some(false) {
                        app.cancel_confirm();
                    } else {
                        app.confirm_hovered = Some(false);
                    }
                } else {
                    app.cancel_confirm();
                }
            } else if app.mode == AppMode::ChannelPicker {
                if let Some(area) = app.channel_picker_area {
                    if rect_contains(area, col, row) {
                        let offset = app.channel_picker_state.offset();
                        if let Some(idx) = list_row_to_index(area, row, offset)
                            && idx < app.available_channels.len()
                        {
                            if app.channel_picker_state.selected() == Some(idx) {
                                app.confirm_channel_pick().await;
                            } else {
                                app.channel_picker_state.select(Some(idx));
                            }
                        }
                    } else {
                        app.close_channel_picker();
                    }
                }
            } else if app.mode == AppMode::ChannelInput {
                if !app
                    .channel_input_area
                    .is_some_and(|a| rect_contains(a, col, row))
                {
                    app.close_channel_input();
                }
            } else if app.mode == AppMode::ClassicConfirm {
                let on_yes = app
                    .confirm_yes_area
                    .is_some_and(|a| rect_contains(a, col, row));
                let on_no = app
                    .confirm_no_area
                    .is_some_and(|a| rect_contains(a, col, row));
                if on_yes {
                    if app.confirm_hovered == Some(true) {
                        app.confirm_classic().await;
                    } else {
                        app.confirm_hovered = Some(true);
                    }
                } else if on_no {
                    if app.confirm_hovered == Some(false) {
                        app.cancel_classic();
                    } else {
                        app.confirm_hovered = Some(false);
                    }
                } else if !app
                    .classic_confirm_area
                    .is_some_and(|a| rect_contains(a, col, row))
                {
                    app.cancel_classic();
                }
            } else if app.mode == AppMode::SlotPicker {
                if let Some(area) = app.slot_picker_area {
                    if rect_contains(area, col, row) {
                        let offset = app.slot_picker_state.offset();
                        if let Some(idx) = list_row_to_index(area, row, offset)
                            && idx < app.slot_picker_items.len()
                        {
                            if app.slot_picker_state.selected() == Some(idx) {
                                app.confirm_slot_pick().await;
                            } else {
                                app.slot_picker_state.select(Some(idx));
                            }
                        }
                    } else {
                        app.close_slot_picker();
                    }
                }
            } else {
                // Tab bar clicks work from any non-overlay mode.
                {
                    if app
                        .snaps_tab_area
                        .is_some_and(|a| rect_contains(a, col, row))
                    {
                        match app.mode {
                            AppMode::Changes | AppMode::Manage => app.mode = AppMode::Browse,
                            _ => {}
                        }
                    } else if app
                        .changes_tab_area
                        .is_some_and(|a| rect_contains(a, col, row))
                    {
                        if app.mode != AppMode::Changes {
                            app.mode = AppMode::Changes;
                            app.load_changes().await;
                        }
                    } else {
                        match app.mode {
                            AppMode::Browse => {
                                if let Some(area) = app.search_area
                                    && rect_contains(area, col, row)
                                {
                                    app.search_focused = true;
                                } else if let Some(area) = app.snap_list_area
                                    && rect_contains(area, col, row)
                                {
                                    let offset = app.list_state.offset();
                                    if let Some(idx) = list_row_to_index(area, row, offset)
                                        && idx < app.display_snaps().len()
                                    {
                                        if app.list_state.selected() == Some(idx) {
                                            // Second click on already-selected item: open manage.
                                            app.open_manage();
                                        } else {
                                            app.list_state.select(Some(idx));
                                            app.search_focused = false;
                                        }
                                    }
                                }
                            }
                            AppMode::Manage => {
                                if let Some(area) = app.manage_actions_area
                                    && rect_contains(area, col, row)
                                {
                                    let offset = app.manage_state.offset();
                                    if let Some(idx) = list_row_to_index(area, row, offset)
                                        && idx < app.manage_actions.len()
                                    {
                                        app.right_pane_focused = false;
                                        let already_selected =
                                            app.manage_state.selected() == Some(idx);
                                        app.manage_state.select(Some(idx));
                                        if already_selected && !app.manage_activated {
                                            // Second explicit click on same item: execute
                                            app.execute_selected_action().await;
                                        } else {
                                            // First click (or activated state): just select
                                            app.manage_activated = false;
                                            app.connections_activated = false;
                                        }
                                    }
                                } else if let Some(inner) = app.connections_inner_area
                                    && rect_contains(inner, col, row)
                                {
                                    let offset = app.connections_state.offset();
                                    if let Some(idx) = inner_list_row_to_index(inner, row, offset)
                                        && idx < app.connection_items().len()
                                    {
                                        app.right_pane_focused = true;
                                        let already_selected =
                                            app.connections_state.selected() == Some(idx);
                                        app.connections_state.select(Some(idx));
                                        if already_selected && !app.connections_activated {
                                            app.activate_selected_connection().await;
                                        } else {
                                            app.connections_activated = false;
                                            app.manage_activated = false;
                                        }
                                    }
                                } else if let Some(inner) = app.components_inner_area
                                    && rect_contains(inner, col, row)
                                {
                                    let offset = app.components_state.offset();
                                    if let Some(idx) = inner_list_row_to_index(inner, row, offset)
                                        && idx < app.snap_components.len()
                                    {
                                        app.right_pane_focused = true;
                                        let already_selected =
                                            app.components_state.selected() == Some(idx);
                                        app.components_state.select(Some(idx));
                                        if already_selected && !app.components_activated {
                                            app.request_confirm_component_toggle();
                                        } else {
                                            app.components_activated = false;
                                            app.manage_activated = false;
                                        }
                                    }
                                } else if let Some(inner) = app.services_inner_area
                                    && rect_contains(inner, col, row)
                                {
                                    let offset = app.services_state.offset();
                                    if let Some(idx) = inner_list_row_to_index(inner, row, offset)
                                        && idx < app.snap_services.len()
                                    {
                                        app.right_pane_focused = true;
                                        let already_selected =
                                            app.services_state.selected() == Some(idx);
                                        app.services_state.select(Some(idx));
                                        if already_selected && !app.services_activated {
                                            app.open_service_action_menu();
                                        } else {
                                            app.services_activated = false;
                                            app.manage_activated = false;
                                        }
                                    }
                                } else if app
                                    .left_pane_area
                                    .is_some_and(|a| rect_contains(a, col, row))
                                {
                                    if let Some(area) = app.snap_list_area
                                        && rect_contains(area, col, row)
                                    {
                                        let offset = app.list_state.offset();
                                        if let Some(idx) = list_row_to_index(area, row, offset)
                                            && idx < app.display_snaps().len()
                                        {
                                            app.list_state.select(Some(idx));
                                        }
                                    }
                                    app.close_manage();
                                }
                            }
                            AppMode::ChannelPicker
                            | AppMode::ChannelInput
                            | AppMode::ClassicConfirm
                            | AppMode::SlotPicker => {
                                // handled above before the in_overlay check
                            }
                            AppMode::Changes => {
                                if let Some(area) = app.changes_list_area
                                    && rect_contains(area, col, row)
                                {
                                    let offset = app.changes_list_state.offset();
                                    if let Some(idx) = list_row_to_index(area, row, offset)
                                        && idx < app.changes_list.len()
                                    {
                                        app.changes_list_state.select(Some(idx));
                                        app.changes_focus_detail = false;
                                    }
                                } else if let Some(area) = app.changes_detail_area
                                    && rect_contains(area, col, row)
                                {
                                    app.changes_focus_detail = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            } // end else (not Confirm mode)
        }
        _ => {}
    }
}
