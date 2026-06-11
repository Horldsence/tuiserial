# TuiSerial - Terminal Serial Port Debugger

A modern TUI serial port debugging tool built with Rust + Ratatui, featuring a JS/TS plugin system, complete keyboard and mouse interaction.

> **v0.2.0** — Plugin system is now feature-gated. Build without `--features plugin` for a slimmer binary, or enable it for the full plugin experience.

[中文文档](README-CN.md) | English

![](image/image.png)

## 📦 Installation

### Install from crates.io

```bash
# Basic (no plugin support)
cargo install tuiserial

# With full plugin support
cargo install tuiserial --features plugin
```

Run:
```bash
tuiserial
```

### Build from Source

1. Clone the repository: `git clone https://github.com/horldsence/tuiserial.git`
2. Enter directory: `cd tuiserial`
3. Build (without plugin support): `cargo build --release`
4. Or build with full plugin support: `cargo build --release --features plugin`
5. Run: `./target/release/tuiserial`

### Use Pre-compiled Binaries

1. Download binary: [Download Link](https://github.com/horldsence/tuiserial/releases)
2. Extract the archive
3. Run: `./tuiserial`

## ✨ Features

### Core Features
- **Complete Serial Configuration**: Port selection, baud rate, data bits, parity, stop bits, flow control
- **Configuration Persistence**: Auto save/load config to `~/.config/tuiserial/config.json` 💾
- **Config Lock Mechanism**: Auto-lock config after connection to prevent misoperations, unlock after disconnect 🔒
- **Smart Status Display**: Real-time connection status and complete config info (8-N-1 format)
- **Internationalization Support**: English and Chinese, default English 🌍
- **Menu Bar Navigation**: Standard menu bar (File/Session/View/Settings/Plugins/Help), supports keyboard and mouse
- **Dual Display Modes**: HEX and TEXT modes, real-time switching
- **Clean Message Format**: `[Time] ◄ RX (Bytes) Data` - clear and intuitive
- **Bidirectional Data Transfer**: Support HEX/ASCII send modes
- **Flexible Append Options**: Choose to append `\n`, `\r`, `\r\n`, `\n\r` or none
- **Real-time Data Reception**: Efficient circular buffer, supports up to 10000 log lines
- **Auto/Manual Scroll**: Smart auto-tracking or manual browsing of historical data
- **Quick Operations**: Fast toggle between configs and display modes

### Plugin System 🧩 (feature-gated: `--features plugin`)
- **JS/TS Plugin Runtime**: Embeddable JavaScript engine powered by [boa_engine](https://crates.io/crates/boa_engine), write plugins in JS/TypeScript
- **Plugin Hooks**: `onLoad`, `onUnload`, `onConnect`, `onDisconnect`, `onRx(data)`, `onTx(data)` - intercept and transform serial data
- **Plugin Manager UI**: Built-in modal to view installed plugins, check status, reload on the fly
- **Plugin Registry**: Browse, search, and install plugins from the online registry via Git
- **Git-based Updates**: `Check for Updates` / `Update All` keeps plugins up to date
- **Plugin API**: `tuiserial.log.*`, `tuiserial.config.get()`, `tuiserial.require()`, `tuiserial.fs.read()`
- **Plugin Discovery**: Drop plugin folders under `~/.config/tuiserial/plugins/<name>/`, auto-loaded on startup
- **Security**: Path traversal blocked; plugins run in a sandboxed JS runtime

### Interaction Features
- **Full Keyboard Control**: Vim-style shortcuts + standard navigation + F10 menu
- **Comprehensive Mouse Support**: Click, right-click, middle-click, scroll wheel, menu bar clicks
- **Clipboard Paste**: Paste hex or ASCII data directly into the input field
- **Real-time Statistics**: Tx/Rx byte count and connection status
- **Notification System**: Operation feedback and error alerts, multilingual support

### UI Optimizations
- **Status Panel**:
  - Connection status: `✓ Connected` / `✗ Disconnected`
  - Config status: `🔓 Modifiable` / `🔒 Locked`
  - Complete config info: Port, Baud rate, Config format (8-N-1)
  - Plugin count indicator
- **Message Log**:
  - Clean title: `Message - HEX | 123 items [x toggle | c clear]`
  - Unified format: `[Time] ◄ RX (Bytes) Data`
  - Smart hints: Show connection status and shortcuts when log is empty
- **Config Lock Indicator**: Display `[Locked]` marker when connected, border turns gray
- **Append Option Selector**: Independent right panel for quick line ending selection
- **Highlight Hints**: Focused field in yellow, selected items bold, locked fields in gray
- **Shortcuts Overlay**: Press `F1` or `?` to view all keyboard shortcuts

## 📦 Project Structure

Modular architecture managed with Cargo Workspace:

```
tuiserial/
├── Cargo.toml                 # Workspace configuration
├── crates/
│   ├── tuiserial-core/        # Core data models, state management, i18n
│   ├── tuiserial-serial/      # Serial communication library (wraps serialport)
│   ├── tuiserial-ui/          # UI rendering components (based on ratatui)
│   ├── tuiserial-tabs/        # Multi-tab and split-pane layout management
│   ├── tuiserial-plugin/      # JS/TS plugin runtime (based on boa_engine)
│   └── tuiserial-cli/         # Main binary package (published as "tuiserial")
├── docs/
│   └── PLUGIN_DEVELOPMENT_GUIDE_CN.md  # Plugin development guide (Chinese)
├── ARCHITECTURE.md            # Detailed architecture documentation
├── README.md                  # This document (English)
└── README-CN.md               # Chinese documentation
```

**Note**: The directory is named `tuiserial-cli` but the package is published as `tuiserial` on crates.io. Users should install with `cargo install tuiserial`.

## 🚀 Quick Start

### Compile

```bash
cd tuiserial

# Basic — without plugin support (smaller binary, faster compilation)
cargo build --release

# Full — with JS/TS plugin system
cargo build --release --features plugin
```

### Run

```bash
./target/release/tuiserial
```

Or run directly:

```bash
cargo run --release --bin tuiserial
```

> **Note**: The plugin system is feature-gated. When running a build without `--features plugin`, the plugin manager UI is still accessible but plugin operations will prompt you to enable the feature. Run with `tuiserial --help` for more info.

## ⌨️ Keyboard Shortcuts

### Global Controls
| Shortcut | Function |
|----------|----------|
| `Ctrl+C` / `Ctrl+Q` / `q` / `Esc` | Quit program |
| `F10` | Open/Close menu bar |
| `F1` / `?` | Toggle keyboard shortcuts overlay |
| `Tab` | Switch focus to next field |
| `Shift+Tab` | Switch focus to previous field |
| `o` | Open/Close serial connection (locks config when connected) |
| `r` | Refresh serial port list |
| `p` | Open/Close plugin manager |
| `Ctrl+S` | Save config |
| `Ctrl+O` | Load config |

### Menu Bar Navigation (F10 to activate)
| Shortcut | Function |
|----------|----------|
| `←` / `→` | Switch menu items |
| `↑` / `↓` | Select in dropdown menu |
| `Enter` | Execute selected menu item |
| `Esc` | Close menu/return to parent |

### Config Panel Navigation (⚠️ Auto-locks after connection)
| Shortcut | Function |
|----------|----------|
| `↑` / `k` | List up/decrease value |
| `↓` / `j` | List down/increase value |
| `←` / `h` | Decrease baud rate |
| `→` / `l` | Increase baud rate |
| `p` | Toggle parity (None → Even → Odd) |
| `f` | Toggle flow control (None → Hardware → Software) |

**Note**: After connecting to serial port, all config parameters are automatically locked and cannot be modified. You must disconnect first to adjust config.

### Log Area
| Shortcut | Function |
|----------|----------|
| `x` | Toggle HEX/TEXT display mode |
| `c` | Clear log |
| `a` | Toggle auto-scroll |
| `PgUp` | Scroll up (10 lines) |
| `PgDn` | Scroll down (10 lines) |
| `Home` | Jump to log beginning |
| `End` | Jump to log end (and enable auto-scroll) |

### Send Area (when focused on input box)
| Shortcut | Function |
|----------|----------|
| `Character keys` | Input characters |
| `Paste` | Paste hex or ASCII data |
| `Backspace` | Delete previous character |
| `Delete` | Delete next character |
| `←` / `→` | Move cursor |
| `Home` / `End` | Move cursor to start/end |
| `↑` / `↓` | Toggle HEX/ASCII mode |
| `n` | Cycle through append options |
| `Enter` | Send data |
| `Esc` | Clear input |

### Plugin Manager Modal
| Shortcut | Function |
|----------|----------|
| `p` | Open/Close plugin manager (Local view) |
| `↑` / `↓` | Navigate plugin list |
| `Enter` | Install selected plugin (Registry view) |
| `r` | Reload all plugins |
| `Esc` / `q` / `p` | Close plugin manager |

## 🖱️ Mouse Interaction

### Left Click
- **Menu Bar** → Open menu dropdown
- **Menu Item** → Execute corresponding function
- **Config Panel** → Switch focus and directly select list item
- **Log Area** → Switch focus to log area
- **Input Box** → Switch focus and position cursor
- **Append Options** → Directly select append mode

### Right Click
- **Log Area** → Quick toggle HEX/TEXT display mode
- **Input Box** → Quick toggle HEX/ASCII send mode
- **Append Options** → Cycle through append modes
- **Statistics Area** → Toggle auto-scroll

### Middle Click
- **Log Area** → Quick clear log
- **Input Box** → Quick clear input

### Scroll Wheel
- **Log Area** → Scroll log up/down (3 lines)
- **Config List** → Select up/down in list
- **Append Options** → Cycle through append modes
- **Plugin Modal** → Navigate plugin list

## 📊 Data Format

### Receive Display Format
```
[14:32:45.123] ◄ RX (   5 B) 48 65 6C 6C 6F
[14:32:45.456] ◄ RX (   5 B) Hello
```

**Format Description**:
- Timestamp accurate to milliseconds
- `◄ RX` Receive direction (cyan bold)
- `► TX` Transmit direction (green bold)
- Byte count right-aligned for easy viewing

### Send Modes
1. **ASCII Mode**: Enter text directly, e.g., `Hello`
2. **HEX Mode**: Enter hexadecimal, space-separated, e.g., `48 65 6C 6C 6F`

### Append Options
- **None**: Don't add any characters
- **\n**: Add line feed (LF, 0x0A)
- **\r**: Add carriage return (CR, 0x0D)
- **\r\n**: Add carriage return line feed (CRLF, 0x0D 0x0A)
- **\n\r**: Add line feed carriage return (LFCR, 0x0A 0x0D)

## 🛠️ Tech Stack

- **Ratatui 0.29**: Modern Rust TUI framework
- **Crossterm 0.28**: Cross-platform terminal control
- **Serialport 4.3+**: Cross-platform serial port access
- **Boa Engine 0.21**: Pure-Rust JavaScript runtime for the plugin system
- **Tokio 1.40**: Async runtime
- **Chrono 0.4**: Timestamp handling
- **Color-eyre 0.6**: Error handling
- **PHF 0.11**: Compile-time perfect hash maps for i18n

## 📈 Development Status

### ✅ Implemented
- ✅ Serial port config management (all common parameters)
- ✅ **Configuration persistence** (auto save/load config file)
- ✅ **Menu bar system** (File/Session/View/Settings/Plugins/Help, keyboard and mouse support)
- ✅ **Internationalization support** (English/Chinese toggle, compile-time zero overhead)
- ✅ **Plugin system** (JS/TS runtime, plugin manager, registry, Git-based updates)
- ✅ **Config lock mechanism** (auto-lock after connection, prevent misoperations)
- ✅ **Smart status display** (connection status, config status, complete config info)
- ✅ Data reception display (HEX/TEXT modes)
- ✅ Data transmission (HEX/ASCII modes)
- ✅ Append options (\n, \r, \r\n, \n\r, none)
- ✅ Clipboard paste support
- ✅ Full keyboard control (including F10 menu, F1/? shortcuts overlay)
- ✅ Full mouse interaction (click, right-click, middle-click, scroll, menu bar)
- ✅ Optimized message format (clean and intuitive log display)
- ✅ Auto/manual scroll
- ✅ Real-time statistics and notification system
- ✅ Modular architecture (6 crates in Workspace)

### 🔄 Planned / Roadmap
- 🔄 Plugin ecosystem expansion — more registry plugins, plugin SDK improvements
- 🔄 Multi-session and tab management (session crate ready, UI integration in progress)
- 🔄 Split-pane layout (single, horizontal, vertical, 2×2 grid)
- 🔄 Command presets and quick send
- 🔄 Log export (TXT/CSV/JSON)
- 🔄 Search and filter functionality in log area
- 🔄 Data analysis and real-time charts
- 🔄 Multiple serial port simultaneous monitoring
- 🔄 Macro recording and playback
- 🔄 More language support (Japanese, Korean, etc.)

## 📚 Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) - Detailed architecture design document
- [docs/PLUGIN_DEVELOPMENT_GUIDE_CN.md](docs/PLUGIN_DEVELOPMENT_GUIDE_CN.md) - Plugin development guide (Chinese)
- [Cargo Workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) - Cargo Workspace official documentation

## 🧩 Plugin System Overview

> **Feature Gate**: The plugin system is optional. Enable it at build time: `cargo build --release --features plugin`. When disabled, the plugin UI remains accessible and any plugin actions will show a message guiding you to enable the feature.

TuiSerial features a built-in JS/TS plugin system powered by [boa_engine](https://crates.io/crates/boa_engine), a pure-Rust JavaScript runtime. Plugins can intercept and transform serial data in real time.

### Quick Start

1. Create a plugin directory: `~/.config/tuiserial/plugins/my-plugin/`
2. Create `plugin.ts` (or `plugin.js`) as the entry point:

```js
// Called when serial port connects
function onConnect() {
    tuiserial.log.success("Connected! My plugin is ready.");
}

// Intercept received data
function onRx(data) {
    // data is number[] (byte array)
    tuiserial.log.info("RX: " + data.length + " bytes");
    return data; // pass through unchanged, or return modified data
}

function onTx(data) {
    return data; // return null to suppress transmission
}
```

3. Press `p` to open the Plugin Manager, or press `r` to reload plugins.

### Plugin API

| API | Description |
|-----|-------------|
| `tuiserial.log.info(msg)` | Log info message |
| `tuiserial.log.warn(msg)` | Log warning message |
| `tuiserial.log.error(msg)` | Log error message |
| `tuiserial.log.success(msg)` | Log success message |
| `tuiserial.config.get()` | Get current serial config |
| `tuiserial.require(path)` | Load another JS/TS file from plugin dir |
| `tuiserial.fs.read(path)` | Read a file as UTF-8 string |
| `tuiserial.fs.readBinary(path)` | Read a file as `number[]` |

For detailed plugin development documentation, see [docs/PLUGIN_DEVELOPMENT_GUIDE_CN.md](docs/PLUGIN_DEVELOPMENT_GUIDE_CN.md).

## 🔐 Core Feature: Config Lock Mechanism

**Why do we need config locking?**
After establishing a serial connection, modifying parameters may cause:
- Communication interruption or data corruption
- Device abnormal response
- Debug information confusion

**Our Solution:**
1. ✅ **Auto-lock on connection** - After pressing `o` to connect, all config parameters are immediately locked
2. ✅ **Visual feedback** - Config panel shows `[Locked]` marker, border turns gray
3. ✅ **Operation interception** - Any modification attempt shows warning: "Config locked, please disconnect first"
4. ✅ **Unlock on disconnect** - Press `o` again to disconnect, config returns to modifiable state
5. ✅ **Status sync** - Status panel displays current config and lock status in real-time

**Actual Effect:**
```
When disconnected:
  Status: ✗ Disconnected
  Config: 🔓 Modifiable
  → Can freely adjust all parameters

When connected:
  Status: ✓ Connected
  Config: 🔒 Locked
  Port: /dev/ttyUSB0
  Baud: 115200
  Config: 8-N-1
  → Parameters locked, cannot modify

After disconnect:
  Status: ✗ Disconnected
  Config: 🔓 Modifiable
  → Returns to modifiable state
```

## 🤝 Contributing

Issues and Pull Requests are welcome!

1. Fork this project
2. Create feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to branch (`git push origin feature/AmazingFeature`)
5. Open Pull Request

## 📝 License

MIT License

## 👨‍💻 Author

pengheng <m18511047688@163.com>

## 🌍 Internationalization

Currently supported:
- **English** (default)
- **中文** (Chinese)

Switch language:
- Press `F10` to open menu
- Select `Settings` → `Toggle Language`
- Or click directly on menu bar

Technical implementation:
- Uses `phf` for compile-time static HashMap
- Zero runtime overhead, all translations embedded at compile time
- Fallback mechanism: returns key itself if translation not found
- Simple and direct, no complex framework dependencies

## 💾 Configuration File

Config auto-saves to:
- **Linux/macOS**: `~/.config/tuiserial/config.json`
- **Windows**: `%APPDATA%\tuiserial\config.json`

Config content:
```json
{
  "port": "/dev/ttyUSB0",
  "baud_rate": 115200,
  "data_bits": 8,
  "parity": "None",
  "stop_bits": 1,
  "flow_control": "None"
}
```

Operations:
- **Save Config**: Menu → File → Save Config (or `Ctrl+S`)
- **Load Config**: Menu → File → Load Config (auto-loads on startup, or `Ctrl+O`)
- Uses default config if config file is corrupted, no crashes

---

**⭐ If this project helps you, please give it a Star!**
