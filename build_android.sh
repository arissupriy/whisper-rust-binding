#!/bin/bash

# Whisper Rust Binding - Android Build Script
# This script builds the whisper-rust-binding library for all Android architectures

# Exit on any error
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

PRINT_ERROR() {
    echo -e "${RED}Error: $1${NC}"
}

PRINT_SUCCESS() {
    echo -e "${GREEN}✓ $1${NC}"
}

PRINT_WARNING() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Function to check if a command exists
check_command() {
    if ! command -v $1 &> /dev/null; then
        PRINT_ERROR "$1 is not installed or not in PATH"
        return 1
    fi
    return 0
}

# Function to check system requirements
check_requirements() {
    PRINT_SECTION "Checking system requirements"
    
    local all_good=true
    
    # Check for Rust
    if check_command rustc; then
        local rust_version=$(rustc --version)
        PRINT_SUCCESS "Rust found: $rust_version"
    else
        PRINT_ERROR "Rust is not installed. Please install Rust from https://rustup.rs/"
        all_good=false
    fi
    
    # Check for Cargo
    if check_command cargo; then
        local cargo_version=$(cargo --version)
        PRINT_SUCCESS "Cargo found: $cargo_version"
    else
        PRINT_ERROR "Cargo is not installed."
        all_good=false
    fi
    
    # Check for CMake
    if check_command cmake; then
        local cmake_version=$(cmake --version | head -n1)
        PRINT_SUCCESS "CMake found: $cmake_version"
    else
        PRINT_ERROR "CMake is not installed. Please install CMake."
        all_good=false
    fi
    
    if [ "$all_good" != true ]; then
        echo -e "\n${RED}Please install the missing dependencies and try again.${NC}"
        exit 1
    fi
}

# Function to check Android NDK
check_android_ndk() {
    PRINT_SECTION "Checking Android NDK"
    
    if [ -z "$ANDROID_NDK_HOME" ]; then
        PRINT_ERROR "ANDROID_NDK_HOME environment variable is not set."
        echo -e "${YELLOW}Please set it to the path of your Android NDK installation.${NC}"
        echo -e "${YELLOW}Example: export ANDROID_NDK_HOME=/path/to/android-ndk${NC}"
        echo -e "${YELLOW}You can download Android NDK from: https://developer.android.com/ndk/downloads${NC}"
        exit 1
    fi
    
    if [ ! -d "$ANDROID_NDK_HOME" ]; then
        PRINT_ERROR "ANDROID_NDK_HOME directory does not exist: $ANDROID_NDK_HOME"
        exit 1
    fi
    
    # Check if NDK seems valid
    if [ ! -f "$ANDROID_NDK_HOME/build/cmake/android.toolchain.cmake" ]; then
        PRINT_ERROR "Invalid Android NDK installation. Missing toolchain file."
        exit 1
    fi
    
    local ndk_version=""
    if [ -f "$ANDROID_NDK_HOME/source.properties" ]; then
        ndk_version=$(grep "Pkg.Revision" "$ANDROID_NDK_HOME/source.properties" | cut -d'=' -f2 | tr -d ' ')
    fi
    
    PRINT_SUCCESS "Android NDK found: $ANDROID_NDK_HOME"
    if [ -n "$ndk_version" ]; then
        echo -e "  Version: ${BLUE}$ndk_version${NC}"
    fi
}

# Function to setup Android targets
setup_android_targets() {
    PRINT_SECTION "Setting up Android targets"
    
    local targets=(
        "aarch64-linux-android"
        "armv7-linux-androideabi" 
        "i686-linux-android"
        "x86_64-linux-android"
    )
    
    for target in "${targets[@]}"; do
        if rustup target list | grep -q "$target (installed)"; then
            PRINT_SUCCESS "$target already installed"
        else
            echo "Installing $target..."
            if rustup target add "$target"; then
                PRINT_SUCCESS "$target installed successfully"
            else
                PRINT_ERROR "Failed to install $target"
                exit 1
            fi
        fi
    done
}

# Function to check and clone whisper.cpp if needed
setup_whisper_cpp() {
    PRINT_SECTION "Setting up whisper.cpp"
    
    if [ ! -d "whisper.cpp" ]; then
        echo "whisper.cpp directory not found. Cloning..."
        git clone https://github.com/ggerganov/whisper.cpp.git
        PRINT_SUCCESS "whisper.cpp cloned successfully"
    else
        PRINT_SUCCESS "whisper.cpp directory already exists"
        
        # Check if it's a valid git repository
        if [ -d "whisper.cpp/.git" ]; then
            echo "Updating whisper.cpp to latest version..."
            cd whisper.cpp
            git pull origin master
            cd ..
            PRINT_SUCCESS "whisper.cpp updated"
        fi
    fi
    
    # Verify essential files exist
    if [ ! -f "whisper.cpp/CMakeLists.txt" ]; then
        PRINT_ERROR "whisper.cpp appears to be corrupted or incomplete"
        exit 1
    fi
    
    PRINT_SUCCESS "whisper.cpp setup complete"
}

# Function to clean previous builds
clean_build() {
    PRINT_SECTION "Cleaning previous builds"
    
    if [ -d "target" ]; then
        echo "Removing target directory..."
        rm -rf target
        PRINT_SUCCESS "Target directory cleaned"
    fi
    
    if [ -d "android/output" ]; then
        echo "Removing Android output directory..."
        rm -rf android/output
        PRINT_SUCCESS "Android output directory cleaned"
    fi
}

# Function to create output directories
create_output_dirs() {
    local output_dir="android/output"
    mkdir -p "$output_dir/arm64-v8a"
    mkdir -p "$output_dir/armeabi-v7a" 
    mkdir -p "$output_dir/x86"
    mkdir -p "$output_dir/x86_64"
    mkdir -p "$output_dir/include"
}

# Function to build for a specific Android architecture
build_for_arch() {
    local arch_name=$1
    local rust_target=$2
    local display_name=$3
    
    PRINT_SECTION "Building for $display_name ($arch_name)"
    
    # Set Android-specific environment variables
    export ANDROID_API_LEVEL=21
    export AR="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
    export CC="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/${rust_target}${ANDROID_API_LEVEL}-clang"
    export CXX="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/${rust_target}${ANDROID_API_LEVEL}-clang++"
    export RANLIB="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ranlib"
    export STRIP="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip"
    
    # Build flags
    local build_flags="--target $rust_target --release"
    
    if [ "$ENABLE_JNI" = true ]; then
        build_flags="$build_flags --features android-jni"
    fi
    
    echo "Building with: cargo build $build_flags"
    
    if cargo build $build_flags; then
        PRINT_SUCCESS "Build completed for $display_name"
        
        # Copy the built library
        local lib_file="target/$rust_target/release/libwhisper_rust.so"
        if [ -f "$lib_file" ]; then
            cp "$lib_file" "android/output/$arch_name/"
            PRINT_SUCCESS "Library copied to android/output/$arch_name/"
            
            # Show library size
            local size=$(du -h "$lib_file" | cut -f1)
            echo -e "  📦 Library size: ${BLUE}$size${NC}"
        else
            PRINT_ERROR "Built library not found: $lib_file"
            return 1
        fi
    else
        PRINT_ERROR "Build failed for $display_name"
        return 1
    fi
}

# Function to build all architectures
build_all_architectures() {
    PRINT_SECTION "Building for all Android architectures"
    
    create_output_dirs
    
    # Architecture configurations
    # Format: arch_name:rust_target:display_name
    local architectures=(
        "arm64-v8a:aarch64-linux-android:ARM64 (64-bit)"
        "armeabi-v7a:armv7-linux-androideabi:ARM (32-bit)"
        "x86:i686-linux-android:x86 (32-bit)"
        "x86_64:x86_64-linux-android:x86_64 (64-bit)"
    )
    
    local failed_builds=()
    local successful_builds=()
    
    for arch_config in "${architectures[@]}"; do
        IFS=':' read -r arch_name rust_target display_name <<< "$arch_config"
        
        if build_for_arch "$arch_name" "$rust_target" "$display_name"; then
            successful_builds+=("$arch_name")
        else
            failed_builds+=("$arch_name")
            if [ "$CONTINUE_ON_ERROR" != true ]; then
                PRINT_ERROR "Build failed for $arch_name. Use --continue-on-error to build other architectures."
                exit 1
            fi
        fi
    done
    
    # Report results
    echo -e "\n${GREEN}Successful builds: ${#successful_builds[@]}${NC}"
    for arch in "${successful_builds[@]}"; do
        echo -e "  ✓ $arch"
    done
    
    if [ ${#failed_builds[@]} -gt 0 ]; then
        echo -e "\n${RED}Failed builds: ${#failed_builds[@]}${NC}"
        for arch in "${failed_builds[@]}"; do
            echo -e "  ✗ $arch"
        done
    fi
}

# Function to copy header files
copy_headers() {
    PRINT_SECTION "Copying header files"
    
    # Create a C header file for the exported functions
    local header_file="android/output/include/whisper_rust.h"
    
    cat > "$header_file" << 'EOF'
#ifndef WHISPER_RUST_H
#define WHISPER_RUST_H

#ifdef __cplusplus
extern "C" {
#endif

// Initialize whisper with model file path
// Returns instance ID on success, -1 on failure
int whisper_rust_init(const char* model_path);

// Free whisper instance
// Returns true on success
bool whisper_rust_free(int instance_id);

// Check if instance is valid
bool whisper_rust_is_valid(int instance_id);

// Process audio data
// Returns true on success, result written to result_buffer
bool whisper_rust_process_audio(
    int instance_id,
    const float* audio_data,
    int audio_len,
    const char* language,
    char* result_buffer,
    int result_buffer_size
);

// Process audio with sliding window
bool whisper_rust_process_audio_sliding_window(
    int instance_id,
    const float* audio_data,
    int audio_len,
    float window_size_sec,
    float step_size_sec,
    int sample_rate,
    const char* language,
    char* result_buffer,
    int result_buffer_size
);

// Validate word against dictionary
bool whisper_rust_validate_word(
    const char* word,
    const char** global_data_words,
    int global_data_words_len
);

// Get model information
bool whisper_rust_get_model_info(
    int instance_id,
    char* info_buffer,
    int info_buffer_size
);

#ifdef __cplusplus
}
#endif

#endif // WHISPER_RUST_H
EOF
    
    PRINT_SUCCESS "Header file created: $header_file"
}

# Function to show build results
show_results() {
    PRINT_SECTION "Build Results"
    
    local output_dir="android/output"
    
    echo -e "${GREEN}Built libraries for Android:${NC}"
    
    local architectures=("arm64-v8a" "armeabi-v7a" "x86" "x86_64")
    
    for arch in "${architectures[@]}"; do
        local lib_file="$output_dir/$arch/libwhisper_rust.so"
        if [ -f "$lib_file" ]; then
            local size=$(du -h "$lib_file" | cut -f1)
            echo -e "  📦 $arch: ${BLUE}$lib_file${NC} (${size})"
        else
            echo -e "  ❌ $arch: ${RED}Not built${NC}"
        fi
    done
    
    echo -e "\n${GREEN}Header file:${NC}"
    echo -e "  📄 ${BLUE}$output_dir/include/whisper_rust.h${NC}"
    
    echo -e "\n${YELLOW}Usage in Android project:${NC}"
    echo -e "  1. Copy .so files to: ${BLUE}app/src/main/jniLibs/<arch>/libwhisper_rust.so${NC}"
    echo -e "  2. Copy header to: ${BLUE}app/src/main/cpp/include/whisper_rust.h${NC}"
    echo -e "  3. Add to CMakeLists.txt: ${BLUE}target_link_libraries(your_target whisper_rust)${NC}"
    
    if [ "$ENABLE_JNI" = true ]; then
        echo -e "\n${YELLOW}JNI support enabled.${NC}"
        echo -e "  You can call the native functions directly from Java/Kotlin."
    fi
}

# Function to create example Android integration
create_android_example() {
    PRINT_SECTION "Creating Android integration example"
    
    local example_dir="android/example"
    mkdir -p "$example_dir"
    
    # Create example CMakeLists.txt
    cat > "$example_dir/CMakeLists.txt" << 'EOF'
cmake_minimum_required(VERSION 3.18.1)

project("whisper_example")

# Add the whisper_rust library
add_library(whisper_rust SHARED IMPORTED)
set_target_properties(whisper_rust PROPERTIES
    IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/../output/${ANDROID_ABI}/libwhisper_rust.so)

# Your native library
add_library(whisper_example SHARED
    src/main/cpp/whisper_example.cpp)

# Include directories
target_include_directories(whisper_example PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../output/include)

# Link libraries
target_link_libraries(whisper_example
    whisper_rust
    android
    log)
EOF
    
    # Create example C++ file
    mkdir -p "$example_dir/src/main/cpp"
    cat > "$example_dir/src/main/cpp/whisper_example.cpp" << 'EOF'
#include <jni.h>
#include <android/log.h>
#include <string>
#include "whisper_rust.h"

#define LOG_TAG "WhisperExample"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

extern "C" JNIEXPORT jint JNICALL
Java_com_example_whisper_WhisperNative_initWhisper(JNIEnv *env, jobject thiz, jstring model_path) {
    const char *path = env->GetStringUTFChars(model_path, nullptr);
    int result = whisper_rust_init(path);
    env->ReleaseStringUTFChars(model_path, path);
    return result;
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_example_whisper_WhisperNative_freeWhisper(JNIEnv *env, jobject thiz, jint instance_id) {
    return whisper_rust_free(instance_id);
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_example_whisper_WhisperNative_processAudio(JNIEnv *env, jobject thiz, jint instance_id,
                                                   jfloatArray audio_data, jstring language) {
    jfloat *audio = env->GetFloatArrayElements(audio_data, nullptr);
    jsize audio_len = env->GetArrayLength(audio_data);
    
    const char *lang = language ? env->GetStringUTFChars(language, nullptr) : nullptr;
    
    char result_buffer[10240];
    bool success = whisper_rust_process_audio(instance_id, audio, audio_len, lang, 
                                            result_buffer, sizeof(result_buffer));
    
    env->ReleaseFloatArrayElements(audio_data, audio, 0);
    if (language && lang) {
        env->ReleaseStringUTFChars(language, lang);
    }
    
    if (success) {
        return env->NewStringUTF(result_buffer);
    } else {
        return nullptr;
    }
}
EOF
    
    PRINT_SUCCESS "Android integration example created in $example_dir"
}

# Main script execution
main() {
    local start_time=$(date +%s)
    
    PRINT_SECTION "Whisper Rust Binding - Android Build"
    echo -e "Target: ${BLUE}Android (all architectures)${NC}"
    echo -e "Mode: ${BLUE}Release${NC}"
    echo -e "Date: ${BLUE}$(date)${NC}"
    
    # Parse command line arguments
    local clean_first=false
    ENABLE_JNI=false
    CONTINUE_ON_ERROR=false
    local create_example=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --clean)
                clean_first=true
                shift
                ;;
            --with-jni)
                ENABLE_JNI=true
                shift
                ;;
            --continue-on-error)
                CONTINUE_ON_ERROR=true
                shift
                ;;
            --create-example)
                create_example=true
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --clean            Clean previous builds before building"
                echo "  --with-jni         Enable JNI support for direct Java integration"
                echo "  --continue-on-error Continue building other architectures if one fails"
                echo "  --create-example   Create Android integration example"
                echo "  --help, -h         Show this help message"
                echo ""
                echo "Environment variables:"
                echo "  ANDROID_NDK_HOME   Path to Android NDK (required)"
                exit 0
                ;;
            *)
                PRINT_ERROR "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
    
    # Execute build steps
    check_requirements
    check_android_ndk
    setup_whisper_cpp
    setup_android_targets
    
    if [ "$clean_first" = true ]; then
        clean_build
    fi
    
    build_all_architectures
    copy_headers
    
    if [ "$create_example" = true ]; then
        create_android_example
    fi
    
    show_results
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    PRINT_SECTION "Build completed in ${duration} seconds"
    PRINT_SUCCESS "Android build ready!"
    
    if [ "$ENABLE_JNI" = true ]; then
        echo -e "\n${YELLOW}JNI support was enabled. The library can be used directly from Java/Kotlin.${NC}"
    fi
}

# Run the main function with all arguments
main "$@"
