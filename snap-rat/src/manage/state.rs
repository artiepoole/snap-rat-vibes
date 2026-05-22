use ratatui::widgets::ListState;

use crate::app::{App, AppMode, ManageAction};

impl App {
    pub fn manage_next(&mut self) {
        let len = self.manage_actions.len();
        if len == 0 {
            return;
        }
        let i = match self.manage_state.selected() {
            Some(i) => (i + 1).min(len - 1),
            None => 0,
        };
        self.manage_state.select(Some(i));
    }

    pub fn manage_prev(&mut self) {
        let len = self.manage_actions.len();
        if len == 0 {
            return;
        }
        let i = match self.manage_state.selected() {
            Some(0) | None => 0,
            Some(i) => i - 1,
        };
        self.manage_state.select(Some(i));
    }

    pub fn open_manage(&mut self) {
        let Some(snap) = self.selected_snap() else {
            return;
        };
        let mut actions = vec![];
        if snap.installed {
            actions.push(ManageAction::Refresh);
            actions.push(ManageAction::SwitchChannel);
            actions.push(ManageAction::Revert);
            actions.push(ManageAction::Enable);
            actions.push(ManageAction::Disable);
            actions.push(ManageAction::Uninstall);
            actions.push(ManageAction::UninstallPurge);
        } else if snap.is_local_file {
            actions.push(ManageAction::InstallLocalFile);
        } else {
            actions.push(ManageAction::Install);
            actions.push(ManageAction::InstallFromChannel);
        }
        actions.push(ManageAction::OpenStorePage);
        if snap.contact.is_some() {
            actions.push(ManageAction::OpenContactPage);
        }
        self.managed_snap_name = Some(snap.name);
        self.manage_actions = actions;
        let mut ms = ListState::default();
        ms.select(Some(0));
        self.manage_state = ms;
        self.manage_activated = true;
        self.snap_interfaces.clear();
        self.snap_connections.clear();
        self.interfaces_loading = false;
        self.connections_mode = false;
        self.connections_state = ListState::default();
        self.connections_activated = false;
        self.mode = AppMode::Manage;
        self.error = None;
        self.status_message = None;
    }

    pub async fn load_snap_interfaces(&mut self, snap_name: &str) {
        self.interfaces_loading = true;
        self.snap_interfaces.clear();
        self.snap_connections.clear();
        self.connections_state = ListState::default();
        // Fetch interfaces (for plug/slot topology) and active connections
        // (for connected state) in parallel — select=all does NOT populate
        // Plug.connections, so we must cross-reference with /v2/connections.
        let (iface_result, conn_result) = {
            let c = &self.client;
            tokio::join!(c.list_snap_interfaces(snap_name), c.list_connections())
        };
        match iface_result {
            Ok(interfaces) => {
                self.snap_interfaces = interfaces;
            }
            Err(_) => {
                self.snap_interfaces.clear();
            }
        }
        if let Ok(connections) = conn_result {
            self.snap_connections = connections;
        }
        // Pre-select index 0 so the ghost arrow shows immediately (connections_activated
        // prevents this from counting as a "second click").
        if !self.connection_items().is_empty() {
            self.connections_state.select(Some(0));
            self.connections_activated = true;
        } else {
            self.connections_state.select(None);
        }
        self.interfaces_loading = false;
    }

    pub fn close_manage(&mut self) {
        self.mode = AppMode::Browse;
        self.manage_actions.clear();
        // Keep tracking any active change so the app can refresh after leaving the pane.
        self.available_channels.clear();
        self.channel_picker_state = ListState::default();
        self.channel_input.clear();
        self.pending_channel_action = None;
        self.classic_pending = None;
        self.classic_local_path = None;
        self.confirm_pending = None;
        self.confirm_message = None;
        self.managed_snap_name = None;
        self.snap_interfaces.clear();
        self.snap_connections.clear();
        self.interfaces_loading = false;
        self.connections_mode = false;
        self.connections_state = ListState::default();
        self.slot_picker_plug = None;
        self.slot_picker_items.clear();
        self.slot_picker_state = ListState::default();
    }
    pub fn action_needs_confirm(action: &ManageAction) -> bool {
        matches!(
            action,
            ManageAction::Uninstall
                | ManageAction::UninstallPurge
                | ManageAction::Revert
                | ManageAction::Disable
        )
    }

    pub fn selected_manage_action(&self) -> Option<&ManageAction> {
        let idx = self.manage_state.selected()?;
        self.manage_actions.get(idx)
    }

    pub async fn execute_selected_action(&mut self) {
        let action = match self.selected_manage_action().cloned() {
            Some(a) => a,
            None => return,
        };
        if action.needs_channel_input() {
            self.open_channel_picker(action).await;
            return;
        }
        if Self::action_needs_confirm(&action) {
            self.request_confirm_action(action);
            return;
        }
        let snap = match self.selected_snap() {
            Some(s) => s,
            None => return,
        };
        if matches!(action, ManageAction::InstallLocalFile) {
            let path = match snap.local_file_path.clone() {
                Some(p) => p,
                None => return,
            };
            self.execute_action(path, action, None).await;
        } else {
            self.execute_action(snap.name, action, None).await;
        }
    }
}
