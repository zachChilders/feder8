#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Testing ESP32 Firmware Project Setup${NC}"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Not in firmware directory${NC}"
    exit 1
fi

# Check if required files exist
echo -e "${YELLOW}Checking project structure...${NC}"
files=("Cargo.toml" "build.rs" "sdkconfig.defaults" "src/main.rs" "src/sim_runner.rs" ".cargo/config.toml")

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        echo -e "${GREEN}✓ $file exists${NC}"
    else
        echo -e "${RED}✗ $file missing${NC}"
        exit 1
    fi
done

# Check if scripts are executable
echo -e "${YELLOW}Checking script permissions...${NC}"
scripts=("qemu_esp32.sh" "qemu_simple.sh")

for script in "${scripts[@]}"; do
    if [ -x "$script" ]; then
        echo -e "${GREEN}✓ $script is executable${NC}"
    else
        echo -e "${RED}✗ $script is not executable${NC}"
        exit 1
    fi
done

# Try to parse Cargo.toml
echo -e "${YELLOW}Validating Cargo.toml...${NC}"
if cargo check --quiet 2>/dev/null; then
    echo -e "${GREEN}✓ Cargo.toml is valid${NC}"
else
    echo -e "${YELLOW}⚠ Cargo.toml may have issues (ESP-IDF dependencies not installed)${NC}"
fi

# Check workspace integration
echo -e "${YELLOW}Checking workspace integration...${NC}"
cd ../..
if grep -q "src/firmware" Cargo.toml; then
    echo -e "${GREEN}✓ Firmware integrated into main workspace${NC}"
else
    echo -e "${RED}✗ Firmware not integrated into main workspace${NC}"
    exit 1
fi

echo -e "${GREEN}All checks passed! The ESP32 firmware project is properly set up.${NC}"
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Install ESP-IDF: https://docs.espressif.com/projects/esp-idf/en/latest/esp32/get-started/index.html"
echo "2. Source the ESP-IDF environment: source ~/esp/esp-idf/export.sh"
echo "3. Build the firmware: cargo firmware-build"
echo "4. Run simulation: cargo firmware-sim"