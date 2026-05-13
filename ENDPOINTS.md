# snapd API Endpoints

## System Info & Warnings

- [ ] `GET /v2/system-info` — Get system information (version, architecture, sandbox info, etc.)
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/system-info` — System info actions (`advise-system-key-mismatch`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/system-info/storage-encrypted` — Get storage encryption status
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/warnings` — List current warnings
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/warnings` — Acknowledge warnings (`okay`)
  - [ ] Documented
  - [ ] Implemented

## State / Changes

- [ ] `GET /v2/changes` — List all async state changes
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/changes/{id}` — Get a specific change by ID
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/changes/{id}` — Abort a change (`abort`)
  - [ ] Documented
  - [ ] Implemented

## Authentication & Users

- [ ] `POST /v2/login` — Log in to the snap store (Macaroon auth)
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/logout` — Log out of the snap store
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/create-user` — Create a local system user (deprecated, use `POST /v2/users`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/users` — List all users
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/users` — Create or remove a user (`create`, `remove`)
  - [ ] Documented
  - [ ] Implemented

## Snaps

- [ ] `GET /v2/snaps` — List all installed snaps
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/snaps` — Multi-snap operations (`install`, `refresh`, `revert`, `switch`, `hold`, `unhold`, `snapshot`, `remove`, `enable`, `disable`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/snaps/{name}` — Get info for a specific snap
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/snaps/{name}` — Single-snap operations (`install`, `refresh`, `revert`, `switch`, `hold`, `unhold`, `remove`, `enable`, `disable`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/snaps/{name}/conf` — Get snap configuration
  - [ ] Documented
  - [ ] Implemented

- [ ] `PUT /v2/snaps/{name}/conf` — Set snap configuration
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/snaps/{name}/file` — Download the `.snap` file for an installed snap
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/icons/{name}/icon` — Get the icon for an installed snap
  - [ ] Documented
  - [ ] Implemented

## Store / Discovery

- [ ] `GET /v2/find` — Search the snap store (`q`, `name`, `category`, `section`, `scope`, `select`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/sections` — List store sections (deprecated, see `/v2/categories`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/categories` — List store categories
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/download` — Download a snap from the store with resume support (`download`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/cohorts` — Create cohort keys for snaps (`create`)
  - [ ] Documented
  - [ ] Implemented

## Snap Purchase

- [ ] `POST /v2/buy` — Buy a snap (currently unsupported)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/buy/ready` — Check if the user is ready to buy (currently unsupported)
  - [ ] Documented
  - [ ] Implemented

## Interfaces & Connections

- [ ] `GET /v2/interfaces` — List interface connections or available interfaces (`?select=`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/interfaces` — Connect or disconnect interfaces (`connect`, `disconnect`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/connections` — List all plug/slot connections with filtering support
  - [ ] Documented
  - [ ] Implemented

## Assertions

- [ ] `GET /v2/assertions` — List available assertion type names
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/assertions` — Add a new assertion to the local store
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/assertions/{assertType}` — Find assertions by type (with header filter query params)
  - [ ] Documented
  - [ ] Implemented

## Apps & Services

- [ ] `GET /v2/apps` — List all snap apps and services with status
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/apps` — Start, stop, or restart snap services (`start`, `stop`, `restart`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/logs` — Stream or retrieve journald logs for snap services
  - [ ] Documented
  - [ ] Implemented

## Aliases

- [ ] `GET /v2/aliases` — List all snap command aliases
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/aliases` — Manage aliases (`alias`, `unalias`, `prefer`)
  - [ ] Documented
  - [ ] Implemented

## Snapshots

- [ ] `GET /v2/snapshots` — List saved snapshots
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/snapshots` — Manage snapshots (`check`, `restore`, `forget`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/snapshots/{id}/export` — Export a snapshot archive
  - [ ] Documented
  - [ ] Implemented

## Model & Device

- [ ] `GET /v2/model` — Get the current device model assertion
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/model` — Remodel the device (apply a new model assertion)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/model/serial` — Get the device serial assertion
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/model/serial` — Manage the serial assertion (`forget`)
  - [ ] Documented
  - [ ] Implemented

## Recovery Systems

- [ ] `GET /v2/systems` — List all available recovery/seed systems
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/systems` — Perform system-level actions (`reboot`, `create`, `install`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/systems/{label}` — Get details of a specific recovery system
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/systems/{label}` — Actions on a labeled system (`do`, `reboot`, `install`, `create`, `remove`, `check-passphrase-quality`, `check-pin-quality`, `fix-encryption-support`)
  - [ ] Documented
  - [ ] Implemented

## Validation Sets

- [ ] `GET /v2/validation-sets` — List all tracked validation sets
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/validation-sets/{account}/{name}` — Get a specific validation set
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/validation-sets/{account}/{name}` — Apply or forget a validation set (`forget`, `apply`)
  - [ ] Documented
  - [ ] Implemented

## Themes / Accessories

- [ ] `GET /v2/accessories/themes` — Check availability/status of GTK, icon, and sound themes
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/accessories/themes` — Install themes from the store
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/accessories/changes/{id}` — Get status of an accessories (theme install) change
  - [ ] Documented
  - [ ] Implemented

## Quota Groups

- [ ] `GET /v2/quotas` — List all resource quota groups
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/quotas` — Manage quota groups (`ensure`, `remove`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/quotas/{group}` — Get details of a specific quota group
  - [ ] Documented
  - [ ] Implemented

## Confdb

- [ ] `GET /v2/confdb/{account}/{confdb-schema}/{view}` — Read values from a confdb view
  - [ ] Documented
  - [ ] Implemented

- [ ] `PUT /v2/confdb/{account}/{confdb-schema}/{view}` — Write values to a confdb view
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/confdb` — Confdb control actions (`delegate`, `undelegate`)
  - [ ] Documented
  - [ ] Implemented

## Notices

- [ ] `GET /v2/notices` — List notices, with filtering and long-poll support
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/notices` — Add a new notice (`add`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/notices/{id}` — Get a specific notice by ID
  - [ ] Documented
  - [ ] Implemented

## Prompting

- [ ] `POST /v2/interfaces/requests` — Post an interface access request from within a snap (`ask`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/interfaces/requests/prompts` — List pending prompts for the current user
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/interfaces/requests/prompts/{id}` — Get a specific prompt by ID
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/interfaces/requests/prompts/{id}` — Reply to a prompt (`allow`, `deny`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/interfaces/requests/rules` — List all prompting rules for the current user
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/interfaces/requests/rules` — Add or remove prompting rules (`add`, `remove`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/interfaces/requests/rules/{id}` — Get a specific prompting rule by ID
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/interfaces/requests/rules/{id}` — Modify or remove a prompting rule (`patch`, `remove`)
  - [ ] Documented
  - [ ] Implemented

## System Recovery Keys

- [ ] `GET /v2/system-recovery-keys` — Retrieve FDE recovery keys
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/system-recovery-keys` — Remove recovery keys (`remove`)
  - [ ] Documented
  - [ ] Implemented

## System Secureboot

- [ ] `POST /v2/system-secureboot` — EFI Secure Boot database actions (`efi-secureboot-update-startup`, `efi-secureboot-update-db-cleanup`, `efi-secureboot-update-db-prepare`)
  - [ ] Documented
  - [ ] Implemented

## System Volumes

- [ ] `GET /v2/system-volumes` — List encrypted volumes and key slot information
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/system-volumes` — FDE key management actions (`generate-recovery-key`, `check-recovery-key`, `add-recovery-key`, `replace-recovery-key`, `replace-platform-key`, `check-passphrase-quality`, `check-pin-quality`, `change-passphrase`, `change-pin`)
  - [ ] Documented
  - [ ] Implemented

## Snapctl

- [ ] `POST /v2/snapctl` — Execute a snapctl command from within a snap hook
  - [ ] Documented
  - [ ] Implemented

## Debug

- [ ] `GET /v2/debug` — Get debug info (`seeding`, `raa`, `connectivity`, `base-declaration`, `timings`, `features`, `change-timings`, `state`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `POST /v2/debug` — Debug actions (`add-warning`, `unshow-warnings`, `ensure-state-soon`, `can-manage-refreshes`, `prune`, `stacktraces`, `create-recovery-system`, `migrate-home`)
  - [ ] Documented
  - [ ] Implemented

- [ ] `GET /v2/debug/pprof/` — Go pprof profiling endpoints (`cmdline`, `profile`, `symbol`, `trace`, `heap`, `allocs`, `block`, `threadcreate`, `goroutine`, `mutex`)
  - [ ] Documented
  - [ ] Implemented

## Internal

- [ ] `POST /v2/internal/console-conf-start` — Called by `console-conf` at startup to pause snap auto-refresh
  - [ ] Documented
  - [ ] Implemented
