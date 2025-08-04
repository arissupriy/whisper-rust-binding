#!/bin/bash

echo "🔗 Building Whisper Rust Binding as .so for Dual-Project Integration"
echo "===================================================================="

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Build for Linux (development)
echo "🖥️ Building for Linux..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Linux build successful"
    
    # Copy .so file to a standard location
    mkdir -p lib/linux
    cp target/release/libwhisper_rust_binding.so lib/linux/
    echo "📦 Library copied to: lib/linux/libwhisper_rust_binding.so"
else
    echo "❌ Linux build failed"
    exit 1
fi

# Build for Android
echo "🤖 Building for Android ARM64..."
if command -v aarch64-linux-android-gcc &> /dev/null; then
    cargo build --release --target aarch64-linux-android
    
    if [ $? -eq 0 ]; then
        echo "✅ Android ARM64 build successful"
        
        # Copy .so file for Android
        mkdir -p lib/android/arm64-v8a
        cp target/aarch64-linux-android/release/libwhisper_rust_binding.so lib/android/arm64-v8a/
        echo "📦 Android library copied to: lib/android/arm64-v8a/libwhisper_rust_binding.so"
    else
        echo "⚠️ Android build failed - continuing with Linux only"
    fi
else
    echo "⚠️ Android NDK not found - skipping Android build"
fi

echo ""
echo "📋 Build Summary:"
echo "================"
if [ -f "lib/linux/libwhisper_rust_binding.so" ]; then
    echo "✅ Linux: lib/linux/libwhisper_rust_binding.so"
    echo "   Size: $(du -h lib/linux/libwhisper_rust_binding.so | cut -f1)"
fi

if [ -f "lib/android/arm64-v8a/libwhisper_rust_binding.so" ]; then
    echo "✅ Android: lib/android/arm64-v8a/libwhisper_rust_binding.so"
    echo "   Size: $(du -h lib/android/arm64-v8a/libwhisper_rust_binding.so | cut -f1)"
fi

echo ""
echo "🔗 Integration Notes:"
echo "===================="
echo "1. Copy libwhisper_rust_binding.so to your Flutter project"
echo "2. Generate FRB bindings with: flutter_rust_bridge_codegen generate"
echo "3. Register validation callback from quran_assistant_engine"
echo "4. Use QuranWhisperApi for integrated functionality"

echo ""
echo "🎯 Ready for dual-project integration!"

# Create integration example
echo ""
echo "📝 Creating integration example..."
cat > integration_example.dart << 'EOF'
// Example: Integrating whisper-rust-binding with quran_assistant_engine
import 'generated_bindings_whisper.dart' as whisper;
import 'generated_bindings_quran.dart' as quran;

class IntegratedQuranTranscriber {
  late whisper.WhisperRustBinding _whisperBinding;
  late quran.QuranAssistantEngine _quranEngine;
  
  Future<void> initialize() async {
    // Initialize both libraries
    _whisperBinding = whisper.WhisperRustBinding(
      dylib: DynamicLibrary.open('libwhisper_rust_binding.so')
    );
    _quranEngine = quran.QuranAssistantEngine(
      dylib: DynamicLibrary.open('libquran_assistant_engine.so')
    );
    
    // Register validation callback
    await _whisperBinding.whisperRegisterQuranValidator(
      callback: _validateWithQuranEngine,
    );
  }
  
  ValidationResponse _validateWithQuranEngine(
    String transcribedText, 
    int ayahId, 
    int surahId
  ) {
    // Use quran_assistant_engine for validation
    final ayahData = _quranEngine.getAyah(surahId: surahId, ayahId: ayahId);
    final similarity = _quranEngine.calculateSimilarity(
      text1: transcribedText, 
      text2: ayahData.text
    );
    
    return ValidationResponse(
      isValid: similarity > 0.8,
      similarityScore: similarity,
      correctText: ayahData.text,
      wordCountMatch: _countMatchingWords(transcribedText, ayahData.text),
      ayahPosition: ayahData.position,
    );
  }
  
  Future<void> startMurajahSession({
    required int surahId,
    required int startingAyahId,
  }) async {
    await _whisperBinding.startQuranSession(
      instanceId: 'murajaah_session',
      surahId: surahId,
      startingAyahId: startingAyahId,
      sessionConfig: const QuranSessionConfig(),
    );
  }
}
EOF

echo "✅ Integration example created: integration_example.dart"
