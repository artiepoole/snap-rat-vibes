# Copilot Project Instructions

## Project overview

This is a Rust workspace with three crates:

| Crate | Path | Purpose |
|---|---|---|
| `snap-rat` | `snap-rat/` | TUI snap manager (ratatui + crossterm) — the main app |
| `snapd-rs` | `snapd-rs/` | Async snapd API client (Unix socket via hyper) |
| `libprompting` | `libprompting/` | Shared prompting utilities |

`snapd-rs` and `app-center` in the repo root are **reference only** — do not treat them as part of snap-rat.

---

## snap-rat architecture

### Entry points

- `snap-rat/src/main.rs` — calls `snap_rat::run()`
- `snap-rat/src/lib.rs` — crate root; event loop, top-level render dispatch, `mod` declarations

### Module layout (vertical slices — one directory per UI concern)

```
src/
  app.rs          Core App struct, all state fields, ManageAction/AppMode/ConfirmPending enums
  lib.rs          Event loop, render dispatch, mod declarations
  keyboard.rs     All keyboard event handling
  mouse.rs        All mouse event handling
  layout.rs       Shared layout helpers (centered_popup, format_size, truncate_text)
  types.rs        Shared types (DisplaySnap)
  browse/         Snap list + search bar
  changes/        Changes tab (active snapd changes)
  channels/       Channel picker overlay + custom channel input
  confirm/        Confirm dialog + classic-confirm dialog
  connections/    Connections pane (plug/slot management)
  detail/         Detail panel + status bar
  help/           Help overlay
  manage/         Manage pane (actions list)
  slots/          Slot picker overlay
```

Each subdirectory has `mod.rs` (re-exports), `render.rs` (drawing), and `state.rs` (logic/mutations on `App`).

### Key types in `app.rs`

```rust
pub enum AppMode {
    Browse, Manage, ChannelPicker, ChannelInput,
    ClassicConfirm, Changes, SlotPicker, Confirm,
}

pub enum ManageAction {
    Install, InstallFromChannel, Refresh, SwitchChannel,
    Revert, Enable, Disable, Uninstall, UninstallPurge,
    OpenStorePage, OpenContactPage,
}

pub enum ConfirmPending {
    Action(ManageAction),
    Connect,
    Disconnect,
    AutoConnect { plug_snap, plug_name, interface_name, slot },
}
```

### Interaction model

- **Single click** on a list item → selects/highlights it
- **Double click** (or second click on already-selected) → executes
- **Confirm dialogs**: first click highlights a button, second click confirms; Enter confirms the highlighted button (default: No/Cancel); `h`/`←` moves to Yes, `l`/`→` moves to No; `y` confirms directly, `n`/`Esc` cancels
- Tab switching: `c` → Changes, `s` → Browse/Snaps (from any non-popup context); switching tabs preserves browse/manage state (no `close_manage()` on tab switch)

### Keyboard bindings summary

| Key | Context | Action |
|---|---|---|
| `j` / `↓` | Browse/Manage | Next item |
| `k` / `↑` | Browse/Manage | Prev item |
| `l` / `→` / `↵` | Browse | Open manage panel / confirm search |
| `h` / `←` / `Esc` | Browse | Close manage / cancel search |
| `l` / `→` / `↵` | Manage | Select action / confirm |
| `h` / `←` / `Esc` | Manage | Close manage panel |
| `h` / `←` | Confirm | Move focus to Yes |
| `l` / `→` | Confirm | Move focus to No (default) |
| `Tab` | Manage | Toggle actions ↔ connections pane |
| `/` | Browse | Focus search |
| `Delete` | Search focused | Clear entire search query |
| `Backspace` | Search focused | Delete last char |
| `i` | Browse | Toggle installed-only filter |
| `o` | Browse | Cycle sort order |
| `r` | Browse/Changes | Refresh |
| `p` | Browse/Manage/Changes | Toggle changes sidebar |
| `c` | Any non-popup | Switch to Changes tab |
| `s` | Any non-popup | Switch to Snaps/Browse tab |
| `?` / `F1` | Global | Toggle help overlay |
| `q` | Global | Quit |
| `Esc` ×2 fast | Browse | Quit |
| `Ctrl-C` | Global | Force quit |

**Important**: All vim keys (`h`/`j`/`k`/`l`) that are bound to actions MUST be guarded with `if !app.search_focused` in `keyboard.rs` to avoid intercepting search input. Search submit uses `Enter | Right` (no `Char('l')`), cancel uses `Esc | Left` (no `Char('h')`).

### Confirm / ClassicConfirm parity

`ClassicConfirm` (classic confinement install prompt) must always have identical keyboard and mouse behaviour to `Confirm`. Both share `app.confirm_yes_area`, `app.confirm_no_area`, and `app.confirm_hovered`. When opening either dialog, set `confirm_hovered = Some(false)` (default to No/Cancel). Clear `confirm_hovered = None` on accept or cancel.

### Auto-connect prompts after install

After a successful install, `tick()` calls `queue_auto_connect_prompts(snap_name)` which loads the snap's interfaces, finds unconnected plugs with exactly one system slot (`slot.snap` is `"snapd"`, `""`, or `None`), and queues `ConfirmPending::AutoConnect` entries. These are shown sequentially via `pop_auto_connect_prompt()`. Cancelling skips to the next prompt; accepting connects the interface.

---

## snapd-rs API client

- Communicates over the snapd Unix socket at `/run/snapd.socket`
- All snap operations live in `snapd-rs/src/api/snaps.rs`
- Key methods: `install_snap`, `install_snap_classic`, `remove_snap`, `remove_snap_purge`, `refresh_snap`, `revert_snap`, `enable_snap`, `disable_snap`, `list_snaps`, `find_snaps`, `list_connections`, `connect_interface`, `disconnect_interface`, `list_changes`, `get_change`, `abort_change`
- `remove_snap_purge` sends `{ "action": "remove", "purge": true }` — deletes all snap data
- Returns `ChangeId` for async operations; poll `/v2/changes/{id}` to track progress

---

## Development workflow

### Before every commit

```bash
bash /project/artie_sandbox/checks.sh
```

This runs: `cargo check`, `cargo clippy --fix`, `yamlfmt .` (non-fatal, may not be installed), `cargo fmt`.

### Commit style

Conventional commits: `feat:`, `fix:`, `refactor:`, `docs:`, `chore:`

Always include the co-author trailer:
```
Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

### Active branch

`artie/iface-prompt` (branched from `artie/refactor`)

---

## Render conventions

- All render functions take `frame: &mut Frame, app: &mut App` (or `app: &App` if read-only)
- Popups: use `crate::layout::centered_popup(width, height, area)` + `frame.render_widget(Clear, popup)`
- Button rows in confirm dialogs: use `block.inner(popup)` + `Layout` split with `Constraint::Length(1)` at the bottom to pin the button row; store button `Rect`s in `app.confirm_yes_area` / `app.confirm_no_area` for mouse hit-testing
- Status bar: rendered by `detail::render_status_bar`; tooltip shows `?/F1  help` (not `h  help`)
- Destructive actions (Uninstall, UninstallPurge, Revert, Disconnect): red border on confirm dialog

---

## Coding conventions

- Logic (state mutations, async calls) goes in `state.rs` of the relevant module or in `app.rs` for cross-cutting concerns
- Rendering goes in `render.rs`; keep it free of business logic
- `lib.rs` is the crate root and event loop only — no business logic
- `app.rs` holds the `App` struct and all state fields; methods are `impl App` blocks split across module `state.rs` files
- Prefer `Option<Rect>` fields on `App` for storing rendered areas used in mouse hit-testing
- `yamlfmt` may not be installed — non-fatal, ignore its errors in checks.sh
