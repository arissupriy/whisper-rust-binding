#!/bin/bash

# Exit on error
set -e

# Colors for output
GREEN="\033[0;32m"
YELLOW="\033[1;33m"
BLUE="\033[0;34m"
RED="\033[0;31m"
NC="\033[0m" # No Color

echo -e "${BLUE}=========================================${NC}"
echo -e "${YELLOW}Running build with detailed debugging${NC}"
echo -e "${BLUE}=========================================${NC}\n"

# Check for whisper.cpp directory
if [ ! -d "whisper.cpp" ]; then
    echo -e "${RED}Error: whisper.cpp directory not found!${NC}"
    echo -e "The build process needs the whisper.cpp repository as a subdirectory."

    echo -e "\n${BLUE}Cloning whisper.cpp repository...${NC}"
    git clone https://github.com/ggerganov/whisper.cpp.git

    if [ $? -ne 0 ]; then
        echo -e "${RED}Error: Failed to clone whisper.cpp repository.${NC}"
        exit 1
    fi

    echo -e "${GREEN}Successfully cloned whisper.cpp repository.${NC}"
fi

# Check for CMake
if ! command -v cmake &> /dev/null; then
    echo -e "${RED}Error: CMake is not installed!${NC}"
    echo -e "Please install CMake and try again."
    exit 1
fi

# Check for C++ compiler
if ! command -v g++ &> /dev/null && ! command -v clang++ &> /dev/null; then
    echo -e "${RED}Error: No C++ compiler found!${NC}"
    echo -e "Please install g++ or clang++ and try again."
    exit 1
fi

# Set environment variables for cargo
export RUST_BACKTRACE=1

# Run Linux build with verbose output
echo -e "${BLUE}Building for Linux with verbose output...${NC}"
cargo build --release -vv

if [ $? -eq 0 ]; then
    echo -e "${GREEN}Linux build completed successfully!${NC}"
    mkdir -p output/linux
    cp target/release/libwhisper_rust.so output/linux/
    echo -e "Library copied to output/linux/libwhisper_rust.so"
else
    echo -e "${RED}Linux build failed!${NC}"
    exit 1
fi

# Check if we should proceed with Android build
if [ -z "$ANDROID_NDK_HOME" ]; then
    echo -e "\n${YELLOW}ANDROID_NDK_HOME environment variable is not set.${NC}"
    echo -e "Skipping Android build."
    exit 0
fi

# Print version information
echo -e "\n${BLUE}Environment information:${NC}"
echo -e "Rust version: $(rustc --version)"
echo -e "CMake version: $(cmake --version | head -n 1)"
echo -e "C++ compiler: $(g++ --version 2>/dev/null | head -n 1 || clang++ --version 2>/dev/null | head -n 1)"
echo -e "ANDROID_NDK_HOME: $ANDROID_NDK_HOME"

echo -e "\n${GREEN}Build completed successfully!${NC}"
