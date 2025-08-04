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
│   ├── libwhisper_rust_binding.so
│   └── libquran_assistant_engine.so
└── armeabi-v7a/
    ├── libwhisper_rust_binding.so
    └── libquran_assistant_engine.so
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

#### 1. Buat flutter_rust_bridge.yaml

```yaml
# flutter_rust_bridge.yaml
rust_input: 
  - "../../whisper-rust-binding/src/lib.rs"
  - "../../quran_assistant_engine/src/lib.rs"
dart_output: "lib/generated"
c_output: "ios/Runner"
rust_crate_dir: "../../"
extra_headers: |
  // Extra headers for mobile optimization
  #ifdef __ANDROID__
  #include <android/log.h>
  #endif
```

#### 2. Generate Bindings

```bash
# Generate Flutter Rust Bridge bindings
flutter_rust_bridge_codegen generate

# Build Rust libraries for Android
cd ../../whisper-rust-binding
./build_mobile_android.sh

cd ../../quran_assistant_engine  
flutter_rust_bridge_codegen build-android
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
# Solution: Copy libraries correctly
cp ../whisper-rust-binding/target/aarch64-linux-android/release/libwhisper_rust_binding.so \
   android/app/src/main/jniLibs/arm64-v8a/
```

**Issue**: Permission denied for audio recording
```dart
// Solution: Check permissions in runtime
await Permission.microphone.request();
```

**Issue**: FRB generation fails
```bash
# Solution: Clean and regenerate
flutter clean
flutter_rust_bridge_codegen generate --force
```
