#!/bin/bash

# Exit on error
set -e

# Check for arguments
if [ $# -lt 1 ]; then
    echo "Usage: $0 <audio_file> [language]"
    echo "  audio_file: Path to audio file (WAV or MP3)"
    echo "  language: Optional language code (e.g., 'en', 'ar') or omit for auto-detection"
    exit 1
fi

AUDIO_FILE="$1"
LANGUAGE="$2"

# Check if file exists
if [ ! -f "$AUDIO_FILE" ]; then
    echo "Error: Audio file not found: $AUDIO_FILE"
    exit 1
fi

# Check if model exists
if [ ! -f "ggml-tiny.bin" ]; then
    echo "Error: Model file not found: ggml-tiny.bin"
    echo "Please place the model file in the project root directory."
    exit 1
fi

# Build the example if needed
if [ ! -f "target/release/examples/transcribe_file" ]; then
    echo "Building example..."
    cargo build --release --example transcribe_file
fi

# Run transcription
echo "Transcribing audio file: $AUDIO_FILE"

if [ -z "$LANGUAGE" ]; then
    LOG_LEVEL=info ./target/release/examples/transcribe_file "./ggml-tiny.bin" "$AUDIO_FILE"
else
    LOG_LEVEL=info ./target/release/examples/transcribe_file "./ggml-tiny.bin" "$AUDIO_FILE" "$LANGUAGE"
fi
