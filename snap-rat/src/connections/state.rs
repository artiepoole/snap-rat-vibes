use ratatui::widgets::ListState;
use snapd_rs::api::interfaces::SlotRef;

use crate::app::{App, AppMode, ConnectionItem};

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

    pub fn toggle_connections_mode(&mut self) {
        self.connections_mode = !self.connections_mode;
        if self.connections_mode {
            // Entering connections pane: show ghost arrow on manage, highlight connections
            self.manage_state
                .select(Some(self.manage_state.selected().unwrap_or(0)));
            if self.connections_state.selected().is_none() && !self.connection_items().is_empty() {
                self.connections_state.select(Some(0));
            }
            self.connections_activated = true;
        } else {
            // Returning to manage actions: show ghost arrow on connections, highlight manage
            self.connections_state
                .select(Some(self.connections_state.selected().unwrap_or(0)));
            if self.manage_state.selected().is_none() && !self.manage_actions.is_empty() {
                self.manage_state.select(Some(0));
            }
            self.manage_activated = true;
        }
    }

    pub fn close_connections_mode(&mut self) {
        self.connections_mode = false;
        // Keep connections position as ghost, restore manage highlight
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
