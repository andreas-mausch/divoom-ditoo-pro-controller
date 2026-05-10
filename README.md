# Divoom Ditoo Pro Controller

A command-line tool to send Bluetooth commands to a [Divoom Ditoo Pro](https://www.divoom.com/products/ditoo-pro) over RFCOMM/SPP.

Supports **macOS** (IOBluetooth) and **Linux** (BlueZ).

---

## Table of Contents

- [Requirements](#requirements)
- [Building](#building)
- [Finding your device](#finding-your-device)
- [Commands](#commands)
  - [get-settings](#get-settings)
  - [set-channel](#set-channel)
  - [restore-state](#restore-state)
  - [animation](#animation)
  - [set-date-time](#set-date-time)
  - [alarm](#alarm)
- [Image conversion](#image-conversion)
- [Claude Code notification hook](#claude-code-notification-hook)
- [Protocol references](#protocol-references)

---

## Requirements

### macOS

- macOS 15 (Sequoia) or later
- Rust stable (`rustup`)
- `make`
- Python 3 (for mascot generation only)
- The Ditoo Pro must be **paired** via System Settings → Bluetooth before use

### Linux

- Rust stable
- `libbluetooth-dev` (BlueZ headers): `sudo apt install libbluetooth-dev`

---

## Building

### macOS

```bash
make build
```

This compiles the release binary **and** applies an ad-hoc code signature with the `com.apple.security.device.bluetooth` entitlement, which macOS 15 requires for Bluetooth access.

The signed binary is at `target/release/divoom-ditoo-pro-controller`.

### Linux

```bash
cargo build --release
```

---

## Finding your device

### macOS

List all paired Bluetooth devices:

```bash
make run ARGS="list-devices"
# or directly:
./target/release/divoom-ditoo-pro-controller list-devices
```

Example output:

```
aa-bb-cc-dd-ee-ff  DitooPro-Audio
```

Use the address (e.g. `aa-bb-cc-dd-ee-ff`) as the `<device>` argument in all commands below.

### Linux

```bash
./target/release/divoom-ditoo-pro-controller list-devices
```

Scans for 20 seconds and prints discovered devices. Use the MAC address (`AA:BB:CC:DD:EE:FF` format).

---

## Commands

All commands follow the pattern:

```
./target/release/divoom-ditoo-pro-controller send <device> <subcommand>
```

### get-settings

Query the current display state (channel and brightness).

```bash
./target/release/divoom-ditoo-pro-controller send aa-bb-cc-dd-ee-ff get-settings
# channel=0 brightness=80
```

Channel values:

| Value | Face |
|-------|------|
| 0 | Clock |
| 1 | Cloud channel |
| 2 | Equalizer |
| 3 | Custom / animation |
| 4 | Scoreboard |
| 5 | Stopwatch |

### set-channel

Switch the display to a built-in face:

```bash
# Switch to clock face
./target/release/divoom-ditoo-pro-controller send aa-bb-cc-dd-ee-ff set-channel 0

# Switch to custom animation channel
./target/release/divoom-ditoo-pro-controller send aa-bb-cc-dd-ee-ff set-channel 3
```

### restore-state

Restore a previously saved channel and brightness:

```bash
./target/release/divoom-ditoo-pro-controller send aa-bb-cc-dd-ee-ff restore-state 0 80
```

Useful in scripts: read the state, make changes, then restore:

```bash
STATE=$(./target/release/divoom-ditoo-pro-controller send aa-bb-cc-dd-ee-ff get-settings)
# → channel=0 brightness=80

./target/release/divoom-ditoo-pro-controller send aa-bb-cc-dd-ee-ff animation images/witch.divoom16

# Restore (parse the saved values)
CHANNEL=$(echo "$STATE" | grep -o 'channel=[0-9]*' | cut -d= -f2)
BRIGHTNESS=$(echo "$STATE" | grep -o 'brightness=[0-9]*' | cut -d= -f2)
./target/release/divoom-ditoo-pro-controller send aa-bb-cc-dd-ee-ff restore-state "$CHANNEL" "$BRIGHTNESS"
```

### animation

Send a 16×16 Divoom animation file to the display:

```bash
./target/release/divoom-ditoo-pro-controller send aa-bb-cc-dd-ee-ff animation images/witch.divoom16
```

The device switches to the custom channel and plays the animation. Use `set-channel 0` (or `restore-state`) to return to the clock face.

### set-date-time

Sync the device's internal clock:

```bash
./target/release/divoom-ditoo-pro-controller send aa-bb-cc-dd-ee-ff set-date-time 2025-03-25T21:22:59
```

This updates the clock but does not exit animation mode. Send `set-channel 0` afterwards to show the clock face.

### alarm

```bash
./target/release/divoom-ditoo-pro-controller send aa-bb-cc-dd-ee-ff alarm true
./target/release/divoom-ditoo-pro-controller send aa-bb-cc-dd-ee-ff alarm false
```

---

## Image conversion

Convert between GIF and Divoom's `.divoom16` wire format:

```bash
# GIF → Divoom format (to send to device)
./target/release/divoom-ditoo-pro-controller convert to-divoom16 images/witch.gif images/witch.divoom16

# Divoom format → GIF (to inspect an existing file)
./target/release/divoom-ditoo-pro-controller convert to-gif images/witch.divoom16 out.gif

# Show detailed frame info for a Divoom file
./target/release/divoom-ditoo-pro-controller debug-image images/witch.divoom16
```

Images must be **16×16 pixels**. Any GIF palette size is supported.

---

## Claude Code notification hook

When [Claude Code](https://claude.ai/code) finishes a task, the device can automatically flash the **Clawd** mascot (the 8-bit pixel crab) and play an alert sound, then restore the previous display.

### One-time setup

**1. Generate the Clawd mascot:**

```bash
python3 scripts/create_mascot.py
# Wrote images/claude.gif
# Wrote images/claude.divoom16
```

**2. (Optional) Add the Family Mart jingle:**

Save the sound file as `sounds/familymart.mp3`. If it's missing the hook falls back to the macOS system `Ping` sound.

**3. The hook script is already executable at `hooks/notify.sh`.** No further setup needed for the script itself.

**4. Wire it into Claude Code** by adding to `~/.claude/settings.json`:

```json
{
  "env": {
    "DIVOOM_DEVICE": "aa-bb-cc-dd-ee-ff"
  },
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "/Users/yourname/path/to/divoom-ditoo-pro-controller/hooks/notify.sh"
          }
        ]
      }
    ]
  }
}
```

Replace `aa-bb-cc-dd-ee-ff` with your device address (from `list-devices`) and update the path to match your local checkout. The script reads `DIVOOM_DEVICE` from the environment and exits silently if it is not set.

### How it works

Each time Claude Code finishes a response:

1. Queries current device state (`get-settings`)
2. Sends the Clawd animation
3. Plays the alert sound (`afplay`)
4. Restores the display to what it was (`restore-state`)

The hook exits immediately and runs all Bluetooth operations in the background, so Claude Code is never blocked.

### Overriding the sound

Pass a custom audio file path in the hook command:

```json
"command": "/path/to/hooks/notify.sh /path/to/custom-sound.mp3"
```

Or set `DIVOOM_DEVICE` to use a different device address:

```json
"command": "DIVOOM_DEVICE=aa-bb-cc-dd-ee-ff /path/to/hooks/notify.sh"
```

### Debugging

```bash
# Run manually
./hooks/notify.sh

# Check the log
cat hooks/notify.log
```

---

## Protocol references

- [Divoom protocol docs](https://docin.divoom-gz.com/web/#/5/146)
- [Animation command 0x8b](https://docin.divoom-gz.com/web/#/5/293)
- [node-divoom-timebox-evo PROTOCOL.md](https://github.com/RomRider/node-divoom-timebox-evo/blob/0.3.0/PROTOCOL.md) — same family, documents `0x46` GetSettings response format
- [futpib's extended fork](https://github.com/futpib/divoom-ditoo-pro-controller)
- [Original blog post](https://andreas-mausch.de/blog/2023-08-14-divoom-ditoo-pro/)
- [16×16 pixel art gallery](https://pixeljoint.com/pixels/new_icons.asp?search=&dimo=%3D&dim=16&colorso=%3E%3D&colors=2&tran=&anim=&iso=&av=&owner=&d=&dosearch=1&ob=search&action=search)
