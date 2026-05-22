use ratatui::widgets::ListState;
use snapd_rs::ChannelSnapInfo;

use crate::app::{App, AppMode, ManageAction, channel_sort_key, empty_channel_info};

impl App {
    pub fn channel_picker_next(&mut self) {
        let len = self.available_channels.len();
        if len == 0 {
            return;
        }
        let i = match self.channel_picker_state.selected() {
            Some(i) => (i + 1).min(len - 1),
            None => 0,
        };
        self.channel_picker_state.select(Some(i));
    }

    pub fn channel_picker_prev(&mut self) {
        let len = self.available_channels.len();
        if len == 0 {
            return;
        }
        let i = match self.channel_picker_state.selected() {
            Some(0) | None => 0,
            Some(i) => i - 1,
        };
        self.channel_picker_state.select(Some(i));
    }

    pub fn close_channel_picker(&mut self) {
        self.mode = AppMode::Manage;
        self.available_channels.clear();
        self.channel_picker_state = ListState::default();
        self.channel_input.clear();
        self.pending_channel_action = None;
    }

    pub async fn open_channel_picker(&mut self, action: ManageAction) {
        let Some(name) = self.selected_snap().map(|snap| snap.name) else {
            return;
        };

        self.loading = true;
        self.error = None;
        self.channel_input.clear();
        self.pending_channel_action = Some(action);

        match self.client.find_snap_by_name(&name).await {
            Ok(store_snap) => {
                let mut channels: Vec<(String, ChannelSnapInfo)> = store_snap
                    .map(|snap| snap.channels.into_iter().collect())
                    .unwrap_or_default();
                channels.sort_by_key(|a| channel_sort_key(&a.0));
                channels.push((String::new(), empty_channel_info()));
                self.available_channels = channels;
                self.channel_picker_state = ListState::default().with_selected(Some(0));
                self.mode = AppMode::ChannelPicker;
            }
            Err(e) => {
                self.pending_channel_action = None;
                self.error = Some(e.to_string());
            }
        }

        self.loading = false;
    }

    pub fn open_custom_channel_input(&mut self) {
        self.channel_input.clear();
        self.mode = AppMode::ChannelInput;
    }

    pub fn close_channel_input(&mut self) {
        self.channel_input.clear();
        if self.available_channels.is_empty() {
            self.mode = AppMode::Manage;
            self.pending_channel_action = None;
        } else {
            self.mode = AppMode::ChannelPicker;
        }
    }

    pub async fn confirm_channel_pick(&mut self) {
        let Some(idx) = self.channel_picker_state.selected() else {
            return;
        };
        let Some((channel, _)) = self.available_channels.get(idx).cloned() else {
            return;
        };

        if channel.is_empty() {
            self.open_custom_channel_input();
            return;
        }

        let name = match self.selected_snap().map(|s| s.name.clone()) {
            Some(n) => n,
            None => return,
        };
        let action = match self.pending_channel_action.take() {
            Some(a) => a,
            None => return,
        };

        self.mode = AppMode::Manage;
        self.available_channels.clear();
        self.channel_picker_state = ListState::default();
        self.execute_action(name, action, Some(channel.as_str()))
            .await;
    }

    pub async fn execute_channel_action(&mut self) {
        let name = match self.selected_snap().map(|s| s.name.clone()) {
            Some(n) => n,
            None => return,
        };
        let action = match self.pending_channel_action.take() {
            Some(a) => a,
            None => return,
        };
        let channel = self.channel_input.trim().to_string();
        let channel_opt = if channel.is_empty() {
            None
        } else {
            Some(channel.clone())
        };
        self.mode = AppMode::Manage;
        self.available_channels.clear();
        self.channel_picker_state = ListState::default();
        self.channel_input.clear();
        self.execute_action(name, action, channel_opt.as_deref())
            .await;
    }
}
