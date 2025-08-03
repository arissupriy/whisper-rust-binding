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
echo -e "${YELLOW}Precisely fixing lib.rs${NC}"
echo -e "${BLUE}=============================================${NC}\n"

# Create a backup of the original file
cp src/lib.rs src/lib.rs.bak
echo -e "${BLUE}Created backup at src/lib.rs.bak${NC}"

# Now let's fix the file
echo -e "${YELLOW}Fixing lib.rs...${NC}"

# Fix the duplicate 'unsafe' keywords
sed -i 's/unsafe unsafe extern "C"/unsafe extern "C"/g' src/lib.rs

# Fix the whisper_rust_validate_word function
sed -i 's/unsafe { unsafe { whisper_rust_validate_word/unsafe { whisper_rust_validate_word/g' src/lib.rs

# Fix all exported functions to be unsafe
sed -i '/^#\[no_mangle\]$/{n;s/pub extern "C"/pub unsafe extern "C"/g}' src/lib.rs

# Fix the public Rust API functions to wrap their unsafe calls
sed -i '/whisper_rust_process_audio(/{/unsafe {/!s/whisper_rust_process_audio(/unsafe { whisper_rust_process_audio(/g}' src/lib.rs
sed -i '/whisper_rust_process_audio_sliding_window(/{/unsafe {/!s/whisper_rust_process_audio_sliding_window(/unsafe { whisper_rust_process_audio_sliding_window(/g}' src/lib.rs
sed -i '/whisper_rust_get_model_info(/{/unsafe {/!s/whisper_rust_get_model_info(/unsafe { whisper_rust_get_model_info(/g}' src/lib.rs
sed -i '/whisper_rust_validate_word(/{/unsafe {/!s/whisper_rust_validate_word(/unsafe { whisper_rust_validate_word(/g}' src/lib.rs

# Add closing brackets for unsafe blocks where needed
sed -i '/buffer_size as i32);/{/} *;/!s/buffer_size as i32);/buffer_size as i32) };/g}' src/lib.rs
sed -i '/c_word_ptrs\.len() as i32)/{/} *$/!s/c_word_ptrs\.len() as i32)/c_word_ptrs\.len() as i32) }/g}' src/lib.rs

# Remove unused imports
sed -i 's/use anyhow::{anyhow, Result};/use anyhow::Result;/g' src/lib.rs
sed -i 's/use log::{info, error, debug};/use log::error;/g' src/lib.rs

echo -e "${GREEN}Successfully fixed lib.rs!${NC}"
