# snap-rat-vibes

[![Crates.io](https://img.shields.io/crates/v/snap-rat-vibes.svg)](https://crates.io/crates/snap-rat-vibes)
[![Documentation](https://docs.rs/snap-rat-vibes/badge.svg)](https://docs.rs/snap-rat-vibes)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

The initial vibe-coded PoC for snap-rat - a Ratatui terminal user interface (TUI) for interacting with the snap store.

See also:

- [snap-rat](https://github.com/artiepoole/snap-rat) for the artisanal re-write of this PoC.
- [snapd-rs-artie](https://github.com/artiepoole/snapd-rs-artie) for the unofficial snapd api bindings used by this
  project (until the official version from Canonical is available)

## snap-rat screenshots and highlighs
Main window:
<img width="1263" height="801" alt="main installed snap management window" src="https://github.com/user-attachments/assets/38ae8850-e237-454e-bc85-a61fd311e273" />
Component managment:
<img width="1047" height="765" alt="compoenents management window for installed snap" src="https://github.com/user-attachments/assets/8a2ed178-d40d-4df2-90d1-ee30159215bf" />
Channel install options and switch mechanism:
<img width="1047" height="765" alt="switch channel menu" src="https://github.com/user-attachments/assets/2802288f-f595-4c4f-a3de-97652344292e" />
Search functionality including hidden snaps (on exact string match):
<img width="1047" height="765" alt="search window" src="https://github.com/user-attachments/assets/6d48bd1e-74f7-49a3-9a29-abd85b588235" />
Changes view
<img width="1047" height="765" alt="changes window" src="https://github.com/user-attachments/assets/c39b7e90-18e7-4894-bbe1-23317bd8df0e" />
Connections management including between snaps (e.g. content interfaces) and connection prompts on install for non-autoconnections
<img width="1047" height="765" alt="connections window" src="https://github.com/user-attachments/assets/dc146b57-5ec2-462f-929b-62fa89e035e7" />
Services management
<img width="1047" height="765" alt="services window" src="https://github.com/user-attachments/assets/ec3504e0-ece9-473d-a3ed-02b331f9e416" />


### Build requirements

snap-rat statically links [libchafa](https://hpjansson.org/chafa/) for rich terminal image rendering. Install the
development package before building:

```
sudo apt install libchafa-dev
```

On terminals that support Kitty, Sixel, or iTerm2 graphics, snap-rat uses those protocols for icon rendering. On other
terminals (including Linux VTs and minimal/ASCII terminals), it falls back to chafa's character-art renderer, which
selects the best-fitting character and colour for each cell — no runtime `.so` dependency needed.

#### Build and run with snapcraft
build:
```
snapcraft pack
```
install:
```
sudo snap install snap-rat-vibes*.snap --dangerous
sudo snap connect snap-rat-vibes:snapd
```
use:
```
sudo snap-rat-vibes
```

**note**: `sudo` is only necessary for write operations like "install", browsing can be done without sudo. If you run the application without root permissions and try to do something which requires them, snap-rat will attempt to escalate and re-exec, but may not be the most seamless behaviour. There be dragons. 

## setup

For megademo.ai. The agent has access to snapd source, so please use

```
git clone git@github.com:canonical/snapd.git
git clone git@github.com:ubuntu/app-center.git
```

to add snapd and appcenter as a subdir.

to update snapd to use 6 in order for workshop to work.

```
sudo snap refresh --channel=6/stable lxd
```

and install workshop

```
sudo snap install workshop --channel=latest/edge
```

then to initialise

```
workshop launch
```

and finally

```
workshop shell
> copilot
```

to enter the shell tool or alternatively

```
# Run copilot interactively
workshop run copilot
# Run copilot with a given prompt
workshop run copilot-prompt <prompt>
# E.g.
workshop run copilot-prompt how many times does the letter p occur in raspberry?
```

to go yolo mode.

# workshop usage

```
workshop launch
```

To see the list of all workshop quick actions, see `workshop.yaml`.

