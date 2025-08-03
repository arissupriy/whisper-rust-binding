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
echo -e "${YELLOW}Cloning whisper.cpp repository${NC}"
echo -e "${BLUE}=============================================${NC}\n"

# Check if whisper.cpp directory already exists
if [ -d "whisper.cpp" ]; then
    echo -e "${YELLOW}whisper.cpp directory already exists.${NC}"
    read -p "Do you want to remove it and clone again? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Removing existing whisper.cpp directory..."
        rm -rf whisper.cpp
    else
        echo "Keeping existing whisper.cpp directory."
        exit 0
    fi
fi

# Clone the repository
echo -e "Cloning whisper.cpp repository from GitHub..."
git clone https://github.com/ggerganov/whisper.cpp.git

if [ $? -eq 0 ]; then
    echo -e "\n${GREEN}Successfully cloned whisper.cpp repository!${NC}"
    echo -e "You can now build the project with ./build_all.sh"
else
    echo -e "\n${RED}Failed to clone whisper.cpp repository!${NC}"
    echo -e "Please check your internet connection and try again."
    exit 1
fi
