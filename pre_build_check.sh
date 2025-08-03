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
echo -e "${YELLOW}Checking build prerequisites${NC}"
echo -e "${BLUE}=============================================${NC}\n"

# Check for Rust
if ! command -v rustc &> /dev/null; then
    echo -e "${RED}Error: Rust is not installed!${NC}"
    echo -e "Please install Rust using https://rustup.rs/ and try again."
    exit 1
fi

# Check Rust version
RUST_VERSION=$(rustc --version | awk '{print $2}')
echo -e "Rust version: ${GREEN}$RUST_VERSION${NC}"

# Check for CMake
if ! command -v cmake &> /dev/null; then
    echo -e "${RED}Error: CMake is not installed!${NC}"
    echo -e "Please install CMake and try again."
    echo -e "On Ubuntu/Debian: sudo apt install cmake"
    echo -e "On macOS: brew install cmake"
    exit 1
fi

# Check CMake version
CMAKE_VERSION=$(cmake --version | head -n 1 | awk '{print $3}')
echo -e "CMake version: ${GREEN}$CMAKE_VERSION${NC}"

# Check for C++ compiler
CPP_COMPILER=""
if command -v g++ &> /dev/null; then
    CPP_COMPILER="g++"
    CPP_VERSION=$(g++ --version | head -n 1)
elif command -v clang++ &> /dev/null; then
    CPP_COMPILER="clang++"
    CPP_VERSION=$(clang++ --version | head -n 1)
else
    echo -e "${RED}Error: No C++ compiler found!${NC}"
    echo -e "Please install g++ or clang++ and try again."
    echo -e "On Ubuntu/Debian: sudo apt install g++"
    echo -e "On macOS: xcode-select --install"
    exit 1
fi

echo -e "C++ compiler: ${GREEN}$CPP_VERSION${NC}"

# Check for whisper.cpp directory
if [ ! -d "whisper.cpp" ]; then
    echo -e "\n${YELLOW}Warning: whisper.cpp directory not found!${NC}"
    echo -e "The build process needs the whisper.cpp repository as a subdirectory."
    echo -e "Would you like to clone it now? (y/n)"
    read -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "\n${BLUE}Cloning whisper.cpp repository...${NC}"
        git clone https://github.com/ggerganov/whisper.cpp.git

        if [ $? -ne 0 ]; then
            echo -e "${RED}Error: Failed to clone whisper.cpp repository.${NC}"
            exit 1
        fi

        echo -e "${GREEN}Successfully cloned whisper.cpp repository.${NC}"
    else
        echo -e "${YELLOW}Please clone the whisper.cpp repository before building:${NC}"
        echo -e "git clone https://github.com/ggerganov/whisper.cpp.git"
        exit 1
    fi
else
    echo -e "whisper.cpp directory: ${GREEN}Found${NC}"
fi

# Check for Android NDK if building for Android
if [ "$1" == "android" ]; then
    if [ -z "$ANDROID_NDK_HOME" ]; then
        echo -e "\n${YELLOW}Warning: ANDROID_NDK_HOME environment variable is not set.${NC}"
        echo -e "This is required for Android builds."
        echo -e "Please set ANDROID_NDK_HOME to your Android NDK installation path."
        exit 1
    else
        echo -e "Android NDK: ${GREEN}$ANDROID_NDK_HOME${NC}"

        # Check if directory exists
        if [ ! -d "$ANDROID_NDK_HOME" ]; then
            echo -e "${RED}Error: Android NDK directory not found: $ANDROID_NDK_HOME${NC}"
            exit 1
        fi

        # Check for Android targets
        if ! rustup target list | grep -q "aarch64-linux-android (installed)"; then
            echo -e "\n${YELLOW}Android targets not installed. Installing now...${NC}"
            rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
        else
            echo -e "Android Rust targets: ${GREEN}Installed${NC}"
        fi
    fi
fi

echo -e "\n${GREEN}All prerequisites satisfied!${NC}"
echo -e "You can now run ./build_all.sh to build the project."
