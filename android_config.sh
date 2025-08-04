# 🔧 Flutter Rust Bridge (FRB) Configuration
# Compatible setup for dual-library Android deployment
# quran_assistant_engine + whisper-rust-binding
# Optimized for mobile development (NDK version: 29.0.13599879)

# Android SDK Configuration (your actual path)
ANDROID_SDK_ROOT=~/Android/Sdk
ANDROID_HOME=~/Android/Sdk

# NDK Configuration (detected version: 29.0.13599879)
NDK_VERSION="29.0.13599879"
NDK_PATH="$ANDROID_SDK_ROOT/ndk/$NDK_VERSION"

# Build targets for mobile Android deployment (prioritized for mobile)
ANDROID_TARGETS=(
    "aarch64-linux-android"      # ARM64 - PRIMARY for modern mobile devices
    "armv7-linux-androideabi"    # ARMv7 - SECONDARY for older mobile devices
    # x86/x86_64 removed - not needed for mobile deployment
)

# API levels optimized for mobile
MIN_SDK_VERSION=21     # Android 5.0 (covers 99%+ of mobile devices)
TARGET_SDK_VERSION=34  # Latest target API level for mobile
COMPILE_SDK_VERSION=34 # Latest compile SDK for mobile features

# Mobile-focused build optimization
MOBILE_OPTIMIZED=true
STRIP_DEBUG_SYMBOLS=true
OPTIMIZE_SIZE=true

# FRB-specific configurations for mobile
FRB_DART_OUTPUT_DIR="lib/generated"
FRB_C_OUTPUT_DIR="ios/Classes"

# Library naming (must match between both projects)
WHISPER_LIB_NAME="libwhisper_rust_binding.so"
QURAN_LIB_NAME="libquran_assistant_engine.so"

# Build optimization for mobile devices
RUST_LOG=warn  # Reduced logging for mobile
CARGO_BUILD_JOBS=4

# Mobile-optimized cross-compilation flags
export RUSTFLAGS="-C target-cpu=generic -C opt-level=s -C strip=symbols"

# Android-specific environment variables for FRB (with detected NDK)
export FLUTTER_RUST_BRIDGE_ANDROID_NDK_HOME="$NDK_PATH"
export FLUTTER_RUST_BRIDGE_ANDROID_SDK_ROOT="$ANDROID_SDK_ROOT"

# Ensure consistent paths across both projects
export PATH="$ANDROID_SDK_ROOT/tools:$ANDROID_SDK_ROOT/platform-tools:$PATH"

# Build profiles for mobile
DEBUG_BUILD=false
RELEASE_BUILD=true
STRIP_SYMBOLS=true

echo "📱 Mobile-optimized FRB Android configuration loaded"
echo "SDK: $ANDROID_SDK_ROOT"
echo "NDK: $NDK_VERSION (detected)"
echo "Mobile targets: ${ANDROID_TARGETS[*]}"
echo "API: Min $MIN_SDK_VERSION, Target $TARGET_SDK_VERSION"
echo "🎯 Optimized for mobile deployment"
