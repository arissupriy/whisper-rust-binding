# 🧹 Example Files Cleanup Guide
## Menentukan File Examples yang Benar-benar Berguna

### 📊 Current Examples Analysis

Saat ini ada **18 file examples** di folder `/examples/`, tapi tidak semuanya berguna. Berikut analisis:

### ✅ **KEEP - Examples yang Berguna**

#### 1. **Core Functionality** (Essential)
```bash
# Basic functionality tests
examples/simple_test.rs              # 6 lines   - Import test only
examples/test_model_only.rs          # 30 lines  - Model loading test
examples/test_transcription.rs       # 77 lines  - Basic transcription
examples/transcribe_file.rs          # 116 lines - File transcription (MOST USEFUL)
```

#### 2. **Production Ready** (Recommended)
```bash
# Real production examples
examples/production_test.rs          # 98 lines  - Production test
examples/test_direct_whisper.rs      # 131 lines - Direct whisper.cpp integration
```

#### 3. **Flutter Integration** (For FRB)
```bash
# Flutter-specific examples
examples/flutter_api_demo.rs         # 197 lines - Flutter API simulation
examples/dual_project_integration.rs # 146 lines - Shows standalone concept
```

### ❌ **DELETE - Redundant/Incomplete Examples**

#### 1. **Mock/Demo Files** (Not Real Implementation)
```bash
examples/flutter_api_mock.rs         # 150 lines - Mock only, no real function
examples/simple_integration_test.rs  # 65 lines  - Redundant with test_transcription.rs
```

#### 2. **Sliding Window Duplicates** (Too Many Similar)
```bash
# Keep only 1-2 sliding window examples
examples/sliding_window.rs           # 252 lines - Complex
examples/simple_sliding_window.rs    # 203 lines - Simpler (KEEP)
examples/sliding_window_transcribe.rs # 213 lines - Redundant
examples/realtime_sliding_window.rs  # 201 lines - Redundant
examples/hybrid_sliding_window.rs    # 260 lines - Most complex, but keep for advanced
```

#### 3. **Specialized Use Cases** (Niche)
```bash
examples/murajaah_chunks.rs          # 234 lines - Very specific to Quran
examples/realtime_murajaah.rs        # 249 lines - Very specific to Quran
examples/flutter_realtime_demo.rs    # 218 lines - Redundant with flutter_api_demo.rs
```

### 🎯 **Recommended Examples Structure**

#### Keep Only These 7 Files:
```bash
examples/
├── 01_basic/
│   ├── simple_test.rs              # Import verification
│   ├── test_model_only.rs          # Model loading only
│   └── test_transcription.rs       # Basic transcription
├── 02_production/
│   ├── transcribe_file.rs          # ⭐ MAIN EXAMPLE - File transcription
│   ├── production_test.rs          # Production-ready test
│   └── test_direct_whisper.rs      # Direct whisper.cpp
├── 03_flutter/
│   ├── flutter_api_demo.rs         # Flutter integration demo
│   └── dual_project_integration.rs # Standalone project concept
└── 04_advanced/
    ├── simple_sliding_window.rs    # Basic sliding window
    └── hybrid_sliding_window.rs    # Advanced sliding window
```

### 🧹 Cleanup Script

Buat script untuk cleanup examples:

```bash
#!/bin/bash
# cleanup_examples.sh

echo "🧹 Cleaning up redundant example files..."

# Create organized directory structure
mkdir -p examples/01_basic
mkdir -p examples/02_production  
mkdir -p examples/03_flutter
mkdir -p examples/04_advanced

# Move useful files to organized structure
mv examples/simple_test.rs examples/01_basic/
mv examples/test_model_only.rs examples/01_basic/
mv examples/test_transcription.rs examples/01_basic/

mv examples/transcribe_file.rs examples/02_production/
mv examples/production_test.rs examples/02_production/
mv examples/test_direct_whisper.rs examples/02_production/

mv examples/flutter_api_demo.rs examples/03_flutter/
mv examples/dual_project_integration.rs examples/03_flutter/

mv examples/simple_sliding_window.rs examples/04_advanced/
mv examples/hybrid_sliding_window.rs examples/04_advanced/

# Delete redundant files
echo "🗑️ Deleting redundant examples..."
rm -f examples/flutter_api_mock.rs
rm -f examples/simple_integration_test.rs
rm -f examples/sliding_window.rs
rm -f examples/sliding_window_transcribe.rs
rm -f examples/realtime_sliding_window.rs
rm -f examples/murajaah_chunks.rs
rm -f examples/realtime_murajaah.rs
rm -f examples/flutter_realtime_demo.rs

echo "✅ Cleanup completed!"
echo "📂 New structure:"
tree examples/
```

### 📋 **Main Example Usage**

#### Primary Example: `transcribe_file.rs`
```bash
# Build and run the main example
cargo build --example transcribe_file

# Usage
./target/debug/examples/transcribe_file ggml-tiny.bin audio.wav ar

# Expected output:
# Loading model from: ggml-tiny.bin
# Processing audio file: audio.wav  
# Language: ar
# Model loaded successfully! Instance ID: 12345
# Transcription: السلام عليكم ورحمة الله وبركاته
```

### 🎯 **Cargo.toml Examples Section**

Update Cargo.toml to reflect organized examples:

```toml
# Add to Cargo.toml
[[example]]
name = "transcribe_file"
path = "examples/02_production/transcribe_file.rs"

[[example]]
name = "flutter_api_demo"
path = "examples/03_flutter/flutter_api_demo.rs"

[[example]]
name = "sliding_window"
path = "examples/04_advanced/simple_sliding_window.rs"
```

### ✅ **FINAL STATUS - Examples yang Bisa Digunakan**

Setelah cleanup dan fixes, berikut examples yang **benar-benar bisa digunakan**:

#### 🟢 **WORKING EXAMPLES** (Ready to Use)

```bash
# Basic Examples (Learning)
cargo run --example simple_test                    # ✅ Import verification
cargo run --example test_model_only ggml-tiny.bin # ✅ Model loading test  
cargo run --example test_transcription ggml-tiny.bin output.wav # ✅ Basic transcription

# Production Examples (Real Usage)
cargo run --example transcribe_file ggml-tiny.bin audio.wav ar  # ✅ ⭐ MAIN EXAMPLE
cargo run --example production_test ggml-tiny.bin               # ✅ Production test
cargo run --example test_direct_whisper ggml-tiny.bin output.wav # ✅ Direct whisper.cpp

# Flutter Examples (FRB Integration)
cargo run --example flutter_api_demo               # ✅ Flutter API simulation
cargo run --example dual_project_integration       # ✅ Standalone concept

# Advanced Examples (Complex Features)
cargo run --example simple_sliding_window ggml-tiny.bin output.wav # ✅ Sliding window
cargo run --example hybrid_sliding_window ggml-tiny.bin output.wav  # ✅ Advanced sliding
```

#### 🔧 **Key Fixes Applied**:
1. ✅ **Added `rlib` to Cargo.toml** → Examples bisa import library
2. ✅ **Fixed common module paths** → `#[path = "../00_common/mod.rs"]`
3. ✅ **Organized structure** → 01_basic/, 02_production/, 03_flutter/, 04_advanced/
4. ✅ **Removed 8 redundant files** → Dari 18 files → 10 useful files
5. ✅ **Updated Cargo.toml paths** → Semua examples registered

#### 🎯 **RECOMMENDED USAGE**:

**Primary Example**: `transcribe_file.rs` - **100% Working** 
```bash
# Contoh penggunaan utama:
cargo run --example transcribe_file ggml-tiny.bin test_short.wav ar

# Expected output:
# Loading model from: ggml-tiny.bin
# Processing audio file: test_short.wav
# Language: ar
# Model loaded successfully! Instance ID: 12345
# Transcription: [Arabic text hasil transcription]
# Processing time: 1.23 seconds
```

#### 📂 **Final Organized Structure**:
```
examples/
├── 00_common/               # Shared utilities
│   ├── audio_utils.rs       # Audio processing helpers
│   └── mod.rs              # Module exports
├── 01_basic/               # Learning & Basic Testing
│   ├── simple_test.rs       # Import verification
│   ├── test_model_only.rs   # Model loading only
│   └── test_transcription.rs # Basic transcription
├── 02_production/          # Real Usage (MAIN)
│   ├── transcribe_file.rs   # ⭐ File transcription
│   ├── production_test.rs   # Production test
│   └── test_direct_whisper.rs # Direct whisper.cpp
├── 03_flutter/             # Flutter Integration
│   ├── flutter_api_demo.rs  # Flutter API simulation
│   └── dual_project_integration.rs # Standalone concept
└── 04_advanced/            # Advanced Features
    ├── simple_sliding_window.rs # Sliding window basic
    └── hybrid_sliding_window.rs # Sliding window advanced
```
