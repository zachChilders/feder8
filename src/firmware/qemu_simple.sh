#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}ESP32 Wokwi Simulation Runner${NC}"
echo -e "${YELLOW}Building firmware...${NC}"

# Build the firmware
cd "$(dirname "$0")"

# Set ESP-IDF environment
export ESP_IDF_VERSION=5.1.2
export ESP_IDF_TOOLS_INSTALL_DIR=~/.espressif

# Build the project
cargo build --release

# Check if wokwi-server is available
if ! command -v wokwi-server &> /dev/null; then
    echo -e "${YELLOW}wokwi-server not found. Installing...${NC}"
    
    # Install wokwi-server
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        curl -L https://github.com/wokwi/wokwi-server/releases/latest/download/wokwi-server-linux-x64.tar.gz | tar -xz
        sudo mv wokwi-server /usr/local/bin/
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        curl -L https://github.com/wokwi/wokwi-server/releases/latest/download/wokwi-server-macos-x64.tar.gz | tar -xz
        sudo mv wokwi-server /usr/local/bin/
    else
        echo -e "${RED}Please install wokwi-server manually for your platform${NC}"
        echo -e "${YELLOW}Visit: https://github.com/wokwi/wokwi-server${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}Starting ESP32 simulation...${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop the simulation${NC}"

# Find the built ELF file
ELF_FILE=$(find target -name "feder8-firmware" -type f | head -1)

if [ -z "$ELF_FILE" ]; then
    echo -e "${RED}Error: Could not find built firmware ELF file${NC}"
    echo -e "${YELLOW}Make sure the build completed successfully${NC}"
    exit 1
fi

echo -e "${GREEN}Running: $ELF_FILE${NC}"

# Create a simple wokwi diagram
cat > diagram.json << EOF
{
  "version": 1,
  "author": "Feder8 ESP32 Firmware",
  "editor": "wokwi",
  "parts": [
    { "type": "wokwi-esp32-devkit-v1", "id": "esp", "top": 0, "left": 0, "attrs": {} },
    { "type": "wokwi-led", "id": "led1", "top": 0, "left": 200, "attrs": { "color": "red" } },
    { "type": "wokwi-resistor", "id": "r1", "top": 0, "left": 100, "attrs": { "value": "220" } }
  ],
  "connections": [
    [ "esp:TX0", "\$serialMonitor:RX", "", [] ],
    [ "esp:RX0", "\$serialMonitor:TX", "", [] ],
    [ "esp:D2", "r1:1", "", [] ],
    [ "r1:2", "led1:A", "", [] ],
    [ "led1:C", "esp:GND.1", "", [] ]
  ]
}
EOF

# Run wokwi-server
echo -e "${GREEN}Starting Wokwi simulation...${NC}"
echo -e "${YELLOW}Open http://localhost:9012 in your browser to see the simulation${NC}"

wokwi-server --chip esp32 --elf "$ELF_FILE" --diagram diagram.json