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
echo -e "${YELLOW}Testing compilation of whisper-rust-binding${NC}"
echo -e "${BLUE}=============================================${NC}\n"

# Test Linux compilation
echo -e "${BLUE}Testing compilation for Linux...${NC}"
cargo check --release

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Linux compilation check passed!${NC}"
else
    echo -e "${RED}✗ Linux compilation check failed!${NC}"
    exit 1
fi

# Test Android compilation if NDK is available
if [ -n "$ANDROID_NDK_HOME" ]; then
    echo -e "\n${BLUE}Testing compilation for Android arm64-v8a...${NC}"
    cargo check --target aarch64-linux-android --release

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Android compilation check passed!${NC}"
    else
        echo -e "${RED}✗ Android compilation check failed!${NC}"
        exit 1
    fi
else
    echo -e "\n${YELLOW}Skipping Android compilation check (ANDROID_NDK_HOME not set)${NC}"
fi

echo -e "\n${GREEN}All compilation tests passed!${NC}"
