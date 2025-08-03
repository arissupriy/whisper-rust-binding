#!/bin/bash

# Whisper Rust Binding - Linux Build Script
# This script builds the whisper-rust-binding library for Linux x86_64

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
    
    # Check for Make
    if check_command make; then
        local make_version=$(make --version | head -n1)
        PRINT_SUCCESS "Make found: $make_version"
    else
        PRINT_ERROR "Make is not installed. Please install build-essential."
        all_good=false
    fi
    
    # Check for GCC/G++
    if check_command gcc && check_command g++; then
        local gcc_version=$(gcc --version | head -n1)
        PRINT_SUCCESS "GCC found: $gcc_version"
    else
        PRINT_ERROR "GCC/G++ is not installed. Please install build-essential."
        all_good=false
    fi
    
    if [ "$all_good" != true ]; then
        echo -e "\n${RED}Please install the missing dependencies and try again.${NC}"
        echo -e "${YELLOW}On Ubuntu/Debian: sudo apt update && sudo apt install build-essential cmake${NC}"
        exit 1
    fi
    
    PRINT_SUCCESS "All requirements satisfied"
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
    
    if [ -d "whisper.cpp/build" ]; then
        echo "Removing whisper.cpp build directory..."
        rm -rf whisper.cpp/build
        PRINT_SUCCESS "whisper.cpp build directory cleaned"
    fi
}

# Function to build the library
build_library() {
    PRINT_SECTION "Building whisper-rust-binding for Linux"
    
    # Build in release mode for optimal performance
    echo "Building Rust library..."
    
    # Set environment variables for optimization
    export RUSTFLAGS="-C target-cpu=native -C opt-level=3"
    
    if cargo build --release; then
        PRINT_SUCCESS "Build completed successfully"
    else
        PRINT_ERROR "Build failed"
        exit 1
    fi
}

# Function to run tests
run_tests() {
    PRINT_SECTION "Running tests"
    
    if cargo test; then
        PRINT_SUCCESS "All tests passed"
    else
        PRINT_ERROR "Some tests failed"
        echo -e "${YELLOW}Note: Test failures might be due to missing model files${NC}"
    fi
}

# Function to build examples
build_examples() {
    PRINT_SECTION "Building examples"
    
    if cargo build --examples --release; then
        PRINT_SUCCESS "Examples built successfully"
    else
        PRINT_ERROR "Failed to build examples"
        return 1
    fi
}

# Function to show build results
show_results() {
    PRINT_SECTION "Build Results"
    
    local lib_path="target/release"
    
    echo -e "${GREEN}Built libraries:${NC}"
    
    # Check for different library formats
    if [ -f "$lib_path/libwhisper_rust.so" ]; then
        local size=$(du -h "$lib_path/libwhisper_rust.so" | cut -f1)
        echo -e "  📦 Dynamic library: ${BLUE}$lib_path/libwhisper_rust.so${NC} (${size})"
    fi
    
    if [ -f "$lib_path/libwhisper_rust.a" ]; then
        local size=$(du -h "$lib_path/libwhisper_rust.a" | cut -f1)
        echo -e "  📦 Static library:  ${BLUE}$lib_path/libwhisper_rust.a${NC} (${size})"
    fi
    
    echo -e "\n${GREEN}Examples:${NC}"
    if [ -f "$lib_path/examples/test_transcription" ]; then
        echo -e "  🚀 ${BLUE}$lib_path/examples/test_transcription${NC}"
    fi
    
    if [ -f "$lib_path/examples/transcribe_file" ]; then
        echo -e "  🚀 ${BLUE}$lib_path/examples/transcribe_file${NC}"
    fi
    
    echo -e "\n${YELLOW}Usage:${NC}"
    echo -e "  To use the library in C/C++, link against: ${BLUE}$lib_path/libwhisper_rust.so${NC}"
    echo -e "  To use as Rust dependency, add to Cargo.toml: ${BLUE}whisper-rust-binding = { path = \"$(pwd)\" }${NC}"
    
    echo -e "\n${YELLOW}Next steps:${NC}"
    echo -e "  1. Download a model: ${BLUE}bash download_model.sh${NC} (if available)"
    echo -e "  2. Test with: ${BLUE}$lib_path/examples/test_transcription <audio_file> <model_file>${NC}"
}

# Main script execution
main() {
    local start_time=$(date +%s)
    
    PRINT_SECTION "Whisper Rust Binding - Linux Build"
    echo -e "Target: ${BLUE}Linux x86_64${NC}"
    echo -e "Mode: ${BLUE}Release${NC}"
    echo -e "Date: ${BLUE}$(date)${NC}"
    
    # Parse command line arguments
    local clean_first=false
    local run_tests_flag=false
    local build_examples_flag=true
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --clean)
                clean_first=true
                shift
                ;;
            --test)
                run_tests_flag=true
                shift
                ;;
            --no-examples)
                build_examples_flag=false
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --clean       Clean previous builds before building"
                echo "  --test        Run tests after building"
                echo "  --no-examples Don't build examples"
                echo "  --help, -h    Show this help message"
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
    setup_whisper_cpp
    
    if [ "$clean_first" = true ]; then
        clean_build
    fi
    
    build_library
    
    if [ "$build_examples_flag" = true ]; then
        build_examples
    fi
    
    if [ "$run_tests_flag" = true ]; then
        run_tests
    fi
    
    show_results
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    PRINT_SECTION "Build completed in ${duration} seconds"
    PRINT_SUCCESS "Linux build ready!"
}

# Run the main function with all arguments
main "$@"
