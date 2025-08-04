# 📱 Setup dan Instalasi
## Flutter + Whisper Rust Binding Setup

### 🔧 Prerequisites

```yaml
# pubspec.yaml dependencies
dependencies:
  flutter:
    sdk: flutter
  
  # State Management
  flutter_riverpod: ^2.4.9
  riverpod_annotation: ^2.3.3
  
  # Audio Recording
  record: ^5.0.4
  permission_handler: ^11.0.1
  
  # Flutter Rust Bridge
  flutter_rust_bridge: ^2.0.0
  
  # Path utilities
  path_provider: ^2.1.1
  path: ^1.8.3
  
  # UI utilities
  flutter_hooks: ^0.20.3
  
dev_dependencies:
  flutter_test:
    sdk: flutter
  
  # Code generation
  build_runner: ^2.4.7
  riverpod_generator: ^2.3.9
  flutter_rust_bridge_codegen: ^2.0.0
```

### 📱 Android Configuration

#### 1. android/app/src/main/AndroidManifest.xml

```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <!-- Audio permissions -->
    <uses-permission android:name="android.permission.RECORD_AUDIO" />
    <uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" />
    <uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" />
    
    <application
        android:label="quran_whisper_app"
        android:exported="true"
        android:icon="@mipmap/ic_launcher">
        
        <activity
            android:name=".MainActivity"
            android:exported="true"
            android:launchMode="singleTop"
            android:theme="@style/LaunchTheme"
            android:configChanges="orientation|keyboardHidden|keyboard|screenSize|smallestScreenSize|locale|layoutDirection|fontScale|screenLayout|density|uiMode"
            android:hardwareAccelerated="true"
            android:windowSoftInputMode="adjustResize">
            
            <meta-data
              android:name="io.flutter.embedding.android.NormalTheme"
              android:resource="@style/NormalTheme" />
              
            <intent-filter android:autoVerify="true">
                <action android:name="android.intent.action.MAIN"/>
                <category android:name="android.intent.category.LAUNCHER"/>
            </intent-filter>
        </activity>
        
        <meta-data
            android:name="flutterEmbedding"
            android:value="2" />
    </application>
</manifest>
```

#### 2. Native Libraries Setup

```bash
# Directory structure untuk native libraries
android/app/src/main/jniLibs/
├── arm64-v8a/
│   ├── libwhisper_rust_binding.so     # Main whisper library
│   └── libc++_shared.so               # NDK C++ runtime (REQUIRED)
└── armeabi-v7a/
    ├── libwhisper_rust_binding.so     # Main whisper library
    └── libc++_shared.so               # NDK C++ runtime (REQUIRED)
```

> ⚠️ **Critical**: Selalu sertakan `libc++_shared.so` karena:
> - whisper.cpp adalah C++ library yang memerlukan C++ runtime
> - Rust FFI dengan C++ memerlukan shared C++ library
> - Android memerlukan explicit linking untuk dynamic libraries
> - Tanpa file ini: `UnsatisfiedLinkError` atau `library not found`

**Lokasi libc++_shared.so di NDK**:
```bash
# NDK r25+ (recommended path)
$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_shared.so
$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/arm-linux-androideabi/libc++_shared.so

# NDK legacy path (fallback)
$ANDROID_NDK_ROOT/sources/cxx-stl/llvm-libc++/libs/arm64-v8a/libc++_shared.so
$ANDROID_NDK_ROOT/sources/cxx-stl/llvm-libc++/libs/armeabi-v7a/libc++_shared.so
```

#### 3. android/app/build.gradle

```gradle
android {
    compileSdkVersion 34
    ndkVersion "29.0.13599879"

    compileOptions {
        sourceCompatibility JavaVersion.VERSION_1_8
        targetCompatibility JavaVersion.VERSION_1_8
    }

    defaultConfig {
        applicationId "com.example.quran_whisper_app"
        minSdkVersion 21
        targetSdkVersion 34
        versionCode flutterVersionCode.toInteger()
        versionName flutterVersionName
        
        // Native library configuration
        ndk {
            abiFilters 'arm64-v8a', 'armeabi-v7a'
        }
    }

    buildTypes {
        release {
            signingConfig signingConfigs.debug
            minifyEnabled true
            shrinkResources true
        }
    }
}
```

### 🔄 Flutter Rust Bridge Setup

> **Note**: whisper-rust-binding adalah **standalone project** yang terpisah dari Flutter project. Flutter project hanya menggunakan hasil `.so` files melalui FRB.

#### 1. Struktur Project

```
workspace/
├── whisper-rust-binding/          # Standalone Rust project
│   ├── src/lib.rs                 # Core whisper functions
│   ├── Cargo.toml
│   ├── build_android.sh          # Build script untuk .so
│   └── target/
│       └── aarch64-linux-android/release/
│           └── libwhisper_rust_binding.so  # Output .so file
│
└── flutter_quran_app/            # Terpisah Flutter project
    ├── lib/
    ├── android/app/src/main/jniLibs/  # Copy .so files ke sini
    ├── flutter_rust_bridge.yaml
    └── pubspec.yaml
```

#### 2. Flutter Project - flutter_rust_bridge.yaml

```yaml
# Di flutter_quran_app/flutter_rust_bridge.yaml
rust_input: 
  - "../whisper-rust-binding/src/lib.rs"  # Reference ke standalone project
dart_output: "lib/generated"
c_output: "ios/Runner"
rust_crate_dir: "../whisper-rust-binding/"  # Path ke standalone project
extra_headers: |
  // Headers for mobile optimization
  #ifdef __ANDROID__
  #include <android/log.h>
  #endif
```

#### 3. Build & Integration Process

```bash
# Step 1: Build whisper-rust-binding standalone
cd whisper-rust-binding/
cargo build --target aarch64-linux-android --release

# Step 2: Copy .so files ke Flutter project
cp target/aarch64-linux-android/release/libwhisper_rust_binding.so \
   ../flutter_quran_app/android/app/src/main/jniLibs/arm64-v8a/

# Step 3: Generate FRB bindings di Flutter project
cd ../flutter_quran_app/
flutter_rust_bridge_codegen generate

# Step 4: Build Flutter app
flutter build apk
```

### 📁 Project Structure

```
lib/
├── main.dart
├── generated/          # FRB generated files
│   ├── bridge_generated.dart
│   └── bridge_definitions.dart
├── models/            # Data models
│   ├── whisper_model.dart
│   ├── transcription_result.dart
│   └── audio_data.dart
├── providers/         # Riverpod providers
│   ├── whisper_provider.dart
│   ├── audio_provider.dart
│   └── quran_provider.dart
├── services/          # Service layer
│   ├── whisper_service.dart
│   ├── audio_service.dart
│   └── permission_service.dart
├── ui/               # UI components
│   ├── screens/
│   ├── widgets/
│   └── components/
└── utils/            # Utilities
    ├── constants.dart
    └── helpers.dart
```

### 🔧 Environment Setup

#### 1. lib/utils/constants.dart

```dart
class WhisperConstants {
  // Model paths
  static const String defaultModelPath = 'assets/models/ggml-tiny.bin';
  static const String arabicModelPath = 'assets/models/ggml-base-ar.bin';
  
  // Audio configuration
  static const int sampleRate = 16000;
  static const int channels = 1;
  static const int bitRate = 16;
  
  // Transcription settings
  static const double windowSizeSec = 10.0;
  static const double stepSizeSec = 5.0;
  static const String defaultLanguage = 'ar';
  
  // File paths
  static const String modelsDirectory = 'models';
  static const String audioDirectory = 'audio';
}
```

#### 2. Assets Configuration - pubspec.yaml

```yaml
flutter:
  assets:
    - assets/models/
    - assets/images/
  
  uses-material-design: true
```

### ✅ Verification

Setelah setup selesai, verifikasi dengan:

```bash
# 1. Check dependencies
flutter pub get

# 2. Check platform setup
flutter doctor -v

# 3. Generate code
dart run build_runner build

# 4. Test build
flutter build apk --debug
```

### 🔄 Next Steps

1. ✅ Setup selesai → Lanjut ke `02-frb-integration.md`
2. Configure Flutter Rust Bridge integration
3. Implement data models
4. Setup Riverpod providers

### 🐛 Common Issues

**Issue**: Native library not found
```bash
# Solution: Ensure .so files are copied correctly from standalone project
cd whisper-rust-binding/
cargo build --target aarch64-linux-android --release

# Copy ke Flutter project
cp target/aarch64-linux-android/release/libwhisper_rust_binding.so \
   ../flutter_quran_app/android/app/src/main/jniLibs/arm64-v8a/

# IMPORTANT: Copy libc++_shared.so juga
cp $ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_shared.so \
   ../flutter_quran_app/android/app/src/main/jniLibs/arm64-v8a/
```

**Issue**: UnsatisfiedLinkError atau library load failed
```bash
# Solution: Pastikan libc++_shared.so ada
ls -la android/app/src/main/jniLibs/arm64-v8a/
# Harus ada:
# - libwhisper_rust_binding.so
# - libc++_shared.so

# Check dependencies dengan objdump
objdump -p android/app/src/main/jniLibs/arm64-v8a/libwhisper_rust_binding.so | grep NEEDED
```

**Issue**: Permission denied for audio recording
```dart
// Solution: Check permissions in runtime (di Flutter project)
await Permission.microphone.request();
```

**Issue**: FRB generation fails
```bash
# Solution: Clean and regenerate (di Flutter project)
cd flutter_quran_app/
flutter clean
flutter_rust_bridge_codegen generate --force
```

**Issue**: Rust crate path not found
```yaml
# Solution: Fix path di flutter_rust_bridge.yaml
rust_crate_dir: "../whisper-rust-binding/"  # Correct relative path
```

**Issue**: App crashes on startup with "library not found"
```bash
# Solution: Verify ALL required libraries
# 1. Main library
file android/app/src/main/jniLibs/arm64-v8a/libwhisper_rust_binding.so
# 2. C++ runtime
file android/app/src/main/jniLibs/arm64-v8a/libc++_shared.so
# 3. Check Android logs
adb logcat | grep -i "dlopen\|library\|whisper"
```
