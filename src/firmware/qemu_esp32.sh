#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}ESP32 QEMU Runner${NC}"
echo -e "${YELLOW}Building firmware...${NC}"

# Build the firmware
cd "$(dirname "$0")"
export ESP_IDF_VERSION=5.1.2
export ESP_IDF_TOOLS_INSTALL_DIR=~/.espressif

# Build the project
cargo build --release

# Check if QEMU is available
if ! command -v qemu-system-xtensa &> /dev/null; then
    echo -e "${RED}Error: qemu-system-xtensa not found!${NC}"
    echo -e "${YELLOW}Installing ESP32 QEMU...${NC}"
    
    # Install QEMU ESP32
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        sudo apt-get update
        sudo apt-get install -y qemu-system-misc
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        brew install qemu
    else
        echo -e "${RED}Please install QEMU manually for your platform${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}Starting ESP32 emulation...${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop the emulation${NC}"

# Find the built ELF file
ELF_FILE=$(find target -name "feder8-firmware" -type f | head -1)

if [ -z "$ELF_FILE" ]; then
    echo -e "${RED}Error: Could not find built firmware ELF file${NC}"
    echo -e "${YELLOW}Make sure the build completed successfully${NC}"
    exit 1
fi

echo -e "${GREEN}Running: $ELF_FILE${NC}"

# Run QEMU with ESP32 emulation
qemu-system-xtensa \
    -machine esp32 \
    -cpu esp32 \
    -m 4M \
    -nographic \
    -kernel "$ELF_FILE" \
    -s \
    -S &

QEMU_PID=$!

# Function to cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}Stopping QEMU...${NC}"
    kill $QEMU_PID 2>/dev/null || true
    wait $QEMU_PID 2>/dev/null || true
    echo -e "${GREEN}QEMU stopped${NC}"
}

# Set up signal handlers
trap cleanup EXIT INT TERM

# Wait for QEMU to start
sleep 2

echo -e "${GREEN}QEMU ESP32 emulation started successfully!${NC}"
echo -e "${YELLOW}Connect with GDB using: gdb -ex 'target remote localhost:1234' $ELF_FILE${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop${NC}"

# Keep the script running
wait $QEMU_PID