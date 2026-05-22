use crate::app::{App, AppMode, ConfirmPending, ManageAction};

impl App {
    pub fn request_confirm_action(&mut self, action: ManageAction) {
        let snap_name = self
            .selected_snap()
            .map(|s| s.title.unwrap_or(s.name))
            .unwrap_or_default();
        self.confirm_message = Some(match &action {
            ManageAction::Uninstall => format!("Uninstall {snap_name}?"),
            ManageAction::UninstallPurge => {
                format!("Uninstall {snap_name} and permanently delete all its data?")
            }
            ManageAction::Revert => format!("Revert {snap_name} to the previous version?"),
            ManageAction::Disable => format!("Disable {snap_name}?"),
            _ => format!("Run \"{}\" on {snap_name}?", action.label()),
        });
        self.confirm_pending = Some(ConfirmPending::Action(action));
        self.confirm_hovered = Some(false); // default to No
        self.mode = AppMode::Confirm;
    }
    pub fn request_confirm_connection(&mut self) {
        let Some(item) = self.selected_connection() else {
            return;
        };
        let (pending, msg) = if item.connected {
            (
                ConfirmPending::Disconnect,
                format!(
                    "Disconnect {}:{} from {}:{}?",
                    item.plug_snap, item.plug_name, item.slot_snap, item.slot_name
                ),
            )
        } else {
            (
                ConfirmPending::Connect,
                format!(
                    "Connect {}:{} to {}:{}?",
                    item.plug_snap, item.plug_name, item.slot_snap, item.slot_name
                ),
            )
        };
        self.confirm_message = Some(msg);
        self.confirm_pending = Some(pending);
        self.confirm_hovered = Some(false); // default to No
        self.mode = AppMode::Confirm;
    }

    pub fn cancel_confirm(&mut self) {
        self.confirm_pending = None;
        self.confirm_message = None;
        self.confirm_hovered = None;
        // If there are queued auto-connect prompts, drop the current one and show the next.
        if !self.auto_connect_queue.is_empty() {
            self.auto_connect_queue.remove(0);
            self.mode = AppMode::Browse;
            self.pop_auto_connect_prompt();
        } else {
            self.mode = AppMode::Browse;
        }
    }

    pub async fn execute_confirm(&mut self) {
        let Some(pending) = self.confirm_pending.take() else {
            return;
        };
        self.confirm_message = None;
        match pending {
            ConfirmPending::Action(action) => {
                self.mode = AppMode::Manage;
                let name = match self.selected_snap().map(|s| s.name.clone()) {
                    Some(n) => n,
                    None => return,
                };
                self.execute_action(name, action, None).await;
            }
            ConfirmPending::Connect => {
                self.mode = AppMode::Manage;
                self.connect_selected().await;
            }
            ConfirmPending::Disconnect => {
                self.mode = AppMode::Manage;
                self.disconnect_selected().await;
            }
            ConfirmPending::AutoConnect {
                plug_snap,
                plug_name,
                slot,
                ..
            } => {
                // Remove this item from the queue and connect.
                if !self.auto_connect_queue.is_empty() {
                    self.auto_connect_queue.remove(0);
                }
                self.mode = AppMode::Browse;
                self.loading = true;
                self.error = None;
                self.status_message = None;
                match self
                    .client
                    .connect_interface(&plug_snap, &plug_name, &slot.snap, &slot.slot)
                    .await
                {
                    Ok(change_id) => {
                        self.active_change_id = Some(change_id.0);
                        self.active_change = None;
                        self.status_message = Some("Connecting…".to_string());
                    }
                    Err(e) => {
                        self.error = Some(e.to_string());
                    }
                }
                self.loading = false;
                // Show next prompt if any (after this change completes, the queue
                // is already drained for this item).
                self.pop_auto_connect_prompt();
            }
        }
    }
    pub fn cancel_classic(&mut self) {
        self.classic_pending = None;
        self.classic_local_path = None;
        self.confirm_hovered = None;
        self.mode = AppMode::Manage;
        self.error = None;
    }
    pub async fn confirm_classic(&mut self) {
        if let Some(path) = self.classic_local_path.take() {
            self.confirm_hovered = None;
            self.mode = AppMode::Manage;
            self.loading = true;
            self.error = None;
            self.status_message = None;
            match self.client.sideload_snap_classic(&path).await {
                Ok(change_id) => {
                    self.active_change_id = Some(change_id.0);
                    self.active_change = None;
                    self.status_message = Some("Sideloading (classic)…".to_string());
                    self.active_change_action = Some(ManageAction::InstallLocalFile);
                    self.active_change_snap = Some(path);
                }
                Err(e) => {
                    self.error = Some(e.to_string());
                }
            }
            self.loading = false;
            return;
        }
        let Some((name, channel)) = self.classic_pending.take() else {
            return;
        };
        self.confirm_hovered = None;
        self.mode = AppMode::Manage;
        self.loading = true;
        self.error = None;
        self.status_message = None;
        match self
            .client
            .install_snap_classic(&name, channel.as_deref())
            .await
        {
            Ok(change_id) => {
                self.active_change_id = Some(change_id.0);
                self.active_change = None;
                self.status_message = Some("Installing (classic)…".to_string());
                self.active_change_action = Some(ManageAction::InstallFromChannel);
                self.active_change_snap = Some(name.clone());
            }
            Err(ref e) if crate::resume::is_elevation_needed(e) => {
                self.try_elevate_and_exec(
                    &name,
                    Some(crate::resume::ResumeAction::InstallClassic {
                        snap_name: name.clone(),
                        channel: channel.clone(),
                    }),
                );
                self.error = Some(e.to_string());
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }
        self.loading = false;
    }
}
