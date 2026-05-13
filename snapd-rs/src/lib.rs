pub mod api;
pub mod client;
pub mod error;
pub mod types;

pub use api::changes::{Change, Task, TaskProgress};
pub use api::store::{ChannelSnapInfo, StoreSnap};
pub use client::SnapdClient;
pub use client::{SNAPD_SNAP_SOCKET, SNAPD_SOCKET, SocketAddress};
pub use error::{Error, Result};
pub use types::{
    AliasStatusKind, ChangeId, ChangeStatus, DaemonScope, DaemonType, NoticeType, PromptLifespan,
    PromptOutcome, Revision, SnapConfinement, SnapStatus, SnapType, SystemMode, ValidationSetMode,
};
