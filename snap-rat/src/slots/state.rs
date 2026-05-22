use ratatui::widgets::ListState;
use snapd_rs::api::interfaces::SlotRef;

use crate::app::{App, AppMode, ConnectionItem};

impl App {
    pub fn slot_picker_next(&mut self) {
        let len = self.slot_picker_items.len();
        if len == 0 {
            return;
        }
        let i = match self.slot_picker_state.selected() {
            Some(i) => (i + 1) % len,
            None => 0,
        };
        self.slot_picker_state.select(Some(i));
    }

    pub fn slot_picker_prev(&mut self) {
        let len = self.slot_picker_items.len();
        if len == 0 {
            return;
        }
        let i = match self.slot_picker_state.selected() {
            Some(0) | None => len - 1,
            Some(i) => i - 1,
        };
        self.slot_picker_state.select(Some(i));
    }

    pub fn close_slot_picker(&mut self) {
        self.mode = AppMode::Manage;
        self.slot_picker_plug = None;
        self.slot_picker_items.clear();
        self.slot_picker_state = ListState::default();
    }

    pub async fn confirm_slot_pick(&mut self) {
        let Some(idx) = self.slot_picker_state.selected() else {
            return;
        };
        let Some(target) = self.slot_picker_items.get(idx).cloned() else {
            return;
        };
        let Some(plug) = self.slot_picker_plug.take() else {
            return;
        };
        self.slot_picker_items.clear();
        self.slot_picker_state = ListState::default();
        self.mode = AppMode::Manage;
        self.do_connect_to_slot(&plug, target).await;
    }

    pub(crate) async fn do_connect_to_slot(&mut self, plug: &ConnectionItem, target: SlotRef) {
        self.loading = true;
        self.error = None;
        self.status_message = None;
        match self
            .client
            .connect_interface(&plug.plug_snap, &plug.plug_name, &target.snap, &target.slot)
            .await
        {
            Ok(change_id) => {
                self.active_change_id = Some(change_id.0);
                self.active_change = None;
                self.status_message = Some("Connecting…".to_string());
            }
            Err(ref e) if crate::resume::is_elevation_needed(e) => {
                self.try_elevate_and_exec(&plug.plug_snap, None);
                self.error = Some(e.to_string());
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }
        self.loading = false;
    }
}
