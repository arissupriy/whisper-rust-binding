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
echo -e "${YELLOW}Fixing whisper.cpp repository${NC}"
echo -e "${BLUE}=============================================${NC}\n"

# Check if whisper.cpp directory exists
if [ ! -d "whisper.cpp" ]; then
    echo -e "${YELLOW}whisper.cpp directory not found. Cloning from GitHub...${NC}"
    git clone https://github.com/ggerganov/whisper.cpp.git
else
    echo -e "${YELLOW}Removing existing whisper.cpp directory and cloning again...${NC}"
    rm -rf whisper.cpp
    git clone https://github.com/ggerganov/whisper.cpp.git
fi

# Verify the repository
if [ -f "whisper.cpp/CMakeLists.txt" ]; then
    echo -e "\n${GREEN}Successfully cloned whisper.cpp repository!${NC}"
    echo -e "You can now run ./build_all.sh to build the project."
else
    echo -e "\n${RED}Error: Failed to clone a valid whisper.cpp repository.${NC}"
    echo -e "Please check your internet connection and try again."
    exit 1
fi
