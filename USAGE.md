# Using Whisper Rust Binding

This document explains how to use the Whisper Rust Binding for audio transcription.

## Prerequisites

- Rust installed
- A Whisper model file (e.g., `ggml-tiny.bin`)
- Audio files to transcribe (WAV or MP3)

## Quick Start

### Step 1: Make Scripts Executable

First, make all shell scripts executable:

```bash
./make_executable.sh
```

### Step 2: Transcribe an Audio File

Use the `transcribe.sh` script to transcribe an audio file:

```bash
./transcribe.sh output.wav
```

You can also specify a language:

```bash
./transcribe.sh output.wav en
```

## Available Scripts

### run_transcript.sh

Quickly transcribe the `output.wav` file in the root directory using the `ggml-tiny.bin` model:

```bash
./run_transcript.sh
```

### transcribe.sh

Transcribe any audio file (WAV or MP3) with optional language specification:

```bash
./transcribe.sh path/to/audio.mp3 [language_code]
```

### run_test_local.sh

Run a comprehensive test using the local model and audio file:

```bash
./run_test_local.sh
```

## Examples Source Code

You can find example code in the `examples` directory:

- `test_transcription.rs`: Simple WAV file transcription
- `transcribe_file.rs`: More advanced example with MP3 support

## Building for Different Platforms

### Linux

```bash
./build_linux.sh
```

### Android

```bash
./android/build_all.sh
```

### Both Linux and Android

```bash
./build_all.sh
```

## Troubleshooting

### Audio File Issues

If you encounter issues with audio files:

1. Ensure your WAV files are in PCM format
2. For MP3 files, make sure ffmpeg is installed
3. Try converting your audio to 16kHz mono WAV format manually:

```bash
ffmpeg -i input.mp3 -ar 16000 -ac 1 output.wav
```

### Model Issues

If the model fails to load:

1. Verify you're using a compatible Whisper model file
2. Try a different model size (tiny, base, small, etc.)
3. Make sure the model file is in the correct location

## Advanced Usage

For more advanced usage, you can integrate the library into your own Rust, C, or other language projects. See the README.md for API examples.
