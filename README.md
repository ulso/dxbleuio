# Example Dioxus/Rust application monitoring sensor devices using the [BleuIO](https://bleuio.com) dongle

This is a desktop application developed in Rust using the [Dioxus](https://dioxuslabs.com) framework (v0.7). It monitors Bluetooth LE sensor data via a BleuIO USB dongle.

The application is specifically designed to monitor sensors from [Smart Sensor Devices AB](https://smartsensordevices.com), primarily the **HibouAIR** series.

## Supported Sensors & Data
The application parses advertisement data from HibouAIR sensors, supporting various metrics depending on the sensor model:
- **Environment**: Temperature, Humidity, Barometric Pressure, Ambient Light.
- **Air Quality**: CO2 (Carbon Dioxide), VOC (Volatile Organic Compounds), PM1.0, PM2.5, PM10.
- **Other**: PIR (Motion), Noise levels.

## How it works
1. **Discovery**: The app searches for a USB device matching the BleuIO Vendor ID and Product ID.
2. **Serial Connection**: It opens the corresponding USB Serial port using `serialport5` and `serial2-tokio`.
3. **Initialization**: 
    - Turns echo off (`ATE0`).
    - Enables verbose mode (`ATV1`).
4. **Scanning**: Starts scanning for specific sensor advertisement data using the command `AT+FINDSCANDATA=FF5B07` (where `5B07` is the HibouAIR manufacturer ID).
5. **Persistence**: The application uses `confy` to remember window size and position across restarts.

## Screenshots
![Dashboard](/img/SCR-20260210-unyj.png)
![Sensor Details](/img/SCR-20260120-lqgp.png)

## Installation & Setup

### Prerequisites
- **Rust**: [Install Rust](https://rustup.rs/)
- **Dioxus CLI**:
  ```bash
  cargo install dioxus-cli
  ```
- **System Dependencies (Linux/Raspberry Pi)**:
  You may need `libudev` development headers:
  ```bash
  sudo apt-get install pkg-config libudev-dev
  ```
  Ensure your user has permission to access the serial port:
  ```bash
  sudo usermod -a -G dialout $USER
  ```
  *(Log out and back in for changes to take effect)*

### Running the App
To run in development mode with hot-reloading:
```bash
dx serve
```

To build for release:
```bash
dx build --release --platform desktop
```

## Project Structure
The project follows a modular Dioxus 0.7 structure:

```
src/
├── components/          # UI Components
│   ├── dashboard.rs     # Main layout and sensor grid
│   ├── sensor_panel.rs  # Individual sensor display cards
│   └── mod.rs
├── hooks/               # Custom Reactive Hooks
│   ├── use_bleuio.rs    # Serial port communication & data stream
│   ├── use_window_config.rs # Window state persistence
│   └── mod.rs
├── models/              # Data Logic & Parsing
│   ├── bleuio.rs        # Dongle responses and AT commands
│   ├── hibouair.rs      # HibouAir binary protocol parsing (using zerocopy)
│   ├── sensor_data.rs   # Unified sensor state models
│   ├── config.rs        # App configuration (window size, etc.)
│   └── mod.rs
└── main.rs              # App entry point and window configuration
```

## Compatibility
The application has been tested and verified on:
- **macOS** (with `macos-app-nap` prevention)
- **Raspberry Pi 5** (latest Debian Trixie)

## License
MIT
