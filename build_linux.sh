#!/bin/bash

# Exit on error
set -e

# Colors for output
GREEN="\033[0;32m"
YELLOW="\033[1;33m"
BLUE="\033[0;34m"
RED="\033[0;31m"
NC="\033[0m" # No Color

echo -e "${BLUE}=============================================${NC}"
echo -e "${YELLOW}Building whisper-rust-binding for Linux${NC}"
echo -e "${BLUE}=============================================${NC}\n"

# Create output directory
mkdir -p "output/linux"

# Build for Linux
cargo build --release

if [ $? -eq 0 ]; then
    cp target/release/libwhisper_rust.so output/linux/
    echo -e "\n${GREEN}✓ Build completed successfully!${NC}"
    echo -e "\nOutput: ${BLUE}$(pwd)/output/linux/libwhisper_rust.so${NC}"
else
    echo -e "\n${RED}✗ Build failed!${NC}"
    exit 1
fi
