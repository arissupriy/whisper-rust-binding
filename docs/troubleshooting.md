# Troubleshooting Guide

Comprehensive troubleshooting guide for Whisper Rust Binding.

## 🔧 Overview

This guide covers:
- 🚨 **Common Issues** - Frequently encountered problems and solutions
- 🏗️ **Build Problems** - Compilation and linking issues
- 🎵 **Audio Issues** - Audio processing and format problems
- ⚡ **Performance Issues** - Speed and memory optimization
- 🐛 **Runtime Errors** - Crashes and unexpected behavior
- 🔍 **Debugging Tools** - Advanced debugging techniques

## 🚨 Common Issues

### Issue 1: Compilation Errors

#### Error: `#[no_mangle]` attribute is unsafe
```
error: `#[no_mangle]` requires unsafe function or block
```

**Solution:**
```rust
// ❌ Old syntax (Rust < 1.82)
#[no_mangle]
pub extern "C" fn function_name() {
    // function body
}

// ✅ New syntax (Rust >= 1.82)
#[unsafe(no_mangle)]
pub extern "C" fn function_name() {
    // function body
}
```

**Prevention:**
- Update Rust: `rustup update`
- Check Rust version: `rustc --version`

#### Error: Struct alignment issues
```
error: mismatched types in FFI bindings
```

**Solution:**
Ensure struct definitions match exactly between Rust and C:

```rust
// ✅ Correct struct definition
#[repr(C)]
#[derive(Debug, Clone)]
pub struct WhisperFullParams {
    pub strategy: whisper_sampling_strategy,
    pub n_threads: c_int,
    pub n_max_text_ctx: c_int,
    pub offset_ms: c_int,
    pub duration_ms: c_int,
    pub translate: bool,
    pub no_context: bool,
    pub no_timestamps: bool,
    pub single_segment: bool,
    pub print_special: bool,
    pub print_progress: bool,
    pub print_realtime: bool,
    pub print_timestamps: bool,
    pub token_timestamps: bool,
    pub thold_pt: c_float,
    pub thold_ptsum: c_float,
    pub max_len: c_int,
    pub split_on_word: bool,
    pub max_tokens: c_int,
    pub speed_up: bool,
    pub debug_mode: bool,
    pub audio_ctx: c_int,
    pub tdrz_enable: bool,
    pub suppress_regex: *const c_char,
    pub initial_prompt: *const c_char,
    pub prompt_tokens: *const whisper_token,
    pub prompt_n_tokens: c_int,
    pub language: *const c_char,
    pub detect_language: bool,
    pub suppress_blank: bool,
    pub suppress_non_speech_tokens: bool,
    pub temperature: c_float,
    pub max_initial_ts: c_float,
    pub length_penalty: c_float,
    pub temperature_inc: c_float,
    pub entropy_thold: c_float,
    pub logprob_thold: c_float,
    pub no_speech_thold: c_float,
    pub greedy: whisper_greedy_params,
    pub beam_search: whisper_beam_search_params,
    pub new_segment_callback: whisper_new_segment_callback,
    pub new_segment_callback_user_data: *mut c_void,
    pub progress_callback: whisper_progress_callback,
    pub progress_callback_user_data: *mut c_void,
    pub encoder_begin_callback: whisper_encoder_begin_callback,
    pub encoder_begin_callback_user_data: *mut c_void,
    pub abort_callback: whisper_abort_callback,
    pub abort_callback_user_data: *mut c_void,
    pub logits_filter_callback: whisper_logits_filter_callback,
    pub logits_filter_callback_user_data: *mut c_void,
    pub grammar_rules: *const whisper_grammar_element,
    pub n_grammar_rules: usize,
    pub i_start_rule: usize,
    pub grammar_penalty: c_float,
}
```

### Issue 2: Build Script Failures

#### Error: `whisper.cpp` not found
```
error: whisper.cpp directory not found
```

**Solution:**
```bash
# Clone whisper.cpp repository
./clone_whisper_cpp.sh

# Or manually:
git clone https://github.com/ggerganov/whisper.cpp.git
cd whisper.cpp
make
```

#### Error: CMake not found
```
error: cmake command not found
```

**Solution:**
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install cmake build-essential

# CentOS/RHEL
sudo yum install cmake gcc gcc-c++ make

# macOS
brew install cmake

# Arch Linux
sudo pacman -S cmake base-devel
```

#### Error: Missing development headers
```
error: cannot find -lwhisper
```

**Solution:**
```bash
# Ensure whisper.cpp is built
cd whisper.cpp
make clean
make

# Check if library exists
ls -la libwhisper.a libwhisper.so

# If missing, rebuild with debug info
make GGML_DEBUG=1
```

### Issue 3: Model Loading Problems

#### Error: Model file not found
```
Error: Failed to load model from ggml-tiny.bin
```

**Solution:**
```bash
# Download model files
./download_model.sh

# Or manually download specific models
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin

# Verify file integrity
sha256sum ggml-tiny.bin
# Should match: 61b4ce6c61a16cfdc085e3b9a8d3fb54b3db6b08
```

#### Error: Corrupted model file
```
Error: Invalid model format
```

**Solution:**
```bash
# Re-download the model
rm ggml-tiny.bin
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin

# Verify file size
ls -lh ggml-tiny.bin
# Should be approximately 39MB for tiny model

# Test with whisper.cpp directly
cd whisper.cpp
./main -m ../ggml-tiny.bin -f ../output.wav
```

## 🏗️ Build Problems

### Detailed Build Diagnostics

#### Build Environment Check

```bash
#!/bin/bash
# build_diagnostics.sh

echo "🔍 Build Environment Diagnostics"
echo "================================"

# Check Rust installation
echo "📦 Rust Version:"
rustc --version
cargo --version

# Check C++ compiler
echo -e "\n🔧 C++ Compiler:"
if command -v g++ &> /dev/null; then
    g++ --version | head -1
else
    echo "❌ g++ not found"
fi

if command -v clang++ &> /dev/null; then
    clang++ --version | head -1
else
    echo "❌ clang++ not found"
fi

# Check CMake
echo -e "\n🏗️  CMake:"
if command -v cmake &> /dev/null; then
    cmake --version | head -1
else
    echo "❌ CMake not found"
fi

# Check pkg-config
echo -e "\n📋 pkg-config:"
if command -v pkg-config &> /dev/null; then
    pkg-config --version
else
    echo "❌ pkg-config not found"
fi

# Check whisper.cpp
echo -e "\n🎙️  Whisper.cpp:"
if [ -d "whisper.cpp" ]; then
    echo "✅ whisper.cpp directory exists"
    if [ -f "whisper.cpp/libwhisper.a" ]; then
        echo "✅ libwhisper.a found"
        ls -lh whisper.cpp/libwhisper.a
    else
        echo "❌ libwhisper.a not found"
    fi
    
    if [ -f "whisper.cpp/include/whisper.h" ]; then
        echo "✅ whisper.h found"
    else
        echo "❌ whisper.h not found"
    fi
else
    echo "❌ whisper.cpp directory not found"
fi

# Check model files
echo -e "\n🤖 Model Files:"
for model in ggml-tiny.bin ggml-base.bin ggml-small.bin; do
    if [ -f "$model" ]; then
        echo "✅ $model found ($(du -h $model | cut -f1))"
    else
        echo "❌ $model not found"
    fi
done

# Check system libraries
echo -e "\n📚 System Libraries:"
ldconfig -p | grep -E "(pthread|dl|m)" | head -3

# Check available disk space
echo -e "\n💾 Disk Space:"
df -h . | tail -1

# Check memory
echo -e "\n🧠 Memory:"
free -h | head -2

echo -e "\n✅ Diagnostics complete!"
```

#### Build Script Debugging

```bash
#!/bin/bash
# debug_build.sh

set -e  # Exit on any error
set -x  # Print each command

echo "🐛 Debug Build Process"
echo "====================="

# Step 1: Environment setup
export RUST_BACKTRACE=full
export CARGO_LOG=debug

# Step 2: Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean
rm -rf target/

# Step 3: Check dependencies
echo "🔍 Checking dependencies..."
./build_diagnostics.sh

# Step 4: Build whisper.cpp with debug info
echo "🔨 Building whisper.cpp..."
cd whisper.cpp
make clean
make GGML_DEBUG=1 WHISPER_DEBUG=1
cd ..

# Step 5: Build Rust project with verbose output
echo "🦀 Building Rust project..."
cargo build --verbose --release

# Step 6: Verify build artifacts
echo "✅ Verifying build artifacts..."
if [ -f "target/release/libwhisper_rust_binding.so" ]; then
    echo "✅ Shared library built successfully"
    ldd target/release/libwhisper_rust_binding.so
else
    echo "❌ Shared library not found"
    exit 1
fi

echo "🎉 Build completed successfully!"
```

### Android Build Issues

#### NDK Configuration Problems

```bash
# Check NDK installation
echo "📱 Android NDK Diagnostics"
echo "=========================="

if [ -z "$ANDROID_NDK_ROOT" ]; then
    echo "❌ ANDROID_NDK_ROOT not set"
    echo "💡 Set it to your NDK path:"
    echo "export ANDROID_NDK_ROOT=/path/to/android-ndk-r25c"
else
    echo "✅ ANDROID_NDK_ROOT: $ANDROID_NDK_ROOT"
fi

# Check available targets
rustup target list | grep android

# Check if targets are installed
for target in aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android; do
    if rustup target list --installed | grep -q $target; then
        echo "✅ $target installed"
    else
        echo "❌ $target not installed"
        echo "💡 Install with: rustup target add $target"
    fi
done
```

#### Cross-compilation Setup

```bash
#!/bin/bash
# setup_android_toolchain.sh

echo "🔧 Setting up Android toolchain"

# Set NDK path
export ANDROID_NDK_ROOT="/path/to/android-ndk-r25c"
export PATH="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH"

# Configure cargo for cross-compilation
mkdir -p ~/.cargo
cat > ~/.cargo/config.toml << EOF
[target.aarch64-linux-android]
ar = "aarch64-linux-android-ar"
linker = "aarch64-linux-android28-clang++"

[target.armv7-linux-androideabi]
ar = "arm-linux-androideabi-ar"
linker = "armv7a-linux-androideabi28-clang++"

[target.i686-linux-android]
ar = "i686-linux-android-ar"
linker = "i686-linux-android28-clang++"

[target.x86_64-linux-android]
ar = "x86_64-linux-android-ar"
linker = "x86_64-linux-android28-clang++"
EOF

echo "✅ Android toolchain configured"
```

## 🎵 Audio Issues

### Audio Format Problems

#### Unsupported Audio Format

```rust
// Debug audio file information
pub fn debug_audio_file(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;
    
    println!("🎵 Audio File Debug: {}", file_path);
    println!("===================");
    
    // Use ffprobe to get detailed information
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            file_path
        ])
        .output()?;
    
    if output.status.success() {
        let info = String::from_utf8_lossy(&output.stdout);
        println!("📊 File Information:");
        println!("{}", info);
    } else {
        println!("❌ Could not read file information");
        
        // Try basic file check
        let metadata = std::fs::metadata(file_path)?;
        println!("📁 File size: {} bytes", metadata.len());
        
        // Check if it's a valid audio file
        let output = Command::new("file")
            .arg(file_path)
            .output()?;
        
        println!("📄 File type: {}", String::from_utf8_lossy(&output.stdout));
    }
    
    Ok(())
}

// Convert any audio file to the required format
pub fn convert_to_whisper_format(input_file: &str, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;
    
    println!("🔄 Converting {} to Whisper format...", input_file);
    
    let output = Command::new("ffmpeg")
        .args(&[
            "-i", input_file,
            "-ar", "16000",      // 16kHz sample rate
            "-ac", "1",          // Mono
            "-c:a", "pcm_s16le", // 16-bit PCM
            "-y",                // Overwrite output
            output_file
        ])
        .output()?;
    
    if output.status.success() {
        println!("✅ Conversion successful");
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("❌ Conversion failed: {}", error);
        return Err(format!("FFmpeg error: {}", error).into());
    }
    
    Ok(())
}
```

#### Audio Quality Issues

```rust
pub fn analyze_audio_quality(audio_data: &[f32]) -> AudioQualityReport {
    let sample_rate = 16000;
    let duration = audio_data.len() as f32 / sample_rate as f32;
    
    // Check for silence
    let max_amplitude = audio_data.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
    let rms = (audio_data.iter().map(|&x| x * x).sum::<f32>() / audio_data.len() as f32).sqrt();
    
    // Detect clipping
    let clipped_samples = audio_data.iter().filter(|&&x| x.abs() >= 0.99).count();
    let clipping_percentage = (clipped_samples as f32 / audio_data.len() as f32) * 100.0;
    
    // Check dynamic range
    let sorted_levels: Vec<f32> = {
        let mut levels: Vec<f32> = audio_data.iter().map(|&x| x.abs()).collect();
        levels.sort_by(|a, b| a.partial_cmp(b).unwrap());
        levels
    };
    
    let noise_floor = sorted_levels[sorted_levels.len() / 10];
    let signal_peak = sorted_levels[sorted_levels.len() * 9 / 10];
    let snr = if noise_floor > 0.0 {
        20.0 * (signal_peak / noise_floor).log10()
    } else {
        60.0
    };
    
    println!("🔊 Audio Quality Analysis");
    println!("========================");
    println!("Duration: {:.2}s", duration);
    println!("Peak Amplitude: {:.3}", max_amplitude);
    println!("RMS Level: {:.3}", rms);
    println!("SNR: {:.1} dB", snr);
    println!("Clipping: {:.1}%", clipping_percentage);
    
    // Recommendations
    if max_amplitude < 0.01 {
        println!("⚠️  Audio is very quiet - consider amplifying");
    }
    if clipping_percentage > 1.0 {
        println!("⚠️  Audio clipping detected - reduce input gain");
    }
    if snr < 20.0 {
        println!("⚠️  High noise level - consider noise reduction");
    }
    if duration < 1.0 {
        println!("⚠️  Audio too short - minimum 1 second recommended");
    }
    
    AudioQualityReport {
        duration,
        max_amplitude,
        rms,
        snr,
        clipping_percentage,
        quality_score: calculate_quality_score(max_amplitude, rms, snr, clipping_percentage),
    }
}

fn calculate_quality_score(peak: f32, rms: f32, snr: f32, clipping: f32) -> f32 {
    let mut score = 100.0;
    
    if peak < 0.1 { score -= 30.0; }
    if snr < 20.0 { score -= (20.0 - snr) * 2.0; }
    if clipping > 0.0 { score -= clipping * 5.0; }
    if rms < 0.05 { score -= 20.0; }
    
    score.max(0.0).min(100.0)
}

pub struct AudioQualityReport {
    pub duration: f32,
    pub max_amplitude: f32,
    pub rms: f32,
    pub snr: f32,
    pub clipping_percentage: f32,
    pub quality_score: f32,
}
```

### Sample Rate Issues

```rust
pub fn detect_sample_rate_issues(expected_rate: u32, actual_rate: u32) {
    if expected_rate != actual_rate {
        println!("⚠️  Sample Rate Mismatch");
        println!("Expected: {} Hz", expected_rate);
        println!("Actual: {} Hz", actual_rate);
        
        let ratio = actual_rate as f64 / expected_rate as f64;
        if ratio > 1.5 || ratio < 0.67 {
            println!("❌ Significant sample rate difference - transcription quality will be poor");
            println!("💡 Resample audio to {} Hz", expected_rate);
        } else {
            println!("⚠️  Minor sample rate difference - consider resampling for best results");
        }
    }
}

pub fn auto_resample_if_needed(audio_data: Vec<f32>, input_rate: u32, target_rate: u32) -> Vec<f32> {
    if input_rate == target_rate {
        return audio_data;
    }
    
    println!("🔄 Resampling from {} Hz to {} Hz", input_rate, target_rate);
    
    let ratio = input_rate as f64 / target_rate as f64;
    let output_length = (audio_data.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(output_length);
    
    for i in 0..output_length {
        let src_pos = i as f64 * ratio;
        let src_index = src_pos as usize;
        let frac = src_pos - src_index as f64;
        
        let sample = if src_index + 1 < audio_data.len() {
            // Linear interpolation
            audio_data[src_index] * (1.0 - frac as f32) + 
            audio_data[src_index + 1] * frac as f32
        } else if src_index < audio_data.len() {
            audio_data[src_index]
        } else {
            0.0
        };
        
        output.push(sample);
    }
    
    println!("✅ Resampled {} samples to {} samples", audio_data.len(), output.len());
    output
}
```

## ⚡ Performance Issues

### Memory Problems

#### Memory Leaks Detection

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct MemoryTracker;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for MemoryTracker {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);
        ALLOCATIONS.fetch_sub(1, Ordering::Relaxed);
    }
}

#[global_allocator]
static GLOBAL: MemoryTracker = MemoryTracker;

pub fn print_memory_stats() {
    let allocated = ALLOCATED.load(Ordering::Relaxed);
    let allocations = ALLOCATIONS.load(Ordering::Relaxed);
    
    println!("💾 Memory Stats:");
    println!("   Allocated: {} bytes ({:.2} MB)", allocated, allocated as f64 / 1024.0 / 1024.0);
    println!("   Active Allocations: {}", allocations);
    
    if allocations > 1000 {
        println!("⚠️  High number of allocations - possible memory leak");
    }
}
```

#### Out of Memory Handler

```rust
pub fn check_available_memory() -> Result<(), String> {
    use std::fs;
    
    let meminfo = fs::read_to_string("/proc/meminfo")
        .map_err(|_| "Could not read memory information")?;
    
    let mut mem_total = 0;
    let mut mem_available = 0;
    
    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            mem_total = parse_memory_line(line).unwrap_or(0);
        } else if line.starts_with("MemAvailable:") {
            mem_available = parse_memory_line(line).unwrap_or(0);
        }
    }
    
    let available_mb = mem_available / 1024;
    let total_mb = mem_total / 1024;
    let usage_percent = ((total_mb - available_mb) * 100) / total_mb;
    
    println!("💾 Memory Status:");
    println!("   Total: {} MB", total_mb);
    println!("   Available: {} MB", available_mb);
    println!("   Usage: {}%", usage_percent);
    
    if available_mb < 500 {
        return Err(format!("Low memory: only {} MB available", available_mb));
    }
    
    if usage_percent > 90 {
        println!("⚠️  High memory usage - consider using a smaller model");
    }
    
    Ok(())
}

fn parse_memory_line(line: &str) -> Option<u64> {
    line.split_whitespace().nth(1)?.parse().ok()
}
```

### CPU Performance Issues

#### CPU Usage Monitoring

```rust
use std::time::{Duration, Instant};
use std::thread;

pub struct CpuMonitor {
    start_time: Instant,
    last_cpu_time: u64,
    last_check: Instant,
}

impl CpuMonitor {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            last_cpu_time: Self::get_cpu_time().unwrap_or(0),
            last_check: Instant::now(),
        }
    }
    
    pub fn check_cpu_usage(&mut self) -> f32 {
        let current_time = Instant::now();
        let current_cpu_time = Self::get_cpu_time().unwrap_or(0);
        
        let time_diff = current_time.duration_since(self.last_check);
        let cpu_diff = current_cpu_time - self.last_cpu_time;
        
        self.last_check = current_time;
        self.last_cpu_time = current_cpu_time;
        
        if time_diff.as_nanos() > 0 {
            let cpu_usage = (cpu_diff as f64 / time_diff.as_nanos() as f64) * 100.0;
            cpu_usage as f32
        } else {
            0.0
        }
    }
    
    fn get_cpu_time() -> Option<u64> {
        use std::fs;
        
        let stat = fs::read_to_string("/proc/self/stat").ok()?;
        let parts: Vec<&str> = stat.split_whitespace().collect();
        
        if parts.len() >= 16 {
            let utime: u64 = parts[13].parse().ok()?;
            let stime: u64 = parts[14].parse().ok()?;
            Some(utime + stime)
        } else {
            None
        }
    }
    
    pub fn print_performance_warning(&self, cpu_usage: f32) {
        if cpu_usage > 95.0 {
            println!("🔥 Critical: CPU usage at {:.1}% - system may become unresponsive", cpu_usage);
        } else if cpu_usage > 85.0 {
            println!("⚠️  High CPU usage: {:.1}% - consider reducing thread count", cpu_usage);
        }
    }
}
```

### Thread Optimization

```rust
use std::sync::Arc;
use std::thread;
use rayon::prelude::*;

pub fn optimize_thread_count() -> usize {
    let cpu_count = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    
    let optimal_threads = match cpu_count {
        1 => 1,
        2..=4 => cpu_count,
        5..=8 => cpu_count - 1, // Leave one core for system
        _ => cpu_count - 2,     // Leave two cores for system
    };
    
    println!("🧵 Thread Optimization:");
    println!("   Available CPUs: {}", cpu_count);
    println!("   Optimal threads: {}", optimal_threads);
    
    optimal_threads
}

pub fn configure_rayon_threadpool() {
    let optimal_threads = optimize_thread_count();
    
    rayon::ThreadPoolBuilder::new()
        .num_threads(optimal_threads)
        .thread_name(|index| format!("whisper-worker-{}", index))
        .build_global()
        .expect("Failed to configure thread pool");
    
    println!("✅ Rayon thread pool configured with {} threads", optimal_threads);
}
```

## 🐛 Runtime Errors

### Segmentation Faults

#### Core Dump Analysis

```bash
#!/bin/bash
# analyze_crash.sh

echo "💥 Crash Analysis"
echo "================"

# Enable core dumps
ulimit -c unlimited

# Check if core dump exists
if ls core* 1> /dev/null 2>&1; then
    echo "📁 Core dump found:"
    ls -la core*
    
    # Use gdb to analyze
    echo "🔍 Starting GDB analysis..."
    gdb -batch -ex "bt" -ex "info registers" -ex "quit" target/release/your_binary core*
else
    echo "❌ No core dump found"
    echo "💡 Enable core dumps with: ulimit -c unlimited"
fi

# Check system logs
echo -e "\n📋 Recent system logs:"
dmesg | tail -20 | grep -E "(segfault|killed|fault)"

# Check memory maps
echo -e "\n🗺️  Memory maps:"
if [ -f "/proc/$$/maps" ]; then
    cat /proc/$$/maps | head -10
fi
```

#### Memory Safety Checks

```rust
use std::panic;

pub fn setup_panic_handler() {
    panic::set_hook(Box::new(|panic_info| {
        eprintln!("💥 PANIC OCCURRED!");
        eprintln!("Location: {:?}", panic_info.location());
        eprintln!("Message: {:?}", panic_info.payload().downcast_ref::<&str>());
        
        // Print stack trace
        eprintln!("Stack trace:");
        let backtrace = std::backtrace::Backtrace::capture();
        eprintln!("{}", backtrace);
        
        // Try to save state before crashing
        save_crash_state();
    }));
}

fn save_crash_state() {
    use std::fs;
    use std::time::SystemTime;
    
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let crash_file = format!("crash_report_{}.txt", timestamp);
    
    let report = format!(
        "Crash Report\n\
         ============\n\
         Timestamp: {}\n\
         Process ID: {}\n\
         \n\
         Memory Usage:\n\
         {}\n\
         \n\
         Environment:\n\
         RUST_BACKTRACE: {:?}\n\
         PWD: {:?}\n",
        timestamp,
        std::process::id(),
        format_memory_usage(),
        std::env::var("RUST_BACKTRACE"),
        std::env::current_dir()
    );
    
    if let Err(e) = fs::write(&crash_file, report) {
        eprintln!("Could not save crash report: {}", e);
    } else {
        eprintln!("Crash report saved to: {}", crash_file);
    }
}

fn format_memory_usage() -> String {
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        let mut result = String::new();
        for line in status.lines() {
            if line.starts_with("VmSize:") || 
               line.starts_with("VmRSS:") || 
               line.starts_with("VmPeak:") {
                result.push_str(line);
                result.push('\n');
            }
        }
        result
    } else {
        "Memory information not available".to_string()
    }
}
```

### Null Pointer Dereferences

```rust
use std::ptr;

pub fn safe_whisper_call(instance_id: i32, audio_data: &[f32]) -> Result<String, String> {
    // Validate inputs
    if instance_id < 0 {
        return Err("Invalid instance ID".to_string());
    }
    
    if audio_data.is_empty() {
        return Err("Empty audio data".to_string());
    }
    
    if audio_data.len() > 1_000_000 {
        return Err("Audio data too large".to_string());
    }
    
    // Check for NaN or infinite values
    for (i, &sample) in audio_data.iter().enumerate() {
        if !sample.is_finite() {
            return Err(format!("Invalid audio sample at index {}: {}", i, sample));
        }
    }
    
    let mut result_buffer = vec![0i8; 4096];
    
    let success = unsafe {
        // Validate pointers before use
        if audio_data.as_ptr().is_null() || result_buffer.as_mut_ptr().is_null() {
            return Err("Null pointer detected".to_string());
        }
        
        whisper_rust_process_audio(
            instance_id,
            audio_data.as_ptr(),
            audio_data.len() as i32,
            ptr::null(), // Language auto-detect
            result_buffer.as_mut_ptr(),
            result_buffer.len() as i32,
        )
    };
    
    if success {
        let result = unsafe {
            std::ffi::CStr::from_ptr(result_buffer.as_ptr())
        };
        
        match result.to_str() {
            Ok(text) => Ok(text.to_string()),
            Err(e) => Err(format!("Invalid UTF-8 in result: {}", e)),
        }
    } else {
        Err("Whisper processing failed".to_string())
    }
}

extern "C" {
    fn whisper_rust_process_audio(
        instance_id: i32,
        audio_data: *const f32,
        audio_len: i32,
        language: *const i8,
        result_buffer: *mut i8,
        result_buffer_size: i32,
    ) -> bool;
}
```

## 🔍 Debugging Tools

### Comprehensive Debug Mode

```rust
pub struct DebugConfig {
    pub enable_logging: bool,
    pub log_audio_stats: bool,
    pub save_intermediate_files: bool,
    pub memory_tracking: bool,
    pub performance_profiling: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enable_logging: true,
            log_audio_stats: true,
            save_intermediate_files: false,
            memory_tracking: true,
            performance_profiling: true,
        }
    }
}

pub struct WhisperDebugWrapper {
    instance_id: i32,
    config: DebugConfig,
    stats: DebugStats,
}

#[derive(Default)]
struct DebugStats {
    total_calls: usize,
    successful_calls: usize,
    total_audio_duration: f32,
    total_processing_time: f32,
    errors: Vec<String>,
}

impl WhisperDebugWrapper {
    pub fn new(model_path: &str, config: DebugConfig) -> Result<Self, String> {
        let instance_id = unsafe { 
            whisper_rust_init(model_path.as_ptr() as *const i8) 
        };
        
        if instance_id < 0 {
            return Err("Failed to initialize Whisper".to_string());
        }
        
        if config.enable_logging {
            println!("🐛 Debug mode enabled");
            println!("   Model: {}", model_path);
            println!("   Instance ID: {}", instance_id);
        }
        
        Ok(Self {
            instance_id,
            config,
            stats: DebugStats::default(),
        })
    }
    
    pub fn process_audio_debug(&mut self, audio_data: &[f32], language: Option<&str>) -> Result<String, String> {
        let start_time = std::time::Instant::now();
        self.stats.total_calls += 1;
        
        // Log audio statistics
        if self.config.log_audio_stats {
            self.log_audio_stats(audio_data);
        }
        
        // Save audio to file if requested
        if self.config.save_intermediate_files {
            self.save_debug_audio(audio_data, self.stats.total_calls);
        }
        
        // Memory tracking
        if self.config.memory_tracking {
            print_memory_stats();
        }
        
        // Process audio
        let result = safe_whisper_call(self.instance_id, audio_data);
        
        // Update statistics
        let processing_time = start_time.elapsed().as_secs_f32();
        self.stats.total_processing_time += processing_time;
        self.stats.total_audio_duration += audio_data.len() as f32 / 16000.0;
        
        match &result {
            Ok(_) => {
                self.stats.successful_calls += 1;
                if self.config.enable_logging {
                    println!("✅ Processing successful ({:.2}ms)", processing_time * 1000.0);
                }
            }
            Err(e) => {
                self.stats.errors.push(e.clone());
                if self.config.enable_logging {
                    println!("❌ Processing failed: {}", e);
                }
            }
        }
        
        result
    }
    
    fn log_audio_stats(&self, audio_data: &[f32]) {
        let duration = audio_data.len() as f32 / 16000.0;
        let max_amplitude = audio_data.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
        let rms = (audio_data.iter().map(|&x| x * x).sum::<f32>() / audio_data.len() as f32).sqrt();
        
        println!("🎵 Audio Stats:");
        println!("   Duration: {:.2}s", duration);
        println!("   Samples: {}", audio_data.len());
        println!("   Peak: {:.3}", max_amplitude);
        println!("   RMS: {:.3}", rms);
    }
    
    fn save_debug_audio(&self, audio_data: &[f32], call_number: usize) {
        let filename = format!("debug_audio_{:04}.raw", call_number);
        
        // Convert float to i16 for saving
        let pcm_data: Vec<i16> = audio_data.iter()
            .map(|&x| (x * 32767.0).clamp(-32767.0, 32767.0) as i16)
            .collect();
        
        let bytes: Vec<u8> = pcm_data.iter()
            .flat_map(|&sample| sample.to_le_bytes())
            .collect();
        
        if let Err(e) = std::fs::write(&filename, bytes) {
            println!("⚠️  Could not save debug audio: {}", e);
        } else {
            println!("💾 Saved debug audio: {}", filename);
        }
    }
    
    pub fn print_debug_summary(&self) {
        println!("\n📊 Debug Summary");
        println!("================");
        println!("Total calls: {}", self.stats.total_calls);
        println!("Successful: {}", self.stats.successful_calls);
        println!("Success rate: {:.1}%", 
                 (self.stats.successful_calls as f32 / self.stats.total_calls as f32) * 100.0);
        println!("Total audio duration: {:.2}s", self.stats.total_audio_duration);
        println!("Total processing time: {:.2}s", self.stats.total_processing_time);
        
        if self.stats.total_audio_duration > 0.0 {
            let rtf = self.stats.total_processing_time / self.stats.total_audio_duration;
            println!("Average RTF: {:.3}", rtf);
        }
        
        if !self.stats.errors.is_empty() {
            println!("\n❌ Errors encountered:");
            for (i, error) in self.stats.errors.iter().enumerate() {
                println!("   {}: {}", i + 1, error);
            }
        }
    }
}

extern "C" {
    fn whisper_rust_init(model_path: *const i8) -> i32;
}
```

### Automated Testing Framework

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_whisper_initialization() {
        let instance_id = unsafe { whisper_rust_init("ggml-tiny.bin\0".as_ptr() as *const i8) };
        assert!(instance_id >= 0, "Failed to initialize Whisper");
        
        unsafe { whisper_rust_free(instance_id) };
    }
    
    #[test]
    fn test_audio_processing() {
        let instance_id = unsafe { whisper_rust_init("ggml-tiny.bin\0".as_ptr() as *const i8) };
        assert!(instance_id >= 0);
        
        // Generate test audio (1 second of 440Hz tone)
        let sample_rate = 16000;
        let duration = 1.0;
        let frequency = 440.0;
        let samples = (sample_rate as f32 * duration) as usize;
        
        let audio_data: Vec<f32> = (0..samples)
            .map(|i| (2.0 * std::f32::consts::PI * frequency * i as f32 / sample_rate as f32).sin() * 0.1)
            .collect();
        
        let result = safe_whisper_call(instance_id, &audio_data);
        assert!(result.is_ok(), "Audio processing failed: {:?}", result);
        
        unsafe { whisper_rust_free(instance_id) };
    }
    
    #[test]
    fn test_invalid_inputs() {
        let instance_id = unsafe { whisper_rust_init("ggml-tiny.bin\0".as_ptr() as *const i8) };
        assert!(instance_id >= 0);
        
        // Test empty audio
        let result = safe_whisper_call(instance_id, &[]);
        assert!(result.is_err());
        
        // Test invalid instance ID
        let audio_data = vec![0.0; 16000];
        let result = safe_whisper_call(-1, &audio_data);
        assert!(result.is_err());
        
        unsafe { whisper_rust_free(instance_id) };
    }
    
    extern "C" {
        fn whisper_rust_free(instance_id: i32) -> bool;
    }
}
```

This comprehensive troubleshooting guide should help you resolve most issues encountered when using the Whisper Rust Binding library. For additional support, check the project's GitHub issues or create a new issue with detailed information about your problem.
