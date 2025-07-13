#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}ESP32 Firmware Test Suite${NC}"
echo "=================================="

# Compile and run the test runner
echo -e "${YELLOW}Compiling test runner...${NC}"
if rustc --edition=2021 test_runner.rs -o test_runner; then
    echo -e "${GREEN}✓ Test runner compiled successfully${NC}"
else
    echo -e "${RED}✗ Failed to compile test runner${NC}"
    exit 1
fi

echo -e "${YELLOW}Running firmware tests...${NC}"
echo ""

# Run the test runner
./test_runner