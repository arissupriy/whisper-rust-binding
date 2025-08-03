#!/bin/bash

# Whisper Model Download Script
# Downloads Whisper models for testing

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

# Available models with their URLs and sizes
declare -A MODELS=(
    ["tiny"]="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
    ["tiny.en"]="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin"
    ["base"]="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
    ["base.en"]="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin"
    ["small"]="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
    ["small.en"]="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin"
    ["medium"]="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin"
    ["medium.en"]="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.en.bin"
    ["large-v1"]="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v1.bin"
    ["large-v2"]="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v2.bin"
    ["large-v3"]="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin"
)

declare -A MODEL_SIZES=(
    ["tiny"]="39 MB"
    ["tiny.en"]="39 MB"
    ["base"]="142 MB"
    ["base.en"]="142 MB"
    ["small"]="244 MB"
    ["small.en"]="244 MB"
    ["medium"]="769 MB"
    ["medium.en"]="769 MB"
    ["large-v1"]="1.5 GB"
    ["large-v2"]="1.5 GB"
    ["large-v3"]="1.5 GB"
)

# Function to show available models
show_models() {
    echo -e "${YELLOW}Available Whisper models:${NC}\n"
    
    echo -e "${BLUE}Multilingual models:${NC}"
    for model in tiny base small medium large-v1 large-v2 large-v3; do
        echo -e "  • ${GREEN}$model${NC} (${MODEL_SIZES[$model]})"
    done
    
    echo -e "\n${BLUE}English-only models (faster):${NC}"
    for model in tiny.en base.en small.en medium.en; do
        echo -e "  • ${GREEN}$model${NC} (${MODEL_SIZES[$model]})"
    done
    
    echo -e "\n${YELLOW}Recommended for testing: ${GREEN}tiny${NC} (smallest, fastest)"
    echo -e "${YELLOW}Recommended for production: ${GREEN}base${NC} or ${GREEN}small${NC} (good balance)"
    echo -e "${YELLOW}Best quality: ${GREEN}large-v3${NC} (largest, slowest)"
}

# Function to download a model
download_model() {
    local model_name="$1"
    local output_dir="${2:-models}"
    
    if [ -z "${MODELS[$model_name]}" ]; then
        PRINT_ERROR "Unknown model: $model_name"
        echo "Use --list to see available models"
        return 1
    fi
    
    local url="${MODELS[$model_name]}"
    local filename="ggml-${model_name}.bin"
    local output_path="$output_dir/$filename"
    
    # Create output directory
    mkdir -p "$output_dir"
    
    # Check if model already exists
    if [ -f "$output_path" ]; then
        echo -e "Model ${GREEN}$model_name${NC} already exists at: ${BLUE}$output_path${NC}"
        local size=$(du -h "$output_path" | cut -f1)
        echo -e "File size: $size"
        
        read -p "Overwrite existing file? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            PRINT_SUCCESS "Using existing model file"
            return 0
        fi
    fi
    
    PRINT_SECTION "Downloading $model_name model"
    echo -e "Model: ${GREEN}$model_name${NC}"
    echo -e "Size: ${YELLOW}${MODEL_SIZES[$model_name]}${NC}"
    echo -e "URL: ${BLUE}$url${NC}"
    echo -e "Output: ${BLUE}$output_path${NC}"
    
    # Download with progress bar
    if command -v wget >/dev/null 2>&1; then
        echo -e "\nDownloading with wget..."
        wget --progress=bar --show-progress -O "$output_path" "$url"
    elif command -v curl >/dev/null 2>&1; then
        echo -e "\nDownloading with curl..."
        curl -L --progress-bar -o "$output_path" "$url"
    else
        PRINT_ERROR "Neither wget nor curl is available"
        echo "Please install wget or curl to download models"
        return 1
    fi
    
    # Verify download
    if [ -f "$output_path" ]; then
        local downloaded_size=$(du -h "$output_path" | cut -f1)
        PRINT_SUCCESS "Model downloaded successfully"
        echo -e "Downloaded: ${BLUE}$output_path${NC} (${downloaded_size})"
        
        # Suggest next steps
        echo -e "\n${YELLOW}Next steps:${NC}"
        echo -e "  Test with: ${BLUE}./target/release/examples/transcribe_file audio.wav $output_path${NC}"
        echo -e "  Or use directly in your application"
    else
        PRINT_ERROR "Download failed"
        return 1
    fi
}

# Function to show help
show_help() {
    echo "Whisper Model Download Script"
    echo ""
    echo "Usage: $0 [MODEL] [OPTIONS]"
    echo ""
    echo "Models:"
    echo "  tiny, tiny.en      Smallest models (39 MB)"
    echo "  base, base.en      Small models (142 MB)"
    echo "  small, small.en    Medium models (244 MB)"
    echo "  medium, medium.en  Large models (769 MB)"
    echo "  large-v1           Very large model (1.5 GB)"
    echo "  large-v2           Very large model (1.5 GB)"
    echo "  large-v3           Latest large model (1.5 GB)"
    echo ""
    echo "Options:"
    echo "  --list, -l         List all available models"
    echo "  --output-dir DIR   Output directory (default: models)"
    echo "  --all-small        Download all small models (tiny, base, small)"
    echo "  --help, -h         Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 tiny                    # Download tiny model"
    echo "  $0 base --output-dir .     # Download base model to current directory"
    echo "  $0 --all-small             # Download tiny, base, and small models"
    echo "  $0 --list                  # List available models"
}

# Main script execution
main() {
    PRINT_SECTION "Whisper Model Download Script"
    
    # Parse arguments
    local model_name=""
    local output_dir="models"
    local download_all_small=false
    local list_models=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --list|-l)
                list_models=true
                shift
                ;;
            --output-dir)
                output_dir="$2"
                shift 2
                ;;
            --all-small)
                download_all_small=true
                shift
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            -*)
                PRINT_ERROR "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
            *)
                if [ -z "$model_name" ]; then
                    model_name="$1"
                else
                    PRINT_ERROR "Multiple model names specified"
                    exit 1
                fi
                shift
                ;;
        esac
    done
    
    # Handle list models
    if [ "$list_models" = true ]; then
        show_models
        exit 0
    fi
    
    # Handle download all small models
    if [ "$download_all_small" = true ]; then
        echo -e "Downloading all small models to: ${BLUE}$output_dir${NC}\n"
        local small_models=("tiny" "base" "small")
        local failed_downloads=()
        
        for model in "${small_models[@]}"; do
            if download_model "$model" "$output_dir"; then
                echo ""
            else
                failed_downloads+=("$model")
            fi
        done
        
        if [ ${#failed_downloads[@]} -eq 0 ]; then
            PRINT_SUCCESS "All small models downloaded successfully"
        else
            PRINT_ERROR "Failed to download: ${failed_downloads[*]}"
            exit 1
        fi
        exit 0
    fi
    
    # Handle single model download
    if [ -z "$model_name" ]; then
        PRINT_ERROR "No model specified"
        echo "Use --list to see available models or --help for usage"
        exit 1
    fi
    
    download_model "$model_name" "$output_dir"
}

# Run the main function with all arguments
main "$@"
