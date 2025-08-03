# Building whisper-rust-binding

This document provides detailed instructions for building the whisper-rust-binding library for different platforms.

## Prerequisites

### Common Requirements

- Rust 1.63+ (2021 edition)
- CMake 3.12+
- C++ compiler (GCC, Clang, or MSVC)
- Git (to clone the repository)

### For Android

- Android NDK (r21+ recommended)
- Android SDK (optional)

## Setting Up the Build Environment

### Installing Rust

If you don't have Rust installed, you can install it using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen instructions to complete the installation.

### Installing CMake

**Ubuntu/Debian:**

```bash
sudo apt update
sudo apt install cmake
```

**macOS (with Homebrew):**

```bash
brew install cmake
```

**Windows:**

Download and install from the [official CMake website](https://cmake.org/download/).

### Setting Up for Android

1. Install Android Studio from the [official site](https://developer.android.com/studio)

2. Install the NDK through Android Studio:
   - Open Android Studio
   - Go to Settings/Preferences > Appearance & Behavior > System Settings > Android SDK
   - Select the "SDK Tools" tab
   - Check "NDK (Side by side)" and click "Apply"

3. Set environment variables:

```bash
export ANDROID_NDK_HOME=/path/to/android/ndk
# Optional: Set Android SDK path if not using default location
export ANDROID_SDK_HOME=/path/to/android/sdk
```

4. Install Rust targets for Android:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

## Building the Library

### Building for Linux

Use the provided build script:

```bash
./build_linux.sh
```

Or manually:

```bash
cargo build --release
```

The compiled library will be at `target/release/libwhisper_rust.so`.

### Building for Android

Use the provided build script to build for all architectures:

```bash
./android/build_all.sh
```

To build with JNI support:

```bash
./android/build_all.sh --with-jni
```

Or manually for specific architectures:

```bash
# For arm64-v8a (64-bit ARM)
cargo build --target aarch64-linux-android --release

# For armeabi-v7a (32-bit ARM)
cargo build --target armv7-linux-androideabi --release

# For x86 (32-bit Intel)
cargo build --target i686-linux-android --release

# For x86_64 (64-bit Intel)
cargo build --target x86_64-linux-android --release
```

### Building for All Platforms

To build for both Linux and Android in one command:

```bash
./build_all.sh
```

## Output Locations

After successful builds, the libraries can be found at:

- Linux: `output/linux/libwhisper_rust.so`
- Android:
  - `output/android/arm64-v8a/libwhisper_rust.so`
  - `output/android/armeabi-v7a/libwhisper_rust.so`
  - `output/android/x86/libwhisper_rust.so`
  - `output/android/x86_64/libwhisper_rust.so`

## Troubleshooting

### CMake Not Found

If you encounter issues with CMake not being found:

1. Verify CMake is installed: `cmake --version`
2. For Android builds, check the NDK path in `build.rs`

### Android Build Issues

If you face problems building for Android:

1. Make sure `ANDROID_NDK_HOME` is correctly set
2. Verify you have the correct Android targets installed: `rustup target list | grep android`
3. Check if the NDK version is compatible (r21+ recommended)

### C++ Standard Library Errors

If you see errors related to the C++ standard library:

1. For Android, make sure your app is configured to use the same C++ runtime (`c++_shared`)
2. For Linux, ensure you have the appropriate development packages installed

## Testing the Build

To verify that the library compiles correctly without running the full build:

```bash
./compile_test.sh
```

## Using Mock Implementation for Testing

The library includes a mock implementation that can be used for testing without loading actual models:

```rust
use whisper_rust_binding::mock::{init_mock, process_audio_mock};

fn main() {
    let instance_id = init_mock().unwrap();
    let audio_data = vec![0.0f32; 16000]; // 1 second of silence at 16kHz
    let transcript = process_audio_mock(&audio_data, Some("ar")).unwrap();
    println!("Mock transcript: {}", transcript);
}
```

This is useful for testing your application's integration with the library without requiring actual models or audio processing.
