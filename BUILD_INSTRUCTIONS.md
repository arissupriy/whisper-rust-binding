# Whisper Rust Binding - Build Instructions

This project provides Rust bindings for OpenAI's Whisper speech-to-text model, with support for both Linux and Android platforms.

## Quick Start

### Prerequisites

#### For Linux builds:
- Rust (install from [rustup.rs](https://rustup.rs/))
- CMake
- Build tools (`build-essential` on Ubuntu/Debian)

```bash
# Ubuntu/Debian
sudo apt update && sudo apt install build-essential cmake

# Fedora/RHEL
sudo dnf install gcc gcc-c++ cmake make

# macOS
xcode-select --install
brew install cmake
```

#### For Android builds:
- All Linux prerequisites above
- Android NDK
- `ANDROID_NDK_HOME` environment variable set

```bash
# Download Android NDK from https://developer.android.com/ndk/downloads
# Extract and set environment variable
export ANDROID_NDK_HOME=/path/to/android-ndk
```

### Building

#### Option 1: Build for specific platform

```bash
# Build for Linux only
./build_linux.sh

# Build for Android only
./build_android.sh

# Build for both platforms
./build_all.sh all
```

#### Option 2: Use the universal build script

```bash
# Linux with tests
./build_all.sh linux --clean --test

# Android with JNI support
./build_all.sh android --with-jni --create-example

# Both platforms, clean build
./build_all.sh all --clean
```

## Build Scripts

### `build_linux.sh` - Linux Build Script

Builds the library for Linux x86_64 platform.

```bash
./build_linux.sh [OPTIONS]

Options:
  --clean       Clean previous builds before building
  --test        Run tests after building
  --no-examples Don't build examples
  --help, -h    Show help message
```

**Output:**
- `target/release/libwhisper_rust.so` - Dynamic library
- `target/release/libwhisper_rust.a` - Static library
- `target/release/examples/` - Example binaries

### `build_android.sh` - Android Build Script

Builds the library for all Android architectures.

```bash
./build_android.sh [OPTIONS]

Options:
  --clean            Clean previous builds before building
  --with-jni         Enable JNI support for direct Java integration
  --continue-on-error Continue building other architectures if one fails
  --create-example   Create Android integration example
  --help, -h         Show help message

Environment variables:
  ANDROID_NDK_HOME   Path to Android NDK (required)
```

**Output:**
- `android/output/arm64-v8a/libwhisper_rust.so` - ARM64 library
- `android/output/armeabi-v7a/libwhisper_rust.so` - ARM32 library
- `android/output/x86/libwhisper_rust.so` - x86 library
- `android/output/x86_64/libwhisper_rust.so` - x86_64 library
- `android/output/include/whisper_rust.h` - C header file
- `android/example/` - Android integration example (if --create-example used)

### `build_all.sh` - Universal Build Script

Builds for one or both platforms using a unified interface.

```bash
./build_all.sh [PLATFORM] [OPTIONS]

Platforms:
  linux              Build for Linux x86_64
  android            Build for Android (all architectures)
  all                Build for both Linux and Android

Options:
  --clean            Clean previous builds before building
  --test             Run tests after building (Linux only)
  --no-examples      Don't build examples (Linux only)
  --with-jni         Enable JNI support (Android only)
  --continue-on-error Continue building other architectures if one fails (Android only)
  --create-example   Create Android integration example (Android only)
  --help, -h         Show help message
```

### `download_model.sh` - Model Download Script

Downloads Whisper models for testing.

```bash
./download_model.sh [MODEL] [OPTIONS]

Models:
  tiny, tiny.en      Smallest models (39 MB)
  base, base.en      Small models (142 MB)
  small, small.en    Medium models (244 MB)
  medium, medium.en  Large models (769 MB)
  large-v1           Very large model (1.5 GB)
  large-v2           Very large model (1.5 GB)
  large-v3           Latest large model (1.5 GB)

Options:
  --list, -l         List all available models
  --output-dir DIR   Output directory (default: models)
  --all-small        Download all small models (tiny, base, small)
  --help, -h         Show help message
```

## Usage Examples

### Complete Build Process

```bash
# 1. Clean build for both platforms
./build_all.sh all --clean

# 2. Download a model for testing
./download_model.sh tiny

# 3. Test the Linux build
./target/release/examples/transcribe_file audio.wav models/ggml-tiny.bin
```

### Linux Development Workflow

```bash
# Development build with tests
./build_linux.sh --clean --test

# Quick rebuild during development
./build_linux.sh

# Production build
./build_linux.sh --clean
```

### Android Development Workflow

```bash
# Set up Android NDK
export ANDROID_NDK_HOME=/path/to/android-ndk

# Build with JNI support and create example
./build_android.sh --clean --with-jni --create-example

# Copy libraries to your Android project
cp android/output/*/libwhisper_rust.so /path/to/your/android/project/app/src/main/jniLibs/
```

## Build Outputs

### Linux Build Artifacts

```
target/release/
├── libwhisper_rust.so          # Dynamic library
├── libwhisper_rust.a           # Static library
└── examples/
    ├── test_transcription      # Test example
    └── transcribe_file         # File transcription example
```

### Android Build Artifacts

```
android/output/
├── arm64-v8a/
│   └── libwhisper_rust.so      # ARM64 library
├── armeabi-v7a/
│   └── libwhisper_rust.so      # ARM32 library
├── x86/
│   └── libwhisper_rust.so      # x86 library
├── x86_64/
│   └── libwhisper_rust.so      # x86_64 library
├── include/
│   └── whisper_rust.h          # C header file
└── example/                    # Android integration example
    ├── CMakeLists.txt
    └── src/main/cpp/
        └── whisper_example.cpp
```

## Integration

### Using in C/C++ (Linux)

```c
#include "whisper_rust.h"

// Link with: -lwhisper_rust -L./target/release

int main() {
    int instance = whisper_rust_init("models/ggml-tiny.bin");
    if (instance >= 0) {
        // Use the library...
        whisper_rust_free(instance);
    }
    return 0;
}
```

### Using in Android Project

1. Copy libraries to your Android project:
```bash
cp android/output/*/libwhisper_rust.so app/src/main/jniLibs/
```

2. Copy header file:
```bash
cp android/output/include/whisper_rust.h app/src/main/cpp/include/
```

3. Update your `CMakeLists.txt`:
```cmake
# Add the whisper_rust library
add_library(whisper_rust SHARED IMPORTED)
set_target_properties(whisper_rust PROPERTIES
    IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/../../jniLibs/${ANDROID_ABI}/libwhisper_rust.so)

# Link against whisper_rust
target_link_libraries(your_target whisper_rust)
```

### Using as Rust Dependency

Add to your `Cargo.toml`:
```toml
[dependencies]
whisper-rust-binding = { path = "/path/to/whisper-rust-binding" }
```

```rust
use whisper_rust_binding::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance_id = init_whisper("models/ggml-tiny.bin")?;
    
    // Process audio...
    let result = process_audio(instance_id, &audio_data, Some("en"))?;
    println!("Transcription: {}", result);
    
    free_whisper(instance_id)?;
    Ok(())
}
```

## Troubleshooting

### Common Issues

1. **whisper.cpp not found**
   ```
   Error: whisper.cpp directory not found!
   ```
   - The script will automatically clone whisper.cpp if missing
   - Make sure you have git installed

2. **Android NDK not set**
   ```
   Error: ANDROID_NDK_HOME environment variable is not set
   ```
   - Download Android NDK and set `ANDROID_NDK_HOME`
   - `export ANDROID_NDK_HOME=/path/to/android-ndk`

3. **Build tools missing**
   ```
   Error: cmake is not installed
   ```
   - Install required build tools for your platform
   - See Prerequisites section above

4. **Rust targets missing**
   ```
   Error: target not installed
   ```
   - The scripts automatically install required targets
   - Manual install: `rustup target add aarch64-linux-android`

### Performance Tips

1. **Use appropriate model size:**
   - `tiny`: Fast, lower accuracy
   - `base/small`: Good balance
   - `large-v3`: Best accuracy, slower

2. **Build optimizations:**
   - Release builds are automatically optimized
   - Use `--clean` for production builds

3. **Android optimizations:**
   - Focus on ARM64 for modern devices
   - Test on actual hardware, not emulator

## Contributing

When modifying the build scripts:

1. Test on both platforms
2. Update documentation
3. Maintain backward compatibility
4. Follow the existing code style

## License

This project follows the same license as the whisper.cpp project it depends on.
