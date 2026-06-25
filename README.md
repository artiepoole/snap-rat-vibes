# snap-rat-vibes

The initial vibe-cded PoC for snap-rat - a Ratatui terminal user interface (TUI) for interacting with the snap store.

See also:

- [snap-rat](https://github.com/artiepoole/snap-rat) for the artisanal re-write of this PoC.
- [snapd-rs-artie](https://github.com/artiepoole/snapd-rs-artie) for the unofficial snapd api bindings used by this
  project (until the official version from Canonical is available)

## snap-rat

<img width="1920" height="1097" alt="image (3)" src="https://github.com/user-attachments/assets/1d7f01f1-8854-4d7f-8815-54fb61a04803" />
<img width="1920" height="1097" alt="image (2)" src="https://github.com/user-attachments/assets/76c83573-05a1-4982-b394-c20af9fcea90" />
<img width="1920" height="1097" alt="image (1)" src="https://github.com/user-attachments/assets/f500c79a-4e77-4f38-afa8-7f1ffd7beae9" />


<img width="1920" height="1151" alt="image" src="https://github.com/user-attachments/assets/958d968b-c8e3-478d-b126-4dbfbe523b1e" />

### Build requirements

snap-rat statically links [libchafa](https://hpjansson.org/chafa/) for rich terminal image rendering. Install the
development package before building:

```
sudo apt install libchafa-dev
```

On terminals that support Kitty, Sixel, or iTerm2 graphics, snap-rat uses those protocols for icon rendering. On other
terminals (including Linux VTs and minimal/ASCII terminals), it falls back to chafa's character-art renderer, which
selects the best-fitting character and colour for each cell — no runtime `.so` dependency needed.

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

