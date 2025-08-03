# Getting Started with Whisper Rust Binding

Welcome to Whisper Rust Binding! This guide will help you get up and running quickly.

## 📋 Prerequisites

Before you begin, ensure you have the following installed:

### System Requirements

- **Operating System**: Linux (Ubuntu 18.04+, CentOS 7+) or Android API 21+
- **Rust**: Version 1.88+ with unsafe attributes support
- **CMake**: Version 3.10 or higher
- **C++ Compiler**: 
  - GCC 7+ or Clang 5+ on Linux
  - Android NDK for Android builds
- **FFmpeg**: For audio file processing
- **Git**: For cloning repositories

### Checking Prerequisites

```bash
# Check Rust version
rustc --version
# Should show: rustc 1.88.0 or higher

# Check CMake
cmake --version
# Should show: cmake version 3.10.0 or higher

# Check C++ compiler
g++ --version
# or
clang++ --version

# Check FFmpeg
ffmpeg -version

# Check Git
git --version
```

## 🚀 Installation

### Step 1: Clone the Repository

```bash
git clone --recursive https://github.com/your-username/whisper-rust-binding.git
cd whisper-rust-binding
```

**Important**: Use `--recursive` to include the whisper.cpp submodule.

### Step 2: Build the Project

#### For Linux Development

```bash
# Make build script executable
chmod +x build_linux.sh

# Run the build
./build_linux.sh
```

This script will:
- Build whisper.cpp with optimizations
- Compile the Rust library
- Build all examples
- Run basic tests

#### For Android Development

```bash
# Make build script executable
chmod +x build_android.sh

# Run Android build
./build_android.sh
```

### Step 3: Download a Whisper Model

```bash
# Make download script executable
chmod +x download_model.sh

# Download the tiny model (39 MB)
./download_model.sh

# Or manually download specific models:
# Tiny model (faster, less accurate)
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin

# Base model (balanced)
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin

# Small model (more accurate)
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin
```

## 🎵 First Test

### Test with Sample Audio

Let's verify everything works with a quick test:

```bash
# Create a simple test audio (3 seconds of silence for testing)
ffmpeg -f lavfi -i "sine=frequency=1000:duration=3" -ar 16000 -ac 1 test_audio.wav

# Test basic transcription
./target/debug/examples/transcribe_file ggml-tiny.bin test_audio.wav en

# Test with Arabic (if you have Arabic audio)
./target/debug/examples/transcribe_file ggml-tiny.bin your_arabic_audio.wav ar
```

Expected output:
```
Loading model from: ggml-tiny.bin
Processing audio file: test_audio.wav
Language: en
Model loaded successfully! Instance ID: 0
...
Transcription completed in 0.65s:
-------------------------------------------
[Transcription result will appear here]
-------------------------------------------
Resources freed successfully
```

## 📝 Your First Rust Program

Create a new file called `my_first_transcription.rs`:

```rust
use whisper_rust_binding::{init_whisper, process_audio, free_whisper};
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎙️ My First Whisper Transcription!");
    
    // Step 1: Initialize the model
    println!("Loading model...");
    let instance_id = init_whisper("ggml-tiny.bin")?;
    println!("✅ Model loaded with ID: {}", instance_id);
    
    // Step 2: Load audio data (this is simplified - see audio-processing.md for details)
    println!("Loading audio...");
    let audio_data = load_audio_file("test_audio.wav")?;
    println!("✅ Audio loaded: {} samples", audio_data.len());
    
    // Step 3: Transcribe
    println!("Transcribing...");
    let result = process_audio(instance_id, &audio_data, Some("en"))?;
    println!("✅ Transcription: {}", result);
    
    // Step 4: Clean up
    free_whisper(instance_id)?;
    println!("✅ Resources freed");
    
    Ok(())
}

// Simplified audio loading (use hound crate in real applications)
fn load_audio_file(path: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    // This is a placeholder - see audio-processing.md for proper implementation
    println!("ℹ️  Note: This is a simplified example. See docs/audio-processing.md for proper audio loading");
    Ok(vec![0.0; 16000]) // 1 second of silence at 16kHz
}
```

Compile and run:

```bash
# Add to Cargo.toml dependencies if creating a new project
cargo add whisper-rust-binding

# Or compile directly
rustc --extern whisper_rust_binding=target/debug/libwhisper_rust_binding.rlib my_first_transcription.rs

# Run
./my_first_transcription
```

## 🎯 Quick Examples

### Example 1: Arabic Transcription

```bash
# Using the command-line example
./target/debug/examples/transcribe_file ggml-tiny.bin arabic_audio.wav ar
```

### Example 2: Murajaah (Review) Mode

Perfect for studying Quran recitations:

```bash
# Break audio into 2-second chunks for easy review
./target/debug/examples/murajaah_chunks ggml-tiny.bin quran_recitation.wav ar 2
```

Output:
```
🎵 Murajaah (Review) Chunk-Based Transcription
==============================================
...
[0s-2s] أَوْ قَصَيِّبٍ مِنْ
[2s-4s] السَّمَاءُ
[4s-6s] إِنْ يُرْمَاتُهُ وَرَهُ
...
```

### Example 3: Batch Processing

```rust
use whisper_rust_binding::{init_whisper, process_audio, free_whisper};

fn process_multiple_files(model_path: &str, audio_files: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
    // Load model once
    let instance_id = init_whisper(model_path)?;
    
    // Process multiple files
    for audio_file in audio_files {
        println!("Processing: {}", audio_file);
        let audio_data = load_audio_file(audio_file)?;
        let result = process_audio(instance_id, &audio_data, Some("ar"))?;
        println!("Result: {}\n", result);
    }
    
    // Clean up once
    free_whisper(instance_id)?;
    Ok(())
}
```

## 🔧 Configuration

### Model Selection

| Model | Size | Speed | Accuracy | Use Case |
|-------|------|-------|----------|----------|
| `ggml-tiny.bin` | 39 MB | Fastest | Good | Development, testing |
| `ggml-base.bin` | 142 MB | Fast | Better | General purpose |
| `ggml-small.bin` | 466 MB | Medium | Best | Production use |
| `ggml-medium.bin` | 1.5 GB | Slow | Excellent | High accuracy needs |

### Language Codes

| Language | Code | Auto-detect |
|----------|------|-------------|
| Arabic | `ar` | ✅ Excellent |
| English | `en` | ✅ Excellent |
| Auto-detect | `null` or omit | ✅ Available |

## 🐛 Troubleshooting

### Common Issues

1. **"Model file not found"**
   ```bash
   # Make sure you downloaded the model
   ls -la ggml-*.bin
   # If missing, run download script
   ./download_model.sh
   ```

2. **"Failed to load audio"**
   ```bash
   # Check audio format (should be WAV, 16kHz, mono)
   ffprobe your_audio.wav
   
   # Convert if needed
   ffmpeg -i input.wav -ar 16000 -ac 1 output.wav
   ```

3. **"Compilation errors"**
   ```bash
   # Check Rust version
   rustc --version
   
   # Update if needed
   rustup update
   ```

4. **"Segmentation fault"**
   - Use single instance for multiple calls
   - Ensure proper resource cleanup
   - Check audio data validity

### Getting Help

- 📖 Read the [troubleshooting guide](./troubleshooting.md)
- 🐛 [Report bugs](https://github.com/your-username/whisper-rust-binding/issues)
- 💬 [Ask questions](https://github.com/your-username/whisper-rust-binding/discussions)

## ➡️ Next Steps

Now that you have the basics working:

1. 📚 **Learn the API**: Read the [API Reference](./api-reference.md)
2. 🎵 **Audio Processing**: Learn about [audio formats and processing](./audio-processing.md)
3. 📱 **Android Development**: Check the [Android integration guide](./android-integration.md)
4. ⚡ **Performance**: Optimize with the [performance guide](./performance-guide.md)

## 🎯 Quick Reference Card

```bash
# Essential commands
./build_linux.sh                    # Build for Linux
./download_model.sh                 # Download models
cargo build --examples              # Build examples
cargo test                          # Run tests

# Basic usage
./target/debug/examples/transcribe_file MODEL AUDIO LANG
./target/debug/examples/murajaah_chunks MODEL AUDIO LANG CHUNK_SIZE

# Model files
ggml-tiny.bin                       # 39 MB, fastest
ggml-base.bin                       # 142 MB, balanced
ggml-small.bin                      # 466 MB, accurate
```

Happy transcribing! 🎉
