# API Reference

Complete API documentation for Whisper Rust Binding.

## 📚 Table of Contents

- [Rust API](#rust-api)
- [C API](#c-api)
- [Error Handling](#error-handling)
- [Data Types](#data-types)
- [Examples](#examples)

## 🦀 Rust API

The Rust API provides a safe, idiomatic interface for Rust applications.

### Core Functions

#### `init_whisper(model_path: &str) -> Result<i32, WhisperError>`

Initialize a Whisper model instance.

**Parameters:**
- `model_path`: Path to the GGML model file (e.g., "ggml-tiny.bin")

**Returns:**
- `Ok(instance_id)`: Unique instance identifier on success
- `Err(WhisperError)`: Error details on failure

**Example:**
```rust
use whisper_rust_binding::init_whisper;

let instance_id = init_whisper("ggml-tiny.bin")?;
println!("Model loaded with ID: {}", instance_id);
```

---

#### `process_audio(instance_id: i32, audio: &[f32], language: Option<&str>) -> Result<String, WhisperError>`

Transcribe audio data.

**Parameters:**
- `instance_id`: Model instance ID from `init_whisper`
- `audio`: Audio samples as f32 slice (16kHz, mono, normalized)
- `language`: Optional language code ("ar", "en", etc.) or None for auto-detect

**Returns:**
- `Ok(transcription)`: Transcribed text on success
- `Err(WhisperError)`: Error details on failure

**Example:**
```rust
use whisper_rust_binding::process_audio;

let audio_data: Vec<f32> = load_audio_file("speech.wav")?;
let result = process_audio(instance_id, &audio_data, Some("ar"))?;
println!("Transcription: {}", result);
```

---

#### `free_whisper(instance_id: i32) -> Result<(), WhisperError>`

Free a Whisper model instance and its resources.

**Parameters:**
- `instance_id`: Model instance ID to free

**Returns:**
- `Ok(())`: Success
- `Err(WhisperError)`: Error details on failure

**Example:**
```rust
use whisper_rust_binding::free_whisper;

free_whisper(instance_id)?;
println!("Resources freed successfully");
```

---

#### `get_model_info(instance_id: i32) -> Result<String, WhisperError>`

Get model version and information.

**Parameters:**
- `instance_id`: Model instance ID

**Returns:**
- `Ok(info)`: Model information string
- `Err(WhisperError)`: Error details on failure

**Example:**
```rust
use whisper_rust_binding::get_model_info;

let info = get_model_info(instance_id)?;
println!("Model info: {}", info);
```

---

#### `is_valid_model(instance_id: i32) -> bool`

Check if a model instance is valid and loaded.

**Parameters:**
- `instance_id`: Model instance ID to check

**Returns:**
- `true`: Model is valid and ready
- `false`: Model is invalid or not loaded

**Example:**
```rust
use whisper_rust_binding::is_valid_model;

if is_valid_model(instance_id) {
    println!("Model is ready for use");
} else {
    println!("Model is not available");
}
```

---

#### `process_audio_sliding_window(...) -> Result<String, WhisperError>`

**⚠️ Experimental**: Process audio with sliding window approach.

```rust
pub fn process_audio_sliding_window(
    instance_id: i32,
    audio: &[f32],
    window_size_sec: f32,
    step_size_sec: f32,
    sample_rate: i32,
    language: Option<&str>
) -> Result<String, WhisperError>
```

**Note**: Currently experimental. Use `murajaah_chunks` example for production chunk-based processing.

---

#### `validate_word(word: &str, global_data_words: &[&str]) -> bool`

Validate a word against a dictionary.

**Parameters:**
- `word`: Word to validate
- `global_data_words`: Dictionary words array

**Returns:**
- `true`: Word found in dictionary
- `false`: Word not found

**Example:**
```rust
use whisper_rust_binding::validate_word;

let dictionary = vec!["hello", "world", "rust"];
let is_valid = validate_word("hello", &dictionary);
println!("Word is valid: {}", is_valid);
```

### Complete Usage Pattern

```rust
use whisper_rust_binding::{init_whisper, process_audio, free_whisper, WhisperError};

fn transcribe_file(model_path: &str, audio_path: &str) -> Result<String, WhisperError> {
    // 1. Initialize model
    let instance_id = init_whisper(model_path)?;
    
    // 2. Load audio (implementation depends on your audio loading library)
    let audio_data = load_audio_16khz_mono(audio_path)?;
    
    // 3. Process audio
    let result = process_audio(instance_id, &audio_data, Some("ar"))?;
    
    // 4. Clean up
    free_whisper(instance_id)?;
    
    Ok(result)
}
```

## 🔗 C API

The C API provides FFI functions for integration with other languages.

### Core C Functions

#### `whisper_rust_init(model_path: *const c_char) -> i32`

Initialize Whisper model from C.

**Parameters:**
- `model_path`: Null-terminated C string with model file path

**Returns:**
- `>= 0`: Instance ID on success
- `-1`: Failure

**Example (C++):**
```cpp
#include "whisper_rust_binding.h"

int instance_id = whisper_rust_init("ggml-tiny.bin");
if (instance_id >= 0) {
    printf("Model loaded with ID: %d\n", instance_id);
}
```

---

#### `whisper_rust_process_audio(...) -> bool`

Process audio from C.

```c
bool whisper_rust_process_audio(
    int32_t instance_id,
    const float* audio_data,
    int32_t audio_len,
    const char* language,
    char* result_buffer,
    int32_t result_buffer_size
);
```

**Parameters:**
- `instance_id`: Model instance ID
- `audio_data`: Audio samples array (f32, 16kHz, mono)
- `audio_len`: Number of audio samples
- `language`: Language code ("ar", "en") or NULL for auto-detect
- `result_buffer`: Output buffer for transcription
- `result_buffer_size`: Size of output buffer

**Returns:**
- `true`: Success, result in buffer
- `false`: Failure

**Example (C++):**
```cpp
float audio_data[16000]; // 1 second at 16kHz
char result_buffer[1024];

bool success = whisper_rust_process_audio(
    instance_id,
    audio_data,
    16000,
    "ar",
    result_buffer,
    sizeof(result_buffer)
);

if (success) {
    printf("Transcription: %s\n", result_buffer);
}
```

---

#### `whisper_rust_free(instance_id: i32) -> bool`

Free model instance from C.

**Parameters:**
- `instance_id`: Instance ID to free

**Returns:**
- `true`: Success
- `false`: Failure

---

#### `whisper_rust_is_valid(instance_id: i32) -> bool`

Check if instance is valid from C.

---

#### `whisper_rust_get_model_info(...) -> bool`

Get model information from C.

```c
bool whisper_rust_get_model_info(
    int32_t instance_id,
    char* info_buffer,
    int32_t info_buffer_size
);
```

### C Header File

```c
// whisper_rust_binding.h
#ifndef WHISPER_RUST_BINDING_H
#define WHISPER_RUST_BINDING_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Core functions
int32_t whisper_rust_init(const char* model_path);
bool whisper_rust_free(int32_t instance_id);
bool whisper_rust_is_valid(int32_t instance_id);

bool whisper_rust_process_audio(
    int32_t instance_id,
    const float* audio_data,
    int32_t audio_len,
    const char* language,
    char* result_buffer,
    int32_t result_buffer_size
);

bool whisper_rust_get_model_info(
    int32_t instance_id,
    char* info_buffer,
    int32_t info_buffer_size
);

bool whisper_rust_validate_word(
    const char* word,
    const char* const* global_data_words,
    int32_t global_data_words_len
);

#ifdef __cplusplus
}
#endif

#endif // WHISPER_RUST_BINDING_H
```

## ⚠️ Error Handling

### WhisperError Enum

```rust
#[derive(Error, Debug)]
pub enum WhisperError {
    #[error("Failed to initialize model: {0}")]
    ModelInitError(String),

    #[error("Invalid model: {0}")]
    InvalidModel(String),

    #[error("Failed to process audio: {0}")]
    ProcessingError(String),

    #[error("Invalid audio data")]
    InvalidAudioData,

    #[error("Internal error: {0}")]
    InternalError(String),
}
```

### Error Handling Patterns

```rust
use whisper_rust_binding::{init_whisper, WhisperError};

match init_whisper("ggml-tiny.bin") {
    Ok(instance_id) => {
        println!("Success: {}", instance_id);
    }
    Err(WhisperError::ModelInitError(msg)) => {
        eprintln!("Model initialization failed: {}", msg);
    }
    Err(WhisperError::InvalidModel(msg)) => {
        eprintln!("Invalid model: {}", msg);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## 📊 Data Types

### Audio Data Format

```rust
// Audio must be:
let audio_data: Vec<f32> = vec![
    // - 32-bit floating point samples
    // - 16kHz sample rate
    // - Mono channel
    // - Normalized to [-1.0, 1.0] range
];
```

### Language Codes

| Language | Code | Support Level |
|----------|------|---------------|
| Arabic | `"ar"` | ⭐⭐⭐⭐⭐ Excellent |
| English | `"en"` | ⭐⭐⭐⭐⭐ Excellent |
| Auto-detect | `None` | ⭐⭐⭐⭐ Good |
| Spanish | `"es"` | ⭐⭐⭐⭐ Good |
| French | `"fr"` | ⭐⭐⭐⭐ Good |
| German | `"de"` | ⭐⭐⭐⭐ Good |

### Model Files

| Model | File | Size | Memory | Speed | Accuracy |
|-------|------|------|--------|-------|----------|
| Tiny | `ggml-tiny.bin` | 39 MB | ~120 MB | Fastest | Good |
| Base | `ggml-base.bin` | 142 MB | ~210 MB | Fast | Better |
| Small | `ggml-small.bin` | 466 MB | ~600 MB | Medium | Best |
| Medium | `ggml-medium.bin` | 1.5 GB | ~1.8 GB | Slow | Excellent |

## 🎯 Examples

### Example 1: Basic Transcription

```rust
use whisper_rust_binding::{init_whisper, process_audio, free_whisper};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance_id = init_whisper("ggml-tiny.bin")?;
    
    // Simulate 1 second of audio (16000 samples at 16kHz)
    let audio_data: Vec<f32> = vec![0.0; 16000];
    
    let result = process_audio(instance_id, &audio_data, Some("ar"))?;
    println!("Result: {}", result);
    
    free_whisper(instance_id)?;
    Ok(())
}
```

### Example 2: Batch Processing

```rust
use whisper_rust_binding::{init_whisper, process_audio, free_whisper};

fn process_multiple_files(files: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let instance_id = init_whisper("ggml-tiny.bin")?;
    
    for file in files {
        let audio_data = load_audio_file(file)?;
        let result = process_audio(instance_id, &audio_data, Some("ar"))?;
        println!("File: {} -> {}", file, result);
    }
    
    free_whisper(instance_id)?;
    Ok(())
}
```

### Example 3: Error Handling

```rust
use whisper_rust_binding::{init_whisper, process_audio, free_whisper, WhisperError};

fn robust_transcription(model_path: &str, audio_data: &[f32]) -> Result<String, WhisperError> {
    // Initialize with error handling
    let instance_id = init_whisper(model_path)
        .map_err(|e| {
            eprintln!("Failed to load model: {}", e);
            e
        })?;
    
    // Process with automatic cleanup
    let result = process_audio(instance_id, audio_data, Some("ar"))
        .and_then(|transcription| {
            // Always clean up, even on success
            free_whisper(instance_id)?;
            Ok(transcription)
        })
        .or_else(|e| {
            // Clean up on error too
            let _ = free_whisper(instance_id);
            Err(e)
        })?;
    
    Ok(result)
}
```

### Example 4: Thread Safety

```rust
use whisper_rust_binding::{init_whisper, process_audio, free_whisper};
use std::sync::Arc;
use std::thread;

fn threaded_processing() -> Result<(), Box<dyn std::error::Error>> {
    // Note: Each thread needs its own instance
    let handles: Vec<_> = (0..4).map(|i| {
        thread::spawn(move || {
            let instance_id = init_whisper("ggml-tiny.bin")?;
            
            // Process thread-specific audio
            let audio_data = generate_test_audio(i);
            let result = process_audio(instance_id, &audio_data, Some("ar"))?;
            
            free_whisper(instance_id)?;
            
            Ok::<String, Box<dyn std::error::Error + Send + Sync>>(result)
        })
    }).collect();
    
    for handle in handles {
        let result = handle.join().unwrap()?;
        println!("Thread result: {}", result);
    }
    
    Ok(())
}
```

## 📋 Performance Tips

1. **Reuse Instances**: Load model once, process multiple audio files
2. **Appropriate Model Size**: Use tiny for development, small for production
3. **Audio Format**: Pre-convert audio to 16kHz mono f32 for best performance
4. **Memory Management**: Always call `free_whisper()` to prevent leaks
5. **Thread Safety**: Use separate instances per thread

## 🔧 Integration Patterns

### Pattern 1: Service Layer

```rust
pub struct WhisperService {
    instance_id: Option<i32>,
}

impl WhisperService {
    pub fn new(model_path: &str) -> Result<Self, WhisperError> {
        let instance_id = init_whisper(model_path)?;
        Ok(Self { instance_id: Some(instance_id) })
    }
    
    pub fn transcribe(&self, audio: &[f32], language: Option<&str>) -> Result<String, WhisperError> {
        let instance_id = self.instance_id.ok_or_else(|| 
            WhisperError::InvalidModel("Service not initialized".to_string()))?;
        process_audio(instance_id, audio, language)
    }
}

impl Drop for WhisperService {
    fn drop(&mut self) {
        if let Some(instance_id) = self.instance_id {
            let _ = free_whisper(instance_id);
        }
    }
}
```

### Pattern 2: RAII Wrapper

```rust
pub struct WhisperInstance {
    id: i32,
}

impl WhisperInstance {
    pub fn new(model_path: &str) -> Result<Self, WhisperError> {
        let id = init_whisper(model_path)?;
        Ok(Self { id })
    }
    
    pub fn transcribe(&self, audio: &[f32], language: Option<&str>) -> Result<String, WhisperError> {
        process_audio(self.id, audio, language)
    }
    
    pub fn get_info(&self) -> Result<String, WhisperError> {
        get_model_info(self.id)
    }
}

impl Drop for WhisperInstance {
    fn drop(&mut self) {
        let _ = free_whisper(self.id);
    }
}
```

---

For more detailed examples, see the [examples directory](../examples/) and other documentation files.
