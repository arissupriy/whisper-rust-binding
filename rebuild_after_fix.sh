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
echo -e "${YELLOW}Rebuilding after fixing Rust code issues${NC}"
echo -e "${BLUE}=============================================${NC}\n"

# Make script executable
chmod +x *.sh

# Check if whisper.cpp directory exists
if [ ! -d "whisper.cpp" ]; then
    echo -e "${YELLOW}whisper.cpp directory not found. Running fix script...${NC}"
    ./fix_whisper_repo.sh
fi

# Build for Linux
echo -e "${BLUE}Building for Linux...${NC}"
cargo build --release

if [ $? -eq 0 ]; then
    echo -e "\n${GREEN}Build completed successfully!${NC}"

    # Create output directory
    mkdir -p output/linux
    cp target/release/libwhisper_rust.so output/linux/

    echo -e "Library copied to output/linux/libwhisper_rust.so"

    # Build example
    echo -e "\n${BLUE}Building example...${NC}"
    cargo build --release --example test_transcription

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}Example built successfully!${NC}"
    else
        echo -e "${RED}Failed to build example.${NC}"
    fi
else
    echo -e "\n${RED}Build failed!${NC}"
    exit 1
fi

echo -e "\n${GREEN}You can now run your program with the model and audio files:${NC}"
echo -e "./run_transcript.sh"
