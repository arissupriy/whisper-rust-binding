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
echo -e "${YELLOW}One-click fix and build for whisper-rust-binding${NC}"
echo -e "${BLUE}=============================================${NC}\n"

# Make all scripts executable
chmod +x *.sh

echo -e "${BLUE}Step 1: Fixing whisper.cpp repository${NC}\n"

# Remove existing whisper.cpp directory
if [ -d "whisper.cpp" ]; then
    echo "Removing existing whisper.cpp directory..."
    rm -rf whisper.cpp
fi

# Clone the official repository
echo "Cloning official whisper.cpp repository..."
git clone https://github.com/ggerganov/whisper.cpp.git

# Verify the repository
if [ -f "whisper.cpp/CMakeLists.txt" ]; then
    echo -e "\n${GREEN}Successfully cloned whisper.cpp repository!${NC}\n"
else
    echo -e "\n${RED}Error: Failed to clone a valid whisper.cpp repository.${NC}"
    exit 1
fi

echo -e "${BLUE}Step 2: Building whisper-rust-binding${NC}\n"

# Build for Linux
cargo build --release

if [ $? -eq 0 ]; then
    echo -e "\n${GREEN}Build completed successfully!${NC}"

    # Create output directory
    mkdir -p output/linux
    cp target/release/libwhisper_rust.so output/linux/

    echo -e "Library copied to output/linux/libwhisper_rust.so"
else
    echo -e "\n${RED}Build failed!${NC}"
    exit 1
fi

echo -e "\n${BLUE}Step 3: Testing with local model and audio${NC}\n"

# Check if model and audio files exist
if [ -f "ggml-tiny.bin" ] && [ -f "output.wav" ]; then
    echo -e "Found model file: ggml-tiny.bin"
    echo -e "Found audio file: output.wav"

    # Build the example if needed
    if [ ! -f "target/release/examples/test_transcription" ]; then
        echo -e "Building example..."
        cargo build --release --example test_transcription
    fi

    echo -e "\n${GREEN}Running transcription test...${NC}\n"
    LOG_LEVEL=info ./target/release/examples/test_transcription "./ggml-tiny.bin" "./output.wav"
else
    echo -e "\n${YELLOW}Skipping test - model or audio file not found.${NC}"
    echo -e "To run a test, make sure you have:"
    echo -e "  - ggml-tiny.bin (model file) in the project root"
    echo -e "  - output.wav (audio file) in the project root"
fi

echo -e "\n${GREEN}All done! whisper-rust-binding is now ready to use.${NC}"
