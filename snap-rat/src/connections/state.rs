use ratatui::widgets::ListState;
use snapd_rs::api::interfaces::SlotRef;

use crate::app::{App, AppMode, ConfirmPending, ConnectionItem, RightPane};

impl App {
    pub fn connections_next(&mut self) {
        let len = self.connection_items().len();
        if len == 0 {
            return;
        }
        let i = match self.connections_state.selected() {
            Some(i) => (i + 1).min(len - 1),
            None => 0,
        };
        self.connections_state.select(Some(i));
    }

    pub fn connections_prev(&mut self) {
        let len = self.connection_items().len();
        if len == 0 {
            return;
        }
        let i = match self.connections_state.selected() {
            Some(0) | None => 0,
            Some(i) => i - 1,
        };
        self.connections_state.select(Some(i));
    }

    pub fn connections_page_down(&mut self) {
        let len = self.connection_items().len();
        if len == 0 {
            return;
        }
        let i = self
            .connections_state
            .selected()
            .unwrap_or(0)
            .saturating_add(10)
            .min(len - 1);
        self.connections_state.select(Some(i));
    }

    pub fn connections_page_up(&mut self) {
        let len = self.connection_items().len();
        if len == 0 {
            return;
        }
        let i = self
            .connections_state
            .selected()
            .unwrap_or(0)
            .saturating_sub(10);
        self.connections_state.select(Some(i));
    }

    pub fn components_next(&mut self) {
        let len = self.snap_components.len();
        if len == 0 {
            return;
        }
        let i = self
            .components_state
            .selected()
            .map(|i| (i + 1).min(len - 1))
            .unwrap_or(0);
        self.components_state.select(Some(i));
    }

    pub fn components_prev(&mut self) {
        let len = self.snap_components.len();
        if len == 0 {
            return;
        }
        let i = match self.components_state.selected() {
            Some(0) | None => 0,
            Some(i) => i - 1,
        };
        self.components_state.select(Some(i));
    }

    pub fn services_next(&mut self) {
        let len = self.snap_services.len();
        if len == 0 {
            return;
        }
        let i = self
            .services_state
            .selected()
            .map(|i| (i + 1).min(len - 1))
            .unwrap_or(0);
        self.services_state.select(Some(i));
    }

    pub fn services_prev(&mut self) {
        let len = self.snap_services.len();
        if len == 0 {
            return;
        }
        let i = match self.services_state.selected() {
            Some(0) | None => 0,
            Some(i) => i - 1,
        };
        self.services_state.select(Some(i));
    }

    pub fn right_pane_next(&mut self) {
        match self.active_right_pane {
            RightPane::None => {}
            RightPane::Connections => self.connections_next(),
            RightPane::Components => self.components_next(),
            RightPane::Services => {
                if self.service_actions_open {
                    self.service_action_next();
                } else {
                    self.services_next();
                }
            }
        }
    }

    pub fn right_pane_prev(&mut self) {
        match self.active_right_pane {
            RightPane::None => {}
            RightPane::Connections => self.connections_prev(),
            RightPane::Components => self.components_prev(),
            RightPane::Services => {
                if self.service_actions_open {
                    self.service_action_prev();
                } else {
                    self.services_prev();
                }
            }
        }
    }

    pub fn right_pane_page_down(&mut self) {
        match self.active_right_pane {
            RightPane::None => {}
            RightPane::Connections => self.connections_page_down(),
            RightPane::Components => {
                for _ in 0..10 {
                    self.components_next();
                }
            }
            RightPane::Services => {
                for _ in 0..10 {
                    self.services_next();
                }
            }
        }
    }

    pub fn right_pane_page_up(&mut self) {
        match self.active_right_pane {
            RightPane::None => {}
            RightPane::Connections => self.connections_page_up(),
            RightPane::Components => {
                for _ in 0..10 {
                    self.components_prev();
                }
            }
            RightPane::Services => {
                for _ in 0..10 {
                    self.services_prev();
                }
            }
        }
    }

    pub fn close_right_pane_focus(&mut self) {
        self.right_pane_focused = false;
        self.active_right_pane = crate::app::RightPane::None;
        self.close_service_action_menu();
        if self.manage_state.selected().is_none() && !self.manage_actions.is_empty() {
            self.manage_state.select(Some(0));
        }
        self.manage_activated = true;
    }

    pub fn selected_connection(&self) -> Option<ConnectionItem> {
        let idx = self.connections_state.selected()?;
        self.connection_items().get(idx).cloned()
    }

    pub async fn activate_selected_connection(&mut self) {
        if self.selected_connection().is_none() {
            return;
        };
        self.request_confirm_connection();
    }

    pub async fn activate_right_pane_item(&mut self) {
        match self.active_right_pane {
            RightPane::None => {}
            RightPane::Connections => {
                self.connections_activated = false;
                self.activate_selected_connection().await;
            }
            RightPane::Components => {
                self.components_activated = false;
                self.request_confirm_component_toggle();
            }
            RightPane::Services => {
                self.services_activated = false;
                if self.service_actions_open {
                    self.confirm_selected_service_action();
                } else {
                    self.open_service_action_menu();
                }
            }
        }
    }

    /// Build the action list for the currently-selected service and open the menu.
    pub fn open_service_action_menu(&mut self) {
        let Some(idx) = self.services_state.selected() else {
            return;
        };
        let Some(service) = self.snap_services.get(idx) else {
            return;
        };
        let active = service.active == Some(true);
        // Only treat as explicitly enabled if snapd says so; None means unknown → not enabled.
        let explicitly_enabled = service.enabled == Some(true);
        let explicitly_disabled = service.enabled == Some(false);

        let mut actions: Vec<crate::app::ServiceAction> = vec![];
        if !active {
            actions.push(crate::app::ServiceAction::Start);
        }
        actions.push(crate::app::ServiceAction::Restart);
        if active {
            actions.push(crate::app::ServiceAction::Stop);
        }
        if !explicitly_enabled {
            actions.push(crate::app::ServiceAction::Enable);
        }
        if explicitly_enabled || (active && !explicitly_disabled) {
            actions.push(crate::app::ServiceAction::Disable);
        }

        self.service_actions = actions;
        self.service_actions_state = ratatui::widgets::ListState::default();
        self.service_actions_state.select(Some(0));
        self.service_actions_open = true;
    }

    pub fn close_service_action_menu(&mut self) {
        self.service_actions_open = false;
        self.service_actions = vec![];
        self.service_actions_state = ratatui::widgets::ListState::default();
    }

    pub fn service_action_next(&mut self) {
        let len = self.service_actions.len();
        if len == 0 {
            return;
        }
        let i = match self.service_actions_state.selected() {
            Some(i) => (i + 1).min(len - 1),
            None => 0,
        };
        self.service_actions_state.select(Some(i));
    }

    pub fn service_action_prev(&mut self) {
        let len = self.service_actions.len();
        if len == 0 {
            return;
        }
        let i = match self.service_actions_state.selected() {
            Some(0) | None => 0,
            Some(i) => i - 1,
        };
        self.service_actions_state.select(Some(i));
    }

    pub fn confirm_selected_service_action(&mut self) {
        let Some(action_idx) = self.service_actions_state.selected() else {
            return;
        };
        let Some(action) = self.service_actions.get(action_idx).cloned() else {
            return;
        };
        let Some(svc_idx) = self.services_state.selected() else {
            return;
        };
        let Some(service) = self.snap_services.get(svc_idx) else {
            return;
        };
        let Some(snap_name) = self.managed_snap_name.clone() else {
            return;
        };
        let service_name = service.name.clone();
        self.confirm_message = Some(format!(
            "{} service '{}'?",
            action
                .label()
                .split_once("  ")
                .map(|(l, _)| l)
                .unwrap_or(action.label()),
            service_name
        ));
        self.confirm_pending = Some(ConfirmPending::ServiceAction {
            snap_name,
            service_name,
            action,
        });
        self.confirm_hovered = Some(false);
        self.mode = AppMode::Confirm;
    }

    pub fn request_confirm_component_toggle(&mut self) {
        let Some(idx) = self.components_state.selected() else {
            return;
        };
        let Some(component) = self.snap_components.get(idx).cloned() else {
            return;
        };
        let Some(snap_name) = self.managed_snap_name.clone() else {
            return;
        };
        let install = component.install_date.is_none();
        let verb = if install { "Install" } else { "Remove" };
        self.confirm_message = Some(format!("{verb} component '{}'?", component.name));
        self.confirm_pending = Some(ConfirmPending::ComponentToggle {
            snap_name,
            component_name: component.name.clone(),
            install,
        });
        self.confirm_hovered = Some(false);
        self.mode = AppMode::Confirm;
    }

    pub async fn toggle_selected_component(&mut self) {
        if self.active_change_id.is_some() {
            self.status_message = Some("Operation already in progress".to_string());
            return;
        }
        let Some(idx) = self.components_state.selected() else {
            return;
        };
        let Some(component) = self.snap_components.get(idx).cloned() else {
            return;
        };
        let Some(snap_name) = self.managed_snap_name.clone() else {
            return;
        };

        self.error = None;
        self.status_message = None;

        let result = if component.install_date.is_some() {
            // Component is installed — remove it
            self.client
                .remove_snap_component(&snap_name, &component.name)
                .await
        } else {
            // Component is not installed — install it
            self.client
                .install_snap_component(&snap_name, &component.name)
                .await
        };

        match result {
            Ok(change_id) => {
                self.active_change_id = Some(change_id.0);
                self.active_change = None;
                self.status_message = Some(if component.install_date.is_some() {
                    format!("Removing component '{}'…", component.name)
                } else {
                    format!("Installing component '{}'…", component.name)
                });
            }
            Err(ref e) if crate::resume::is_elevation_needed(e) => {
                self.try_elevate_and_exec(&snap_name, None);
                self.error = Some(e.to_string());
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }
    }

    pub async fn connect_selected(&mut self) {
        if self.active_change_id.is_some() {
            self.status_message = Some("Operation already in progress".to_string());
            return;
        }

        let Some(item) = self.selected_connection() else {
            return;
        };
        if item.connected {
            self.status_message = Some("Connection already active".to_string());
            return;
        }
        if !item.is_plug {
            self.error = Some("Select a plug to create a new connection".to_string());
            return;
        }

        // Collect all available slots for this interface (from any snap).
        let available_slots: Vec<SlotRef> = self
            .snap_interfaces
            .iter()
            .find(|iface| iface.name == item.interface_name)
            .map(|iface| {
                iface
                    .slots
                    .iter()
                    .map(|slot| SlotRef {
                        snap: slot.snap.clone().unwrap_or_default(),
                        slot: slot.slot.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        if available_slots.is_empty() {
            self.error = Some(format!(
                "No available slots for interface '{}'",
                item.interface_name
            ));
            return;
        }

        // If there is exactly one slot, connect immediately; otherwise show the picker.
        if available_slots.len() == 1 {
            let target = available_slots.into_iter().next().unwrap();
            self.do_connect_to_slot(&item, target).await;
        } else {
            let mut state = ListState::default();
            state.select(Some(0));
            self.slot_picker_plug = Some(item);
            self.slot_picker_items = available_slots;
            self.slot_picker_state = state;
            self.mode = AppMode::SlotPicker;
        }
    }

    pub async fn disconnect_selected(&mut self) {
        if self.active_change_id.is_some() {
            self.status_message = Some("Operation already in progress".to_string());
            return;
        }

        let Some(item) = self.selected_connection() else {
            return;
        };
        if !item.connected {
            self.status_message = Some("Connection already disconnected".to_string());
            return;
        }

        self.loading = true;
        self.error = None;
        self.status_message = None;
        match self
            .client
            .disconnect_interface(
                &item.plug_snap,
                &item.plug_name,
                &item.slot_snap,
                &item.slot_name,
            )
            .await
        {
            Ok(change_id) => {
                self.active_change_id = Some(change_id.0);
                self.active_change = None;
                self.status_message = Some("Disconnecting…".to_string());
            }
            Err(ref e) if crate::resume::is_elevation_needed(e) => {
                // For connect/disconnect, just restore position — user can redo.
                self.try_elevate_and_exec(&item.plug_snap, None);
                self.error = Some(e.to_string());
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }
        self.loading = false;
    }

    pub fn connection_items(&self) -> Vec<ConnectionItem> {
        let Some(snap) = self.selected_snap() else {
            return vec![];
        };
        self.connection_items_for_snap(&snap.name)
    }

    /// After an install completes, load interfaces for the snap and queue prompts
    /// for any unconnected plugs that have exactly one system (snapd) slot available.
    pub async fn queue_auto_connect_prompts(&mut self, snap_name: &str) {
        let (iface_result, conn_result) = {
            let c = &self.client;
            tokio::join!(c.list_snap_interfaces(snap_name), c.list_connections())
        };
        let interfaces = match iface_result {
            Ok(i) => i,
            Err(_) => return,
        };
        let connections = conn_result.unwrap_or_default();

        let mut queue = Vec::new();
        for iface in &interfaces {
            for plug in iface
                .plugs
                .iter()
                .filter(|p| p.snap.as_deref() == Some(snap_name))
            {
                // Skip if already connected.
                let already_connected = connections
                    .iter()
                    .any(|c| c.plug.snap == snap_name && c.plug.plug == plug.plug);
                if already_connected {
                    continue;
                }

                // Collect slots that belong to a system snap (snapd or empty snap name).
                let system_slots: Vec<SlotRef> = iface
                    .slots
                    .iter()
                    .filter(|s| matches!(s.snap.as_deref(), Some("snapd") | Some("") | None))
                    .map(|s| SlotRef {
                        snap: s.snap.clone().unwrap_or_default(),
                        slot: s.slot.clone(),
                    })
                    .collect();

                if system_slots.len() == 1 {
                    queue.push(crate::app::ConfirmPending::AutoConnect {
                        plug_snap: snap_name.to_string(),
                        plug_name: plug.plug.clone(),
                        interface_name: iface.name.clone(),
                        slot: system_slots.into_iter().next().unwrap(),
                    });
                }
            }
        }
        self.auto_connect_queue = queue;
        self.pop_auto_connect_prompt();
    }

    /// Pop the next auto-connect prompt from the queue and show it.
    pub fn pop_auto_connect_prompt(&mut self) {
        if self.confirm_pending.is_some() || self.mode == AppMode::Confirm {
            return; // another confirm is already showing
        }
        if let Some(pending) = self.auto_connect_queue.first().cloned()
            && let crate::app::ConfirmPending::AutoConnect {
                ref interface_name, ..
            } = pending
        {
            self.confirm_message = Some(format!("Connect interface '{interface_name}'?"));
            self.confirm_pending = Some(pending);
            self.confirm_hovered = Some(false);
            self.mode = AppMode::Confirm;
        }
    }
}
