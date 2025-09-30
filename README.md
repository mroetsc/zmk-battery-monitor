# ZMK Battery Monitor

Monitor battery levels of ZMK-powered mechanical keyboards via Bluetooth.

## Features

- Read battery levels for both keyboard halves (Central and Peripheral)
- System tray integration with tooltips
- Configurable update intervals

## Requirements

- Linux with BlueZ
- D-Bus access
- System tray support (for tray mode)

## Installation

```bash
cargo build --release
```

## Usage

### CLI Mode
```bash
cargo run --bin zmk-battery-monitor
```

### System Tray
```bash
cargo run --bin zmk-battery-tray
```

### Configuration

Config file location: `~/.config/zmk-battery-monitor/config.toml`

The config file is created automatically on first run. Edit it to set your keyboard's MAC address:

```toml
[[devices]]
name = "My Keyboard"
address = "XX:XX:XX:XX:XX:XX"
enabled = true
```

Find your keyboard's address with:
```bash
bluetoothctl devices
```
