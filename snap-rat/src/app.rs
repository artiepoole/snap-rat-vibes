use std::collections::{HashMap, HashSet};

use image::DynamicImage;
use ratatui::widgets::ListState;
use ratatui_image::picker::{Capability, Picker, ProtocolType, cap_parser::QueryStdioOptions};
use snapd_rs::{
    Change, ChannelSnapInfo, SnapdClient, StoreSnap,
    api::{
        interfaces::{Connection, Interface, SlotRef},
        snaps::Snap,
    },
};

use crate::DisplaySnap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    Browse,
    Manage,
    ChannelPicker,
    ChannelInput,
    /// Waiting for the user to confirm installing a classic-confinement snap.
    /// Holds the snap name and optional channel so we can retry with classic=true.
    ClassicConfirm,
    Changes,
    /// Picking a slot to connect a plug to.
    SlotPicker,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortMode {
    NameAsc,
    NameDesc,
    RevisionDesc,
}

impl SortMode {
    pub fn label(&self) -> &'static str {
        match self {
            SortMode::NameAsc => "A→Z",
            SortMode::NameDesc => "Z→A",
            SortMode::RevisionDesc => "Newest",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManageAction {
    Install,
    InstallFromChannel,
    Refresh,
    SwitchChannel,
    Revert,
    Enable,
    Disable,
    Uninstall,
    OpenStorePage,
    OpenContactPage,
}

impl ManageAction {
    pub fn label(&self) -> &'static str {
        match self {
            ManageAction::Install => "Install",
            ManageAction::InstallFromChannel => "Install from channel…",
            ManageAction::Refresh => "Refresh to latest",
            ManageAction::SwitchChannel => "Switch channel…",
            ManageAction::Revert => "Revert to previous version",
            ManageAction::Enable => "Enable",
            ManageAction::Disable => "Disable",
            ManageAction::Uninstall => "Uninstall",
            ManageAction::OpenStorePage => "Open store page",
            ManageAction::OpenContactPage => "Open contact page",
        }
    }

    pub fn needs_channel_input(&self) -> bool {
        matches!(
            self,
            ManageAction::SwitchChannel | ManageAction::InstallFromChannel
        )
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionItem {
    pub interface_name: String,
    pub plug_snap: String,
    pub plug_name: String,
    pub slot_snap: String,
    pub slot_name: String,
    pub connected: bool,
    pub is_plug: bool,
}

pub struct App {
    pub client: SnapdClient,
    pub installed: Vec<Snap>,
    pub store_results: Vec<StoreSnap>,
    pub search_query: String,
    pub search_focused: bool,
    pub list_state: ListState,
    pub loading: bool,
    pub error: Option<String>,
    pub status_message: Option<String>,
    pub showing_results: bool,
    pub show_installed_only: bool,
    pub sort_mode: SortMode,
    pub mode: AppMode,
    pub manage_actions: Vec<ManageAction>,
    pub manage_state: ListState,
    pub active_change_id: Option<String>,
    pub active_change: Option<Change>,
    pub show_changes_sidebar: bool,
    pub sidebar_changes: Vec<Change>,
    pub changes_list: Vec<Change>,
    pub changes_list_state: ListState,
    pub changes_detail_state: ListState,
    pub changes_focus_detail: bool,
    pub available_channels: Vec<(String, ChannelSnapInfo)>,
    pub channel_picker_state: ListState,
    pub channel_input: String,
    pub pending_channel_action: Option<ManageAction>,
    /// Snap name / channel waiting for classic confirmation.
    pub classic_pending: Option<(String, Option<String>)>,
    /// Name of the snap currently open in the manage panel.
    /// Persists through close_manage so reload() can restore the selection.
    pub managed_snap_name: Option<String>,
    pub snap_interfaces: Vec<Interface>,
    /// Active connections fetched from /v2/connections — used to determine
    /// connected state because select=all does not populate Plug.connections.
    pub snap_connections: Vec<Connection>,
    pub interfaces_loading: bool,
    pub connections_mode: bool,
    pub connections_state: ListState,
    /// Plug being connected — shown in slot picker overlay.
    pub slot_picker_plug: Option<ConnectionItem>,
    /// Available slots to connect to (populated when entering SlotPicker mode).
    pub slot_picker_items: Vec<SlotRef>,
    pub slot_picker_state: ListState,
    pub icon_picker: Option<Picker>,
    pub icon_cache: HashMap<String, Option<DynamicImage>>,
    pub icon_fetching: HashSet<String>,
}

impl App {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            client: SnapdClient::new(),
            installed: vec![],
            store_results: vec![],
            search_query: String::new(),
            search_focused: false,
            list_state,
            loading: false,
            error: None,
            status_message: None,
            showing_results: false,
            show_installed_only: false,
            sort_mode: SortMode::NameAsc,
            mode: AppMode::Browse,
            manage_actions: vec![],
            manage_state: ListState::default(),
            active_change_id: None,
            active_change: None,
            show_changes_sidebar: false,
            sidebar_changes: vec![],
            changes_list: vec![],
            changes_list_state: ListState::default(),
            changes_detail_state: ListState::default(),
            changes_focus_detail: false,
            available_channels: vec![],
            channel_picker_state: ListState::default(),
            channel_input: String::new(),
            pending_channel_action: None,
            classic_pending: None,
            managed_snap_name: None,
            snap_interfaces: vec![],
            snap_connections: vec![],
            interfaces_loading: false,
            connections_mode: false,
            connections_state: ListState::default(),
            slot_picker_plug: None,
            slot_picker_items: vec![],
            slot_picker_state: ListState::default(),
            icon_picker: Picker::from_query_stdio_with_options(QueryStdioOptions {
                // Ask the terminal for its background colour via OSC 11 so that
                // transparent PNG icons are composited correctly instead of
                // rendering transparency as black.
                terminal_background_color_osc: true,
                ..Default::default()
            })
            .ok()
            .map(|mut picker| {
                // Apply the queried background colour, or fall back to a dark
                // default that matches most terminal themes.
                let bg = picker
                    .capabilities()
                    .iter()
                    .find_map(|c| {
                        if let Capability::Background(r, g, b) = c {
                            Some(image::Rgba([*r, *g, *b, 255u8]))
                        } else {
                            None
                        }
                    })
                    .unwrap_or(image::Rgba([30u8, 30u8, 30u8, 255u8]));
                picker.set_background_color(Some(bg));

                // VTE-based terminals (Tilix, GNOME Terminal, etc.) support Sixel
                // but the capability query may not be detected in time. Force Sixel
                // when we know we're in a VTE terminal and the picker fell back to
                // halfblocks.
                let is_vte =
                    std::env::var("VTE_VERSION").is_ok() || std::env::var("TILIX_ID").is_ok();
                if is_vte && picker.protocol_type() == ProtocolType::Halfblocks {
                    picker.set_protocol_type(ProtocolType::Sixel);
                }

                picker
            }),
            icon_cache: HashMap::default(),
            icon_fetching: HashSet::default(),
        }
    }

    pub fn toggle_focus(&mut self) {
        self.search_focused = !self.search_focused;
    }

    pub fn toggle_installed_filter(&mut self) {
        self.show_installed_only = !self.show_installed_only;
        self.list_state.select(Some(0));
    }

    pub fn toggle_changes_sidebar(&mut self) {
        self.show_changes_sidebar = !self.show_changes_sidebar;
        if self.show_changes_sidebar {
            self.sidebar_changes.clear();
        }
    }

    pub fn cycle_sort(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::NameAsc => SortMode::NameDesc,
            SortMode::NameDesc => SortMode::RevisionDesc,
            SortMode::RevisionDesc => SortMode::NameAsc,
        };
        self.list_state.select(Some(0));
    }

    pub fn display_snaps(&self) -> Vec<DisplaySnap> {
        let mut snaps: Vec<DisplaySnap> = if self.showing_results {
            let installed_names: std::collections::HashSet<&str> =
                self.installed.iter().map(|s| s.name.as_str()).collect();
            self.store_results
                .iter()
                .filter(|s| !self.show_installed_only || installed_names.contains(s.name.as_str()))
                .map(|s| {
                    let mut d = DisplaySnap::from(s);
                    if installed_names.contains(s.name.as_str()) {
                        d.installed = true;
                    }
                    d
                })
                .collect()
        } else {
            self.installed.iter().map(DisplaySnap::from).collect()
        };

        snaps.sort_by(|a, b| match self.sort_mode {
            SortMode::NameAsc => a
                .name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then_with(|| a.name.cmp(&b.name)),
            SortMode::NameDesc => b
                .name
                .to_lowercase()
                .cmp(&a.name.to_lowercase())
                .then_with(|| b.name.cmp(&a.name)),
            SortMode::RevisionDesc => b
                .version
                .as_deref()
                .unwrap_or_default()
                .cmp(a.version.as_deref().unwrap_or_default())
                .then_with(|| a.name.cmp(&b.name)),
        });

        if self.showing_results {
            snaps.sort_by_key(|snap| if snap.installed { 0u8 } else { 1u8 });
        }

        snaps
    }

    pub fn selected_snap(&self) -> Option<DisplaySnap> {
        let snaps = self.display_snaps();
        let idx = self.list_state.selected()?;
        snaps.get(idx).cloned()
    }

    pub fn next(&mut self) {
        let len = self.display_snaps().len();
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1).min(len - 1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn prev(&mut self) {
        let len = self.display_snaps().len();
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(0) | None => 0,
            Some(i) => i - 1,
        };
        self.list_state.select(Some(i));
    }

    pub fn page_down(&mut self) {
        let len = self.display_snaps().len();
        if len == 0 {
            return;
        }
        let i = self
            .list_state
            .selected()
            .unwrap_or(0)
            .saturating_add(10)
            .min(len - 1);
        self.list_state.select(Some(i));
    }

    pub fn page_up(&mut self) {
        let len = self.display_snaps().len();
        if len == 0 {
            return;
        }
        let i = self.list_state.selected().unwrap_or(0).saturating_sub(10);
        self.list_state.select(Some(i));
    }

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
        if self.connections_mode && self.connections_state.selected().is_none() {
            self.connections_state
                .select((!self.connection_items().is_empty()).then_some(0));
        }
    }

    pub fn close_connections_mode(&mut self) {
        self.connections_mode = false;
    }

    pub fn selected_connection(&self) -> Option<ConnectionItem> {
        let idx = self.connections_state.selected()?;
        self.connection_items().get(idx).cloned()
    }

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

    pub fn changes_next(&mut self) {
        let len = self.changes_list.len();
        if len == 0 {
            return;
        }
        let i = match self.changes_list_state.selected() {
            Some(i) => (i + 1).min(len - 1),
            None => 0,
        };
        self.changes_list_state.select(Some(i));
        self.changes_detail_state.select(Some(0));
    }

    pub fn changes_prev(&mut self) {
        let len = self.changes_list.len();
        if len == 0 {
            return;
        }
        let i = match self.changes_list_state.selected() {
            Some(0) | None => 0,
            Some(i) => i - 1,
        };
        self.changes_list_state.select(Some(i));
        self.changes_detail_state.select(Some(0));
    }

    pub fn changes_detail_next(&mut self) {
        let len = self
            .selected_change()
            .map(|change| change.tasks.len())
            .unwrap_or(0);
        if len == 0 {
            return;
        }
        let i = match self.changes_detail_state.selected() {
            Some(i) => (i + 1).min(len - 1),
            None => 0,
        };
        self.changes_detail_state.select(Some(i));
    }

    pub fn changes_detail_prev(&mut self) {
        let len = self
            .selected_change()
            .map(|change| change.tasks.len())
            .unwrap_or(0);
        if len == 0 {
            return;
        }
        let i = match self.changes_detail_state.selected() {
            Some(0) | None => 0,
            Some(i) => i - 1,
        };
        self.changes_detail_state.select(Some(i));
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
        let mut state = ListState::default();
        state.select(Some(0));
        self.manage_state = state;
        self.snap_interfaces.clear();
        self.snap_connections.clear();
        self.interfaces_loading = false;
        self.connections_mode = false;
        self.connections_state = ListState::default();
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
        self.connections_state
            .select((!self.connection_items_for_snap(snap_name).is_empty()).then_some(0));
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

    /// Dismiss the classic confirmation and go back to the manage panel.
    pub fn cancel_classic(&mut self) {
        self.classic_pending = None;
        self.mode = AppMode::Manage;
        self.error = None;
    }

    /// Confirm and re-run the install with `classic: true`.
    pub async fn confirm_classic(&mut self) {
        let Some((name, channel)) = self.classic_pending.take() else {
            return;
        };
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
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }
        self.loading = false;
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
                let mut state = ListState::default();
                state.select(Some(0));
                self.channel_picker_state = state;
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

    pub fn selected_manage_action(&self) -> Option<&ManageAction> {
        let idx = self.manage_state.selected()?;
        self.manage_actions.get(idx)
    }

    pub fn selected_change(&self) -> Option<&Change> {
        let idx = self.changes_list_state.selected()?;
        self.changes_list.get(idx)
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
        let name = match self.selected_snap().map(|s| s.name.clone()) {
            Some(n) => n,
            None => return,
        };
        self.execute_action(name, action, None).await;
    }

    pub async fn activate_selected_connection(&mut self) {
        let Some(item) = self.selected_connection() else {
            return;
        };
        if item.connected {
            self.disconnect_selected().await;
        } else {
            self.connect_selected().await;
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

    async fn do_connect_to_slot(&mut self, plug: &ConnectionItem, target: SlotRef) {
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
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }
        self.loading = false;
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
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }
        self.loading = false;
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

    async fn execute_action(&mut self, name: String, action: ManageAction, channel: Option<&str>) {
        if self.active_change_id.is_some() {
            self.status_message = Some("Operation already in progress".to_string());
            return;
        }

        self.loading = true;
        self.error = None;
        self.status_message = None;

        let result: Result<&str, snapd_rs::Error> = match &action {
            ManageAction::Install => match self.client.install_snap(&name, None).await {
                Ok(change_id) => {
                    self.active_change_id = Some(change_id.0);
                    self.active_change = None;
                    Ok("Installing…")
                }
                Err(e) if e.is_kind("snap-needs-classic") => {
                    self.loading = false;
                    self.classic_pending = Some((name, None));
                    self.mode = AppMode::ClassicConfirm;
                    return;
                }
                Err(e) => Err(e),
            },
            ManageAction::InstallFromChannel => {
                match self.client.install_snap(&name, channel).await {
                    Ok(change_id) => {
                        self.active_change_id = Some(change_id.0);
                        self.active_change = None;
                        Ok("Installing…")
                    }
                    Err(e) if e.is_kind("snap-needs-classic") => {
                        self.loading = false;
                        self.classic_pending = Some((name, channel.map(str::to_owned)));
                        self.mode = AppMode::ClassicConfirm;
                        return;
                    }
                    Err(e) => Err(e),
                }
            }
            ManageAction::Refresh => match self.client.refresh_snap(&name, None).await {
                Ok(change_id) => {
                    self.active_change_id = Some(change_id.0);
                    self.active_change = None;
                    Ok("Refreshing…")
                }
                Err(e) => Err(e),
            },
            ManageAction::SwitchChannel => match self.client.refresh_snap(&name, channel).await {
                Ok(change_id) => {
                    self.active_change_id = Some(change_id.0);
                    self.active_change = None;
                    Ok("Switching channel…")
                }
                Err(e) => Err(e),
            },
            ManageAction::Revert => match self.client.revert_snap(&name).await {
                Ok(change_id) => {
                    self.active_change_id = Some(change_id.0);
                    self.active_change = None;
                    Ok("Reverting…")
                }
                Err(e) => Err(e),
            },
            ManageAction::Enable => match self.client.enable_snap(&name).await {
                Ok(change_id) => {
                    self.active_change_id = Some(change_id.0);
                    self.active_change = None;
                    Ok("Enabling…")
                }
                Err(e) => Err(e),
            },
            ManageAction::Disable => match self.client.disable_snap(&name).await {
                Ok(change_id) => {
                    self.active_change_id = Some(change_id.0);
                    self.active_change = None;
                    Ok("Disabling…")
                }
                Err(e) => Err(e),
            },
            ManageAction::Uninstall => match self.client.remove_snap(&name).await {
                Ok(change_id) => {
                    self.active_change_id = Some(change_id.0);
                    self.active_change = None;
                    Ok("Uninstalling…")
                }
                Err(e) => Err(e),
            },
            ManageAction::OpenStorePage => {
                open_url(&format!("https://snapcraft.io/{name}"));
                Ok("Opened store page")
            }
            ManageAction::OpenContactPage => {
                if let Some(contact) = self
                    .installed
                    .iter()
                    .find(|s| s.name == name)
                    .and_then(|s| s.contact.as_deref())
                {
                    open_url(contact);
                }
                Ok("Opened contact page")
            }
        };

        self.loading = false;

        match result {
            Ok(msg) => {
                self.status_message = Some(msg.to_string());
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }
    }

    pub async fn tick(&mut self) {
        if let Some(id) = self.active_change_id.clone() {
            match self.client.get_change(&id).await {
                Ok(change) => {
                    let ready = change.ready;
                    let err = change.err.clone();
                    self.active_change = Some(change);
                    if ready {
                        self.active_change_id = None;
                        self.active_change = None;
                        let in_connections =
                            self.connections_mode || self.mode == AppMode::SlotPicker;
                        if in_connections {
                            // Connection/disconnect complete — stay in manage panel
                            // and reload the interfaces so the connected state updates.
                            if let Some(name) = self.selected_snap().map(|s| s.name) {
                                self.load_snap_interfaces(&name).await;
                            }
                        } else if matches!(
                            self.mode,
                            AppMode::Manage
                                | AppMode::ChannelPicker
                                | AppMode::ChannelInput
                                | AppMode::ClassicConfirm
                        ) {
                            self.close_manage();
                        }
                        if let Some(error) = err {
                            self.error = Some(error);
                        } else {
                            self.status_message = Some("Done".to_string());
                            self.reload().await;
                        }
                    }
                }
                Err(e) => {
                    self.error = Some(e.to_string());
                    self.active_change_id = None;
                    self.active_change = None;
                }
            }
        }

        if self.show_changes_sidebar {
            self.poll_sidebar_changes().await;
        }

        if let Some(icon_url) = self.selected_snap().and_then(|snap| snap.icon_url) {
            self.fetch_icon_if_needed(icon_url).await;
        }
    }

    pub async fn poll_sidebar_changes(&mut self) {
        if let Ok(changes) = self.client.list_changes().await {
            self.sidebar_changes = changes;
        }
    }

    pub async fn load_changes(&mut self) {
        self.loading = true;
        self.error = None;
        match self.client.list_all_changes().await {
            Ok(changes) => {
                self.changes_list = changes;
                self.changes_focus_detail = false;
                self.changes_list_state
                    .select((!self.changes_list.is_empty()).then_some(0));
                self.changes_detail_state.select(
                    self.selected_change()
                        .and_then(|change| (!change.tasks.is_empty()).then_some(0)),
                );
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }
        self.loading = false;
    }

    pub async fn abort_selected_change(&mut self) {
        let Some(change) = self.selected_change().cloned() else {
            return;
        };
        if change.ready {
            self.status_message = Some("Change already finished".to_string());
            return;
        }

        self.loading = true;
        self.error = None;
        match self.client.abort_change(&change.id).await {
            Ok(_) => {
                self.status_message = Some("Abort requested".to_string());
                self.load_changes().await;
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }
        self.loading = false;
    }

    pub async fn reload(&mut self) {
        // Prefer the explicitly-recorded managed snap name (survives close_manage) over
        // inferring from list_state, which can be ambiguous or stale.
        let selected_name = self
            .managed_snap_name
            .clone()
            .or_else(|| self.selected_snap().map(|s| s.name));
        let refresh_search = self.showing_results && !self.search_query.is_empty();

        self.load_installed().await;
        if refresh_search {
            self.perform_search().await;
        }

        self.restore_selection_by_name(selected_name.as_deref());
    }

    pub async fn load_installed(&mut self) {
        self.loading = true;
        self.error = None;
        match self.client.list_snaps().await {
            Ok(snaps) => {
                self.installed = snaps;
                self.showing_results = false;
                self.list_state.select(Some(0));
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }
        self.loading = false;
    }

    pub async fn fetch_icon_if_needed(&mut self, url: String) {
        if self.icon_picker.is_none()
            || self.icon_cache.contains_key(&url)
            || self.icon_fetching.contains(&url)
        {
            return;
        }

        self.icon_fetching.insert(url.clone());

        // Local installed snaps expose their icon via the snapd socket at
        // /v2/icons/<name>/icon — use the snapd client for those.
        // Store snaps have an absolute HTTPS URL — use reqwest for those.
        let image = if url.starts_with("/v2/icons/") {
            let snap_name = url
                .trim_start_matches("/v2/icons/")
                .trim_end_matches("/icon");
            self.client
                .get_snap_icon(snap_name)
                .await
                .ok()
                .and_then(|b| image::load_from_memory(&b).ok())
        } else {
            match reqwest::get(&url).await {
                Ok(response) => response
                    .bytes()
                    .await
                    .ok()
                    .and_then(|b| image::load_from_memory(&b).ok()),
                Err(_) => None,
            }
        };

        self.icon_cache.insert(url.clone(), image);
        self.icon_fetching.remove(&url);
    }

    pub async fn perform_search(&mut self) {
        if self.search_query.is_empty() {
            self.showing_results = false;
            self.list_state.select(Some(0));
            return;
        }

        self.loading = true;
        self.error = None;
        let query = self.search_query.clone();
        let (fuzzy_result, exact_result) = {
            let client = &self.client;
            tokio::join!(client.find_snaps(&query), client.find_snap_by_name(&query))
        };

        let mut results = fuzzy_result.as_ref().ok().cloned().unwrap_or_default();
        if let Ok(Some(exact)) = &exact_result
            && !results.iter().any(|result| result.name == exact.name)
        {
            results.insert(0, exact.clone());
        }

        if results.is_empty()
            && let Some(error) = fuzzy_result.err().or_else(|| exact_result.err())
        {
            self.error = Some(error.to_string());
        }

        self.store_results = results;
        self.showing_results = true;
        self.list_state.select(Some(0));
        self.loading = false;
    }

    #[allow(dead_code)]
    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.store_results.clear();
        self.showing_results = false;
        self.list_state.select(Some(0));
    }

    pub fn connection_items(&self) -> Vec<ConnectionItem> {
        let Some(snap) = self.selected_snap() else {
            return vec![];
        };
        self.connection_items_for_snap(&snap.name)
    }

    fn connection_items_for_snap(&self, snap_name: &str) -> Vec<ConnectionItem> {
        let mut items = Vec::new();
        for interface in &self.snap_interfaces {
            for plug in interface
                .plugs
                .iter()
                .filter(|plug| plug.snap.as_deref() == Some(snap_name))
            {
                // Find any active connection for this plug from /v2/connections.
                let active = self.snap_connections.iter().find(|c| {
                    c.plug.snap == plug.snap.as_deref().unwrap_or("") && c.plug.plug == plug.plug
                });
                items.push(ConnectionItem {
                    interface_name: interface.name.clone(),
                    plug_snap: plug.snap.clone().unwrap_or_else(|| snap_name.to_string()),
                    plug_name: plug.plug.clone(),
                    slot_snap: active.map(|c| c.slot.snap.clone()).unwrap_or_default(),
                    slot_name: active.map(|c| c.slot.slot.clone()).unwrap_or_default(),
                    connected: active.is_some(),
                    is_plug: true,
                });
            }

            for slot in interface
                .slots
                .iter()
                .filter(|slot| slot.snap.as_deref() == Some(snap_name))
            {
                let active = self.snap_connections.iter().find(|c| {
                    c.slot.snap == slot.snap.as_deref().unwrap_or("") && c.slot.slot == slot.slot
                });
                items.push(ConnectionItem {
                    interface_name: interface.name.clone(),
                    plug_snap: active.map(|c| c.plug.snap.clone()).unwrap_or_default(),
                    plug_name: active.map(|c| c.plug.plug.clone()).unwrap_or_default(),
                    slot_snap: slot.snap.clone().unwrap_or_else(|| snap_name.to_string()),
                    slot_name: slot.slot.clone(),
                    connected: active.is_some(),
                    is_plug: false,
                });
            }
        }

        items.sort_by(|a, b| {
            a.interface_name
                .cmp(&b.interface_name)
                .then_with(|| a.is_plug.cmp(&b.is_plug))
                .then_with(|| a.plug_name.cmp(&b.plug_name))
                .then_with(|| a.slot_name.cmp(&b.slot_name))
        });
        items
    }

    fn restore_selection_by_name(&mut self, name: Option<&str>) {
        let snaps = self.display_snaps();
        let selected = name
            .and_then(|name| snaps.iter().position(|snap| snap.name == name))
            .or_else(|| (!snaps.is_empty()).then_some(0));
        self.list_state.select(selected);
    }
}

fn empty_channel_info() -> ChannelSnapInfo {
    ChannelSnapInfo {
        revision: None,
        confinement: None,
        version: None,
        channel: None,
        size: None,
        released_at: None,
    }
}

fn channel_sort_key(channel: &str) -> (u8, String, u8, String) {
    let mut parts = channel.split('/');
    let first = parts.next().unwrap_or_default();
    let second = parts.next();
    let (track, risk, branch) = match second {
        Some(risk) => (first, risk, parts.collect::<Vec<_>>().join("/")),
        None => ("latest", first, String::new()),
    };
    let track_rank = if track == "latest" { 0 } else { 1 };
    let risk_rank = match risk {
        "stable" => 0,
        "candidate" => 1,
        "beta" => 2,
        "edge" => 3,
        _ => 4,
    };

    (track_rank, track.to_string(), risk_rank, branch)
}

fn open_url(url: &str) {
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}
