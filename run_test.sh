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
echo -e "${YELLOW}Whisper Rust Binding - Linux Test${NC}"
echo -e "${BLUE}=============================================${NC}\n"

# Make scripts executable
chmod +x test_linux.sh download_model.sh

# Check if model exists
if [ ! -f "models/ggml-tiny.bin" ]; then
    echo -e "${YELLOW}Model not found. Running download script...${NC}"
    ./download_model.sh tiny
fi

# Check if audio file exists
if [ ! -f "output.wav" ]; then
    echo -e "${YELLOW}Audio file 'output.wav' not found.${NC}"

    # Check if we can generate a test audio file
    if command -v ffmpeg &> /dev/null; then
        echo -e "Generating a test audio file using ffmpeg..."
        ffmpeg -f lavfi -i "sine=frequency=440:duration=3" -ar 16000 -ac 1 output.wav
        echo -e "${GREEN}Test audio file generated!${NC}"
    else
        echo -e "${RED}Warning: output.wav not found and ffmpeg is not installed to generate a test file.${NC}"
        echo -e "Please provide an audio file named output.wav."
        exit 1
    fi
fi

# Run the test
LOG_LEVEL=info ./test_linux.sh

echo -e "\n${GREEN}Test completed successfully!${NC}"
