# Feder8 ESP32 Firmware

This is an ESP-IDF Rust hello world project that demonstrates basic ESP32 functionality with LED blinking and serial output.

## Features

- **LED Blinking**: Toggles the onboard LED (GPIO2) every second
- **Serial Output**: Prints hello messages and LED status to the console
- **QEMU Simulation**: Run the firmware in emulation without physical hardware
- **Wokwi Integration**: Visual simulation with web-based interface

## Prerequisites

Before building and running the firmware, you need to install the ESP-IDF development environment:

```bash
# Install ESP-IDF (if not already installed)
curl -LO https://github.com/espressif/esp-idf/archive/refs/tags/v5.1.2.zip
unzip v5.1.2.zip
cd esp-idf-5.1.2
./install.sh
source export.sh
```

## Building the Firmware

From the root directory of the project:

```bash
# Build the firmware
cargo firmware-build

# Or build directly in the firmware directory
cd src/firmware
cargo build --release
```

## Running in Simulation

### Option 1: Using Wokwi (Recommended)

Wokwi provides a visual simulation with a web interface:

```bash
# Run with Wokwi simulation
cargo firmware-sim

# Or run directly
cd src/firmware
./qemu_simple.sh
```

This will:
1. Build the firmware
2. Start a Wokwi simulation server
3. Open http://localhost:9012 in your browser
4. Show a virtual ESP32 board with LED visualization

### Option 2: Using QEMU

For command-line QEMU simulation:

```bash
# Run with QEMU simulation
cd src/firmware
./qemu_esp32.sh

# Or use the sim runner with qemu flag
cargo run --bin run-sim qemu
```

## Project Structure

```
src/firmware/
├── Cargo.toml              # Firmware project dependencies
├── build.rs                # Build script for ESP-IDF
├── sdkconfig.defaults      # ESP-IDF configuration
├── README.md              # This file
├── qemu_esp32.sh          # QEMU simulation script
├── qemu_simple.sh         # Wokwi simulation script
└── src/
    ├── main.rs            # Main firmware application
    └── sim_runner.rs      # Simulation runner binary
```

## Understanding the Code

The main firmware application (`src/main.rs`) demonstrates:

1. **GPIO Control**: Configures GPIO2 as an output pin for LED control
2. **FreeRTOS Integration**: Uses FreeRTOS delays for timing
3. **Logging**: Implements both `println!` and `log::info!` for output
4. **Infinite Loop**: Runs continuously, typical for embedded applications

## Customization

To modify the firmware:

1. **Change LED Pin**: Edit the GPIO pin number in `src/main.rs`
2. **Adjust Timing**: Modify the delay values in the main loop
3. **Add Features**: Include additional ESP-IDF components in `Cargo.toml`
4. **Configuration**: Update `sdkconfig.defaults` for ESP-IDF settings

## Troubleshooting

### Build Issues

If you encounter build errors:

1. Ensure ESP-IDF is properly installed and sourced
2. Check that the ESP-IDF version matches the project requirements
3. Verify all dependencies are installed

### Simulation Issues

If simulation doesn't work:

1. For Wokwi: Check that the server starts and port 9012 is available
2. For QEMU: Ensure `qemu-system-xtensa` is installed
3. Check that the firmware binary was built successfully

### Performance

The firmware is configured for optimal size (`opt-level = "s"`) to fit within ESP32 memory constraints. You can adjust optimization levels in `Cargo.toml` if needed.

## Integration with Main Project

The firmware is integrated into the main Feder8 project as a workspace member. You can use these commands from the root directory:

- `cargo firmware-build` - Build the firmware
- `cargo firmware-sim` - Run the simulation
- `cargo firmware-qemu` - Run the firmware binary directly

## Next Steps

This hello world project provides a foundation for more complex ESP32 applications. You can extend it with:

- WiFi connectivity
- Bluetooth communication
- Sensor integration
- Web server functionality
- Over-the-air updates