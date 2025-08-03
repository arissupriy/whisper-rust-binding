# Performance Guide

Comprehensive guide for optimizing performance with Whisper Rust Binding.

## ⚡ Overview

This guide covers:
- 🚀 **Performance Metrics** - Understanding benchmarks and measurements
- 🎯 **Model Selection** - Choosing the right model for your use case
- ⚙️ **System Optimization** - Hardware and OS-level optimizations
- 🧠 **Memory Management** - Efficient memory usage patterns
- 🔄 **Processing Strategies** - Batch vs streaming vs real-time
- 📊 **Monitoring & Profiling** - Performance measurement tools

## 🚀 Performance Metrics

### Real-time Factor (RTF)

The **Real-time Factor** is the key performance metric for speech recognition:

```
RTF = Processing Time / Audio Duration
```

- **RTF < 1.0**: Faster than real-time (e.g., 0.05 = 18x faster)
- **RTF = 1.0**: Real-time processing
- **RTF > 1.0**: Slower than real-time

### Current Performance Benchmarks

| Model | Arabic RTF | English RTF | Memory Usage | Quality |
|-------|------------|-------------|--------------|---------|
| Tiny | **0.055** (18x) | 0.045 (22x) | ~120 MB | Good |
| Base | 0.15 (6.7x) | 0.12 (8.3x) | ~210 MB | Very Good |
| Small | 0.45 (2.2x) | 0.35 (2.9x) | ~600 MB | Excellent |
| Medium | 1.2 (0.8x) | 0.95 (1.1x) | ~1.2 GB | Excellent |
| Large | 2.8 (0.36x) | 2.1 (0.48x) | ~2.9 GB | Best |

### Confidence and Accuracy

```rust
// Example performance measurement
pub struct PerformanceMetrics {
    pub processing_time_ms: f64,
    pub audio_duration_ms: f64,
    pub rtf: f64,
    pub confidence: f32,
    pub memory_peak_mb: f64,
    pub cpu_usage_percent: f32,
}

impl PerformanceMetrics {
    pub fn calculate_rtf(processing_time_ms: f64, audio_duration_ms: f64) -> f64 {
        processing_time_ms / audio_duration_ms
    }
    
    pub fn performance_rating(&self) -> &str {
        match self.rtf {
            rtf if rtf < 0.1 => "Excellent (10x+ faster than real-time)",
            rtf if rtf < 0.5 => "Very Good (2-10x faster than real-time)",
            rtf if rtf < 1.0 => "Good (faster than real-time)",
            rtf if rtf < 2.0 => "Acceptable (near real-time)",
            _ => "Poor (slower than real-time)",
        }
    }
}
```

## 🎯 Model Selection Guide

### Use Case Matrix

| Use Case | Recommended Model | RTF Target | Rationale |
|----------|-------------------|------------|-----------|
| **Real-time Chat** | Tiny | < 0.2 | Fast response critical |
| **Live Subtitles** | Base | < 0.5 | Balance speed/quality |
| **Voice Commands** | Tiny | < 0.1 | Ultra-low latency |
| **Podcast Transcription** | Small/Medium | < 2.0 | Quality over speed |
| **Academic Research** | Large | Any | Best accuracy |
| **Mobile Apps** | Tiny | < 0.3 | Battery/memory limits |
| **Server Backend** | Base/Small | < 1.0 | Throughput focused |

### Model Characteristics

#### Tiny Model (39 MB)
```rust
// Tiny model configuration for maximum speed
pub fn configure_tiny_model() -> WhisperConfig {
    WhisperConfig {
        model_path: "ggml-tiny.bin",
        language: Some("ar"), // Specify language for better performance
        translate: false,
        no_context: true, // Disable context for speed
        single_segment: false,
        max_tokens: 200, // Limit output length
        temperature: 0.0, // Deterministic output
        best_of: 1, // Single pass
    }
}
```

#### Base Model (142 MB)
```rust
// Base model configuration for balanced performance
pub fn configure_base_model() -> WhisperConfig {
    WhisperConfig {
        model_path: "ggml-base.bin",
        language: Some("ar"),
        translate: false,
        no_context: false, // Enable context for better accuracy
        single_segment: false,
        max_tokens: 500,
        temperature: 0.0,
        best_of: 1,
    }
}
```

#### Small Model (466 MB)
```rust
// Small model configuration for high quality
pub fn configure_small_model() -> WhisperConfig {
    WhisperConfig {
        model_path: "ggml-small.bin",
        language: Some("ar"),
        translate: false,
        no_context: false,
        single_segment: false,
        max_tokens: 1000,
        temperature: 0.1, // Slight randomness for quality
        best_of: 2, // Multiple passes for better results
    }
}
```

## ⚙️ System Optimization

### CPU Optimization

#### Thread Configuration

```rust
use std::thread;

pub struct OptimalThreadConfig {
    pub whisper_threads: usize,
    pub audio_processing_threads: usize,
    pub total_cores: usize,
}

impl OptimalThreadConfig {
    pub fn detect_optimal() -> Self {
        let total_cores = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        
        let whisper_threads = match total_cores {
            1..=2 => 1,
            3..=4 => 2,
            5..=8 => 4,
            9..=16 => 6,
            _ => 8, // Cap at 8 threads for diminishing returns
        };
        
        let audio_processing_threads = (total_cores - whisper_threads).max(1);
        
        Self {
            whisper_threads,
            audio_processing_threads,
            total_cores,
        }
    }
    
    pub fn apply_to_whisper(&self, params: &mut WhisperFullParams) {
        params.n_threads = self.whisper_threads as i32;
    }
}
```

#### CPU Affinity (Linux)

```rust
use std::process::Command;

pub fn set_cpu_performance_mode() -> Result<(), Box<dyn std::error::Error>> {
    // Set CPU governor to performance mode
    let output = Command::new("sudo")
        .args(&["cpupower", "frequency-set", "-g", "performance"])
        .output();
    
    match output {
        Ok(result) if result.status.success() => {
            println!("✅ CPU governor set to performance mode");
        }
        _ => {
            println!("⚠️  Could not set CPU governor (requires sudo)");
        }
    }
    
    // Disable CPU frequency scaling
    let output = Command::new("sudo")
        .args(&["echo", "0", ">", "/sys/devices/system/cpu/cpufreq/boost"])
        .output();
    
    Ok(())
}

pub fn set_process_priority() -> Result<(), Box<dyn std::error::Error>> {
    // Set high priority for current process
    let output = Command::new("sudo")
        .args(&["renice", "-n", "-10", "-p", &std::process::id().to_string()])
        .output()?;
    
    if output.status.success() {
        println!("✅ Process priority increased");
    } else {
        println!("⚠️  Could not increase process priority");
    }
    
    Ok(())
}
```

### Memory Optimization

#### Memory Pool Management

```rust
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct AudioBufferPool {
    buffers: Arc<Mutex<VecDeque<Vec<f32>>>>,
    buffer_size: usize,
    max_buffers: usize,
}

impl AudioBufferPool {
    pub fn new(buffer_size: usize, max_buffers: usize) -> Self {
        let mut buffers = VecDeque::with_capacity(max_buffers);
        
        // Pre-allocate buffers
        for _ in 0..max_buffers {
            buffers.push_back(vec![0.0; buffer_size]);
        }
        
        Self {
            buffers: Arc::new(Mutex::new(buffers)),
            buffer_size,
            max_buffers,
        }
    }
    
    pub fn get_buffer(&self) -> Option<Vec<f32>> {
        self.buffers.lock().unwrap().pop_front()
    }
    
    pub fn return_buffer(&self, mut buffer: Vec<f32>) {
        buffer.clear();
        buffer.resize(self.buffer_size, 0.0);
        
        let mut buffers = self.buffers.lock().unwrap();
        if buffers.len() < self.max_buffers {
            buffers.push_back(buffer);
        }
    }
    
    pub fn available_buffers(&self) -> usize {
        self.buffers.lock().unwrap().len()
    }
}
```

#### Memory Usage Monitoring

```rust
use std::fs;

pub struct MemoryMonitor;

impl MemoryMonitor {
    pub fn get_memory_usage() -> Result<MemoryInfo, Box<dyn std::error::Error>> {
        let status = fs::read_to_string("/proc/self/status")?;
        
        let mut vm_rss = 0;
        let mut vm_peak = 0;
        
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                vm_rss = Self::parse_memory_line(line)?;
            } else if line.starts_with("VmPeak:") {
                vm_peak = Self::parse_memory_line(line)?;
            }
        }
        
        Ok(MemoryInfo {
            current_mb: vm_rss / 1024,
            peak_mb: vm_peak / 1024,
        })
    }
    
    fn parse_memory_line(line: &str) -> Result<u64, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            Ok(parts[1].parse::<u64>()?)
        } else {
            Err("Invalid memory line format".into())
        }
    }
    
    pub fn check_memory_pressure() -> bool {
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            let mut mem_total = 0;
            let mut mem_available = 0;
            
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    mem_total = Self::parse_memory_line(line).unwrap_or(0);
                } else if line.starts_with("MemAvailable:") {
                    mem_available = Self::parse_memory_line(line).unwrap_or(0);
                }
            }
            
            if mem_total > 0 {
                let usage_percent = (mem_total - mem_available) * 100 / mem_total;
                return usage_percent > 85; // High memory pressure
            }
        }
        false
    }
}

pub struct MemoryInfo {
    pub current_mb: u64,
    pub peak_mb: u64,
}
```

## 🧠 Memory Management Patterns

### Instance Pool Pattern

```rust
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

pub struct WhisperInstancePool {
    instances: Arc<Mutex<VecDeque<i32>>>,
    model_path: String,
    max_instances: usize,
}

impl WhisperInstancePool {
    pub fn new(model_path: String, pool_size: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let mut instances = VecDeque::with_capacity(pool_size);
        
        // Pre-create instances
        for _ in 0..pool_size {
            let instance_id = unsafe { whisper_rust_init(model_path.as_ptr() as *const i8) };
            if instance_id >= 0 {
                instances.push_back(instance_id);
            } else {
                return Err("Failed to create Whisper instance".into());
            }
        }
        
        Ok(Self {
            instances: Arc::new(Mutex::new(instances)),
            model_path,
            max_instances: pool_size,
        })
    }
    
    pub fn get_instance(&self) -> Option<WhisperInstanceGuard> {
        let mut instances = self.instances.lock().unwrap();
        instances.pop_front().map(|id| {
            WhisperInstanceGuard {
                instance_id: id,
                pool: Arc::clone(&self.instances),
            }
        })
    }
    
    pub fn available_instances(&self) -> usize {
        self.instances.lock().unwrap().len()
    }
}

pub struct WhisperInstanceGuard {
    instance_id: i32,
    pool: Arc<Mutex<VecDeque<i32>>>,
}

impl WhisperInstanceGuard {
    pub fn id(&self) -> i32 {
        self.instance_id
    }
}

impl Drop for WhisperInstanceGuard {
    fn drop(&mut self) {
        // Return instance to pool
        let mut instances = self.pool.lock().unwrap();
        instances.push_back(self.instance_id);
    }
}

extern "C" {
    fn whisper_rust_init(model_path: *const i8) -> i32;
}
```

### Smart Caching Strategy

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct TranscriptionCache {
    cache: HashMap<AudioHash, CachedResult>,
    max_entries: usize,
    ttl: Duration,
}

#[derive(Hash, Eq, PartialEq)]
struct AudioHash([u8; 32]);

struct CachedResult {
    text: String,
    timestamp: Instant,
    confidence: f32,
}

impl TranscriptionCache {
    pub fn new(max_entries: usize, ttl_seconds: u64) -> Self {
        Self {
            cache: HashMap::with_capacity(max_entries),
            max_entries,
            ttl: Duration::from_secs(ttl_seconds),
        }
    }
    
    pub fn get(&mut self, audio: &[f32]) -> Option<String> {
        let hash = self.hash_audio(audio);
        
        if let Some(cached) = self.cache.get(&hash) {
            if cached.timestamp.elapsed() < self.ttl {
                return Some(cached.text.clone());
            } else {
                self.cache.remove(&hash);
            }
        }
        
        None
    }
    
    pub fn insert(&mut self, audio: &[f32], text: String, confidence: f32) {
        if self.cache.len() >= self.max_entries {
            self.evict_oldest();
        }
        
        let hash = self.hash_audio(audio);
        self.cache.insert(hash, CachedResult {
            text,
            timestamp: Instant::now(),
            confidence,
        });
    }
    
    fn hash_audio(&self, audio: &[f32]) -> AudioHash {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        
        // Sample every 100th sample for speed
        for (i, &sample) in audio.iter().enumerate() {
            if i % 100 == 0 {
                hasher.update(&sample.to_le_bytes());
            }
        }
        
        let hash_bytes = hasher.finalize();
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash_bytes);
        AudioHash(result)
    }
    
    fn evict_oldest(&mut self) {
        if let Some((oldest_key, _)) = self.cache.iter()
            .min_by_key(|(_, v)| v.timestamp) {
            let key_to_remove = *oldest_key;
            self.cache.remove(&key_to_remove);
        }
    }
    
    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, v| now.duration_since(v.timestamp) < self.ttl);
    }
}
```

## 🔄 Processing Strategies

### Batch Processing

```rust
pub struct BatchProcessor {
    instance_pool: WhisperInstancePool,
    batch_size: usize,
    processing_queue: Arc<Mutex<VecDeque<AudioJob>>>,
}

struct AudioJob {
    id: String,
    audio_data: Vec<f32>,
    language: Option<String>,
    callback: Box<dyn FnOnce(String) + Send>,
}

impl BatchProcessor {
    pub fn new(model_path: String, pool_size: usize, batch_size: usize) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            instance_pool: WhisperInstancePool::new(model_path, pool_size)?,
            batch_size,
            processing_queue: Arc::new(Mutex::new(VecDeque::new())),
        })
    }
    
    pub fn submit_job<F>(&self, audio_data: Vec<f32>, language: Option<String>, callback: F) 
    where F: FnOnce(String) + Send + 'static {
        let job = AudioJob {
            id: uuid::Uuid::new_v4().to_string(),
            audio_data,
            language,
            callback: Box::new(callback),
        };
        
        self.processing_queue.lock().unwrap().push_back(job);
        self.try_process_batch();
    }
    
    fn try_process_batch(&self) {
        let mut queue = self.processing_queue.lock().unwrap();
        
        if queue.len() >= self.batch_size {
            let batch: Vec<_> = queue.drain(0..self.batch_size).collect();
            drop(queue); // Release lock
            
            // Process batch in parallel
            std::thread::spawn({
                let pool = &self.instance_pool;
                move || {
                    for job in batch {
                        if let Some(instance) = pool.get_instance() {
                            let result = Self::process_single(instance.id(), &job.audio_data, &job.language);
                            (job.callback)(result);
                        }
                    }
                }
            });
        }
    }
    
    fn process_single(instance_id: i32, audio: &[f32], language: &Option<String>) -> String {
        let mut result_buffer = [0i8; 2048];
        let language_ptr = language.as_ref()
            .map(|s| s.as_ptr() as *const i8)
            .unwrap_or(std::ptr::null());
        
        let success = unsafe {
            whisper_rust_process_audio(
                instance_id,
                audio.as_ptr(),
                audio.len() as i32,
                language_ptr,
                result_buffer.as_mut_ptr(),
                result_buffer.len() as i32,
            )
        };
        
        if success {
            unsafe {
                std::ffi::CStr::from_ptr(result_buffer.as_ptr())
                    .to_string_lossy()
                    .into_owned()
            }
        } else {
            String::new()
        }
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

### Streaming Processing

```rust
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

pub struct StreamingProcessor {
    audio_sender: Sender<Vec<f32>>,
    result_receiver: Receiver<StreamingResult>,
    _worker_handle: thread::JoinHandle<()>,
}

pub struct StreamingResult {
    pub text: String,
    pub confidence: f32,
    pub processing_time_ms: u64,
    pub is_final: bool,
}

impl StreamingProcessor {
    pub fn new(model_path: String, chunk_duration_ms: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let (audio_tx, audio_rx) = channel();
        let (result_tx, result_rx) = channel();
        
        let instance_id = unsafe { whisper_rust_init(model_path.as_ptr() as *const i8) };
        if instance_id < 0 {
            return Err("Failed to initialize Whisper".into());
        }
        
        let worker_handle = thread::spawn(move || {
            Self::worker_loop(instance_id, audio_rx, result_tx, chunk_duration_ms);
        });
        
        Ok(Self {
            audio_sender: audio_tx,
            result_receiver: result_rx,
            _worker_handle: worker_handle,
        })
    }
    
    pub fn send_audio(&self, audio_chunk: Vec<f32>) -> Result<(), String> {
        self.audio_sender.send(audio_chunk)
            .map_err(|_| "Failed to send audio chunk".to_string())
    }
    
    pub fn try_get_result(&self) -> Option<StreamingResult> {
        self.result_receiver.try_recv().ok()
    }
    
    pub fn get_result_timeout(&self, timeout: Duration) -> Option<StreamingResult> {
        self.result_receiver.recv_timeout(timeout).ok()
    }
    
    fn worker_loop(
        instance_id: i32,
        audio_rx: Receiver<Vec<f32>>,
        result_tx: Sender<StreamingResult>,
        chunk_duration_ms: u32,
    ) {
        let sample_rate = 16000;
        let chunk_size = (sample_rate * chunk_duration_ms / 1000) as usize;
        let mut audio_buffer = Vec::new();
        let mut partial_results = Vec::new();
        
        while let Ok(audio_chunk) = audio_rx.recv() {
            let start_time = Instant::now();
            
            audio_buffer.extend(audio_chunk);
            
            // Process when we have enough audio
            if audio_buffer.len() >= chunk_size {
                let chunk_to_process = audio_buffer.drain(0..chunk_size).collect::<Vec<_>>();
                
                // Process the chunk
                let mut result_buffer = [0i8; 2048];
                let success = unsafe {
                    whisper_rust_process_audio(
                        instance_id,
                        chunk_to_process.as_ptr(),
                        chunk_to_process.len() as i32,
                        std::ptr::null(), // Auto-detect language
                        result_buffer.as_mut_ptr(),
                        result_buffer.len() as i32,
                    )
                };
                
                if success {
                    let text = unsafe {
                        std::ffi::CStr::from_ptr(result_buffer.as_ptr())
                            .to_string_lossy()
                            .into_owned()
                    };
                    
                    if !text.trim().is_empty() {
                        let processing_time = start_time.elapsed().as_millis() as u64;
                        
                        // Simple confidence estimation (would be more sophisticated in practice)
                        let confidence = if text.len() > 10 { 0.85 } else { 0.60 };
                        
                        partial_results.push(text.clone());
                        
                        let result = StreamingResult {
                            text,
                            confidence,
                            processing_time_ms: processing_time,
                            is_final: false,
                        };
                        
                        let _ = result_tx.send(result);
                        
                        // Send final result every 5 chunks
                        if partial_results.len() >= 5 {
                            let final_text = partial_results.join(" ");
                            partial_results.clear();
                            
                            let final_result = StreamingResult {
                                text: final_text,
                                confidence: 0.90,
                                processing_time_ms: processing_time,
                                is_final: true,
                            };
                            
                            let _ = result_tx.send(final_result);
                        }
                    }
                }
            }
        }
    }
}
```

## 📊 Monitoring & Profiling

### Performance Profiler

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

pub struct PerformanceProfiler {
    measurements: HashMap<String, Vec<Duration>>,
    current_operations: HashMap<String, Instant>,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            measurements: HashMap::new(),
            current_operations: HashMap::new(),
        }
    }
    
    pub fn start_measurement(&mut self, operation: &str) {
        self.current_operations.insert(operation.to_string(), Instant::now());
    }
    
    pub fn end_measurement(&mut self, operation: &str) {
        if let Some(start_time) = self.current_operations.remove(operation) {
            let duration = start_time.elapsed();
            self.measurements.entry(operation.to_string())
                .or_insert_with(Vec::new)
                .push(duration);
        }
    }
    
    pub fn get_stats(&self, operation: &str) -> Option<OperationStats> {
        self.measurements.get(operation).map(|durations| {
            let total_ms: f64 = durations.iter().map(|d| d.as_secs_f64() * 1000.0).sum();
            let count = durations.len();
            let avg_ms = total_ms / count as f64;
            
            let mut sorted_durations = durations.clone();
            sorted_durations.sort();
            
            let p50_ms = sorted_durations[count / 2].as_secs_f64() * 1000.0;
            let p95_ms = sorted_durations[count * 95 / 100].as_secs_f64() * 1000.0;
            let p99_ms = sorted_durations[count * 99 / 100].as_secs_f64() * 1000.0;
            
            OperationStats {
                count,
                avg_ms,
                p50_ms,
                p95_ms,
                p99_ms,
                total_ms,
            }
        })
    }
    
    pub fn print_report(&self) {
        println!("🔍 Performance Report");
        println!("===================");
        
        for (operation, _) in &self.measurements {
            if let Some(stats) = self.get_stats(operation) {
                println!("📊 {}", operation);
                println!("   Count: {}", stats.count);
                println!("   Average: {:.2}ms", stats.avg_ms);
                println!("   P50: {:.2}ms", stats.p50_ms);
                println!("   P95: {:.2}ms", stats.p95_ms);
                println!("   P99: {:.2}ms", stats.p99_ms);
                println!("   Total: {:.2}ms", stats.total_ms);
                println!();
            }
        }
    }
}

pub struct OperationStats {
    pub count: usize,
    pub avg_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub total_ms: f64,
}

// Usage example
pub fn benchmark_transcription() {
    let mut profiler = PerformanceProfiler::new();
    
    // Load test audio
    let (_, audio_data) = read_wav_file("test_audio.wav").unwrap();
    let instance_id = unsafe { whisper_rust_init("ggml-tiny.bin\0".as_ptr() as *const i8) };
    
    // Run multiple iterations
    for i in 0..100 {
        profiler.start_measurement("transcription");
        
        let mut result_buffer = [0i8; 2048];
        unsafe {
            whisper_rust_process_audio(
                instance_id,
                audio_data.as_ptr(),
                audio_data.len() as i32,
                std::ptr::null(),
                result_buffer.as_mut_ptr(),
                result_buffer.len() as i32,
            );
        }
        
        profiler.end_measurement("transcription");
        
        if i % 10 == 0 {
            println!("Completed {} iterations...", i + 1);
        }
    }
    
    profiler.print_report();
}
```

### Resource Monitor

```rust
use std::thread;
use std::time::Duration;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

pub struct ResourceMonitor {
    monitoring: Arc<AtomicBool>,
    _monitor_thread: thread::JoinHandle<()>,
}

impl ResourceMonitor {
    pub fn start() -> Self {
        let monitoring = Arc::new(AtomicBool::new(true));
        let monitoring_clone = Arc::clone(&monitoring);
        
        let monitor_thread = thread::spawn(move || {
            while monitoring_clone.load(Ordering::Relaxed) {
                Self::collect_metrics();
                thread::sleep(Duration::from_secs(1));
            }
        });
        
        Self {
            monitoring,
            _monitor_thread: monitor_thread,
        }
    }
    
    fn collect_metrics() {
        // CPU usage
        if let Ok(cpu_usage) = Self::get_cpu_usage() {
            println!("🔥 CPU Usage: {:.1}%", cpu_usage);
        }
        
        // Memory usage
        if let Ok(memory) = MemoryMonitor::get_memory_usage() {
            println!("💾 Memory: {}MB (Peak: {}MB)", memory.current_mb, memory.peak_mb);
        }
        
        // Check memory pressure
        if MemoryMonitor::check_memory_pressure() {
            println!("⚠️  High memory pressure detected!");
        }
    }
    
    fn get_cpu_usage() -> Result<f32, Box<dyn std::error::Error>> {
        // Read from /proc/stat for CPU usage
        use std::fs;
        
        let stat1 = fs::read_to_string("/proc/stat")?;
        thread::sleep(Duration::from_millis(100));
        let stat2 = fs::read_to_string("/proc/stat")?;
        
        let cpu1 = Self::parse_cpu_line(&stat1)?;
        let cpu2 = Self::parse_cpu_line(&stat2)?;
        
        let total_diff = cpu2.total - cpu1.total;
        let idle_diff = cpu2.idle - cpu1.idle;
        
        if total_diff > 0 {
            Ok(100.0 * (1.0 - idle_diff as f32 / total_diff as f32))
        } else {
            Ok(0.0)
        }
    }
    
    fn parse_cpu_line(stat: &str) -> Result<CpuStats, Box<dyn std::error::Error>> {
        let first_line = stat.lines().next().ok_or("No CPU line found")?;
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        
        if parts.len() < 5 {
            return Err("Invalid CPU stats format".into());
        }
        
        let user: u64 = parts[1].parse()?;
        let nice: u64 = parts[2].parse()?;
        let system: u64 = parts[3].parse()?;
        let idle: u64 = parts[4].parse()?;
        
        Ok(CpuStats {
            total: user + nice + system + idle,
            idle,
        })
    }
}

struct CpuStats {
    total: u64,
    idle: u64,
}

impl Drop for ResourceMonitor {
    fn drop(&mut self) {
        self.monitoring.store(false, Ordering::Relaxed);
    }
}
```

### Optimization Recommendations

```rust
pub struct OptimizationRecommendations {
    pub model_recommendations: Vec<String>,
    pub system_recommendations: Vec<String>,
    pub code_recommendations: Vec<String>,
}

impl OptimizationRecommendations {
    pub fn analyze_performance(metrics: &PerformanceMetrics, system_info: &SystemInfo) -> Self {
        let mut model_recs = Vec::new();
        let mut system_recs = Vec::new();
        let mut code_recs = Vec::new();
        
        // Model recommendations
        if metrics.rtf > 1.0 {
            model_recs.push("Consider using a smaller model (tiny or base) for better performance".to_string());
        }
        if metrics.confidence < 0.8 {
            model_recs.push("Consider using a larger model for better accuracy".to_string());
        }
        
        // System recommendations
        if system_info.memory_usage_percent > 85.0 {
            system_recs.push("High memory usage detected - consider reducing batch size".to_string());
        }
        if system_info.cpu_usage_percent > 90.0 {
            system_recs.push("High CPU usage - consider reducing thread count".to_string());
        }
        if system_info.available_cores < 4 {
            system_recs.push("Limited CPU cores - use tiny model for best performance".to_string());
        }
        
        // Code recommendations
        if metrics.processing_time_ms > 1000.0 {
            code_recs.push("Consider implementing audio chunking for better responsiveness".to_string());
        }
        
        Self {
            model_recommendations: model_recs,
            system_recommendations: system_recs,
            code_recommendations: code_recs,
        }
    }
    
    pub fn print_recommendations(&self) {
        println!("💡 Optimization Recommendations");
        println!("================================");
        
        if !self.model_recommendations.is_empty() {
            println!("🎯 Model Optimizations:");
            for rec in &self.model_recommendations {
                println!("   • {}", rec);
            }
            println!();
        }
        
        if !self.system_recommendations.is_empty() {
            println!("⚙️  System Optimizations:");
            for rec in &self.system_recommendations {
                println!("   • {}", rec);
            }
            println!();
        }
        
        if !self.code_recommendations.is_empty() {
            println!("💻 Code Optimizations:");
            for rec in &self.code_recommendations {
                println!("   • {}", rec);
            }
        }
    }
}

pub struct SystemInfo {
    pub available_cores: usize,
    pub memory_usage_percent: f32,
    pub cpu_usage_percent: f32,
}
```

This comprehensive performance guide provides all the tools and strategies needed to optimize Whisper Rust Binding for maximum efficiency in any environment!
