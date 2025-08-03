# Build Guide

Comprehensive guide for building Whisper Rust Binding on different platforms.

## 📋 Overview

This guide covers building the library for:
- 🐧 **Linux** (Ubuntu, CentOS, Arch Linux)
- 🤖 **Android** (ARM64, ARM32, x86, x86_64)
- 🔄 **Cross-compilation**
- 🚀 **Optimization builds**

## 🛠️ Build Requirements

### Common Requirements

- **Rust 1.88+** with unsafe attributes support
- **CMake 3.10+**
- **Git** (for submodules)
- **C++ Compiler** (GCC 7+ or Clang 5+)

### Platform-Specific Requirements

#### Linux
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install cmake build-essential git pkg-config

# CentOS/RHEL
sudo yum install cmake gcc-c++ git pkgconfig
# or (newer versions)
sudo dnf install cmake gcc-c++ git pkgconfig

# Arch Linux
sudo pacman -S cmake gcc git pkgconfig
```

#### Android
```bash
# Android NDK (version 21+)
export ANDROID_NDK_ROOT=/path/to/android-ndk

# Android SDK with API level 21+
export ANDROID_SDK_ROOT=/path/to/android-sdk
```

## 🐧 Linux Build

### Quick Build (Automated)

```bash
# Clone repository
git clone --recursive https://github.com/your-username/whisper-rust-binding.git
cd whisper-rust-binding

# Make script executable
chmod +x build_linux.sh

# Run automated build
./build_linux.sh
```

The script will:
1. Build whisper.cpp with optimizations
2. Build Rust library
3. Build all examples
4. Run basic tests
5. Display build summary

### Manual Build (Step-by-Step)

#### Step 1: Prepare Environment

```bash
# Verify Rust installation
rustc --version
# Should show: rustc 1.88.0 or higher

# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone repository with submodules
git clone --recursive https://github.com/your-username/whisper-rust-binding.git
cd whisper-rust-binding
```

#### Step 2: Build whisper.cpp

```bash
cd whisper.cpp

# Create build directory
mkdir build
cd build

# Configure with optimizations
cmake .. \
    -DCMAKE_BUILD_TYPE=Release \
    -DWHISPER_BUILD_TESTS=OFF \
    -DWHISPER_BUILD_EXAMPLES=OFF

# Build (use all CPU cores)
make -j$(nproc)

# Return to root directory
cd ../..
```

#### Step 3: Build Rust Library

```bash
# Build library
cargo build --release

# Build examples
cargo build --examples --release

# Run tests
cargo test

# Check build artifacts
ls -la target/release/
```

#### Step 4: Verify Build

```bash
# Test basic functionality
./target/release/examples/transcribe_file --help

# Test with sample audio (if available)
./target/release/examples/transcribe_file ggml-tiny.bin test_audio.wav ar
```

### Build Configuration Options

#### Optimization Levels

```bash
# Development build (faster compilation, slower runtime)
cargo build

# Release build (slower compilation, faster runtime)
cargo build --release

# Size-optimized build
RUSTFLAGS="-C opt-level=s" cargo build --release

# Performance-optimized build
RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo build --release
```

#### Feature Flags

```toml
# In Cargo.toml
[features]
default = ["android"]
android = []
mock = []

# Build without Android support
cargo build --no-default-features

# Build with specific features
cargo build --features "android,mock"
```

### Troubleshooting Linux Build

#### Issue: CMake version too old

```bash
# Check version
cmake --version

# Install newer CMake (Ubuntu)
sudo apt remove cmake
sudo snap install cmake --classic

# Or build from source
wget https://cmake.org/files/v3.20/cmake-3.20.0.tar.gz
tar -xf cmake-3.20.0.tar.gz
cd cmake-3.20.0
./bootstrap && make && sudo make install
```

#### Issue: Missing C++ compiler

```bash
# Install GCC
sudo apt install build-essential

# Or install Clang
sudo apt install clang

# Verify
g++ --version
clang++ --version
```

#### Issue: Rust version conflicts

```bash
# Update Rust
rustup update

# Use specific toolchain
rustup toolchain install 1.88.0
rustup default 1.88.0
```

## 🤖 Android Build

### Automated Android Build

```bash
# Set Android NDK path
export ANDROID_NDK_ROOT=/path/to/android-ndk

# Make script executable
chmod +x build_android.sh

# Build for all Android architectures
./build_android.sh
```

### Manual Android Build

#### Step 1: Setup Android Environment

```bash
# Install Android NDK
# Download from: https://developer.android.com/ndk/downloads

# Set environment variables
export ANDROID_NDK_ROOT=/path/to/android-ndk
export PATH=$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH

# Install Android targets
rustup target add aarch64-linux-android    # ARM64
rustup target add armv7-linux-androideabi  # ARM32
rustup target add i686-linux-android       # x86
rustup target add x86_64-linux-android     # x86_64
```

#### Step 2: Configure Cargo

Create or update `.cargo/config.toml`:

```toml
[target.aarch64-linux-android]
ar = "aarch64-linux-android-ar"
linker = "aarch64-linux-android21-clang"

[target.armv7-linux-androideabi]
ar = "arm-linux-androideabi-ar"
linker = "armv7a-linux-androideabi21-clang"

[target.i686-linux-android]
ar = "i686-linux-android-ar"
linker = "i686-linux-android21-clang"

[target.x86_64-linux-android]
ar = "x86_64-linux-android-ar"
linker = "x86_64-linux-android21-clang"
```

#### Step 3: Build whisper.cpp for Android

```bash
cd whisper.cpp

# Build for ARM64
mkdir build-android-arm64
cd build-android-arm64

cmake .. \
    -DCMAKE_TOOLCHAIN_FILE=$ANDROID_NDK_ROOT/build/cmake/android.toolchain.cmake \
    -DANDROID_ABI=arm64-v8a \
    -DANDROID_PLATFORM=android-21 \
    -DCMAKE_BUILD_TYPE=Release

make -j$(nproc)
cd ..

# Repeat for other architectures (armv7, x86, x86_64)
```

#### Step 4: Build Rust Library for Android

```bash
# Build for ARM64
cargo build --target aarch64-linux-android --release

# Build for ARM32
cargo build --target armv7-linux-androideabi --release

# Build for x86
cargo build --target i686-linux-android --release

# Build for x86_64
cargo build --target x86_64-linux-android --release
```

#### Step 5: Package Android Library

```bash
# Create output directory
mkdir -p android/libs

# Copy ARM64 libraries
cp target/aarch64-linux-android/release/libwhisper_rust_binding.so android/libs/arm64-v8a/

# Copy ARM32 libraries
cp target/armv7-linux-androideabi/release/libwhisper_rust_binding.so android/libs/armeabi-v7a/

# Copy x86 libraries
cp target/i686-linux-android/release/libwhisper_rust_binding.so android/libs/x86/

# Copy x86_64 libraries
cp target/x86_64-linux-android/release/libwhisper_rust_binding.so android/libs/x86_64/
```

### Android Integration

#### JNI Headers

Create `android/src/main/cpp/whisper_jni.h`:

```cpp
#include <jni.h>
#include "whisper_rust_binding.h"

extern "C" {
    JNIEXPORT jint JNICALL
    Java_com_yourpackage_WhisperService_initWhisper(JNIEnv *env, jobject thiz, jstring modelPath);

    JNIEXPORT jstring JNICALL
    Java_com_yourpackage_WhisperService_processAudio(JNIEnv *env, jobject thiz, 
                                                     jint instanceId, jfloatArray audioData, jstring language);

    JNIEXPORT jboolean JNICALL
    Java_com_yourpackage_WhisperService_freeWhisper(JNIEnv *env, jobject thiz, jint instanceId);
}
```

#### Gradle Configuration

`android/build.gradle`:

```gradle
android {
    compileSdkVersion 31
    buildToolsVersion "31.0.0"

    defaultConfig {
        minSdkVersion 21
        targetSdkVersion 31
        
        ndk {
            abiFilters 'arm64-v8a', 'armeabi-v7a', 'x86', 'x86_64'
        }
    }

    sourceSets {
        main {
            jniLibs.srcDirs = ['libs']
        }
    }
}
```

### Troubleshooting Android Build

#### Issue: NDK not found

```bash
# Check NDK installation
ls $ANDROID_NDK_ROOT
ls $ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin

# Reinstall NDK if needed
sdkmanager "ndk;21.4.7075529"
```

#### Issue: Target not installed

```bash
# List installed targets
rustup target list --installed

# Install missing targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
```

#### Issue: Linker errors

```bash
# Check linker configuration
cat .cargo/config.toml

# Verify NDK tools
which aarch64-linux-android21-clang
```

## 🔄 Cross-Compilation

### Cross-Compilation for Different Architectures

#### Setup Cross-Compilation

```bash
# Install cross-compilation tools
sudo apt install gcc-aarch64-linux-gnu  # For ARM64
sudo apt install gcc-arm-linux-gnueabihf  # For ARM32

# Install Rust targets
rustup target add aarch64-unknown-linux-gnu
rustup target add armv7-unknown-linux-gnueabihf
```

#### Configure Cross-Compilation

`.cargo/config.toml`:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```

#### Build for Target Architecture

```bash
# Build for ARM64
cargo build --target aarch64-unknown-linux-gnu --release

# Build for ARM32
cargo build --target armv7-unknown-linux-gnueabihf --release
```

## 🚀 Optimization Builds

### Performance Optimization

```bash
# Maximum performance build
RUSTFLAGS="-C opt-level=3 -C target-cpu=native -C lto=fat" cargo build --release

# Profile-guided optimization (requires profdata)
RUSTFLAGS="-C profile-generate=/tmp/pgo-data" cargo build --release
# Run with representative workload
./target/release/examples/transcribe_file ggml-tiny.bin sample.wav ar
# Rebuild with profile data
RUSTFLAGS="-C profile-use=/tmp/pgo-data -C opt-level=3" cargo build --release
```

### Size Optimization

```bash
# Minimum size build
RUSTFLAGS="-C opt-level=s -C lto=fat -C strip=symbols" cargo build --release

# Ultra-minimal build (may affect performance)
RUSTFLAGS="-C opt-level=z -C lto=fat -C strip=symbols -C panic=abort" cargo build --release
```

### Debug Builds

```bash
# Debug build with symbols
cargo build

# Debug build with optimizations
RUSTFLAGS="-C opt-level=1" cargo build

# Release build with debug info
cargo build --release --config profile.release.debug=true
```

## 📊 Build Verification

### Verify Build Artifacts

```bash
# Check library size
ls -lh target/release/libwhisper_rust_binding.so

# Check symbols
nm target/release/libwhisper_rust_binding.so | grep whisper_rust

# Check dependencies
ldd target/release/libwhisper_rust_binding.so

# Run basic test
cargo test
```

### Performance Testing

```bash
# Benchmark build
cargo bench

# Profile memory usage
valgrind --tool=massif ./target/release/examples/transcribe_file ggml-tiny.bin test.wav ar

# Profile CPU usage
perf record ./target/release/examples/transcribe_file ggml-tiny.bin test.wav ar
perf report
```

## 🎯 Build Scripts Reference

### build_linux.sh

```bash
#!/bin/bash
set -e

echo "🐧 Building Whisper Rust Binding for Linux..."

# Check prerequisites
check_prerequisites() {
    command -v rustc >/dev/null 2>&1 || { echo "Rust is required"; exit 1; }
    command -v cmake >/dev/null 2>&1 || { echo "CMake is required"; exit 1; }
    command -v make >/dev/null 2>&1 || { echo "Make is required"; exit 1; }
}

# Build whisper.cpp
build_whisper_cpp() {
    echo "Building whisper.cpp..."
    cd whisper.cpp
    make clean
    make -j$(nproc)
    cd ..
}

# Build Rust library
build_rust() {
    echo "Building Rust library..."
    cargo build --release
    cargo build --examples --release
}

# Run tests
run_tests() {
    echo "Running tests..."
    cargo test
}

# Main build process
main() {
    check_prerequisites
    build_whisper_cpp
    build_rust
    run_tests
    echo "✅ Build completed successfully!"
}

main "$@"
```

### build_android.sh

```bash
#!/bin/bash
set -e

echo "🤖 Building Whisper Rust Binding for Android..."

# Android configuration
ANDROID_TARGETS=(
    "aarch64-linux-android"
    "armv7-linux-androideabi"
    "i686-linux-android"
    "x86_64-linux-android"
)

# Build for each Android target
for target in "${ANDROID_TARGETS[@]}"; do
    echo "Building for $target..."
    cargo build --target $target --release
done

echo "✅ Android build completed!"
```

## 🛠️ Custom Build Configuration

### Environment Variables

```bash
# Rust compilation flags
export RUSTFLAGS="-C opt-level=3 -C target-cpu=native"

# CMake configuration
export CMAKE_BUILD_TYPE=Release

# Number of parallel jobs
export MAKEFLAGS="-j$(nproc)"

# Android NDK path
export ANDROID_NDK_ROOT=/path/to/ndk
```

### Cargo.toml Optimization

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true

[profile.release-with-debug]
inherits = "release"
debug = true
strip = false
```

This comprehensive build guide should help you build the library for any supported platform. For specific issues, refer to the [troubleshooting guide](./troubleshooting.md).
