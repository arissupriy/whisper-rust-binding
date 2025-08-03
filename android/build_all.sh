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

# Check if ANDROID_NDK_HOME is set
if [ -z "$ANDROID_NDK_HOME" ]; then
    echo -e "${RED}Error: ANDROID_NDK_HOME environment variable is not set.${NC}"
    echo "Please set it to the path of your Android NDK installation."
    exit 1
fi

PRINT_SECTION "Building whisper-rust-binding for all Android architectures"

# Check if Android targets are installed
if ! rustup target list | grep -q "aarch64-linux-android (installed)"; then
    PRINT_SECTION "Installing Android targets for Rust"
    rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
fi

# Create output directory
output_dir="output"
mkdir -p "$output_dir/arm64-v8a"
mkdir -p "$output_dir/armeabi-v7a"
mkdir -p "$output_dir/x86"
mkdir -p "$output_dir/x86_64"

# Build with JNI support if requested
JNI_FLAG=""
if [ "$1" == "--with-jni" ]; then
    JNI_FLAG="--features android-jni"
    echo -e "${YELLOW}Building with JNI support enabled${NC}"
fi

# Build for arm64-v8a (64-bit ARM)
PRINT_SECTION "Building for arm64-v8a"
cargo build --target aarch64-linux-android --release $JNI_FLAG
cp target/aarch64-linux-android/release/libwhisper_rust.so "$output_dir/arm64-v8a/"

# Build for armeabi-v7a (32-bit ARM)
PRINT_SECTION "Building for armeabi-v7a"
cargo build --target armv7-linux-androideabi --release $JNI_FLAG
cp target/armv7-linux-androideabi/release/libwhisper_rust.so "$output_dir/armeabi-v7a/"

# Build for x86 (32-bit Intel)
PRINT_SECTION "Building for x86"
cargo build --target i686-linux-android --release $JNI_FLAG
cp target/i686-linux-android/release/libwhisper_rust.so "$output_dir/x86/"

# Build for x86_64 (64-bit Intel)
PRINT_SECTION "Building for x86_64"
cargo build --target x86_64-linux-android --release $JNI_FLAG
cp target/x86_64-linux-android/release/libwhisper_rust.so "$output_dir/x86_64/"

PRINT_SECTION "Build completed successfully!"

echo -e "${GREEN}Libraries copied to:${NC}"
echo -e "  arm64-v8a:    ${BLUE}$(pwd)/$output_dir/arm64-v8a/libwhisper_rust.so${NC}"
echo -e "  armeabi-v7a:  ${BLUE}$(pwd)/$output_dir/armeabi-v7a/libwhisper_rust.so${NC}"
echo -e "  x86:          ${BLUE}$(pwd)/$output_dir/x86/libwhisper_rust.so${NC}"
echo -e "  x86_64:       ${BLUE}$(pwd)/$output_dir/x86_64/libwhisper_rust.so${NC}"

echo -e "\n${YELLOW}To use in your Android project, copy the .so files to:${NC}"
echo -e "app/src/main/jniLibs/<architecture>/libwhisper_rust.so"
