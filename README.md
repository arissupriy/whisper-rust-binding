# Whisper Rust Binding

A Rust library that provides bindings to the [whisper.cpp](https://github.com/ggml-org/whisper.cpp) speech recognition engine. This library enables audio transcription with support for various features including streaming audio with a sliding window approach, particularly useful for Quran recitation (murajaah) in Arabic.

## Features

- Initialize and load Whisper models
- Model validity checking
- Audio transcription
- Sliding window processing for streaming audio
- Word validation against a global dictionary
- Support for multiple languages with specific optimizations for Arabic

## Requirements

- Rust 1.63+ (2021 edition)
- CMake 3.12+
- C++ compiler (GCC, Clang, or MSVC)

## Building

```bash
cargo build --release
```

This will generate a shared library (.so on Linux, .dylib on macOS, .dll on Windows) that can be used in other applications.

## Usage Examples

### Rust API

```rust
use whisper_rust_binding::{init_whisper, process_audio, free_whisper};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize whisper with a model
    let instance_id = init_whisper("path/to/model.bin")?;

    // Process audio (assuming you have audio data as f32)
    let audio_data: Vec<f32> = vec![/* ... */];
    let language = Some("ar"); // Arabic language

    let transcript = process_audio(instance_id, &audio_data, language)?;
    println!("Transcript: {}", transcript);

    // Free resources
    free_whisper(instance_id)?;

    Ok(())
}
```

### C API

```c
#include <stdio.h>
#include <stdlib.h>

// Function declarations for the shared library
extern int whisper_rust_init(const char* model_path);
extern bool whisper_rust_process_audio(int instance_id, const float* audio_data, int audio_len, 
                                       const char* language, char* result_buffer, int result_buffer_size);
extern bool whisper_rust_free(int instance_id);

int main() {
    // Initialize the model
    int instance_id = whisper_rust_init("path/to/model.bin");
    if (instance_id < 0) {
        fprintf(stderr, "Failed to initialize whisper model\n");
        return 1;
    }

    // Process audio (assuming you have audio data)
    float* audio_data = /* ... */;
    int audio_len = /* ... */;
    char result[10240] = {0};

    bool success = whisper_rust_process_audio(
        instance_id,
        audio_data,
        audio_len,
        "ar", // Arabic language
        result,
        sizeof(result)
    );

    if (success) {
        printf("Transcript: %s\n", result);
    } else {
        fprintf(stderr, "Failed to process audio\n");
    }

    // Free resources
    whisper_rust_free(instance_id);

    return 0;
}
```

## Integration with Flutter Rust Bridge

This library can be integrated with Flutter using Flutter Rust Bridge (FRB). Here's a basic example:

```dart
import 'package:quran_engine/ffi.dart';

void main() async {
  // Initialize the engine
  final engineId = await api.initWhisper('path/to/model.bin');

  // Process audio
  final audioData = Float32List.fromList([...]);
  final transcript = await api.processAudio(engineId, audioData, 'ar');

  print('Transcript: $transcript');

  // Free resources
  await api.freeWhisper(engineId);
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [whisper.cpp](https://github.com/ggml-org/whisper.cpp) - C++ implementation of OpenAI's Whisper model
