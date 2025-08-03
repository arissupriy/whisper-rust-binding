#!/bin/bash

# Whisper Rust Binding - Universal Build Script
# This script can build for Linux, Android, or both platforms

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

# Function to show help
show_help() {
    echo "Whisper Rust Binding - Universal Build Script"
    echo ""
    echo "Usage: $0 [PLATFORM] [OPTIONS]"
    echo ""
    echo "Platforms:"
    echo "  linux              Build for Linux x86_64"
    echo "  android            Build for Android (all architectures)"
    echo "  all                Build for both Linux and Android"
    echo ""
    echo "Options:"
    echo "  --clean            Clean previous builds before building"
    echo "  --test             Run tests after building (Linux only)"
    echo "  --no-examples      Don't build examples (Linux only)"
    echo "  --with-jni         Enable JNI support (Android only)"
    echo "  --continue-on-error Continue building other architectures if one fails (Android only)"
    echo "  --create-example   Create Android integration example (Android only)"
    echo "  --help, -h         Show this help message"
    echo ""
    echo "Environment variables (Android only):"
    echo "  ANDROID_NDK_HOME   Path to Android NDK (required for Android builds)"
    echo ""
    echo "Examples:"
    echo "  $0 linux --clean --test"
    echo "  $0 android --with-jni --create-example"
    echo "  $0 all --clean"
}

# Function to check if build scripts exist
check_build_scripts() {
    if [ ! -f "build_linux.sh" ]; then
        PRINT_ERROR "build_linux.sh not found in current directory"
        exit 1
    fi
    
    if [ ! -f "build_android.sh" ]; then
        PRINT_ERROR "build_android.sh not found in current directory"
        exit 1
    fi
    
    # Make sure build scripts are executable
    chmod +x build_linux.sh build_android.sh
}

# Function to build for Linux
build_linux() {
    PRINT_SECTION "Building for Linux"
    
    local linux_args=()
    
    # Pass through relevant arguments
    for arg in "$@"; do
        case $arg in
            --clean|--test|--no-examples)
                linux_args+=("$arg")
                ;;
        esac
    done
    
    echo "Running: ./build_linux.sh ${linux_args[*]}"
    
    if ./build_linux.sh "${linux_args[@]}"; then
        PRINT_SUCCESS "Linux build completed successfully"
        return 0
    else
        PRINT_ERROR "Linux build failed"
        return 1
    fi
}

# Function to build for Android
build_android() {
    PRINT_SECTION "Building for Android"
    
    local android_args=()
    
    # Pass through relevant arguments
    for arg in "$@"; do
        case $arg in
            --clean|--with-jni|--continue-on-error|--create-example)
                android_args+=("$arg")
                ;;
        esac
    done
    
    echo "Running: ./build_android.sh ${android_args[*]}"
    
    if ./build_android.sh "${android_args[@]}"; then
        PRINT_SUCCESS "Android build completed successfully"
        return 0
    else
        PRINT_ERROR "Android build failed"
        return 1
    fi
}

# Function to show final results
show_final_results() {
    local platforms=("$@")
    
    PRINT_SECTION "Final Build Results"
    
    for platform in "${platforms[@]}"; do
        case $platform in
            linux)
                echo -e "${GREEN}Linux build:${NC}"
                if [ -f "target/release/libwhisper_rust.so" ]; then
                    local size=$(du -h "target/release/libwhisper_rust.so" | cut -f1)
                    echo -e "  📦 ${BLUE}target/release/libwhisper_rust.so${NC} (${size})"
                else
                    echo -e "  ❌ ${RED}Library not found${NC}"
                fi
                
                if [ -f "target/release/libwhisper_rust.a" ]; then
                    local size=$(du -h "target/release/libwhisper_rust.a" | cut -f1)
                    echo -e "  📦 ${BLUE}target/release/libwhisper_rust.a${NC} (${size})"
                fi
                ;;
                
            android)
                echo -e "${GREEN}Android build:${NC}"
                local output_dir="android/output"
                local architectures=("arm64-v8a" "armeabi-v7a" "x86" "x86_64")
                
                for arch in "${architectures[@]}"; do
                    local lib_file="$output_dir/$arch/libwhisper_rust.so"
                    if [ -f "$lib_file" ]; then
                        local size=$(du -h "$lib_file" | cut -f1)
                        echo -e "  📦 $arch: ${BLUE}$lib_file${NC} (${size})"
                    fi
                done
                
                if [ -f "$output_dir/include/whisper_rust.h" ]; then
                    echo -e "  📄 ${BLUE}$output_dir/include/whisper_rust.h${NC}"
                fi
                ;;
        esac
        echo ""
    done
    
    echo -e "${YELLOW}Next steps:${NC}"
    
    for platform in "${platforms[@]}"; do
        case $platform in
            linux)
                echo -e "Linux:"
                echo -e "  • Test with examples: ${BLUE}target/release/examples/test_transcription${NC}"
                echo -e "  • Link in C/C++: ${BLUE}-lwhisper_rust -L./target/release${NC}"
                echo -e "  • Use in Rust: Add to Cargo.toml dependencies"
                ;;
                
            android)
                echo -e "Android:"
                echo -e "  • Copy .so files to: ${BLUE}app/src/main/jniLibs/<arch>/${NC}"
                echo -e "  • Copy header to: ${BLUE}app/src/main/cpp/include/${NC}"
                echo -e "  • See android/example/ for integration example"
                ;;
        esac
    done
}

# Main script execution
main() {
    local start_time=$(date +%s)
    
    PRINT_SECTION "Whisper Rust Binding - Universal Build Script"
    echo -e "Date: ${BLUE}$(date)${NC}"
    
    # Parse arguments
    local platform=""
    local build_args=()
    local platforms_to_build=()
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            linux|android|all)
                platform="$1"
                shift
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                build_args+=("$1")
                shift
                ;;
        esac
    done
    
    # Validate platform argument
    if [ -z "$platform" ]; then
        PRINT_ERROR "Platform not specified"
        echo "Use one of: linux, android, all"
        echo "Use --help for more information"
        exit 1
    fi
    
    # Determine which platforms to build
    case $platform in
        linux)
            platforms_to_build=("linux")
            ;;
        android)
            platforms_to_build=("android")
            ;;
        all)
            platforms_to_build=("linux" "android")
            ;;
        *)
            PRINT_ERROR "Invalid platform: $platform"
            echo "Use one of: linux, android, all"
            exit 1
            ;;
    esac
    
    echo -e "Building for: ${BLUE}${platforms_to_build[*]}${NC}"
    
    # Check that build scripts exist
    check_build_scripts
    
    # Track build results
    local successful_builds=()
    local failed_builds=()
    
    # Build for each platform
    for target_platform in "${platforms_to_build[@]}"; do
        case $target_platform in
            linux)
                if build_linux "${build_args[@]}"; then
                    successful_builds+=("linux")
                else
                    failed_builds+=("linux")
                fi
                ;;
            android)
                if build_android "${build_args[@]}"; then
                    successful_builds+=("android")
                else
                    failed_builds+=("android")
                fi
                ;;
        esac
    done
    
    # Show final results
    if [ ${#successful_builds[@]} -gt 0 ]; then
        show_final_results "${successful_builds[@]}"
    fi
    
    # Report build status
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    echo -e "\n${BLUE}=============================================${NC}"
    echo -e "${YELLOW}Build Summary${NC}"
    echo -e "${BLUE}=============================================${NC}"
    
    if [ ${#successful_builds[@]} -gt 0 ]; then
        echo -e "${GREEN}Successful builds (${#successful_builds[@]}):${NC}"
        for platform in "${successful_builds[@]}"; do
            echo -e "  ✓ $platform"
        done
    fi
    
    if [ ${#failed_builds[@]} -gt 0 ]; then
        echo -e "\n${RED}Failed builds (${#failed_builds[@]}):${NC}"
        for platform in "${failed_builds[@]}"; do
            echo -e "  ✗ $platform"
        done
    fi
    
    echo -e "\n${BLUE}Total build time: ${duration} seconds${NC}"
    
    if [ ${#failed_builds[@]} -gt 0 ]; then
        PRINT_ERROR "Some builds failed"
        exit 1
    else
        PRINT_SUCCESS "All builds completed successfully!"
    fi
}

# Run the main function with all arguments
main "$@"
