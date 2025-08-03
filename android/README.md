# Android Build Instructions

## Prerequisites

Before building the library for Android, make sure you have the following installed:

1. Android NDK (recommended version: r25 or newer)
2. Android SDK
3. Rust with Android targets installed

## Setting Up Rust for Android

Install Android targets for Rust:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

## Environment Variables

Set up the following environment variables:

```bash
export ANDROID_NDK_HOME=/path/to/your/android/ndk
export ANDROID_SDK_HOME=/path/to/your/android/sdk # Optional
```

## Building for Android

To build the library for all Android architectures:

```bash
# For arm64-v8a (64-bit ARM)
cargo build --target aarch64-linux-android --release

# For armeabi-v7a (32-bit ARM)
cargo build --target armv7-linux-androideabi --release

# For x86 (32-bit Intel)
cargo build --target i686-linux-android --release

# For x86_64 (64-bit Intel)
cargo build --target x86_64-linux-android --release
```

## Output Location

The compiled libraries will be located at:

```
target/aarch64-linux-android/release/libwhisper_rust.so  # for arm64-v8a
target/armv7-linux-androideabi/release/libwhisper_rust.so  # for armeabi-v7a
target/i686-linux-android/release/libwhisper_rust.so  # for x86
target/x86_64-linux-android/release/libwhisper_rust.so  # for x86_64
```

## Integration with Android App

Copy the generated `.so` files to the appropriate directories in your Android project:

```
app/src/main/jniLibs/arm64-v8a/libwhisper_rust.so
app/src/main/jniLibs/armeabi-v7a/libwhisper_rust.so
app/src/main/jniLibs/x86/libwhisper_rust.so
app/src/main/jniLibs/x86_64/libwhisper_rust.so
```

## JNI Bindings (Optional)

If you want to use JNI bindings directly, build with the `android-jni` feature:

```bash
cargo build --target aarch64-linux-android --release --features android-jni
```

## Troubleshooting

### CMake Not Found

If you encounter issues with CMake not being found in the Android NDK, verify the path in `CMAKE_TOOLCHAIN_FILE`. Different NDK versions may have different paths. You can manually set it in `build.rs`.

### Linking Errors

If you face linking errors related to C++ standard library, make sure your Android app is configured to use the same C++ runtime (`c++_shared` in this case). In your app's `build.gradle`, add:

```gradle
android {
    // ...
    defaultConfig {
        // ...
        externalNativeBuild {
            cmake {
                arguments "-DANDROID_STL=c++_shared"
            }
        }
    }
}
```
