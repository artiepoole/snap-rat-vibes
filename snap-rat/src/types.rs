use snapd_rs_artie::{SnapConfinement, api::snaps::Snap, api::store::StoreSnap};

/// Unified view of a snap (installed local or from store search results).
#[derive(Debug, Clone)]
pub struct DisplaySnap {
    pub name: String,
    pub title: Option<String>,
    pub version: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub confinement: Option<SnapConfinement>,
    pub channel: Option<String>,
    pub contact: Option<String>,
    pub icon_url: Option<String>,
    pub size: Option<u64>,
    pub installed: bool,
    /// ISO 8601 install timestamp from snapd. None for uninstalled store results.
    pub install_date: Option<String>,
    /// If true, this is a local file match rather than a snapd result.
    pub is_local_file: bool,
    pub local_file_path: Option<String>,
}

impl From<&Snap> for DisplaySnap {
    fn from(s: &Snap) -> Self {
        Self {
            name: s.name.clone(),
            title: s.title.clone(),
            version: s.version.clone(),
            summary: s.summary.clone(),
            description: s.description.clone(),
            publisher: s
                .publisher
                .as_ref()
                .and_then(|p| p.display_name.clone().or_else(|| p.username.clone())),
            confinement: s.confinement.clone(),
            channel: s.tracking_channel.clone().or_else(|| s.channel.clone()),
            contact: s.contact.clone(),
            icon_url: s.icon.clone(),
            size: s.installed_size,
            installed: true,
            install_date: s.install_date.clone(),
            is_local_file: false,
            local_file_path: None,
        }
    }
}

impl From<&StoreSnap> for DisplaySnap {
    fn from(s: &StoreSnap) -> Self {
        Self {
            name: s.name.clone(),
            title: s.title.clone(),
            version: s.version.clone(),
            summary: s.summary.clone(),
            description: s.description.clone(),
            publisher: s
                .publisher
                .as_ref()
                .and_then(|p| p.display_name.clone().or_else(|| p.username.clone())),
            confinement: None,
            channel: s.channel.clone(),
            contact: None,
            icon_url: s.icon.clone(),
            size: s.download_size.map(|value| value.max(0) as u64),
            installed: false,
            install_date: None,
            is_local_file: false,
            local_file_path: None,
        }
    }
}

pub fn open_url(url: &str) {
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}
