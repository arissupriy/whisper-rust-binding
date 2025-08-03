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
echo -e "${YELLOW}One-click fix for Rust code issues${NC}"
echo -e "${BLUE}=============================================${NC}\n"

# Make all scripts executable
chmod +x *.sh

# Check if whisper.cpp directory exists and fix if needed
if [ ! -d "whisper.cpp" ]; then
    echo -e "${YELLOW}whisper.cpp directory not found. Cloning from GitHub...${NC}"
    git clone https://github.com/ggerganov/whisper.cpp.git
fi

# Fix Rust code with patches
echo -e "\n${BLUE}Patching Rust code to fix safety issues...${NC}\n"

# Add 'unsafe' to all extern "C" blocks
sed -i 's/extern "C" {/unsafe extern "C" {/g' src/lib.rs

# Add 'unsafe' to all exported C functions
sed -i 's/#\[no_mangle\]\npub extern "C"/#\[no_mangle\]\npub unsafe extern "C"/g' src/lib.rs

# Add 'unsafe' blocks around all FFI function calls
sed -i 's/\(let success = \)whisper_rust_process_audio/\1unsafe { whisper_rust_process_audio/g' src/lib.rs
sed -i 's/\(let success = \)whisper_rust_process_audio_sliding_window/\1unsafe { whisper_rust_process_audio_sliding_window/g' src/lib.rs
sed -i 's/\(let success = \)whisper_rust_get_model_info/\1unsafe { whisper_rust_get_model_info/g' src/lib.rs
sed -i 's/whisper_rust_validate_word(/unsafe { whisper_rust_validate_word(/g' src/lib.rs

# Add missing closing brackets for unsafe blocks
sed -i 's/buffer_size as i32\n    );/buffer_size as i32\n    ) };/g' src/lib.rs
sed -i 's/c_word_ptrs.len() as i32\n    )/c_word_ptrs.len() as i32\n    ) }/g' src/lib.rs

# Remove unused imports
sed -i 's/use anyhow::{anyhow, Result};/use anyhow::Result;/g' src/lib.rs
sed -i 's/use log::{info, error, debug};/use log::error;/g' src/lib.rs

# Build the project
echo -e "\n${BLUE}Building the project...${NC}\n"
cargo build --release

if [ $? -eq 0 ]; then
    echo -e "\n${GREEN}Build completed successfully!${NC}"

    # Create output directory
    mkdir -p output/linux
    cp target/release/libwhisper_rust.so output/linux/

    echo -e "\nLibrary copied to output/linux/libwhisper_rust.so"
else
    echo -e "\n${RED}Build failed. Please check for additional errors.${NC}"
    exit 1
fi

echo -e "\n${GREEN}All Rust code issues have been fixed!${NC}"
echo -e "You can now use the whisper-rust-binding library in your projects."
