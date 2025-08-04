#!/bin/bash
# copy_native_libs.sh
# Helper script untuk copy .so files ke Flutter project dengan libc++_shared.so

set -e

# Colors untuk output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
WHISPER_PROJECT_DIR="$(pwd)"
FLUTTER_PROJECT_DIR="../flutter_quran_transcriber"
JNI_LIBS_DIR="$FLUTTER_PROJECT_DIR/android/app/src/main/jniLibs"

echo -e "${BLUE}🔧 Native Libraries Copy Script${NC}"
echo "================================================"

# Check if we're in whisper-rust-binding directory
if [ ! -f "Cargo.toml" ] || [ ! -d "src" ]; then
    echo -e "${RED}❌ Error: Must run from whisper-rust-binding root directory${NC}"
    exit 1
fi

# Check if Flutter project exists
if [ ! -d "$FLUTTER_PROJECT_DIR" ]; then
    echo -e "${RED}❌ Error: Flutter project not found at $FLUTTER_PROJECT_DIR${NC}"
    echo -e "${YELLOW}💡 Update FLUTTER_PROJECT_DIR variable in this script${NC}"
    exit 1
fi

# Check if Android NDK is available
if [ -z "$ANDROID_NDK_ROOT" ]; then
    echo -e "${RED}❌ Error: ANDROID_NDK_ROOT environment variable not set${NC}"
    echo -e "${YELLOW}💡 Export ANDROID_NDK_ROOT=/path/to/android-ndk${NC}"
    exit 1
fi

# Create JNI directories
echo -e "${BLUE}📁 Creating JNI directories...${NC}"
mkdir -p "$JNI_LIBS_DIR/arm64-v8a"
mkdir -p "$JNI_LIBS_DIR/armeabi-v7a"

# Function to find libc++_shared.so
find_libcpp() {
    local arch=$1
    local ndk_arch
    
    if [ "$arch" = "arm64-v8a" ]; then
        ndk_arch="aarch64-linux-android"
    else
        ndk_arch="arm-linux-androideabi"
    fi
    
    # Try multiple NDK paths
    local paths=(
        "$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/$ndk_arch/libc++_shared.so"
        "$ANDROID_NDK_ROOT/sources/cxx-stl/llvm-libc++/libs/$arch/libc++_shared.so"
        "$ANDROID_NDK_ROOT/sysroot/usr/lib/$ndk_arch/libc++_shared.so"
    )
    
    for path in "${paths[@]}"; do
        if [ -f "$path" ]; then
            echo "$path"
            return 0
        fi
    done
    
    echo ""
    return 1
}

# Copy arm64-v8a libraries
echo -e "${BLUE}📦 Copying arm64-v8a libraries...${NC}"

# Copy main whisper library
WHISPER_LIB_ARM64="target/aarch64-linux-android/release/libwhisper_rust_binding.so"
if [ -f "$WHISPER_LIB_ARM64" ]; then
    cp "$WHISPER_LIB_ARM64" "$JNI_LIBS_DIR/arm64-v8a/"
    echo -e "${GREEN}✅ Copied libwhisper_rust_binding.so (arm64-v8a)${NC}"
else
    echo -e "${RED}❌ Error: $WHISPER_LIB_ARM64 not found${NC}"
    echo -e "${YELLOW}💡 Run: cargo build --target aarch64-linux-android --release${NC}"
    exit 1
fi

# Copy libc++_shared.so for arm64-v8a
LIBCPP_ARM64=$(find_libcpp "arm64-v8a")
if [ -n "$LIBCPP_ARM64" ]; then
    cp "$LIBCPP_ARM64" "$JNI_LIBS_DIR/arm64-v8a/"
    echo -e "${GREEN}✅ Copied libc++_shared.so (arm64-v8a)${NC}"
else
    echo -e "${RED}❌ Error: libc++_shared.so not found for arm64-v8a${NC}"
    echo -e "${YELLOW}💡 Check ANDROID_NDK_ROOT path${NC}"
    exit 1
fi

# Copy armeabi-v7a libraries
echo -e "${BLUE}📦 Copying armeabi-v7a libraries...${NC}"

# Copy main whisper library
WHISPER_LIB_ARM32="target/armv7-linux-androideabi/release/libwhisper_rust_binding.so"
if [ -f "$WHISPER_LIB_ARM32" ]; then
    cp "$WHISPER_LIB_ARM32" "$JNI_LIBS_DIR/armeabi-v7a/"
    echo -e "${GREEN}✅ Copied libwhisper_rust_binding.so (armeabi-v7a)${NC}"
else
    echo -e "${YELLOW}⚠️  Warning: $WHISPER_LIB_ARM32 not found${NC}"
    echo -e "${YELLOW}💡 Run: cargo build --target armv7-linux-androideabi --release${NC}"
fi

# Copy libc++_shared.so for armeabi-v7a
LIBCPP_ARM32=$(find_libcpp "armeabi-v7a")
if [ -n "$LIBCPP_ARM32" ]; then
    cp "$LIBCPP_ARM32" "$JNI_LIBS_DIR/armeabi-v7a/"
    echo -e "${GREEN}✅ Copied libc++_shared.so (armeabi-v7a)${NC}"
else
    echo -e "${YELLOW}⚠️  Warning: libc++_shared.so not found for armeabi-v7a${NC}"
fi

# Verify copied files
echo -e "${BLUE}🔍 Verifying copied files...${NC}"
echo ""

for arch in "arm64-v8a" "armeabi-v7a"; do
    echo -e "${BLUE}$arch:${NC}"
    
    if [ -f "$JNI_LIBS_DIR/$arch/libwhisper_rust_binding.so" ]; then
        size=$(stat -c%s "$JNI_LIBS_DIR/$arch/libwhisper_rust_binding.so")
        echo -e "  ${GREEN}✅ libwhisper_rust_binding.so${NC} ($(numfmt --to=iec $size))"
    else
        echo -e "  ${RED}❌ libwhisper_rust_binding.so${NC}"
    fi
    
    if [ -f "$JNI_LIBS_DIR/$arch/libc++_shared.so" ]; then
        size=$(stat -c%s "$JNI_LIBS_DIR/$arch/libc++_shared.so")
        echo -e "  ${GREEN}✅ libc++_shared.so${NC} ($(numfmt --to=iec $size))"
    else
        echo -e "  ${RED}❌ libc++_shared.so${NC}"
    fi
    
    echo ""
done

# Show final directory structure
echo -e "${BLUE}📂 Final JNI directory structure:${NC}"
tree "$JNI_LIBS_DIR" 2>/dev/null || find "$JNI_LIBS_DIR" -type f | sort

echo ""
echo -e "${GREEN}🎉 Native libraries copy completed!${NC}"
echo -e "${YELLOW}💡 Next steps:${NC}"
echo -e "   1. cd $FLUTTER_PROJECT_DIR"
echo -e "   2. flutter_rust_bridge_codegen generate"
echo -e "   3. flutter build apk"

# Optional: Check dependencies
echo ""
echo -e "${BLUE}🔧 Checking library dependencies...${NC}"
if command -v objdump >/dev/null 2>&1; then
    for arch in "arm64-v8a" "armeabi-v7a"; do
        LIB_PATH="$JNI_LIBS_DIR/$arch/libwhisper_rust_binding.so"
        if [ -f "$LIB_PATH" ]; then
            echo -e "${BLUE}Dependencies for $arch:${NC}"
            objdump -p "$LIB_PATH" | grep NEEDED | sed 's/^/  /'
            echo ""
        fi
    done
else
    echo -e "${YELLOW}⚠️  objdump not available, skipping dependency check${NC}"
fi
