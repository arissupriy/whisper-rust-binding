#!/bin/bash

# Exit on any error
set -e

# Colors for output
GREEN="\033[0;32m"
YELLOW="\033[1;33m"
BLUE="\033[0;34m"
RED="\033[0;31m"
NC="\033[0m" # No Color

echo -e "${BLUE}=============================================${NC}"
echo -e "${YELLOW}Testing whisper-rust-binding build process${NC}"
echo -e "${BLUE}=============================================${NC}\n"

# Check if whisper.cpp directory exists
if [ ! -d "whisper.cpp" ]; then
    echo -e "${RED}Error: whisper.cpp directory is missing.${NC}"
    echo -e "The build requires the whisper.cpp source code in a subdirectory."
    echo -e "Please clone it with: git clone https://github.com/ggml-org/whisper.cpp.git"
    exit 1
fi

# Check if whisper.cpp/CMakeLists.txt exists
if [ ! -f "whisper.cpp/CMakeLists.txt" ]; then
    echo -e "${RED}Error: whisper.cpp/CMakeLists.txt is missing.${NC}"
    echo -e "The whisper.cpp directory doesn't appear to be a valid whisper.cpp repository."
    echo -e "Please clone it with: git clone https://github.com/ggml-org/whisper.cpp.git"
    exit 1
fi

# Try to build just for Linux first
echo -e "${BLUE}Testing Linux build...${NC}"

if cargo build --release 2> build_error.log; then
    echo -e "${GREEN}Linux build succeeded!${NC}"
else
    echo -e "${RED}Linux build failed! See error details below:${NC}"
    cat build_error.log
    echo -e "\n${YELLOW}Analysis of common errors:${NC}"

    if grep -q "No such file or directory" build_error.log; then
        echo -e "${RED}Missing file error detected.${NC} Check that all required files exist."
    fi

    if grep -q "whisper.cpp" build_error.log; then
        echo -e "${RED}whisper.cpp error detected.${NC} There might be issues with the whisper.cpp dependency."
        echo "Make sure whisper.cpp is properly cloned and accessible."
    fi

    if grep -q "CMake" build_error.log; then
        echo -e "${RED}CMake error detected.${NC} Check if CMake is installed and working properly."
        echo "Try: sudo apt-get install cmake"
    fi

    if grep -q "c++" build_error.log || grep -q "stdc++" build_error.log; then
        echo -e "${RED}C++ library error detected.${NC} Check if C++ development tools are installed."
        echo "Try: sudo apt-get install build-essential"
    fi

    if grep -q "linker" build_error.log; then
        echo -e "${RED}Linker error detected.${NC} Check if build-essential package is installed."
    fi

    exit 1
fi
