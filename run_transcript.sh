#!/bin/bash

# Exit on error
set -e

# Build the example if needed
if [ ! -f "target/release/examples/test_transcription" ]; then
    echo "Building example..."
    cargo build --release --example test_transcription
fi

# Run the transcription
echo "Running transcription with local model and audio..."
LOG_LEVEL=info ./target/release/examples/test_transcription "./ggml-tiny.bin" "./output.wav"
