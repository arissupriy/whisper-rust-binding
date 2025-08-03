# Fixing Build Issues

This document helps you resolve common build issues with the whisper-rust-binding project.

## Common Error: Invalid whisper.cpp repository

If you're seeing an error like this:

```
thread 'main' panicked at build.rs:17:9:
Invalid whisper.cpp repository! Make sure you have cloned the correct repository.
```

This means there's an issue with the whisper.cpp repository in your project directory. The build script expects specific files to be present in the whisper.cpp directory.

## Quick Fix

Run our fix script to automatically resolve this issue:

```bash
chmod +x fix_whisper_repo.sh
./fix_whisper_repo.sh
```

This script will:
1. Remove any existing whisper.cpp directory (if present)
2. Clone the official whisper.cpp repository from GitHub
3. Verify that the repository is valid

## Rebuild After Fixing

After fixing the repository, rebuild the project with:

```bash
./rebuild.sh
```

Or manually with:

```bash
cargo build --release
```

## Manual Fix

If the automatic fix doesn't work, you can manually fix the issue:

1. Remove the existing whisper.cpp directory:
   ```bash
   rm -rf whisper.cpp
   ```

2. Clone the official repository:
   ```bash
   git clone https://github.com/ggerganov/whisper.cpp.git
   ```

3. Rebuild the project:
   ```bash
   cargo build --release
   ```

## Other Common Issues

### Missing CMake

If you get errors about CMake, install it with:

```bash
sudo apt-get update
sudo apt-get install cmake
```

### Missing C++ Compiler

If you get errors about g++ or compilation issues:

```bash
sudo apt-get update
sudo apt-get install build-essential
```

### Android NDK Issues

For Android builds, make sure the ANDROID_NDK_HOME environment variable is correctly set:

```bash
export ANDROID_NDK_HOME=/path/to/your/android/ndk
```

## Testing After Fix

After fixing the repository and rebuilding, test the transcription:

```bash
./run_transcript.sh
```

This will use your local model (ggml-tiny.bin) and audio file (output.wav) for testing.
