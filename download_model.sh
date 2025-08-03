#!/bin/bash

# Exit on error
set -e

# Colors for output
GREEN="\033[0;32m"
YELLOW="\033[1;33m"
BLUE="\033[0;34m"
RED="\033[0;31m"
NC="\033[0m" # No Color

# Available models
MODELS=(
    "tiny"
    "tiny.en"
    "base"
    "base.en"
    "small"
    "small.en"
    "medium"
    "medium.en"
    "large"
    "large-v1"
    "large-v2"
    "large-v3"
)

# Create models directory
mkdir -p models

# Display available models
echo -e "${BLUE}Available Whisper models:${NC}"
for i in "${!MODELS[@]}"; do
    echo -e "  $((i+1)). ${YELLOW}${MODELS[$i]}${NC}"
done

# If no arguments provided, ask user to select a model
if [ $# -eq 0 ]; then
    echo -e "\nEnter the number of the model you want to download (1-${#MODELS[@]}): "
    read -r choice

    # Validate choice
    if ! [[ "$choice" =~ ^[0-9]+$ ]] || [ "$choice" -lt 1 ] || [ "$choice" -gt "${#MODELS[@]}" ]; then
        echo -e "${RED}Invalid choice. Please enter a number between 1 and ${#MODELS[@]}.${NC}"
        exit 1
    fi

    # Adjust for zero-based indexing
    MODEL=${MODELS[$((choice-1))]}
else
    # Use first argument as model name
    MODEL=$1

    # Check if it's a valid model
    if ! [[ " ${MODELS[*]} " =~ " ${MODEL} " ]]; then
        echo -e "${RED}Invalid model name: $MODEL${NC}"
        echo -e "Available models: ${YELLOW}${MODELS[*]}${NC}"
        exit 1
    fi
fi

# Model URL
URL="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-${MODEL}.bin"
OUTPUT="models/ggml-${MODEL}.bin"

echo -e "\n${BLUE}Downloading ${YELLOW}ggml-${MODEL}.bin${BLUE}...${NC}"

# Check if model already exists
if [ -f "$OUTPUT" ]; then
    echo -e "${YELLOW}Model already exists. Do you want to download it again? [y/N]${NC}"
    read -r answer
    if [[ ! "$answer" =~ ^[Yy]$ ]]; then
        echo -e "${GREEN}Skipping download.${NC}"
        exit 0
    fi
fi

# Download the model
if command -v curl &> /dev/null; then
    curl -L -o "$OUTPUT" "$URL"
elif command -v wget &> /dev/null; then
    wget -O "$OUTPUT" "$URL"
else
    echo -e "${RED}Error: Neither curl nor wget is installed. Please install one of them.${NC}"
    exit 1
fi

echo -e "\n${GREEN}Model downloaded successfully to $OUTPUT${NC}"
