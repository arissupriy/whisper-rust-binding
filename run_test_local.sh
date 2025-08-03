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
echo -e "${YELLOW}Whisper Rust Binding - Testing with Local Files${NC}"
echo -e "${BLUE}=============================================${NC}\n"

# Check if model and audio files exist
if [ ! -f "ggml-tiny.bin" ]; then
    echo -e "${RED}Error: Model file 'ggml-tiny.bin' not found in project root.${NC}"
    exit 1
fi

if [ ! -f "output.wav" ]; then
    echo -e "${RED}Error: Audio file 'output.wav' not found in project root.${NC}"
    exit 1
fi

echo -e "${GREEN}Found model file: ggml-tiny.bin${NC}"
echo -e "${GREEN}Found audio file: output.wav${NC}\n"

# Build the example
echo -e "${BLUE}Building test example...${NC}"
cargo build --release --example test_transcription

# Run the test with local files
echo -e "\n${BLUE}Running transcription test...${NC}\n"
LOG_LEVEL=info ./target/release/examples/test_transcription "./ggml-tiny.bin" "./output.wav"

echo -e "\n${GREEN}Test completed successfully!${NC}"
