# snapd API Value Types

This document describes all constrained-value string fields in the snapd REST API
and their corresponding Rust enum types in `snapd-rs`.

## SnapConfinement

The confinement level of a snap.

| JSON value | Rust variant | Description |
|------------|--------------|-------------|
| `"strict"` | `Strict` | Full confinement with security sandbox |
| `"classic"` | `Classic` | No sandbox, full system access |
| `"devmode"` | `Devmode` | Security sandbox in non-enforcing mode |

Used in: `Snap.confinement`

## SnapType

The type of a snap package.

| JSON value | Rust variant | Description |
|------------|--------------|-------------|
| `"app"` | `App` | Application snap (default) |
| `"kernel"` | `Kernel` | Kernel snap |
| `"gadget"` | `Gadget` | Gadget snap (board-specific config) |
| `"os"` | `Os` | OS snap (deprecated, see `core`) |
| `"base"` | `Base` | Base snap providing a root filesystem |
| `"core"` | `Core` | Core snap (runtime environment) |
| `"snapd"` | `Snapd` | The snapd snap itself |

Used in: `Snap.type_`, `StoreSnap.type_`

## SnapStatus

The installation status of a snap.

| JSON value | Rust variant | Description |
|------------|--------------|-------------|
| `"installed"` | `Installed` | Snap is installed but not currently active |
| `"active"` | `Active` | Snap is installed and active (current revision) |
| `"available"` | `Available` | Snap is available in the store but not installed |
| `"removed"` | `Removed` | Snap has been removed |

Used in: `Snap.status`

## DaemonType

The type of daemon/service a snap app runs as.

| JSON value | Rust variant | Description |
|------------|--------------|-------------|
| `"simple"` | `Simple` | Long-running process (default daemon type) |
| `"forking"` | `Forking` | Forks and parent exits (classic daemon pattern) |
| `"oneshot"` | `Oneshot` | Runs to completion then exits |
| `"dbus"` | `Dbus` | Activated via D-Bus |
| `"notify"` | `Notify` | Like simple, but signals readiness via sd_notify |

Used in: `AppInfo.daemon`, `SnapApp.daemon`

## DaemonScope

The scope at which a daemon service operates.

| JSON value | Rust variant | Description |
|------------|--------------|-------------|
| `"system"` | `System` | System-wide service |
| `"user"` | `User` | Per-user service |

Used in: `AppInfo.daemon_scope`, `SnapApp.daemon_scope`

## AliasStatusKind

The status of a snap command alias.

| JSON value | Rust variant | Description |
|------------|--------------|-------------|
| `"auto"` | `Auto` | Alias is automatically enabled by the snap |
| `"manual"` | `Manual` | Alias was manually set by the user |
| `"disabled"` | `Disabled` | Alias is disabled |

Used in: `AliasStatus.status`

## ChangeStatus

The status of an async change or individual task.

| JSON value | Rust variant | Description |
|------------|--------------|-------------|
| `"Do"` | `Do` | Queued, not yet started |
| `"Doing"` | `Doing` | Currently in progress |
| `"Done"` | `Done` | Completed successfully |
| `"Abort"` | `Abort` | Abort requested |
| `"Aborting"` | `Aborting` | Currently aborting |
| `"Error"` | `Error` | Failed with an error |
| `"Hold"` | `Hold` | On hold |
| `"Wait"` | `Wait` | Waiting for external input |
| `"Undone"` | `Undone` | Successfully undone/rolled back |
| `"Undoing"` | `Undoing` | Currently rolling back |

Used in: `Change.status`, `Task.status`

## SystemMode

The recovery/operating mode of the system.

| JSON value | Rust variant | Description |
|------------|--------------|-------------|
| `"run"` | `Run` | Normal operating mode |
| `"recover"` | `Recover` | Recovery mode |
| `"install"` | `Install` | Factory reset / installation mode |

Used in: `SystemAction.mode`, `SystemInfo.system_mode`

## ValidationSetMode

The enforcement mode of a validation set.

| JSON value | Rust variant | Description |
|------------|--------------|-------------|
| `"enforce"` | `Enforce` | Validation set is enforced (snaps must match) |
| `"monitor"` | `Monitor` | Validation set is monitored only (mismatches reported) |

Used in: `ValidationSet.mode`

## NoticeType

The type of a notice event.

| JSON value | Rust variant | Description |
|------------|--------------|-------------|
| `"snap-run-inhibit"` | `SnapRunInhibit` | A snap is inhibited from running |
| `"interfaces-requests-prompt"` | `InterfacesRequestsPrompt` | An interface access prompt is pending |
| `"interfaces-requests-rule-update"` | `InterfacesRequestsRuleUpdate` | A prompting rule was created/changed/removed |
| `"change-update"` | `ChangeUpdate` | An async change has been updated |
| `"warning"` | `Warning` | A warning has been issued |
| `"refresh-inhibit"` | `RefreshInhibit` | An auto-refresh is inhibited |

Used in: `Notice.type_`

## PromptOutcome

The outcome/decision for an interface access prompt.

| JSON value | Rust variant | Description |
|------------|--------------|-------------|
| `"allow"` | `Allow` | Access is allowed |
| `"deny"` | `Deny` | Access is denied |

Used in: `PromptRule.outcome`, `reply_to_prompt()`

## PromptLifespan

How long a prompt rule or reply should remain in effect.

| JSON value | Rust variant | Description |
|------------|--------------|-------------|
| `"single"` | `Single` | Applies to this single request only |
| `"session"` | `Session` | Applies for the current user session |
| `"forever"` | `Forever` | Applies permanently |
| `"timespan"` | `Timespan` | Applies until a specified expiration time |

Used in: `PromptRule.lifespan`, `reply_to_prompt()`

## ThemeStatusKind

Availability status for a requested theme.

| JSON value | Rust variant | Description |
|------------|--------------|-------------|
| `"installed"` | `Installed` | Theme is already installed |
| `"available"` | `Available` | Theme is available to install |
| `"unavailable"` | `Unavailable` | Theme is not available |

Used in: `ThemeStatus.gtk_themes`, `ThemeStatus.icon_themes`, `ThemeStatus.sound_themes`

## FdeStatus

Disk encryption activation status for the current boot.

| JSON value | Rust variant |
|------------|--------------|
| `"indeterminate"` | `Indeterminate` |
| `"active"` | `Active` |
| `"inactive"` | `Inactive` |
| `"recovery"` | `Recovery` |
| `"degraded"` | `Degraded` |

Used in: `StorageEncryptionStatus.status`

## AutoRepairResult

Auto-repair outcome for storage encryption.

| JSON value | Rust variant |
|------------|--------------|
| `"not-initialized"` | `NotInitialized` |
| `"not-attempted"` | `NotAttempted` |
| `"failed-platform-init"` | `FailedPlatformInit` |
| `"failed-keyslots"` | `FailedKeyslots` |
| `"failed-encryption-support"` | `FailedEncryptionSupport` |
| `"success"` | `Success` |

Used in: `StorageEncryptionStatus.auto_repair_result`
