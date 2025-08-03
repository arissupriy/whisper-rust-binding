#!/bin/bash

# Exit on error
set -e

# Colors for output
GREEN="\033[0;32m"
YELLOW="\033[1;33m"
BLUE="\033[0;34m"
RED="\033[0;31m"
NC="\033[0m" # No Color

PRINT_SECTION() {
    echo -e "\n${BLUE}=============================================${NC}"
    echo -e "${YELLOW}$1${NC}"
    echo -e "${BLUE}=============================================${NC}\n"
}

PRINT_SUCCESS() {
    echo -e "\n${GREEN}✓ $1${NC}\n"
}

PRINT_ERROR() {
    echo -e "\n${RED}✗ $1${NC}\n"
    exit 1
}

# Create output directories
mkdir -p "output/linux"
mkdir -p "output/android/arm64-v8a"
mkdir -p "output/android/armeabi-v7a"
mkdir -p "output/android/x86"
mkdir -p "output/android/x86_64"

# Build for Linux
PRINT_SECTION "Building whisper-rust-binding for Linux"
cargo build --release

if [ $? -eq 0 ]; then
    cp target/release/libwhisper_rust.so output/linux/
    PRINT_SUCCESS "Linux build completed successfully! Library copied to output/linux/"
else
    PRINT_ERROR "Linux build failed!"
fi

# Check if we should build for Android
if [ -z "$ANDROID_NDK_HOME" ]; then
    echo -e "${YELLOW}Warning: ANDROID_NDK_HOME environment variable is not set.${NC}"
    read -p "Do you want to continue with Android build? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Skipping Android build."
        exit 0
    fi

    # Ask for NDK path
    read -p "Please enter your Android NDK path: " ANDROID_NDK_HOME
    export ANDROID_NDK_HOME

    if [ ! -d "$ANDROID_NDK_HOME" ]; then
        PRINT_ERROR "Invalid NDK path: $ANDROID_NDK_HOME"
    fi
fi

# Check if Android targets are installed
if ! rustup target list | grep -q "aarch64-linux-android (installed)"; then
    PRINT_SECTION "Installing Android targets for Rust"
    rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
fi

# Build for Android arm64-v8a (64-bit ARM)
PRINT_SECTION "Building for Android arm64-v8a"
cargo build --target aarch64-linux-android --release

if [ $? -eq 0 ]; then
    cp target/aarch64-linux-android/release/libwhisper_rust.so output/android/arm64-v8a/
    PRINT_SUCCESS "Android arm64-v8a build completed!"
else
    PRINT_ERROR "Android arm64-v8a build failed!"
fi

# Build for Android armeabi-v7a (32-bit ARM)
PRINT_SECTION "Building for Android armeabi-v7a"
cargo build --target armv7-linux-androideabi --release

if [ $? -eq 0 ]; then
    cp target/armv7-linux-androideabi/release/libwhisper_rust.so output/android/armeabi-v7a/
    PRINT_SUCCESS "Android armeabi-v7a build completed!"
else
    PRINT_ERROR "Android armeabi-v7a build failed!"
fi

# Build for Android x86 (32-bit Intel)
PRINT_SECTION "Building for Android x86"
cargo build --target i686-linux-android --release

if [ $? -eq 0 ]; then
    cp target/i686-linux-android/release/libwhisper_rust.so output/android/x86/
    PRINT_SUCCESS "Android x86 build completed!"
else
    PRINT_ERROR "Android x86 build failed!"
fi

# Build for Android x86_64 (64-bit Intel)
PRINT_SECTION "Building for Android x86_64"
cargo build --target x86_64-linux-android --release

if [ $? -eq 0 ]; then
    cp target/x86_64-linux-android/release/libwhisper_rust.so output/android/x86_64/
    PRINT_SUCCESS "Android x86_64 build completed!"
else
    PRINT_ERROR "Android x86_64 build failed!"
fi

PRINT_SECTION "Build Summary"
echo -e "${GREEN}✓ All builds completed successfully!${NC}"
echo -e "\nOutput locations:"
echo -e "  Linux:   ${BLUE}$(pwd)/output/linux/libwhisper_rust.so${NC}"
echo -e "  Android: ${BLUE}$(pwd)/output/android/${NC}"
echo -e "    - arm64-v8a:    ${BLUE}$(pwd)/output/android/arm64-v8a/libwhisper_rust.so${NC}"
echo -e "    - armeabi-v7a:  ${BLUE}$(pwd)/output/android/armeabi-v7a/libwhisper_rust.so${NC}"
echo -e "    - x86:          ${BLUE}$(pwd)/output/android/x86/libwhisper_rust.so${NC}"
echo -e "    - x86_64:       ${BLUE}$(pwd)/output/android/x86_64/libwhisper_rust.so${NC}"

echo -e "\n${YELLOW}To use in your Android project, copy the .so files to:${NC}"
echo -e "app/src/main/jniLibs/<architecture>/libwhisper_rust.so"
