use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use image::DynamicImage;
use ratatui::{layout::Rect, widgets::ListState};
use ratatui_image::picker::{Capability, Picker, cap_parser::QueryStdioOptions};
use snapd_rs::{
    AppInfo, Change, ChannelSnapInfo, ComponentInfo, SnapdClient, StoreSnap,
    api::{
        interfaces::{Connection, Interface, SlotRef},
        snaps::Snap,
    },
};

use crate::types::DisplaySnap;

/// Which panel is shown on the right side of the manage pane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RightPane {
    /// No pane selected yet — right side shows a placeholder.
    None,
    Connections,
    Components,
    Services,
}

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
    /// Confirmation overlay for a destructive action (Uninstall, Revert,
    /// Disable, or toggling a connection).
    Confirm,
}

/// The action that will execute once the user confirms the `Confirm` dialog.
#[derive(Debug, Clone)]
pub enum ConfirmPending {
    Action(ManageAction),
    Connect,
    Disconnect,
    /// Auto-connect a plug to a system slot after install.
    AutoConnect {
        plug_snap: String,
        plug_name: String,
        interface_name: String,
        slot: SlotRef,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortMode {
    Relevance,
    NameAsc,
    NameDesc,
    RevisionDesc,
}

impl SortMode {
    pub fn label(&self) -> &'static str {
        match self {
            SortMode::Relevance => "Relevance",
            SortMode::NameAsc => "A→Z",
            SortMode::NameDesc => "Z→A",
            SortMode::RevisionDesc => "Recently installed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManageAction {
    Install,
    InstallFromChannel,
    InstallLocalFile,
    Refresh,
    SwitchChannel,
    Revert,
    Enable,
    Disable,
    Uninstall,
    UninstallPurge,
    OpenConnections,
    OpenComponents,
    OpenServices,
    OpenStorePage,
    OpenContactPage,
}

impl ManageAction {
    pub fn label(&self) -> &'static str {
        match self {
            ManageAction::Install => "Install",
            ManageAction::InstallFromChannel => "Install from channel…",
            ManageAction::InstallLocalFile => "Install from local file",
            ManageAction::Refresh => "Refresh to latest",
            ManageAction::SwitchChannel => "Switch channel…",
            ManageAction::Revert => "Revert to previous version",
            ManageAction::Enable => "Enable",
            ManageAction::Disable => "Disable",
            ManageAction::Uninstall => "Uninstall",
            ManageAction::UninstallPurge => "Uninstall and purge data",
            ManageAction::OpenConnections => "Connections →",
            ManageAction::OpenComponents => "Components →",
            ManageAction::OpenServices => "Services →",
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
    pub local_file_results: Vec<DisplaySnap>,
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
    /// True from when the manage pane is opened until the user makes their first explicit
    /// click/keypress to select an item. Used so the pre-selected index 0 doesn't count
    /// as an "already selected" second click.
    pub manage_activated: bool,
    /// Same sentinel for the right-pane sub-pane.
    pub connections_activated: bool,
    pub active_change_id: Option<String>,
    pub active_change: Option<Change>,
    /// The action that started the current active change (used to trigger post-install prompts).
    pub active_change_action: Option<ManageAction>,
    /// The snap name the active change is operating on.
    pub active_change_snap: Option<String>,
    pub show_changes_sidebar: bool,
    pub show_help: bool,
    pub sidebar_changes: Vec<Change>,
    pub changes_list: Vec<Change>,
    pub changes_list_state: ListState,
    pub changes_detail_state: ListState,
    pub changes_focus_detail: bool,
    /// When the Changes view was last polled, used to throttle background refreshes.
    pub changes_last_polled: Option<Instant>,
    pub available_channels: Vec<(String, ChannelSnapInfo)>,
    pub channel_picker_state: ListState,
    pub channel_input: String,
    pub pending_channel_action: Option<ManageAction>,
    /// Snap name / channel waiting for classic confirmation.
    pub classic_pending: Option<(String, Option<String>)>,
    /// Path of a local snap file waiting for classic confirmation.
    pub classic_local_path: Option<String>,
    /// Name of the snap currently open in the manage panel.
    /// Persists through close_manage so reload() can restore the selection.
    pub managed_snap_name: Option<String>,
    pub snap_interfaces: Vec<Interface>,
    /// Active connections fetched from /v2/connections — used to determine
    /// connected state because select=all does not populate Plug.connections.
    pub snap_connections: Vec<Connection>,
    pub interfaces_loading: bool,
    /// Whether the right-side pane (connections / components / services) has keyboard focus.
    pub right_pane_focused: bool,
    /// Which panel is currently shown on the right side of the manage pane.
    pub active_right_pane: RightPane,
    pub connections_state: ListState,
    /// Snap components, populated lazily when the Components right pane is opened.
    pub snap_components: Vec<ComponentInfo>,
    pub components_state: ListState,
    pub components_activated: bool,
    pub components_loading: bool,
    /// Snap services (daemon apps), populated lazily when the Services right pane is opened.
    pub snap_services: Vec<AppInfo>,
    pub services_state: ListState,
    pub services_activated: bool,
    pub services_loading: bool,
    /// Plug being connected — shown in slot picker overlay.
    pub slot_picker_plug: Option<ConnectionItem>,
    /// Available slots to connect to (populated when entering SlotPicker mode).
    pub slot_picker_items: Vec<SlotRef>,
    pub slot_picker_state: ListState,
    pub icon_picker: Option<Picker>,
    pub icon_cache: HashMap<String, Option<DynamicImage>>,
    pub icon_fetching: HashSet<String>,

    /// Pending action waiting for confirmation in `AppMode::Confirm`.
    pub confirm_pending: Option<ConfirmPending>,
    /// Human-readable message shown in the confirmation dialog.
    pub confirm_message: Option<String>,
    /// Queue of auto-connect prompts to show after install.
    pub auto_connect_queue: Vec<ConfirmPending>,
    /// Clickable area for the Yes/Confirm button in the confirmation dialog.
    pub confirm_yes_area: Option<Rect>,
    /// Clickable area for the No/Cancel button in the confirmation dialog.
    pub confirm_no_area: Option<Rect>,
    /// Which confirm button is currently highlighted: Some(true)=yes, Some(false)=no, None=neither.
    pub confirm_hovered: Option<bool>,

    // Areas of interactive widgets, updated on every draw for mouse hit-testing.
    pub snap_list_area: Option<Rect>,
    pub search_area: Option<Rect>,
    pub manage_actions_area: Option<Rect>,
    /// Entire left pane (tabs + search + list), used so clicks outside the
    /// manage area can close the manage panel.
    pub left_pane_area: Option<Rect>,
    /// Inner area of the connections list (border + padding already removed).
    pub connections_inner_area: Option<Rect>,
    /// Inner area of the components list (border + padding already removed).
    pub components_inner_area: Option<Rect>,
    /// Inner area of the services list (border + padding already removed).
    pub services_inner_area: Option<Rect>,
    pub channel_picker_area: Option<Rect>,
    pub slot_picker_area: Option<Rect>,
    pub channel_input_area: Option<Rect>,
    pub classic_confirm_area: Option<Rect>,
    pub changes_list_area: Option<Rect>,
    pub changes_detail_area: Option<Rect>,
    /// Clickable tab areas rendered at the top of the left pane.
    pub snaps_tab_area: Option<Rect>,
    pub changes_tab_area: Option<Rect>,
    /// Area of the help dialog popup, used for click-outside-to-close.
    pub help_area: Option<Rect>,
}

impl App {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            client: SnapdClient::new(),
            installed: vec![],
            store_results: vec![],
            local_file_results: vec![],
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
            manage_activated: false,
            connections_activated: false,
            active_change_id: None,
            active_change: None,
            active_change_action: None,
            active_change_snap: None,
            show_changes_sidebar: false,
            show_help: false,
            sidebar_changes: vec![],
            changes_list: vec![],
            changes_list_state: ListState::default(),
            changes_detail_state: ListState::default(),
            changes_focus_detail: false,
            changes_last_polled: None,
            available_channels: vec![],
            channel_picker_state: ListState::default(),
            channel_input: String::new(),
            pending_channel_action: None,
            classic_pending: None,
            classic_local_path: None,
            managed_snap_name: None,
            snap_interfaces: vec![],
            snap_connections: vec![],
            interfaces_loading: false,
            right_pane_focused: false,
            active_right_pane: RightPane::None,
            connections_state: ListState::default(),
            snap_components: vec![],
            components_state: ListState::default(),
            components_activated: false,
            components_loading: false,
            snap_services: vec![],
            services_state: ListState::default(),
            services_activated: false,
            services_loading: false,
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
                picker
            }),
            icon_cache: HashMap::default(),
            icon_fetching: HashSet::default(),
            confirm_pending: None,
            confirm_message: None,
            auto_connect_queue: Vec::new(),
            confirm_yes_area: None,
            confirm_no_area: None,
            confirm_hovered: None,
            snap_list_area: None,
            search_area: None,
            manage_actions_area: None,
            left_pane_area: None,
            connections_inner_area: None,
            components_inner_area: None,
            services_inner_area: None,
            channel_picker_area: None,
            slot_picker_area: None,
            channel_input_area: None,
            classic_confirm_area: None,
            changes_list_area: None,
            changes_detail_area: None,
            snaps_tab_area: None,
            changes_tab_area: None,
            help_area: None,
        }
    }

    pub fn toggle_focus(&mut self) {
        self.search_focused = !self.search_focused;
    }

    /// True when the current process is running as root (uid 0).
    pub fn is_root() -> bool {
        unsafe { libc::getuid() == 0 }
    }

    /// If `e` is an elevation error and we are not already root, serialize the
    /// current app state along with the intended `action` and re-exec under
    /// pkexec/sudo. If we are already root, this is a no-op (error falls
    /// through to the normal error display path).
    pub fn try_elevate_and_exec(
        &self,
        snap_name: &str,
        action: Option<crate::resume::ResumeAction>,
    ) {
        if Self::is_root() {
            return;
        }
        let resume = crate::resume::ResumeState {
            selected_snap: Some(snap_name.to_string()),
            search_query: self.search_query.clone(),
            show_installed_only: self.show_installed_only,
            pending: action,
        };
        crate::resume::reexec_elevated(&resume);
    }

    /// Restore search state, snap selection, and execute the pending action
    /// from a `--resume` argument. Called once after initial load when the
    /// process was re-exec'd with elevated privileges.
    pub async fn apply_resume(&mut self, state: crate::resume::ResumeState) {
        self.search_query = state.search_query;
        self.show_installed_only = state.show_installed_only;

        // Restore list position by snap name and pin managed_snap_name so
        // reload() after the change completes can restore the selection.
        if let Some(ref name) = state.selected_snap {
            let snaps = self.display_snaps();
            if let Some(idx) = snaps.iter().position(|s| &s.name == name) {
                self.list_state.select(Some(idx));
            }
            self.managed_snap_name = Some(name.clone());
        }

        // Re-run the action that triggered elevation (already confirmed by the user).
        if let Some(action) = state.pending {
            self.execute_resume_action(action).await;
        }
    }

    /// Execute a resume action directly, bypassing confirm dialogs.
    pub async fn execute_resume_action(&mut self, action: crate::resume::ResumeAction) {
        use crate::resume::ResumeAction;
        match action {
            ResumeAction::Install { snap_name, channel } => {
                self.execute_action(snap_name, ManageAction::Install, channel.as_deref())
                    .await;
            }
            ResumeAction::InstallClassic { snap_name, channel } => {
                self.loading = true;
                self.error = None;
                self.status_message = None;
                match self
                    .client
                    .install_snap_classic(&snap_name, channel.as_deref())
                    .await
                {
                    Ok(change_id) => {
                        self.active_change_id = Some(change_id.0);
                        self.active_change = None;
                        self.status_message = Some("Installing (classic)…".to_string());
                        self.active_change_action = Some(ManageAction::InstallFromChannel);
                        self.active_change_snap = Some(snap_name);
                    }
                    Err(e) => {
                        self.error = Some(e.to_string());
                    }
                }
                self.loading = false;
            }
            ResumeAction::Refresh { snap_name, channel } => {
                self.execute_action(snap_name, ManageAction::Refresh, channel.as_deref())
                    .await;
            }
            ResumeAction::SwitchChannel { snap_name, channel } => {
                self.execute_action(snap_name, ManageAction::SwitchChannel, Some(&channel))
                    .await;
            }
            ResumeAction::Revert { snap_name } => {
                self.execute_action(snap_name, ManageAction::Revert, None)
                    .await;
            }
            ResumeAction::Enable { snap_name } => {
                self.execute_action(snap_name, ManageAction::Enable, None)
                    .await;
            }
            ResumeAction::Disable { snap_name } => {
                self.execute_action(snap_name, ManageAction::Disable, None)
                    .await;
            }
            ResumeAction::Uninstall { snap_name } => {
                self.execute_action(snap_name, ManageAction::Uninstall, None)
                    .await;
            }
            ResumeAction::UninstallPurge { snap_name } => {
                self.execute_action(snap_name, ManageAction::UninstallPurge, None)
                    .await;
            }
        }
    }

    pub fn toggle_changes_sidebar(&mut self) {
        self.show_changes_sidebar = !self.show_changes_sidebar;
        if self.show_changes_sidebar {
            self.sidebar_changes.clear();
        }
    }

    pub fn display_snaps(&self) -> Vec<DisplaySnap> {
        let query = self.search_query.trim().to_lowercase();

        let mut snaps: Vec<DisplaySnap> = if self.showing_results {
            let installed_names: std::collections::HashSet<&str> =
                self.installed.iter().map(|s| s.name.as_str()).collect();
            let mut results: Vec<DisplaySnap> = self
                .store_results
                .iter()
                .filter(|s| !self.show_installed_only || installed_names.contains(s.name.as_str()))
                .map(|s| {
                    let mut d = DisplaySnap::from(s);
                    if installed_names.contains(s.name.as_str()) {
                        d.installed = true;
                    }
                    d
                })
                .collect();
            results.extend(self.local_file_results.clone());
            results
        } else {
            self.installed.iter().map(DisplaySnap::from).collect()
        };

        // When searching, default to Relevance unless user has explicitly changed sort.
        let effective_sort = &self.sort_mode;

        match effective_sort {
            SortMode::Relevance => {
                snaps.sort_by(|a, b| {
                    sort_group(a)
                        .cmp(&sort_group(b))
                        .then_with(|| {
                            let qa = match_quality(
                                &a.name,
                                a.title.as_deref(),
                                a.summary.as_deref(),
                                &query,
                            );
                            let qb = match_quality(
                                &b.name,
                                b.title.as_deref(),
                                b.summary.as_deref(),
                                &query,
                            );
                            qa.cmp(&qb)
                        })
                        .then_with(|| a.name.cmp(&b.name))
                });
            }
            SortMode::NameAsc => {
                snaps.sort_by(|a, b| {
                    sort_group(a).cmp(&sort_group(b)).then_with(|| {
                        a.name
                            .to_lowercase()
                            .cmp(&b.name.to_lowercase())
                            .then_with(|| a.name.cmp(&b.name))
                    })
                });
            }
            SortMode::NameDesc => {
                snaps.sort_by(|a, b| {
                    sort_group(a).cmp(&sort_group(b)).then_with(|| {
                        b.name
                            .to_lowercase()
                            .cmp(&a.name.to_lowercase())
                            .then_with(|| b.name.cmp(&a.name))
                    })
                });
            }
            SortMode::RevisionDesc => {
                snaps.sort_by(|a, b| {
                    sort_group(a).cmp(&sort_group(b)).then_with(|| {
                        b.install_date
                            .as_deref()
                            .unwrap_or_default()
                            .cmp(a.install_date.as_deref().unwrap_or_default())
                            .then_with(|| a.name.cmp(&b.name))
                    })
                });
            }
        }

        snaps
    }

    pub fn selected_snap(&self) -> Option<DisplaySnap> {
        let snaps = self.display_snaps();
        let idx = self.list_state.selected()?;
        snaps.get(idx).cloned()
    }

    /// Returns true if `action` is destructive and should require confirmation.
    pub(crate) async fn execute_action(
        &mut self,
        name: String,
        action: ManageAction,
        channel: Option<&str>,
    ) {
        if self.active_change_id.is_some() {
            self.status_message = Some("Operation already in progress".to_string());
            return;
        }

        self.loading = true;
        self.error = None;
        self.status_message = None;
        self.active_change_action = Some(action.clone());
        self.active_change_snap = Some(name.clone());

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
                    self.confirm_hovered = Some(false);
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
                        self.confirm_hovered = Some(false);
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
            ManageAction::UninstallPurge => match self.client.remove_snap_purge(&name).await {
                Ok(change_id) => {
                    self.active_change_id = Some(change_id.0);
                    self.active_change = None;
                    Ok("Uninstalling (purge)…")
                }
                Err(e) => Err(e),
            },
            ManageAction::InstallLocalFile => match self.client.sideload_snap(&name).await {
                Ok(change_id) => {
                    self.active_change_id = Some(change_id.0);
                    self.active_change = None;
                    Ok("Sideloading…")
                }
                Err(e) if e.is_kind("snap-needs-classic") => {
                    self.loading = false;
                    self.classic_local_path = Some(name);
                    self.confirm_hovered = Some(false);
                    self.mode = AppMode::ClassicConfirm;
                    return;
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
            // These are handled by execute_selected_action before reaching here.
            ManageAction::OpenConnections
            | ManageAction::OpenComponents
            | ManageAction::OpenServices => Ok(""),
        };

        self.loading = false;

        match result {
            Ok(msg) => {
                self.status_message = Some(msg.to_string());
            }
            Err(ref e) if crate::resume::is_elevation_needed(e) => {
                use crate::resume::ResumeAction;
                let resume_action = match &action {
                    ManageAction::Install => Some(ResumeAction::Install {
                        snap_name: name.clone(),
                        channel: channel.map(str::to_owned),
                    }),
                    ManageAction::InstallFromChannel => Some(ResumeAction::Install {
                        snap_name: name.clone(),
                        channel: channel.map(str::to_owned),
                    }),
                    ManageAction::InstallLocalFile => None,
                    ManageAction::Refresh => Some(ResumeAction::Refresh {
                        snap_name: name.clone(),
                        channel: channel.map(str::to_owned),
                    }),
                    ManageAction::SwitchChannel => channel.map(|ch| ResumeAction::SwitchChannel {
                        snap_name: name.clone(),
                        channel: ch.to_owned(),
                    }),
                    ManageAction::Revert => Some(ResumeAction::Revert {
                        snap_name: name.clone(),
                    }),
                    ManageAction::Enable => Some(ResumeAction::Enable {
                        snap_name: name.clone(),
                    }),
                    ManageAction::Disable => Some(ResumeAction::Disable {
                        snap_name: name.clone(),
                    }),
                    ManageAction::Uninstall => Some(ResumeAction::Uninstall {
                        snap_name: name.clone(),
                    }),
                    ManageAction::UninstallPurge => Some(ResumeAction::UninstallPurge {
                        snap_name: name.clone(),
                    }),
                    // Non-privileged actions — shouldn't fail with elevation error.
                    ManageAction::OpenStorePage
                    | ManageAction::OpenContactPage
                    | ManageAction::OpenConnections
                    | ManageAction::OpenComponents
                    | ManageAction::OpenServices => None,
                };
                self.try_elevate_and_exec(&name, resume_action);
                // Only reached if already root — show the error normally.
                self.error = Some(result.unwrap_err().to_string());
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
                        let action = self.active_change_action.take();
                        let snap_name = self.active_change_snap.take();
                        let in_connections =
                            self.right_pane_focused || self.mode == AppMode::SlotPicker;
                        if in_connections {
                            // Right-pane operation complete — stay in manage and reload the
                            // active pane data so the state reflects the change.
                            if let Some(name) = self.selected_snap().map(|s| s.name) {
                                match self.active_right_pane {
                                    crate::app::RightPane::None => {}
                                    crate::app::RightPane::Connections => {
                                        self.load_snap_interfaces(&name).await;
                                    }
                                    crate::app::RightPane::Components => {
                                        self.load_snap_components(&name).await;
                                    }
                                    crate::app::RightPane::Services => {
                                        self.load_snap_services(&name).await;
                                    }
                                }
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
                            // After a successful install, prompt to connect unconnected system interfaces.
                            if matches!(
                                action,
                                Some(ManageAction::Install)
                                    | Some(ManageAction::InstallFromChannel)
                            ) && let Some(name) = snap_name
                            {
                                self.queue_auto_connect_prompts(&name).await;
                            }
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

        // Poll the Changes view every 3 s while it is open.
        if self.mode == AppMode::Changes {
            let due = self
                .changes_last_polled
                .is_none_or(|t| t.elapsed() >= Duration::from_secs(3));
            if due {
                self.poll_changes().await;
            }
        }

        if let Some(icon_url) = self.selected_snap().and_then(|snap| snap.icon_url) {
            self.fetch_icon_if_needed(icon_url).await;
        }
    }

    /// Refresh the changes list while preserving the current selection.
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

        // Perform store search in parallel with local file search
        let (store_fuzzy, store_exact, local_files) = tokio::join!(
            {
                let client = &self.client;
                client.find_snaps(&query)
            },
            {
                let client = &self.client;
                client.find_snap_by_name(&query)
            },
            {
                let q = query.clone();
                async move {
                    use std::fs;
                    let mut files = Vec::new();
                    if let Ok(entries) = fs::read_dir(".") {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if let Some(name) = path
                                .file_name()
                                .and_then(|n| n.to_str().map(|s| s.to_owned()))
                                && name.to_lowercase().contains(&q.to_lowercase())
                                && (name.ends_with(".snap") || name.ends_with(".comp"))
                            {
                                let full_path = path
                                    .canonicalize()
                                    .ok()
                                    .and_then(|p| p.to_str().map(|s| s.to_owned()));
                                files.push(DisplaySnap {
                                    name,
                                    title: None,
                                    version: None,
                                    summary: None,
                                    description: None,
                                    publisher: None,
                                    confinement: None,
                                    channel: None,
                                    contact: None,
                                    icon_url: None,
                                    size: None,
                                    installed: false,
                                    install_date: None,
                                    is_local_file: true,
                                    local_file_path: full_path,
                                });
                            }
                        }
                    }
                    files
                }
            }
        );

        let mut results = store_fuzzy.as_ref().ok().cloned().unwrap_or_default();
        if let Ok(Some(exact)) = &store_exact
            && !results.iter().any(|result| result.name == exact.name)
        {
            results.insert(0, exact.clone());
        }

        // Merge local files into results
        self.local_file_results = local_files;

        if results.is_empty()
            && let Some(error) = store_fuzzy.err().or_else(|| store_exact.err())
        {
            self.error = Some(error.to_string());
        }

        self.store_results = results;
        self.showing_results = true;
        self.sort_mode = SortMode::Relevance;
        self.list_state.select(Some(0));
        self.loading = false;
    }

    #[allow(dead_code)]
    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.store_results.clear();
        self.local_file_results.clear();
        self.showing_results = false;
        self.list_state.select(Some(0));
    }

    pub(crate) fn connection_items_for_snap(&self, snap_name: &str) -> Vec<ConnectionItem> {
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

pub(crate) fn empty_channel_info() -> ChannelSnapInfo {
    ChannelSnapInfo {
        revision: None,
        confinement: None,
        version: None,
        channel: None,
        size: None,
        released_at: None,
    }
}

pub(crate) fn channel_sort_key(channel: &str) -> (u8, String, u8, String) {
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

fn sort_group(s: &DisplaySnap) -> u8 {
    if s.installed {
        0
    } else if s.is_local_file {
        1
    } else {
        2
    }
}

/// Score a snap against a search query. Lower = better match.
/// Tiers:
///   0 — exact name match
///   1 — name starts with query
///   2 — name contains query
///   3 — title starts with query
///   4 — title contains query
///   5 — summary contains query
///   6 — no match / anything else
fn match_quality(name: &str, title: Option<&str>, summary: Option<&str>, query: &str) -> u8 {
    if query.is_empty() {
        return 6;
    }
    let name_lc = name.to_lowercase();
    let q = query; // already lowercased by caller
    if name_lc == q {
        return 0;
    }
    if name_lc.starts_with(q) {
        return 1;
    }
    if name_lc.contains(q) {
        return 2;
    }
    if let Some(t) = title {
        let t_lc = t.to_lowercase();
        if t_lc.starts_with(q) {
            return 3;
        }
        if t_lc.contains(q) {
            return 4;
        }
    }
    if let Some(s) = summary
        && s.to_lowercase().contains(q)
    {
        return 5;
    }
    6
}

fn open_url(url: &str) {
    crate::types::open_url(url);
}
