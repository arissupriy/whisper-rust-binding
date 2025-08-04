#!/bin/bash

# 🔧 Dual Library Android Setup Script
# Sets up both whisper-rust-binding and quran_assistant_engine for Android FRB deployment

set -e

# Load configuration
source "$(dirname "$0")/android_config.sh"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

print_header() {
    echo -e "\n${BLUE}=====================================${NC}"
    echo -e "${YELLOW}$1${NC}"
    echo -e "${BLUE}=====================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️ $1${NC}"
}

print_header "Dual Library Android Setup for FRB"

# 1. Verify Android SDK and NDK
print_info "Verifying Android development environment..."

ANDROID_SDK_ROOT=$(eval echo $ANDROID_SDK_ROOT)
if [ ! -d "$ANDROID_SDK_ROOT" ]; then
    print_error "Android SDK not found at: $ANDROID_SDK_ROOT"
    echo "Please install Android SDK and update the path in android_config.sh"
    exit 1
fi

export ANDROID_HOME="$ANDROID_SDK_ROOT"
print_success "Android SDK found: $ANDROID_SDK_ROOT"

# Find and setup NDK
NDK_PATH="$ANDROID_SDK_ROOT/ndk"
if [ ! -d "$NDK_PATH" ]; then
    print_error "NDK not found at: $NDK_PATH"
    echo "Please install Android NDK through Android Studio SDK Manager"
    exit 1
fi

# Use the first available NDK version
NDK_VERSION=$(ls "$NDK_PATH" | head -n1)
if [ -z "$NDK_VERSION" ]; then
    print_error "No NDK versions found"
    exit 1
fi

export NDK_HOME="$NDK_PATH/$NDK_VERSION"
export ANDROID_NDK_HOME="$NDK_HOME"
print_success "Using NDK version: $NDK_VERSION"

# 2. Install Rust Android targets
print_info "Installing Rust Android targets..."

for target in "${ANDROID_TARGETS[@]}"; do
    if ! rustup target list --installed | grep -q "$target"; then
        print_info "Installing target: $target"
        rustup target add "$target"
    else
        print_success "Target already installed: $target"
    fi
done

# 3. Configure Cargo for Android
print_info "Configuring Cargo for Android cross-compilation..."

CARGO_CONFIG_DIR="$HOME/.cargo"
CARGO_CONFIG_FILE="$CARGO_CONFIG_DIR/config.toml"

mkdir -p "$CARGO_CONFIG_DIR"

# Generate Cargo config with proper NDK paths
cat > "$CARGO_CONFIG_FILE" << EOF
# Dual-library Android configuration
# Compatible with both whisper-rust-binding and quran_assistant_engine (FRB)

[target.aarch64-linux-android]
ar = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
linker = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android$TARGET_SDK_VERSION-clang"

[target.armv7-linux-androideabi]
ar = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
linker = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi$TARGET_SDK_VERSION-clang"

[target.i686-linux-android]
ar = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
linker = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android$TARGET_SDK_VERSION-clang"

[target.x86_64-linux-android]
ar = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
linker = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android$TARGET_SDK_VERSION-clang"

# FRB-compatible settings
[env]
CC_aarch64-linux-android = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android$TARGET_SDK_VERSION-clang"
CXX_aarch64-linux-android = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android$TARGET_SDK_VERSION-clang++"
AR_aarch64-linux-android = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"

CC_armv7-linux-androideabi = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi$TARGET_SDK_VERSION-clang"
CXX_armv7-linux-androideabi = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi$TARGET_SDK_VERSION-clang++"
AR_armv7-linux-androideabi = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"

CC_i686-linux-android = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android$TARGET_SDK_VERSION-clang"
CXX_i686-linux-android = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android$TARGET_SDK_VERSION-clang++"
AR_i686-linux-android = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"

CC_x86_64-linux-android = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android$TARGET_SDK_VERSION-clang"
CXX_x86_64-linux-android = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android$TARGET_SDK_VERSION-clang++"
AR_x86_64-linux-android = "$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
EOF

print_success "Cargo configuration created: $CARGO_CONFIG_FILE"

# 4. Install FRB codegen if not present
print_info "Checking Flutter Rust Bridge installation..."

if ! cargo install --list | grep -q flutter_rust_bridge_codegen; then
    print_info "Installing Flutter Rust Bridge codegen..."
    cargo install flutter_rust_bridge_codegen
else
    print_success "Flutter Rust Bridge codegen already installed"
fi

# 5. Create environment setup script
ENV_SCRIPT="setup_android_env.sh"
cat > "$ENV_SCRIPT" << 'EOF'
#!/bin/bash
# Source this file to set up Android environment for dual-library development

# Android SDK
export ANDROID_HOME=~/Android/Sdk
export ANDROID_SDK_ROOT=~/Android/Sdk

# Expand tilde
ANDROID_SDK_ROOT=$(eval echo $ANDROID_SDK_ROOT)
export ANDROID_HOME="$ANDROID_SDK_ROOT"
export ANDROID_SDK_ROOT="$ANDROID_SDK_ROOT"

# Find NDK
NDK_PATH="$ANDROID_SDK_ROOT/ndk"
NDK_VERSION=$(ls "$NDK_PATH" | head -n1)
export NDK_HOME="$NDK_PATH/$NDK_VERSION"
export ANDROID_NDK_HOME="$NDK_HOME"

# Update PATH
export PATH="$ANDROID_SDK_ROOT/tools:$ANDROID_SDK_ROOT/platform-tools:$PATH"

# FRB environment
export FLUTTER_RUST_BRIDGE_ANDROID_NDK_HOME="$NDK_HOME"
export FLUTTER_RUST_BRIDGE_ANDROID_SDK_ROOT="$ANDROID_SDK_ROOT"

echo "🔧 Android environment configured for dual-library development"
echo "SDK: $ANDROID_SDK_ROOT"
echo "NDK: $NDK_HOME"
echo ""
echo "Ready for:"
echo "  • whisper-rust-binding Android builds"
echo "  • quran_assistant_engine FRB builds"
echo "  • Flutter integration"
EOF

chmod +x "$ENV_SCRIPT"
print_success "Environment setup script created: $ENV_SCRIPT"

# 6. Create flutter project integration helper
FLUTTER_INTEGRATION_SCRIPT="integrate_with_flutter.sh"
cat > "$FLUTTER_INTEGRATION_SCRIPT" << 'EOF'
#!/bin/bash
# Helper script to integrate both libraries with Flutter project

if [ -z "$1" ]; then
    echo "Usage: $0 <flutter_project_path>"
    echo "Example: $0 ~/my_quran_app"
    exit 1
fi

FLUTTER_PROJECT="$1"
if [ ! -d "$FLUTTER_PROJECT" ]; then
    echo "Flutter project not found: $FLUTTER_PROJECT"
    exit 1
fi

echo "🔧 Integrating dual libraries with Flutter project: $FLUTTER_PROJECT"

# Create JNI directories
mkdir -p "$FLUTTER_PROJECT/android/app/src/main/jniLibs"/{arm64-v8a,armeabi-v7a,x86,x86_64}

echo "✅ JNI directories created"

# Copy whisper-rust-binding libraries
if [ -d "lib/android" ]; then
    cp lib/android/arm64-v8a/libwhisper_rust_binding.so "$FLUTTER_PROJECT/android/app/src/main/jniLibs/arm64-v8a/" 2>/dev/null || true
    cp lib/android/armeabi-v7a/libwhisper_rust_binding.so "$FLUTTER_PROJECT/android/app/src/main/jniLibs/armeabi-v7a/" 2>/dev/null || true
    cp lib/android/x86_64/libwhisper_rust_binding.so "$FLUTTER_PROJECT/android/app/src/main/jniLibs/x86_64/" 2>/dev/null || true
    cp lib/android/x86/libwhisper_rust_binding.so "$FLUTTER_PROJECT/android/app/src/main/jniLibs/x86/" 2>/dev/null || true
    echo "✅ whisper-rust-binding libraries copied"
else
    echo "⚠️ whisper-rust-binding Android libraries not found - run ./build_android.sh first"
fi

# Instructions for quran_assistant_engine
echo ""
echo "📋 Next steps for quran_assistant_engine integration:"
echo ""
echo "1. Build quran_assistant_engine for Android:"
echo "   cd /path/to/quran_assistant_engine"
echo "   flutter_rust_bridge_codegen build-android"
echo ""
echo "2. Or use FRB codegen in your Flutter project:"
echo "   cd $FLUTTER_PROJECT"
echo "   flutter_rust_bridge_codegen generate"
echo ""
echo "3. Update your pubspec.yaml to include both libraries"
echo "4. Update android/app/build.gradle with NDK configuration"
echo ""
echo "🎯 Ready for dual-library Flutter deployment!"
EOF

chmod +x "$FLUTTER_INTEGRATION_SCRIPT"
print_success "Flutter integration helper created: $FLUTTER_INTEGRATION_SCRIPT"

# 7. Generate example pubspec.yaml configuration
PUBSPEC_EXAMPLE="pubspec_dual_library_example.yaml"
cat > "$PUBSPEC_EXAMPLE" << 'EOF'
# Example pubspec.yaml configuration for dual-library setup
# Copy relevant sections to your Flutter project's pubspec.yaml

name: your_quran_app
description: Quran app with whisper-rust-binding + quran_assistant_engine

dependencies:
  flutter:
    sdk: flutter
  
  # Audio recording for whisper-rust-binding
  record: ^5.0.4
  permission_handler: ^11.0.1
  
  # Flutter Rust Bridge for both libraries
  flutter_rust_bridge: ^2.0.0
  ffi: ^2.1.0
  
  # Additional dependencies for Quran functionality
  # Add your quran_assistant_engine specific dependencies here

dev_dependencies:
  flutter_test:
    sdk: flutter
  flutter_rust_bridge_codegen: ^2.0.0
  ffigen: ^11.0.0
  build_runner: ^2.4.7

# Flutter Rust Bridge configuration
flutter_rust_bridge:
  # Paths to your Rust libraries
  rust_input:
    - ../rust_libraries/quran_assistant_engine/src/api/*.rs
    - ../rust_libraries/whisper_rust_binding/src/lib.rs
  
  # Generated Dart outputs
  dart_output:
    - lib/generated/quran_bindings.dart
    - lib/generated/whisper_bindings.dart
  
  # C header outputs for iOS
  c_output:
    - ios/Classes/quran_bindings.h
    - ios/Classes/whisper_bindings.h
  
  # Android configuration
  android:
    package_name: com.yourcompany.your_quran_app
    
flutter:
  uses-material-design: true
  
  # Include model files
  assets:
    - assets/models/
    - assets/quran_data/
EOF

print_success "Example pubspec.yaml created: $PUBSPEC_EXAMPLE"

# 8. Generate Android build.gradle example
GRADLE_EXAMPLE="android_build_gradle_example.gradle"
cat > "$GRADLE_EXAMPLE" << 'EOF'
// Example android/app/build.gradle configuration for dual-library setup
// Add these configurations to your existing build.gradle

android {
    compileSdkVersion 34
    
    compileOptions {
        sourceCompatibility JavaVersion.VERSION_1_8
        targetCompatibility JavaVersion.VERSION_1_8
    }
    
    defaultConfig {
        applicationId "com.yourcompany.your_quran_app"
        minSdkVersion 21  // Required for both libraries
        targetSdkVersion 34
        versionCode flutterVersionCode.toInteger()
        versionName flutterVersionName
        
        // NDK configuration for dual libraries
        ndk {
            abiFilters 'arm64-v8a', 'armeabi-v7a'
            // Add x86/x86_64 for emulator testing if needed
        }
        
        // Ensure libraries are packaged
        packagingOptions {
            pickFirst '**/libwhisper_rust_binding.so'
            pickFirst '**/libquran_assistant_engine.so'
            pickFirst '**/libc++_shared.so'
        }
    }
    
    buildTypes {
        release {
            signingConfig signingConfigs.debug
            minifyEnabled true
            useProguard true
            proguardFiles getDefaultProguardFile('proguard-android.txt'), 'proguard-rules.pro'
            
            // Optimize native libraries
            ndk {
                debugSymbolLevel 'SYMBOL_TABLE'
            }
        }
    }
}

// Add required permissions to AndroidManifest.xml:
/*
<uses-permission android:name="android.permission.RECORD_AUDIO" />
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" />
<uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" />
*/
EOF

print_success "Example build.gradle created: $GRADLE_EXAMPLE"

# Final summary
print_header "Setup Complete! 🎉"

echo "Your dual-library Android development environment is ready:"
echo ""
echo "📁 Files created:"
echo "  • $CARGO_CONFIG_FILE - Rust Android cross-compilation config"
echo "  • $ENV_SCRIPT - Environment setup script"
echo "  • $FLUTTER_INTEGRATION_SCRIPT - Flutter integration helper"
echo "  • $PUBSPEC_EXAMPLE - Example pubspec.yaml"
echo "  • $GRADLE_EXAMPLE - Example build.gradle"
echo ""
echo "🔧 Environment configured for:"
echo "  • Android SDK: $ANDROID_SDK_ROOT"
echo "  • Android NDK: $NDK_HOME"
echo "  • Rust targets: ${ANDROID_TARGETS[*]}"
echo ""
echo "📱 Next steps:"
echo "1. Build whisper-rust-binding for Android:"
echo "   ./build_android.sh"
echo ""
echo "2. Build quran_assistant_engine for Android:"
echo "   cd /path/to/quran_assistant_engine"
echo "   flutter_rust_bridge_codegen build-android"
echo ""
echo "3. Integrate with Flutter project:"
echo "   ./$FLUTTER_INTEGRATION_SCRIPT /path/to/your/flutter/project"
echo ""
echo "4. Generate FRB bindings and build Flutter app"
echo ""
print_success "Ready for dual-library Android deployment! 🚀"
