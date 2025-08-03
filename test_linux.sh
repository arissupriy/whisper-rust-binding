#!/bin/bash

# Exit on error
set -e

# Colors for output
GREEN="\033[0;32m"
YELLOW="\033[1;33m"
BLUE="\033[0;34m"
RED="\033[0;31m"
NC="\033[0m" # No Color

# Create models directory if it doesn't exist
mkdir -p models

# Check if model exists, download if not
if [ ! -f "models/ggml-tiny.bin" ]; then
    echo -e "${YELLOW}Model not found. Downloading ggml-tiny.bin...${NC}"

    # Check if curl or wget is available
    if command -v curl &> /dev/null; then
        curl -L -o models/ggml-tiny.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin
    elif command -v wget &> /dev/null; then
        wget -O models/ggml-tiny.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin
    else
        echo -e "${RED}Error: Neither curl nor wget is installed. Please install one of them or download the model manually.${NC}"
        exit 1
    fi

    echo -e "${GREEN}Model downloaded successfully!${NC}"
fi

# Check if audio file exists
if [ ! -f "output.wav" ]; then
    echo -e "${YELLOW}Warning: output.wav not found.${NC}"

    # Check if we can generate a test audio file
    if command -v ffmpeg &> /dev/null; then
        echo -e "Generating a test audio file using ffmpeg..."
        ffmpeg -f lavfi -i "sine=frequency=440:duration=3" -ar 16000 -ac 1 output.wav
        echo -e "${GREEN}Test audio file generated!${NC}"
    else
        echo -e "${RED}Error: output.wav not found and ffmpeg is not installed to generate a test file.${NC}"
        echo -e "Please provide an audio file named output.wav or install ffmpeg."
        exit 1
    fi
fi

# Build the example
echo -e "${BLUE}Building test example...${NC}"
cargo build --release --example test_transcription

# Add hound dependency if not already present
if ! grep -q "hound" Cargo.toml; then
    echo -e "${YELLOW}Adding hound dependency for WAV file parsing...${NC}"
    echo -e "\n[dev-dependencies]\nhound = \"3.5.0\"\n\n[[example]]\nname = \"test_transcription\"\npath = \"examples/test_transcription.rs\"" >> Cargo.toml

    cargo build --release --example test_transcription
fi

# Run the example
echo -e "\n${BLUE}=============================================${NC}"
echo -e "${YELLOW}Running transcription test with ggml-tiny.bin${NC}"
echo -e "${BLUE}=============================================${NC}\n"

LOG_LEVEL=info ./target/release/examples/test_transcription "models/ggml-tiny.bin" "output.wav"

echo -e "\n${GREEN}Test completed!${NC}"
