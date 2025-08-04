#!/bin/bash
# Mobile development environment setup
# Source this file: source setup_mobile_env.sh

# Android SDK (your actual path)
export ANDROID_HOME=~/Android/Sdk
export ANDROID_SDK_ROOT=~/Android/Sdk

# Expand tilde
ANDROID_SDK_ROOT=$(eval echo $ANDROID_SDK_ROOT)
export ANDROID_HOME="$ANDROID_SDK_ROOT"
export ANDROID_SDK_ROOT="$ANDROID_SDK_ROOT"

# NDK (your actual version)
export NDK_HOME="$ANDROID_SDK_ROOT/ndk/29.0.13599879"
export ANDROID_NDK_HOME="$NDK_HOME"

# Mobile development tools
export PATH="$ANDROID_SDK_ROOT/tools:$ANDROID_SDK_ROOT/platform-tools:$PATH"

# FRB mobile environment
export FLUTTER_RUST_BRIDGE_ANDROID_NDK_HOME="$NDK_HOME"
export FLUTTER_RUST_BRIDGE_ANDROID_SDK_ROOT="$ANDROID_SDK_ROOT"

# Mobile build optimizations
export RUSTFLAGS="-C target-cpu=generic -C opt-level=s -C strip=symbols"

echo "📱 Mobile development environment configured"
echo "SDK: $ANDROID_SDK_ROOT"
echo "NDK: 29.0.13599879"
echo "Targets: ARM64 (primary), ARMv7 (compatibility)"
echo "🎯 Ready for mobile dual-library development!"
