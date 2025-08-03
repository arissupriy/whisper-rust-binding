# Android Integration Guide

Complete guide for integrating Whisper Rust Binding into Android applications.

## 📱 Overview

This guide covers:
- 🏗️ **Project Setup** - Android Studio configuration
- 🔧 **NDK Integration** - Native library integration
- ☕ **Java/Kotlin API** - High-level Android API
- 🎵 **Audio Processing** - Android audio handling
- 📦 **Packaging** - APK integration
- 🚀 **Performance** - Android-specific optimizations

## 🏗️ Project Setup

### Prerequisites

- **Android Studio 4.0+**
- **Android SDK API 21+** (Android 5.0+)
- **Android NDK 21+**
- **CMake 3.10+**
- **Built whisper-rust-binding** (see [Build Guide](./build-guide.md))

### Create Android Project

1. **Open Android Studio**
2. **Create New Project**
   - Choose "Native C++" template
   - Set minimum SDK to API 21
   - Select C++14 or later

3. **Project Structure**
   ```
   YourApp/
   ├── app/
   │   ├── src/main/
   │   │   ├── java/com/yourpackage/
   │   │   ├── cpp/
   │   │   ├── assets/
   │   │   └── res/
   │   └── libs/
   │       ├── arm64-v8a/
   │       ├── armeabi-v7a/
   │       ├── x86/
   │       └── x86_64/
   ```

### Configure Gradle

#### Module-level build.gradle

```gradle
android {
    compileSdkVersion 31
    buildToolsVersion "31.0.0"

    defaultConfig {
        applicationId "com.yourpackage.whisperapp"
        minSdkVersion 21
        targetSdkVersion 31
        versionCode 1
        versionName "1.0"

        ndk {
            abiFilters 'arm64-v8a', 'armeabi-v7a', 'x86', 'x86_64'
        }

        externalNativeBuild {
            cmake {
                cppFlags "-std=c++14"
                abiFilters 'arm64-v8a', 'armeabi-v7a', 'x86', 'x86_64'
            }
        }
    }

    externalNativeBuild {
        cmake {
            path "src/main/cpp/CMakeLists.txt"
            version "3.10.2"
        }
    }

    sourceSets {
        main {
            jniLibs.srcDirs = ['libs']
        }
    }

    packagingOptions {
        pickFirst '**/libc++_shared.so'
        pickFirst '**/libjsc.so'
    }
}

dependencies {
    implementation 'androidx.appcompat:appcompat:1.4.0'
    implementation 'androidx.constraintlayout:constraintlayout:2.1.0'
    implementation 'com.google.android.material:material:1.4.0'
}
```

### Copy Native Libraries

```bash
# Copy whisper-rust-binding libraries to Android project
cp whisper-rust-binding/target/aarch64-linux-android/release/libwhisper_rust_binding.so \
   YourApp/app/libs/arm64-v8a/

cp whisper-rust-binding/target/armv7-linux-androideabi/release/libwhisper_rust_binding.so \
   YourApp/app/libs/armeabi-v7a/

cp whisper-rust-binding/target/i686-linux-android/release/libwhisper_rust_binding.so \
   YourApp/app/libs/x86/

cp whisper-rust-binding/target/x86_64-linux-android/release/libwhisper_rust_binding.so \
   YourApp/app/libs/x86_64/
```

## 🔧 NDK Integration

### CMakeLists.txt

Create `app/src/main/cpp/CMakeLists.txt`:

```cmake
cmake_minimum_required(VERSION 3.10.2)

project("whisperapp")

# Import whisper-rust-binding library
add_library(whisper-rust-binding SHARED IMPORTED)
set_target_properties(whisper-rust-binding PROPERTIES
    IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/../libs/${ANDROID_ABI}/libwhisper_rust_binding.so)

# Create JNI wrapper library
add_library(whisperapp SHARED
    whisper_jni.cpp
    audio_utils.cpp)

# Include directories
target_include_directories(whisperapp PRIVATE
    ${CMAKE_SOURCE_DIR}
    ${CMAKE_SOURCE_DIR}/whisper-rust-binding/include)

# Link libraries
target_link_libraries(whisperapp
    whisper-rust-binding
    android
    log)
```

### JNI Wrapper

Create `app/src/main/cpp/whisper_jni.cpp`:

```cpp
#include <jni.h>
#include <string>
#include <vector>
#include <android/log.h>

// Include the Rust library header
extern "C" {
    int32_t whisper_rust_init(const char* model_path);
    bool whisper_rust_free(int32_t instance_id);
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
}

#define LOG_TAG "WhisperJNI"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

extern "C" {

JNIEXPORT jint JNICALL
Java_com_yourpackage_WhisperService_initWhisper(JNIEnv *env, jobject thiz, jstring modelPath) {
    const char *model_path_str = env->GetStringUTFChars(modelPath, nullptr);
    if (!model_path_str) {
        LOGE("Failed to get model path string");
        return -1;
    }

    LOGI("Initializing Whisper with model: %s", model_path_str);
    int32_t instance_id = whisper_rust_init(model_path_str);
    
    env->ReleaseStringUTFChars(modelPath, model_path_str);
    
    if (instance_id >= 0) {
        LOGI("Whisper initialized successfully with ID: %d", instance_id);
    } else {
        LOGE("Failed to initialize Whisper");
    }
    
    return instance_id;
}

JNIEXPORT jstring JNICALL
Java_com_yourpackage_WhisperService_processAudio(JNIEnv *env, jobject thiz, 
                                                 jint instanceId, jfloatArray audioData, jstring language) {
    // Get audio data
    jfloat *audio_array = env->GetFloatArrayElements(audioData, nullptr);
    jsize audio_length = env->GetArrayLength(audioData);
    
    if (!audio_array) {
        LOGE("Failed to get audio array");
        return env->NewStringUTF("");
    }

    // Get language string
    const char *language_str = nullptr;
    if (language != nullptr) {
        language_str = env->GetStringUTFChars(language, nullptr);
    }

    // Prepare result buffer
    const int result_buffer_size = 4096;
    char result_buffer[result_buffer_size];
    
    LOGI("Processing audio: %d samples, language: %s", audio_length, language_str ? language_str : "auto");
    
    // Call Rust function
    bool success = whisper_rust_process_audio(
        instanceId,
        audio_array,
        audio_length,
        language_str,
        result_buffer,
        result_buffer_size
    );
    
    // Cleanup
    env->ReleaseFloatArrayElements(audioData, audio_array, JNI_ABORT);
    if (language_str) {
        env->ReleaseStringUTFChars(language, language_str);
    }
    
    if (success) {
        LOGI("Audio processing successful");
        return env->NewStringUTF(result_buffer);
    } else {
        LOGE("Audio processing failed");
        return env->NewStringUTF("");
    }
}

JNIEXPORT jboolean JNICALL
Java_com_yourpackage_WhisperService_freeWhisper(JNIEnv *env, jobject thiz, jint instanceId) {
    LOGI("Freeing Whisper instance: %d", instanceId);
    bool success = whisper_rust_free(instanceId);
    
    if (success) {
        LOGI("Whisper instance freed successfully");
    } else {
        LOGE("Failed to free Whisper instance");
    }
    
    return success;
}

JNIEXPORT jstring JNICALL
Java_com_yourpackage_WhisperService_getModelInfo(JNIEnv *env, jobject thiz, jint instanceId) {
    const int info_buffer_size = 256;
    char info_buffer[info_buffer_size];
    
    bool success = whisper_rust_get_model_info(instanceId, info_buffer, info_buffer_size);
    
    if (success) {
        return env->NewStringUTF(info_buffer);
    } else {
        return env->NewStringUTF("Unknown");
    }
}

} // extern "C"
```

### Audio Utilities

Create `app/src/main/cpp/audio_utils.cpp`:

```cpp
#include <jni.h>
#include <vector>
#include <algorithm>
#include <android/log.h>

#define LOG_TAG "AudioUtils"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)

extern "C" {

JNIEXPORT jfloatArray JNICALL
Java_com_yourpackage_AudioUtils_convertBytesToFloat(JNIEnv *env, jobject thiz, 
                                                    jbyteArray audioBytes, jint sampleRate) {
    jbyte *bytes = env->GetByteArrayElements(audioBytes, nullptr);
    jsize byte_length = env->GetArrayLength(audioBytes);
    
    // Convert 16-bit PCM to float
    int sample_count = byte_length / 2;
    std::vector<float> float_samples(sample_count);
    
    int16_t *pcm_data = reinterpret_cast<int16_t*>(bytes);
    
    for (int i = 0; i < sample_count; i++) {
        float_samples[i] = static_cast<float>(pcm_data[i]) / 32768.0f;
    }
    
    // Resample to 16kHz if necessary
    if (sampleRate != 16000) {
        float_samples = resampleTo16kHz(float_samples, sampleRate);
    }
    
    // Create Java float array
    jfloatArray result = env->NewFloatArray(float_samples.size());
    env->SetFloatArrayRegion(result, 0, float_samples.size(), float_samples.data());
    
    env->ReleaseByteArrayElements(audioBytes, bytes, JNI_ABORT);
    
    return result;
}

JNIEXPORT jfloatArray JNICALL
Java_com_yourpackage_AudioUtils_normalizeAudio(JNIEnv *env, jobject thiz, jfloatArray audioData) {
    jfloat *samples = env->GetFloatArrayElements(audioData, nullptr);
    jsize length = env->GetArrayLength(audioData);
    
    // Find maximum absolute value
    float max_val = 0.0f;
    for (int i = 0; i < length; i++) {
        max_val = std::max(max_val, std::abs(samples[i]));
    }
    
    // Normalize if needed
    if (max_val > 0.0f && max_val != 1.0f) {
        float scale = 1.0f / max_val;
        for (int i = 0; i < length; i++) {
            samples[i] *= scale;
        }
    }
    
    jfloatArray result = env->NewFloatArray(length);
    env->SetFloatArrayRegion(result, 0, length, samples);
    
    env->ReleaseFloatArrayElements(audioData, samples, JNI_ABORT);
    
    return result;
}

} // extern "C"

// Helper function for resampling
std::vector<float> resampleTo16kHz(const std::vector<float>& input, int inputSampleRate) {
    if (inputSampleRate == 16000) {
        return input;
    }
    
    double ratio = static_cast<double>(inputSampleRate) / 16000.0;
    int output_length = static_cast<int>(input.size() / ratio);
    
    std::vector<float> output(output_length);
    
    for (int i = 0; i < output_length; i++) {
        double src_index = i * ratio;
        int src_i = static_cast<int>(src_index);
        
        if (src_i + 1 < input.size()) {
            double frac = src_index - src_i;
            output[i] = input[src_i] * (1.0 - frac) + input[src_i + 1] * frac;
        } else {
            output[i] = input[src_i];
        }
    }
    
    return output;
}
```

## ☕ Java/Kotlin API

### WhisperService.java

Create `app/src/main/java/com/yourpackage/WhisperService.java`:

```java
package com.yourpackage;

import android.content.Context;
import android.content.res.AssetManager;
import android.util.Log;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;

public class WhisperService {
    private static final String TAG = "WhisperService";
    
    // Load native library
    static {
        System.loadLibrary("whisperapp");
    }
    
    private int instanceId = -1;
    private boolean isInitialized = false;
    
    // Native method declarations
    private native int initWhisper(String modelPath);
    private native String processAudio(int instanceId, float[] audioData, String language);
    private native boolean freeWhisper(int instanceId);
    private native String getModelInfo(int instanceId);
    
    /**
     * Initialize Whisper with a model file
     * @param context Application context
     * @param modelFileName Model file name in assets (e.g., "ggml-tiny.bin")
     * @return true if successful, false otherwise
     */
    public boolean initialize(Context context, String modelFileName) {
        try {
            // Copy model from assets to internal storage
            String modelPath = copyAssetToInternalStorage(context, modelFileName);
            
            // Initialize native library
            instanceId = initWhisper(modelPath);
            
            if (instanceId >= 0) {
                isInitialized = true;
                Log.i(TAG, "Whisper initialized successfully with ID: " + instanceId);
                return true;
            } else {
                Log.e(TAG, "Failed to initialize Whisper");
                return false;
            }
            
        } catch (IOException e) {
            Log.e(TAG, "Error copying model file", e);
            return false;
        }
    }
    
    /**
     * Transcribe audio data
     * @param audioData Audio samples (16kHz, mono, float)
     * @param language Language code ("ar", "en", null for auto-detect)
     * @return Transcription result or empty string on error
     */
    public String transcribe(float[] audioData, String language) {
        if (!isInitialized || instanceId < 0) {
            Log.e(TAG, "Whisper not initialized");
            return "";
        }
        
        if (audioData == null || audioData.length == 0) {
            Log.e(TAG, "Invalid audio data");
            return "";
        }
        
        Log.i(TAG, "Transcribing audio: " + audioData.length + " samples");
        String result = processAudio(instanceId, audioData, language);
        
        if (result == null) {
            result = "";
        }
        
        Log.i(TAG, "Transcription result: " + result);
        return result;
    }
    
    /**
     * Get model information
     * @return Model version string
     */
    public String getModelVersion() {
        if (!isInitialized || instanceId < 0) {
            return "Not initialized";
        }
        
        return getModelInfo(instanceId);
    }
    
    /**
     * Check if service is initialized
     * @return true if ready for transcription
     */
    public boolean isReady() {
        return isInitialized && instanceId >= 0;
    }
    
    /**
     * Clean up resources
     */
    public void cleanup() {
        if (isInitialized && instanceId >= 0) {
            boolean success = freeWhisper(instanceId);
            Log.i(TAG, "Cleanup " + (success ? "successful" : "failed"));
            
            instanceId = -1;
            isInitialized = false;
        }
    }
    
    /**
     * Copy asset file to internal storage
     */
    private String copyAssetToInternalStorage(Context context, String assetFileName) throws IOException {
        AssetManager assetManager = context.getAssets();
        InputStream inputStream = assetManager.open(assetFileName);
        
        File outputFile = new File(context.getFilesDir(), assetFileName);
        FileOutputStream outputStream = new FileOutputStream(outputFile);
        
        byte[] buffer = new byte[8192];
        int length;
        while ((length = inputStream.read(buffer)) > 0) {
            outputStream.write(buffer, 0, length);
        }
        
        inputStream.close();
        outputStream.close();
        
        Log.i(TAG, "Model copied to: " + outputFile.getAbsolutePath());
        return outputFile.getAbsolutePath();
    }
}
```

### AudioUtils.java

Create `app/src/main/java/com/yourpackage/AudioUtils.java`:

```java
package com.yourpackage;

import android.media.AudioFormat;
import android.media.AudioRecord;
import android.media.MediaRecorder;
import android.util.Log;

public class AudioUtils {
    private static final String TAG = "AudioUtils";
    
    // Load native library
    static {
        System.loadLibrary("whisperapp");
    }
    
    // Native methods
    private native float[] convertBytesToFloat(byte[] audioBytes, int sampleRate);
    private native float[] normalizeAudio(float[] audioData);
    
    // Audio recording parameters
    private static final int SAMPLE_RATE = 16000;
    private static final int CHANNEL_CONFIG = AudioFormat.CHANNEL_IN_MONO;
    private static final int AUDIO_FORMAT = AudioFormat.ENCODING_PCM_16BIT;
    
    /**
     * Record audio for specified duration
     * @param durationMs Recording duration in milliseconds
     * @return Audio data as float array or null on error
     */
    public float[] recordAudio(int durationMs) {
        int bufferSize = AudioRecord.getMinBufferSize(SAMPLE_RATE, CHANNEL_CONFIG, AUDIO_FORMAT);
        AudioRecord audioRecord = new AudioRecord(
            MediaRecorder.AudioSource.MIC,
            SAMPLE_RATE,
            CHANNEL_CONFIG,
            AUDIO_FORMAT,
            bufferSize
        );
        
        if (audioRecord.getState() != AudioRecord.STATE_INITIALIZED) {
            Log.e(TAG, "AudioRecord not initialized");
            return null;
        }
        
        audioRecord.startRecording();
        
        int totalSamples = (SAMPLE_RATE * durationMs) / 1000;
        byte[] audioData = new byte[totalSamples * 2]; // 16-bit = 2 bytes per sample
        int bytesRead = 0;
        
        while (bytesRead < audioData.length) {
            int result = audioRecord.read(audioData, bytesRead, audioData.length - bytesRead);
            if (result > 0) {
                bytesRead += result;
            } else {
                Log.e(TAG, "Error reading audio data: " + result);
                break;
            }
        }
        
        audioRecord.stop();
        audioRecord.release();
        
        if (bytesRead < audioData.length) {
            Log.w(TAG, "Only recorded " + bytesRead + " bytes out of " + audioData.length);
        }
        
        // Convert to float and normalize
        float[] floatData = convertBytesToFloat(audioData, SAMPLE_RATE);
        return normalizeAudio(floatData);
    }
    
    /**
     * Convert WAV file bytes to float array
     * @param wavBytes WAV file bytes
     * @return Audio data as float array
     */
    public float[] processWavFile(byte[] wavBytes) {
        // Skip WAV header (44 bytes)
        if (wavBytes.length < 44) {
            Log.e(TAG, "Invalid WAV file: too short");
            return null;
        }
        
        byte[] audioBytes = new byte[wavBytes.length - 44];
        System.arraycopy(wavBytes, 44, audioBytes, 0, audioBytes.length);
        
        // Extract sample rate from WAV header
        int sampleRate = extractSampleRateFromWav(wavBytes);
        
        float[] floatData = convertBytesToFloat(audioBytes, sampleRate);
        return normalizeAudio(floatData);
    }
    
    /**
     * Extract sample rate from WAV header
     */
    private int extractSampleRateFromWav(byte[] wavBytes) {
        if (wavBytes.length < 28) {
            return 16000; // Default
        }
        
        // Sample rate is at bytes 24-27 (little endian)
        int sampleRate = (wavBytes[24] & 0xFF) |
                        ((wavBytes[25] & 0xFF) << 8) |
                        ((wavBytes[26] & 0xFF) << 16) |
                        ((wavBytes[27] & 0xFF) << 24);
        
        return sampleRate;
    }
}
```

### Kotlin Version (Optional)

Create `WhisperService.kt`:

```kotlin
package com.yourpackage

import android.content.Context
import android.util.Log

class WhisperService {
    companion object {
        private const val TAG = "WhisperService"
        
        init {
            System.loadLibrary("whisperapp")
        }
    }
    
    private var instanceId: Int = -1
    private var isInitialized: Boolean = false
    
    // Native method declarations
    private external fun initWhisper(modelPath: String): Int
    private external fun processAudio(instanceId: Int, audioData: FloatArray, language: String?): String
    private external fun freeWhisper(instanceId: Int): Boolean
    private external fun getModelInfo(instanceId: Int): String
    
    /**
     * Initialize Whisper with a model file
     */
    fun initialize(context: Context, modelFileName: String): Boolean {
        return try {
            val modelPath = copyAssetToInternalStorage(context, modelFileName)
            instanceId = initWhisper(modelPath)
            
            if (instanceId >= 0) {
                isInitialized = true
                Log.i(TAG, "Whisper initialized successfully with ID: $instanceId")
                true
            } else {
                Log.e(TAG, "Failed to initialize Whisper")
                false
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error initializing Whisper", e)
            false
        }
    }
    
    /**
     * Transcribe audio data
     */
    fun transcribe(audioData: FloatArray, language: String? = null): String {
        if (!isInitialized || instanceId < 0) {
            Log.e(TAG, "Whisper not initialized")
            return ""
        }
        
        if (audioData.isEmpty()) {
            Log.e(TAG, "Invalid audio data")
            return ""
        }
        
        Log.i(TAG, "Transcribing audio: ${audioData.size} samples")
        val result = processAudio(instanceId, audioData, language) ?: ""
        
        Log.i(TAG, "Transcription result: $result")
        return result
    }
    
    /**
     * Get model information
     */
    fun getModelVersion(): String {
        return if (isInitialized && instanceId >= 0) {
            getModelInfo(instanceId)
        } else {
            "Not initialized"
        }
    }
    
    /**
     * Check if service is ready
     */
    fun isReady(): Boolean = isInitialized && instanceId >= 0
    
    /**
     * Clean up resources
     */
    fun cleanup() {
        if (isInitialized && instanceId >= 0) {
            val success = freeWhisper(instanceId)
            Log.i(TAG, "Cleanup ${if (success) "successful" else "failed"}")
            
            instanceId = -1
            isInitialized = false
        }
    }
    
    /**
     * Copy asset file to internal storage
     */
    private fun copyAssetToInternalStorage(context: Context, assetFileName: String): String {
        val inputStream = context.assets.open(assetFileName)
        val outputFile = context.filesDir.resolve(assetFileName)
        
        inputStream.use { input ->
            outputFile.outputStream().use { output ->
                input.copyTo(output)
            }
        }
        
        Log.i(TAG, "Model copied to: ${outputFile.absolutePath}")
        return outputFile.absolutePath
    }
}
```

## 🎵 Audio Processing

### Recording Audio

```java
public class AudioRecorder {
    private static final int SAMPLE_RATE = 16000;
    private static final int CHANNEL = AudioFormat.CHANNEL_IN_MONO;
    private static final int FORMAT = AudioFormat.ENCODING_PCM_16BIT;
    
    private AudioRecord audioRecord;
    private boolean isRecording = false;
    
    public void startRecording(AudioCallback callback) {
        int bufferSize = AudioRecord.getMinBufferSize(SAMPLE_RATE, CHANNEL, FORMAT);
        audioRecord = new AudioRecord(MediaRecorder.AudioSource.MIC, 
                                     SAMPLE_RATE, CHANNEL, FORMAT, bufferSize);
        
        audioRecord.startRecording();
        isRecording = true;
        
        new Thread(() -> {
            byte[] buffer = new byte[bufferSize];
            while (isRecording) {
                int bytesRead = audioRecord.read(buffer, 0, buffer.length);
                if (bytesRead > 0) {
                    callback.onAudioData(buffer, bytesRead);
                }
            }
        }).start();
    }
    
    public void stopRecording() {
        isRecording = false;
        if (audioRecord != null) {
            audioRecord.stop();
            audioRecord.release();
        }
    }
    
    public interface AudioCallback {
        void onAudioData(byte[] data, int length);
    }
}
```

### MainActivity Example

```java
public class MainActivity extends AppCompatActivity {
    private WhisperService whisperService;
    private AudioUtils audioUtils;
    private Button recordButton;
    private TextView resultText;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        
        whisperService = new WhisperService();
        audioUtils = new AudioUtils();
        
        recordButton = findViewById(R.id.recordButton);
        resultText = findViewById(R.id.resultText);
        
        // Initialize Whisper in background thread
        new Thread(() -> {
            boolean success = whisperService.initialize(this, "ggml-tiny.bin");
            runOnUiThread(() -> {
                if (success) {
                    recordButton.setEnabled(true);
                    recordButton.setText("Record");
                } else {
                    recordButton.setText("Initialization Failed");
                }
            });
        }).start();
        
        recordButton.setOnClickListener(v -> recordAndTranscribe());
    }
    
    private void recordAndTranscribe() {
        recordButton.setEnabled(false);
        recordButton.setText("Recording...");
        
        new Thread(() -> {
            // Record 5 seconds of audio
            float[] audioData = audioUtils.recordAudio(5000);
            
            if (audioData != null) {
                // Transcribe audio
                String result = whisperService.transcribe(audioData, "ar");
                
                runOnUiThread(() -> {
                    resultText.setText(result);
                    recordButton.setEnabled(true);
                    recordButton.setText("Record");
                });
            } else {
                runOnUiThread(() -> {
                    resultText.setText("Recording failed");
                    recordButton.setEnabled(true);
                    recordButton.setText("Record");
                });
            }
        }).start();
    }
    
    @Override
    protected void onDestroy() {
        super.onDestroy();
        whisperService.cleanup();
    }
}
```

## 📦 Packaging

### Assets Organization

```
app/src/main/assets/
├── ggml-tiny.bin          # 39 MB - For development
├── ggml-base.bin          # 142 MB - For production
└── test_audio.wav         # Sample audio for testing
```

### Permissions

Add to `AndroidManifest.xml`:

```xml
<uses-permission android:name="android.permission.RECORD_AUDIO" />
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" />
<uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" />
```

### ProGuard Configuration

Add to `proguard-rules.pro`:

```proguard
# Keep native method names
-keepclasseswithmembernames class * {
    native <methods>;
}

# Keep WhisperService methods
-keep class com.yourpackage.WhisperService {
    public *;
}

# Keep AudioUtils methods
-keep class com.yourpackage.AudioUtils {
    public *;
}
```

## 🚀 Performance Optimization

### Model Selection for Android

| Model | APK Size Impact | RAM Usage | Battery Usage | Speed |
|-------|----------------|-----------|---------------|-------|
| Tiny | +39 MB | ~120 MB | Low | Fastest |
| Base | +142 MB | ~210 MB | Medium | Fast |
| Small | +466 MB | ~600 MB | High | Slower |

### Optimization Tips

1. **Use Tiny Model**: For mobile apps, tiny model provides good balance
2. **Background Processing**: Always run transcription in background threads
3. **Audio Chunking**: Process audio in chunks for better responsiveness
4. **Memory Management**: Clean up instances when not needed
5. **Battery Optimization**: Use appropriate CPU cores for processing

### Build Optimizations

```gradle
android {
    buildTypes {
        release {
            minifyEnabled true
            shrinkResources true
            proguardFiles getDefaultProguardFile('proguard-android-optimize.txt'), 'proguard-rules.pro'
            
            ndk {
                debugSymbolLevel 'NONE'
            }
        }
    }
}
```

## 🔍 Testing

### Unit Tests

```java
@RunWith(AndroidJUnit4.class)
public class WhisperServiceTest {
    @Test
    public void testInitialization() {
        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        WhisperService service = new WhisperService();
        
        boolean success = service.initialize(context, "ggml-tiny.bin");
        assertTrue(success);
        assertTrue(service.isReady());
        
        service.cleanup();
    }
    
    @Test
    public void testTranscription() {
        Context context = InstrumentationRegistry.getInstrumentation().getTargetContext();
        WhisperService service = new WhisperService();
        service.initialize(context, "ggml-tiny.bin");
        
        // Generate test audio (1 second of sine wave)
        float[] testAudio = generateTestAudio(16000);
        String result = service.transcribe(testAudio, "en");
        
        assertNotNull(result);
        // Note: Result might be empty for sine wave, but should not crash
        
        service.cleanup();
    }
    
    private float[] generateTestAudio(int samples) {
        float[] audio = new float[samples];
        for (int i = 0; i < samples; i++) {
            audio[i] = (float) Math.sin(2.0 * Math.PI * 440.0 * i / 16000.0);
        }
        return audio;
    }
}
```

## 🎯 Example App

A complete example app is available in the `android/` directory of the repository. To use it:

1. Copy the example to your Android Studio workspace
2. Update package names in Java files
3. Copy model files to assets
4. Build and run

The example demonstrates:
- Model initialization
- Real-time audio recording
- Audio file transcription
- Error handling
- UI integration

This integration guide should help you successfully integrate Whisper Rust Binding into your Android application!
