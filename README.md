# XREAL Virtual 3D Desktop

A pure Rust application for running a virtual 3D desktop on XREAL Air glasses via USB-C on macOS. Uses head tracking from IMU to navigate a 3D space with virtual screens.

## Current Status

This is a simplified working version that includes:
- ✅ Head-tracked 3D environment via Bevy
- ✅ IMU fusion with imu-fusion to minimize drift/jitter
- ✅ Settings UI with bevy_egui for brightness, mode toggle, roll lock
- ✅ Calibration support for gyro, accel, mag to reduce drift
- ✅ Low-pass filter on orientation for anti-jitter
- ✅ Fixed timestep updates for consistency

## Features Not Yet Implemented
- Screen capture functionality (removed due to API compatibility issues)
- Input handling via raycasting
- Display extension configuration via knoll
- Multiple virtual screens with live desktop capture

## Setup
1. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Install dependencies: `brew install libusb` (for hidapi/ar-drivers)
3. Connect XREAL Air via USB-C
4. Run: `cargo run --release`

## Building
```bash
cd /Volumes/samsung_t9/xreal_bevy
cargo build --release
```

## Running
```bash
cargo run --release
```

## Project Structure
- `src/main.rs` - Main application entry point with Bevy setup
- `src/driver.rs` - XREAL glasses driver integration
- `src/tracking.rs` - IMU fusion and head tracking implementation
- `src/render.rs` - 3D scene setup and rendering logic
- `src/ui.rs` - Settings UI with egui integration

## Known Issues
- Screen capture features removed due to thread safety issues with ScreenCaptureKit
- Display configuration features disabled (knoll API changes)
- Input handling not yet implemented

## Next Steps
To fully implement the virtual desktop features:
1. Implement thread-safe screen capture wrapper
2. Add display configuration with updated APIs
3. Implement raycasting-based input
4. Add multiple virtual screen support

## License
MIT