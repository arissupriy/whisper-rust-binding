#!/bin/bash

# Exit on error
set -e

# Check if ANDROID_NDK_HOME is set
if [ -z "$ANDROID_NDK_HOME" ]; then
    echo "Error: ANDROID_NDK_HOME environment variable is not set."
    echo "Please set it to the path of your Android NDK installation."
    exit 1
fi

echo "Building whisper-rust-binding for all Android architectures..."

# Build for arm64-v8a (64-bit ARM)
echo "\nBuilding for arm64-v8a..."
cargo build --target aarch64-linux-android --release

# Build for armeabi-v7a (32-bit ARM)
echo "\nBuilding for armeabi-v7a..."
cargo build --target armv7-linux-androideabi --release

# Build for x86 (32-bit Intel)
echo "\nBuilding for x86..."
cargo build --target i686-linux-android --release

# Build for x86_64 (64-bit Intel)
echo "\nBuilding for x86_64..."
cargo build --target x86_64-linux-android --release

echo "\nBuild completed successfully!"

# Create output directory
output_dir="android/output"
mkdir -p "$output_dir/arm64-v8a"
mkdir -p "$output_dir/armeabi-v7a"
mkdir -p "$output_dir/x86"
mkdir -p "$output_dir/x86_64"

# Copy libraries to output directory
cp target/aarch64-linux-android/release/libwhisper_rust.so "$output_dir/arm64-v8a/"
cp target/armv7-linux-androideabi/release/libwhisper_rust.so "$output_dir/armeabi-v7a/"
cp target/i686-linux-android/release/libwhisper_rust.so "$output_dir/x86/"
cp target/x86_64-linux-android/release/libwhisper_rust.so "$output_dir/x86_64/"

echo "Libraries copied to $output_dir"
echo "You can now copy these files to your Android project's jniLibs directory."
