# 🚀 ESP32 Firmware Integration

## Summary

This PR introduces a second entry point for the project by adding a complete ESP-IDF Rust firmware project under `src/firmware/`. The new firmware module provides a "Hello World" ESP32 application with LED blinking functionality and includes comprehensive QEMU emulation support integrated with Cargo workflows.

## 🎯 **Key Features Added**

### ESP32 Firmware Project
- **Complete ESP-IDF Rust Setup**: Full ESP32 development environment with proper configuration
- **Hello World Application**: LED blinking demo with serial output logging
- **Hardware Abstraction**: Uses ESP-IDF HAL for GPIO control and system management
- **Optimized Builds**: Size-optimized profiles for embedded deployment

### QEMU Emulation Support
- **Dual Emulation Options**: 
  - Traditional QEMU setup (`qemu_esp32.sh`)
  - Modern Wokwi-based simulation (`qemu_simple.sh`) - **recommended**
- **Cargo Integration**: Simulation runner binary (`run-sim`) for seamless development
- **Automated Setup**: Scripts handle QEMU/Wokwi installation and configuration

### Workspace Integration
- **Cargo Workspace**: Properly configured multi-package workspace
- **Shared Profiles**: Optimized build configurations for both web and embedded targets
- **Clean Separation**: Firmware code isolated in dedicated directory structure

## 📁 **Project Structure**

```
src/firmware/
├── .cargo/config.toml      # ESP32 target configuration
├── Cargo.toml              # Firmware project dependencies
├── build.rs                # ESP-IDF build script
├── sdkconfig.defaults      # ESP-IDF configuration
├── README.md               # Comprehensive documentation
├── qemu_esp32.sh           # QEMU simulation script
├── qemu_simple.sh          # Wokwi simulation script (recommended)
├── test_setup.sh           # Setup validation script
└── src/
    ├── main.rs             # ESP32 hello world with LED blinking
    └── sim_runner.rs       # Cargo-integrated simulation runner
```

## 🚀 **How to Use**

### Build the Firmware
```bash
# From project root
cd src/firmware
cargo build --release
```

### Run in Simulation
```bash
# Option 1: Using cargo runner (recommended)
cargo run --bin run-sim

# Option 2: Direct script execution
./qemu_simple.sh        # Wokwi simulation
./qemu_esp32.sh         # Traditional QEMU
```

### Test Project Setup
```bash
cd src/firmware
./test_setup.sh         # Validates project configuration
```

## 🔧 **Technical Details**

### Dependencies
- **ESP-IDF Integration**: `esp-idf-sys`, `esp-idf-hal`, `esp-idf-svc`
- **Embedded Services**: `embedded-svc` for hardware abstraction
- **Build Tools**: `embuild` for ESP-IDF integration
- **Logging**: Standard Rust `log` crate with ESP-IDF backend

### Configuration
- **Target**: `xtensa-esp32-espidf` with custom linker configuration
- **Optimization**: Size-optimized builds (`opt-level = "s"`)
- **SDK Config**: Sensible defaults for ESP32 development
- **GPIO Setup**: Onboard LED (GPIO2) configured for blinking demo

### Emulation Options
1. **Wokwi Simulation** (Recommended):
   - Web-based visual simulation
   - Automatic installation via `qemu_simple.sh`
   - Better debugging and visualization

2. **Traditional QEMU**:
   - Command-line QEMU emulation
   - Handles installation of `qemu-system-xtensa`
   - More traditional embedded development workflow

## 📋 **Verification**

- ✅ **Cargo Formatting**: All code formatted with `cargo fmt`
- ✅ **Linting**: Main project passes `cargo clippy` with zero warnings
- ✅ **Workspace Configuration**: Proper multi-package workspace setup
- ✅ **Build Verification**: Firmware builds successfully for ESP32 target
- ✅ **Script Permissions**: All shell scripts have proper executable permissions

## 🎭 **Demo Application**

The firmware includes a complete "Hello World" application that:
- Initializes ESP-IDF logging system
- Configures GPIO2 (onboard LED) as output
- Blinks LED every 1 second
- Outputs status messages to serial console
- Demonstrates proper ESP-IDF Rust patterns

## 🔮 **Future Enhancements**

This firmware foundation enables:
- IoT sensor integration
- WiFi connectivity
- Bluetooth Low Energy (BLE) support
- Real-time data collection
- Edge computing capabilities
- Integration with the main web application

## 📚 **Documentation**

Comprehensive documentation is provided in `src/firmware/README.md` covering:
- Prerequisites and setup instructions
- Building and flashing procedures
- Emulation workflows
- Troubleshooting guide
- Development best practices

---

This integration transforms the project into a full-stack IoT solution with both web application and embedded firmware capabilities, setting the stage for sophisticated hardware-software integration scenarios.