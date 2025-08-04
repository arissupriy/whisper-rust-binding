# 📋 Complete Integration Examples
## End-to-End Implementation dengan whisper-rust-binding.so

### 🎯 Overview

Contoh lengkap implementasi Flutter app yang menggunakan **whisper-rust-binding.so** sebagai standalone transcription engine melalui Flutter Rust Bridge.

### 🏗️ Project Architecture

```
Development Workspace:
├── whisper-rust-binding/          # 🦀 Standalone Rust Project
│   ├── src/lib.rs                 # Core whisper functions
│   ├── Cargo.toml                 # Rust dependencies
│   ├── build_android.sh          # Build script
│   └── target/
│       └── aarch64-linux-android/release/
│           └── libwhisper_rust_binding.so  # 📦 Output library
│
└── flutter_quran_transcriber/    # 📱 Flutter Project (Terpisah)
    ├── lib/
    │   ├── generated/             # FRB generated bindings
    │   ├── services/              # Service layer
    │   ├── providers/             # Riverpod providers
    │   └── ui/                    # Flutter UI
    ├── android/app/src/main/jniLibs/  # 📂 .so files location
    ├── flutter_rust_bridge.yaml   # FRB configuration
    └── pubspec.yaml               # Flutter dependencies
```

### 📱 Complete Flutter Project Example

#### 1. flutter_quran_transcriber/pubspec.yaml

```yaml
name: flutter_quran_transcriber
description: Quran transcription app using whisper-rust-binding.so

version: 1.0.0+1

environment:
  sdk: '>=3.0.0 <4.0.0'
  flutter: ">=3.10.0"

dependencies:
  flutter:
    sdk: flutter

  # State Management
  flutter_riverpod: ^2.4.9
  riverpod_annotation: ^2.3.3

  # Audio Handling
  record: ^5.0.4
  audioplayers: ^5.2.1
  permission_handler: ^11.0.1

  # Flutter Rust Bridge - komunikasi dengan .so
  flutter_rust_bridge: ^2.0.0

  # File & Path utilities
  path_provider: ^2.1.1
  path: ^1.8.3
  file_picker: ^6.1.1

  # UI Components
  flutter_hooks: ^0.20.3
  freezed_annotation: ^2.4.1
  json_annotation: ^4.8.1

  # HTTP untuk download models
  http: ^1.1.0
  dio: ^5.4.0

dev_dependencies:
  flutter_test:
    sdk: flutter
  flutter_lints: ^3.0.0

  # Code Generation
  build_runner: ^2.4.7
  riverpod_generator: ^2.3.9
  flutter_rust_bridge_codegen: ^2.0.0
  freezed: ^2.4.6
  json_serializable: ^6.7.1

flutter:
  uses-material-design: true
  
  assets:
    - assets/models/
    - assets/images/
    - assets/fonts/

  fonts:
    - family: Amiri
      fonts:
        - asset: assets/fonts/Amiri-Regular.ttf
        - asset: assets/fonts/Amiri-Bold.ttf
          weight: 700
```

#### 2. flutter_quran_transcriber/flutter_rust_bridge.yaml

```yaml
# FRB Configuration - pointing to standalone whisper-rust-binding
rust_input: "../whisper-rust-binding/src/lib.rs"
dart_output: "lib/generated"
c_output: "ios/Runner"
rust_crate_dir: "../whisper-rust-binding/"
class_name: "WhisperBinding"
dart_format_line_length: 120
```

#### 3. android/app/src/main/jniLibs Structure

```bash
# Required .so files for Android
android/app/src/main/jniLibs/
├── arm64-v8a/
│   ├── libwhisper_rust_binding.so     # Main whisper library
│   └── libc++_shared.so               # NDK C++ runtime (REQUIRED)
└── armeabi-v7a/
    ├── libwhisper_rust_binding.so     # Main whisper library  
    └── libc++_shared.so               # NDK C++ runtime (REQUIRED)
```

> ⚠️ **Important**: `libc++_shared.so` diperlukan karena:
> - Rust menggunakan C++ standard library untuk beberapa operasi
> - whisper.cpp (C++) memerlukan C++ runtime
> - Android NDK memerlukan shared C++ library untuk dynamic linking
> - Tanpa file ini, app akan crash dengan error "library not found"

### 🔧 whisper-rust-binding Functions (Standalone)

#### whisper-rust-binding/src/lib.rs

```rust
// Standalone whisper-rust-binding project
use std::collections::HashMap;
use std::sync::Mutex;

// Global instance storage
static WHISPER_INSTANCES: Mutex<HashMap<i32, WhisperInstance>> = Mutex::new(HashMap::new());

#[derive(Debug)]
pub struct WhisperInstance {
    pub id: i32,
    pub model_path: String,
    pub language: String,
    // Include actual whisper.cpp integration here
}

/// Initialize whisper instance with model
/// Returns instance ID for future operations
#[no_mangle]
pub extern "C" fn whisper_init(model_path: String, language: String) -> i32 {
    let instance_id = generate_instance_id();
    
    let instance = WhisperInstance {
        id: instance_id,
        model_path: model_path.clone(),
        language: language.clone(),
    };
    
    // Initialize actual whisper.cpp here
    // ...
    
    let mut instances = WHISPER_INSTANCES.lock().unwrap();
    instances.insert(instance_id, instance);
    
    instance_id
}

/// Transcribe audio data to text
/// Pure transcription function - no UI logic
#[no_mangle]
pub extern "C" fn whisper_transcribe(
    instance_id: i32,
    audio_data: Vec<f64>,
    language: String,
) -> String {
    let instances = WHISPER_INSTANCES.lock().unwrap();
    
    if let Some(instance) = instances.get(&instance_id) {
        // Call actual whisper.cpp transcription
        // This is where the core transcription happens
        transcribe_with_whisper_cpp(&audio_data, &language)
    } else {
        "Error: Invalid instance ID".to_string()
    }
}

/// Get transcription with segments and timestamps
#[no_mangle]
pub extern "C" fn whisper_transcribe_with_segments(
    instance_id: i32,
    audio_data: Vec<f64>,
    language: String,
) -> String {
    // Return JSON string with segments
    // Format: {"text": "...", "segments": [...], "timestamps": [...]}
    let result = whisper_transcribe(instance_id, audio_data, language);
    
    // Convert to structured format
    serde_json::json!({
        "text": result,
        "segments": [],  // Implement segment detection
        "confidence": 0.8,
        "processing_time": 1.5
    }).to_string()
}

/// Free whisper instance
#[no_mangle]
pub extern "C" fn whisper_free(instance_id: i32) -> bool {
    let mut instances = WHISPER_INSTANCES.lock().unwrap();
    instances.remove(&instance_id).is_some()
}

/// Get available models info
#[no_mangle]
pub extern "C" fn whisper_get_models() -> String {
    serde_json::json!([
        {
            "name": "tiny",
            "size_mb": 39,
            "languages": ["ar", "en"],
            "filename": "ggml-tiny.bin"
        },
        {
            "name": "base",
            "size_mb": 74,
            "languages": ["ar", "en"],
            "filename": "ggml-base.bin"
        },
        {
            "name": "small",
            "size_mb": 244,
            "languages": ["ar", "en"],
            "filename": "ggml-small.bin"
        }
    ]).to_string()
}

// Helper functions
fn generate_instance_id() -> i32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i32
}

fn transcribe_with_whisper_cpp(audio_data: &[f64], language: &str) -> String {
    // Actual whisper.cpp integration
    // This is where you integrate with whisper.cpp
    format!("Transcribed text in {} from {} samples", language, audio_data.len())
}
```

### 📱 Flutter Integration Code

#### 3. flutter_quran_transcriber/lib/services/whisper_service.dart

```dart
import 'dart:convert';
import '../generated/bridge_generated.dart';
import '../models/whisper_models.dart';

/// Service untuk komunikasi dengan whisper-rust-binding.so
class WhisperService {
  final WhisperBinding _binding;
  int? _currentInstanceId;

  WhisperService(this._binding);

  /// Initialize whisper dengan model
  Future<bool> initializeWithModel(String modelPath, {String language = 'ar'}) async {
    try {
      // Call whisper_init di .so file
      _currentInstanceId = await _binding.whisperInit(
        modelPath: modelPath,
        language: language,
      );
      
      return _currentInstanceId != null && _currentInstanceId! > 0;
    } catch (e) {
      print('❌ Whisper initialization failed: $e');
      return false;
    }
  }

  /// Transcribe audio menggunakan .so
  Future<TranscriptionResult> transcribeAudio(List<double> audioData, {String? language}) async {
    if (_currentInstanceId == null) {
      throw Exception('Whisper not initialized. Call initializeWithModel() first.');
    }

    try {
      // Call whisper_transcribe_with_segments di .so file
      final resultJson = await _binding.whisperTranscribeWithSegments(
        instanceId: _currentInstanceId!,
        audioData: audioData,
        language: language ?? 'ar',
      );

      // Parse JSON response dari .so
      final data = jsonDecode(resultJson);
      
      return TranscriptionResult(
        id: 'transcription_${DateTime.now().millisecondsSinceEpoch}',
        text: data['text'] ?? '',
        confidence: (data['confidence'] ?? 0.0).toDouble(),
        segments: [], // Parse segments if needed
        timestamp: DateTime.now(),
        language: language ?? 'ar',
        metadata: TranscriptionMetadata(
          processingTimeSec: (data['processing_time'] ?? 0.0).toDouble(),
          audioLengthSec: audioData.length / 16000.0,
          instanceId: _currentInstanceId!,
          config: const WhisperConfig(),
          totalSegments: (data['segments'] as List?)?.length ?? 0,
          totalWords: (data['text'] ?? '').split(' ').length,
        ),
      );

    } catch (e) {
      throw Exception('Transcription failed: $e');
    }
  }

  /// Get available models dari .so
  Future<List<WhisperModel>> getAvailableModels() async {
    try {
      final modelsJson = await _binding.whisperGetModels();
      final List<dynamic> modelsList = jsonDecode(modelsJson);
      
      return modelsList.map((model) => WhisperModel(
        name: model['name'],
        sizeMb: model['size_mb'],
        languages: List<String>.from(model['languages']),
        filename: model['filename'],
        isDownloaded: false, // Check local existence
      )).toList();

    } catch (e) {
      print('❌ Failed to get models: $e');
      return [];
    }
  }

  /// Free resources
  Future<void> dispose() async {
    if (_currentInstanceId != null) {
      try {
        await _binding.whisperFree(_currentInstanceId!);
        _currentInstanceId = null;
      } catch (e) {
        print('❌ Failed to free whisper instance: $e');
      }
    }
  }
}
```

#### 4. flutter_quran_transcriber/lib/ui/pages/transcription_page.dart

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:record/record.dart';
import '../../providers/whisper_provider.dart';
import '../../providers/audio_provider.dart';
import '../../models/whisper_models.dart';

class TranscriptionPage extends ConsumerStatefulWidget {
  const TranscriptionPage({super.key});

  @override
  ConsumerState<TranscriptionPage> createState() => _TranscriptionPageState();
}

class _TranscriptionPageState extends ConsumerState<TranscriptionPage> {
  bool _isRecording = false;
  List<double> _audioBuffer = [];

  @override
  Widget build(BuildContext context) {
    final whisperState = ref.watch(whisperProvider);
    final audioState = ref.watch(audioRecorderProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('مُحَوِّل الكلام - Whisper.so'),
        backgroundColor: Colors.blue,
        foregroundColor: Colors.white,
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            // Model selection
            _buildModelSelector(),
            const SizedBox(height: 20),

            // Recording controls
            _buildRecordingControls(),
            const SizedBox(height: 20),

            // Transcription results
            Expanded(
              child: _buildTranscriptionResults(),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildModelSelector() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'اختيار نموذج Whisper',
              style: TextStyle(
                fontSize: 16,
                fontWeight: FontWeight.bold,
              ),
            ),
            const SizedBox(height: 12),
            Consumer(
              builder: (context, ref, child) {
                return FutureBuilder<List<WhisperModel>>(
                  future: ref.read(whisperProvider.notifier).getAvailableModels(),
                  builder: (context, snapshot) {
                    if (snapshot.connectionState == ConnectionState.waiting) {
                      return const CircularProgressIndicator();
                    }

                    if (!snapshot.hasData || snapshot.data!.isEmpty) {
                      return const Text('لا توجد نماذج متاحة');
                    }

                    return DropdownButton<String>(
                      isExpanded: true,
                      items: snapshot.data!.map((model) {
                        return DropdownMenuItem(
                          value: model.filename,
                          child: Row(
                            children: [
                              Text(model.name),
                              const Spacer(),
                              Text(
                                '${model.sizeMb} MB',
                                style: const TextStyle(
                                  fontSize: 12,
                                  color: Colors.grey,
                                ),
                              ),
                            ],
                          ),
                        );
                      }).toList(),
                      onChanged: (filename) {
                        if (filename != null) {
                          _initializeWhisper(filename);
                        }
                      },
                      hint: const Text('اختر نموذج'),
                    );
                  },
                );
              },
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildRecordingControls() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
          children: [
            // Record button
            ElevatedButton.icon(
              onPressed: _isRecording ? _stopRecording : _startRecording,
              icon: Icon(_isRecording ? Icons.stop : Icons.mic),
              label: Text(_isRecording ? 'إيقاف التسجيل' : 'بدء التسجيل'),
              style: ElevatedButton.styleFrom(
                backgroundColor: _isRecording ? Colors.red : Colors.green,
                foregroundColor: Colors.white,
                padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
              ),
            ),

            // Transcribe button
            ElevatedButton.icon(
              onPressed: _audioBuffer.isNotEmpty ? _transcribeAudio : null,
              icon: const Icon(Icons.translate),
              label: const Text('تحويل النص'),
              style: ElevatedButton.styleFrom(
                padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildTranscriptionResults() {
    final results = ref.watch(transcriptionResultsProvider);
    
    return results.when(
      data: (transcriptions) {
        if (transcriptions.isEmpty) {
          return const Center(
            child: Text(
              'لا توجد نتائج تحويل\nابدأ بتسجيل الصوت',
              textAlign: TextAlign.center,
              style: TextStyle(
                fontSize: 16,
                color: Colors.grey,
              ),
            ),
          );
        }

        return ListView.builder(
          itemCount: transcriptions.length,
          itemBuilder: (context, index) {
            final result = transcriptions[index];
            return Card(
              margin: const EdgeInsets.only(bottom: 12),
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    // Header with confidence
                    Row(
                      children: [
                        const Icon(Icons.psychology, size: 20),
                        const SizedBox(width: 8),
                        Text(
                          'whisper.so - ${(result.confidence * 100).toInt()}%',
                          style: const TextStyle(
                            fontWeight: FontWeight.bold,
                            fontSize: 14,
                          ),
                        ),
                        const Spacer(),
                        Container(
                          padding: const EdgeInsets.symmetric(
                            horizontal: 8,
                            vertical: 4,
                          ),
                          decoration: BoxDecoration(
                            color: Colors.blue.withOpacity(0.1),
                            borderRadius: BorderRadius.circular(12),
                          ),
                          child: Text(
                            '${result.metadata.processingTimeSec.toStringAsFixed(1)}s',
                            style: const TextStyle(
                              fontSize: 12,
                              color: Colors.blue,
                            ),
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 12),

                    // Transcription text
                    Container(
                      width: double.infinity,
                      padding: const EdgeInsets.all(16),
                      decoration: BoxDecoration(
                        color: Colors.grey[50],
                        borderRadius: BorderRadius.circular(8),
                      ),
                      child: SelectableText(
                        result.text,
                        style: const TextStyle(
                          fontSize: 16,
                          height: 1.6,
                          fontFamily: 'Amiri',
                        ),
                        textAlign: TextAlign.right,
                        textDirection: TextDirection.rtl,
                      ),
                    ),

                    const SizedBox(height: 8),

                    // Metadata
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Text(
                          'الكلمات: ${result.metadata.totalWords}',
                          style: const TextStyle(
                            fontSize: 12,
                            color: Colors.grey,
                          ),
                        ),
                        Text(
                          'المدة: ${result.metadata.audioLengthSec.toStringAsFixed(1)}s',
                          style: const TextStyle(
                            fontSize: 12,
                            color: Colors.grey,
                          ),
                        ),
                        Text(
                          result.timestamp.toString().substring(11, 19),
                          style: const TextStyle(
                            fontSize: 12,
                            color: Colors.grey,
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            );
          },
        );
      },
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (error, stack) => Center(
        child: Text(
          'خطأ: $error',
          style: const TextStyle(color: Colors.red),
        ),
      ),
    );
  }

  // Helper methods
  Future<void> _initializeWhisper(String modelFilename) async {
    try {
      await ref.read(whisperProvider.notifier).initializeWithModel(
        'assets/models/$modelFilename',
      );
      
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('تم تحميل النموذج: $modelFilename'),
          backgroundColor: Colors.green,
        ),
      );
    } catch (e) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('فشل تحميل النموذج: $e'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

  Future<void> _startRecording() async {
    try {
      final hasPermission = await ref.read(audioRecorderProvider.notifier).requestPermission();
      if (!hasPermission) {
        throw Exception('إذن الميكروفون غير مُمنوح');
      }

      await ref.read(audioRecorderProvider.notifier).startRecording();
      setState(() => _isRecording = true);
      
    } catch (e) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('فشل بدء التسجيل: $e'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

  Future<void> _stopRecording() async {
    try {
      final audioData = await ref.read(audioRecorderProvider.notifier).stopRecording();
      setState(() {
        _isRecording = false;
        _audioBuffer = audioData ?? [];
      });
      
    } catch (e) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('فشل إيقاف التسجيل: $e'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

  Future<void> _transcribeAudio() async {
    if (_audioBuffer.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('لا توجد بيانات صوتية للتحويل'),
          backgroundColor: Colors.orange,
        ),
      );
      return;
    }

    try {
      // Call whisper.so melalui FRB
      await ref.read(whisperProvider.notifier).transcribeAudio(_audioBuffer);
      
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('تم تحويل الصوت بنجاح'),
          backgroundColor: Colors.green,
        ),
      );
      
    } catch (e) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('فشل التحويل: $e'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }
}
```

### 🔨 Build & Deployment Scripts

#### 5. build_complete.sh

```bash
#!/bin/bash
# Complete build script untuk kedua projects

set -e

echo "🔨 Building Whisper Rust Binding (Standalone)..."

# Step 1: Build whisper-rust-binding.so
cd whisper-rust-binding/
echo "📦 Building Android .so files..."
cargo build --target aarch64-linux-android --release
cargo build --target armv7-linux-androideabi --release

echo "✅ Whisper .so files built successfully"

# Step 2: Copy .so files ke Flutter project (INCLUDING libc++_shared.so)
echo "📂 Copying .so files to Flutter project..."
./copy_native_libs.sh

# Step 3: Build Flutter project
cd ../flutter_quran_transcriber/
echo "🚀 Building Flutter project..."

# Clean dan generate FRB bindings
flutter clean
flutter pub get
flutter_rust_bridge_codegen generate

# Generate other code
dart run build_runner build --delete-conflicting-outputs

# Build APK
flutter build apk --release

echo "🎉 Complete build finished!"
echo "📱 APK location: flutter_quran_transcriber/build/app/outputs/flutter-apk/app-release.apk"
```

#### 6. copy_native_libs.sh (Helper Script)

**Auto-copy script** yang handle semua .so files termasuk `libc++_shared.so`:

```bash
# Usage (di whisper-rust-binding directory):
./copy_native_libs.sh

# Output:
# 🔧 Native Libraries Copy Script
# ================================================
# 📁 Creating JNI directories...
# 📦 Copying arm64-v8a libraries...
# ✅ Copied libwhisper_rust_binding.so (arm64-v8a)
# ✅ Copied libc++_shared.so (arm64-v8a)
# 📦 Copying armeabi-v7a libraries...
# ✅ Copied libwhisper_rust_binding.so (armeabi-v7a)  
# ✅ Copied libc++_shared.so (armeabi-v7a)
# 🎉 Native libraries copy completed!
```

**Features**:
- ✅ Auto-detect Android NDK paths
- ✅ Copy main library + libc++_shared.so
- ✅ Support multiple architectures
- ✅ Verify file sizes and dependencies
- ✅ Error handling dengan clear messages

### 🎯 Key Points

1. **🦀 whisper-rust-binding**: Standalone Rust project, hanya menghasilkan `.so` files
2. **📱 Flutter project**: Terpisah project yang menggunakan `.so` melalui FRB
3. **🔗 FRB**: Bridge layer yang menghandle komunikasi type-safe
4. **📦 .so files**: Copied dari whisper-rust-binding ke Flutter project
5. **🚀 Build process**: Independent builds, lalu integration step

Arsitektur ini memberikan:
- ✅ Clear separation of concerns
- ✅ Reusable whisper.so untuk projects lain
- ✅ Independent development dan testing
- ✅ Type-safe communication melalui FRB
