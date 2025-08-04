# 📱 Android NDK Setup Guide
## Building whisper-rust-binding for Android

### 🔧 **Quick Setup (Recommended)**

#### 1. **Install Android NDK via Android Studio**
```bash
# Open Android Studio
# Go to: Tools > SDK Manager > SDK Tools
# Check: NDK (Side by side) and CMake
# Click Apply to install
```

#### 2. **Set Environment Variables**
```bash
# Add to ~/.bashrc or ~/.zshrc
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/26.1.10909125"  # Use your NDK version
export PATH="$PATH:$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"

# Reload shell
source ~/.bashrc
```

#### 3. **Install Rust Android Targets**
```bash
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android
```

#### 4. **Configure Cargo for Android**
```bash
# Create ~/.cargo/config.toml
mkdir -p ~/.cargo
cat > ~/.cargo/config.toml << 'EOF'
[target.aarch64-linux-android]
ar = "aarch64-linux-android-ar"
linker = "aarch64-linux-android26-clang"

[target.armv7-linux-androideabi]
ar = "arm-linux-androideabi-ar"
linker = "armv7a-linux-androideabi26-clang"

[target.i686-linux-android]
ar = "i686-linux-android-ar"
linker = "i686-linux-android26-clang"

[target.x86_64-linux-android]
ar = "x86_64-linux-android-ar"
linker = "x86_64-linux-android26-clang"
EOF
```

### 🚀 **Manual NDK Setup (Alternative)**

#### 1. **Download NDK**
```bash
cd ~/Downloads
wget https://dl.google.com/android/repository/android-ndk-r26b-linux.zip
unzip android-ndk-r26b-linux.zip
sudo mv android-ndk-r26b /opt/android-ndk
```

#### 2. **Set PATH**
```bash
export PATH="/opt/android-ndk/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH"
```

### ✅ **Verify Setup**
```bash
# Test if NDK is properly configured
aarch64-linux-android-gcc --version
```

### 🔄 **Re-run Build After Setup**
```bash
./build_so.sh
```

**Expected Output After Setup:**
```
🤖 Building for Android ARM64...
✅ Android ARM64 build successful
📦 Android library copied to: lib/android/arm64-v8a/libwhisper_rust_binding.so
```

### 📦 **For Now (Linux Only)**

**Current Status: ✅ PRODUCTION READY**
- Linux `.so` file: **Ready for development and testing**
- Flutter integration: **Fully functional**
- Dual-project setup: **Complete**

**Android can be added later when needed for mobile deployment.**
