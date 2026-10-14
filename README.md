# HA-TUI: Home Assistant Terminal Dashboard

A high-performance, real-time terminal dashboard for Home Assistant and Kinect AI built with **Ratatui**, **Crossterm**, and **Tokio** in Rust.

## Features

- **Live Real-Time Updates**: Subscribes directly to Home Assistant's WebSocket API (`/api/websocket` with event `state_changed`) for instant state feedback and polls the Kinect AI FastAPI service (`http://100.91.201.64:8080`) for real-time audio/motor/motion telemetry.
- **Auto-Discovery Configuration**: Automatically detects `HASS_URL` and `HASS_TOKEN` from CLI flags, environment variables, local `.env` files (including `kinect_home_ai/.env`), or defaults to Tailscale host `http://100.91.201.64:8123`.
- **9 Dedicated Interactive Views**:
  1. **Overview (`[1]`)**: Glance summary cards (Weather & Sun, Media players, Beckphone & System health, Quick Lighting controls, and Live activity stream).
  2. **Lights (`[2]`)**: Interactive lighting management with graphical brightness gauges (`0-100%`), RGB/Color temperature indicator, quick colors (`[c]`), master toggles (`[a]`/`[x]`).
  3. **Kinect (`[3]`)**: Dedicated Xbox 360 Kinect v1 & Voice Assistant control center:
     - Mute/unmute spoken TTS voice responses (`[m]`) while keeping command recognition active.
     - Mute/unmute the 4-channel Kinect microphone array hardware input (`[Shift+M]`).
     - Switch between all 11 assistant voice profiles (`[v]` — Rachel, GLaDOS, HAL 9000, Cave Johnson, Daniel, Adam, George, Bella, Charlie, Karen, Espeak) with optional vocal greeting announcement.
     - Synthesize & speak custom text phrases aloud (`Speak Custom Phrase (TTS)`).
     - Fine-tune the physical motorized camera tilt angle (`-30°` to `+30°` in `5°` increments with `←`/`→` or `-`/`+`, or jump to `+20°`, `0°`, `-20°`).
     - Toggle between 24/7 3D Infrared Depth sensing (0 lux) and Live Camera video streaming, and cycle camera view modes (`RGB`, `Thermal`, `Depth`).
     - Toggle Motion Auto-On Lights, Empty Room Auto-Off, Motion Greeting Speech, Zelda Ocarina detection, and Security Alarm mode.
     - Live voice transcript feed, last heard phrase, matched intent, and motion countdown telemetry.
  4. **Switches & Automations (`[4]`)**: Complete toggle switches for Kinect continuous listen, motion lights, camera mode, Jellyfin ambient sync, and automations.
  5. **Actionable Buttons & Scripts (`[5]`)**: Direct execution of "Find My Phone", Kinect motor tilt presets, PS5 shutdown, speaker tests, and Jellyfin library cleanups.
  6. **Media Players (`[6]`)**: Player controller for Roku Ultra, PlayStation 5, PlayStation 4, and Samsung TV with play/pause, volume sliders, app name, and mute.
  7. **System & Storage Metrics (`[7]`)**: Real-time visual progress gauges for Jellyfin storage pools (`/mnt/media_hdd`, `disk0`, `disk1`, `disk2`, NVMe root), library counts, Kinect health, Beckphone battery, and PSN stats.
  8. **Entity Explorer (`[8]`)**: Searchable, filterable browser for all 140+ Home Assistant entities with live attribute JSON inspector and domain filtering.
  9. **Live Event Log (`[9]`)**: Chronological event feed showing timestamped state transitions in real time.
- **Universal List Search (`[/]`)**: Filter the active list in real time on **any** tab (Overview lights, Lights, Kinect controls, Switches, Buttons, Media players, Sensors, Entities, or Events).
- **Home Assistant Assist (`[Shift+A]`)**: Natural language prompt input to speak/type commands directly to Home Assistant's conversation engine (`/api/conversation/process`).
- **Service Caller (`[s]`)**: In-dashboard modal to dispatch arbitrary HA service calls and JSON payloads to any entity.

---

## Installation & Running

The binary is compiled and symlinked to `~/.local/bin/ha-tui`.

```bash
# Launch directly from terminal:
ha-tui

# Or with custom URL / token:
ha-tui --url http://100.91.201.64:8123 --token <YOUR_TOKEN>
```

### Building from Source

```bash
cd /home/beck/sync/Projects/home_assistant/ha-tui
cargo build --release
```

---

## Keyboard Shortcuts

| Key | Description |
|---|---|
| `1` .. `9` | Jump directly to View 1 through 9 |
| `Tab` / `Shift+Tab` | Cycle next / previous view |
| `↑` / `k` | Select previous item in active list |
| `↓` / `j` | Select next item in active list |
| `/` | Search / filter the current list on any tab |
| `Shift+A` | Open Home Assistant Assist natural language command prompt |
| `Space` | Toggle selected entity or Kinect control |
| `Enter` | Execute selected control/button/script or inspect entity details |
| `+` / `→` | Increase brightness/volume (+10%) or tilt Kinect camera up (+5°) |
| `-` / `←` | Decrease brightness/volume (-10%) or tilt Kinect camera down (-5°) |
| `v` | Open Kinect Assistant Voice Profile Picker (`[3] Kinect` tab) |
| `m` | Mute/Unmute Kinect Voice TTS (`[3] Kinect` tab) or Media Mute (`[6] Media` tab) |
| `Shift+M` | Mute/Unmute Kinect 4-Channel Microphone Array |
| `t` or `0` | Center Kinect motorized camera base to level (`0°`) |
| `a` | Turn ALL lights ON |
| `x` | Turn ALL lights OFF |
| `c` | Open Color Preset Picker modal (Lights view) |
| `w` | Quick set selected light to Warm White (2700K) |
| `p` | Toggle Power (Media view) |
| `o` | Send PS5 Shutdown / Rest Mode command |
| `f` | Trigger "Find My Phone" alert sound |
| `d` | Cycle domain filters in Entities tab |
| `s` | Open Call Service dialog for selected entity |
| `r` | Force refresh all states from Home Assistant & Kinect |
| `?` or `h` | Toggle Help modal |
| `Esc` | Close active modal / Clear active search filter |
| `q` / `Ctrl+C` | Quit application |
