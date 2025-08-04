use std::env;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::VecDeque;
use whisper_rust_binding::{init_whisper, process_audio, free_whisper};

mod common;

struct RealtimeBuffer {
    buffer: Arc<Mutex<VecDeque<f32>>>,
    sample_rate: usize,
    max_size: usize,
}

impl RealtimeBuffer {
    fn new(sample_rate: usize, max_duration_sec: f32) -> Self {
        let max_size = (sample_rate as f32 * max_duration_sec) as usize;
        Self {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            sample_rate,
            max_size,
        }
    }
    
    fn add_audio(&self, samples: &[f32]) {
        let mut buffer = self.buffer.lock().unwrap();
        for &sample in samples {
            buffer.push_back(sample);
            if buffer.len() > self.max_size {
                buffer.pop_front();
            }
        }
    }
    
    fn get_latest_window(&self, duration_sec: f32) -> Vec<f32> {
        let window_size = (self.sample_rate as f32 * duration_sec) as usize;
        let buffer = self.buffer.lock().unwrap();
        
        if buffer.len() >= window_size {
            buffer.iter().rev().take(window_size).rev().cloned().collect()
        } else {
            buffer.iter().cloned().collect()
        }
    }
    
    fn has_enough_data(&self, duration_sec: f32) -> bool {
        let window_size = (self.sample_rate as f32 * duration_sec) as usize;
        let buffer = self.buffer.lock().unwrap();
        buffer.len() >= window_size
    }
}

fn simulate_audio_stream(file_path: &str, buffer: RealtimeBuffer, chunk_duration_ms: u64) -> Result<(), String> {
    use std::fs::File;
    use std::io::{BufReader, Read};
    
    let mut file = BufReader::new(File::open(file_path).map_err(|e| e.to_string())?);
    
    // Skip WAV header
    let mut header = [0u8; 44];
    file.read_exact(&mut header).map_err(|e| e.to_string())?;
    
    // Read all audio data
    let mut pcm_data = Vec::new();
    file.read_to_end(&mut pcm_data).map_err(|e| e.to_string())?;
    
    // Convert to f32 samples
    let mut all_samples = Vec::new();
    for chunk in pcm_data.chunks_exact(2) {
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        all_samples.push(sample as f32 / 32768.0);
    }
    
    // Simulate streaming by sending chunks
    let samples_per_chunk = (16000.0 * chunk_duration_ms as f32 / 1000.0) as usize;
    
    println!("📡 Starting audio stream simulation...");
    println!("   - Total samples: {}", all_samples.len());
    println!("   - Chunk size: {} samples ({} ms)", samples_per_chunk, chunk_duration_ms);
    println!("   - Total duration: {:.2}s", all_samples.len() as f32 / 16000.0);
    
    for chunk in all_samples.chunks(samples_per_chunk) {
        buffer.add_audio(chunk);
        thread::sleep(std::time::Duration::from_millis(chunk_duration_ms));
    }
    
    println!("📡 Audio stream completed!");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <model_path> <audio_file> [language] [window_sec] [hop_ms]", args[0]);
        eprintln!("Example: {} ggml-tiny.bin output.wav ar 2.0 500", args[0]);
        eprintln!("  window_sec: Processing window duration (default: 2.0s)");
        eprintln!("  hop_ms: Processing interval in milliseconds (default: 500ms)");
        return Ok(());
    }

    let model_path = &args[1];
    let audio_path = &args[2];
    let language = args.get(3).map(|s| s.as_str());
    let window_duration = args.get(4)
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(2.0);
    let hop_interval_ms = args.get(5)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(500);

    println!("🎤 Real-Time Audio Processing Simulation");
    println!("=====================================");
    println!("Model: {}", model_path);
    println!("Audio: {}", audio_path);
    println!("Language: {:?}", language);
    println!("Window duration: {:.1}s", window_duration);
    println!("Processing interval: {}ms", hop_interval_ms);
    println!();

    // Initialize Whisper
    println!("🔧 Initializing Whisper model...");
    let instance_id = init_whisper(model_path)?;
    println!("✅ Model loaded! Instance ID: {}", instance_id);

    // Create real-time buffer (keep 10 seconds of audio)
    let buffer = RealtimeBuffer::new(16000, 10.0);
    let buffer_clone = RealtimeBuffer {
        buffer: Arc::clone(&buffer.buffer),
        sample_rate: buffer.sample_rate,
        max_size: buffer.max_size,
    };

    // Start audio streaming thread
    let audio_path_clone = audio_path.to_string();
    let stream_handle = thread::spawn(move || {
        simulate_audio_stream(&audio_path_clone, buffer_clone, 50) // 50ms chunks
    });

    // Real-time processing loop
    println!("🚀 Starting real-time processing...");
    println!("{}", "=".repeat(80));

    let mut window_count = 0;
    let mut total_processing_time = 0.0;
    let mut successful_transcriptions = 0;
    let start_time = Instant::now();
    
    // Wait a bit for initial buffer
    thread::sleep(std::time::Duration::from_millis(1000));

    loop {
        if !buffer.has_enough_data(window_duration) {
            // Check if streaming is done
            if stream_handle.is_finished() {
                break;
            }
            thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        window_count += 1;
        let current_time = start_time.elapsed().as_secs_f32();
        
        println!("🎬 Window #{} [Current time: {:.1}s]", window_count, current_time);

        // Get latest audio window
        let audio_window = buffer.get_latest_window(window_duration);
        
        let process_start = Instant::now();
        
        // Process with Whisper
        match process_audio(instance_id, &audio_window, language) {
            Ok(result) => {
                let process_time = process_start.elapsed();
                total_processing_time += process_time.as_secs_f32();
                
                let rtf = process_time.as_secs_f32() / window_duration;
                
                if !result.trim().is_empty() {
                    successful_transcriptions += 1;
                    println!("   ✅ Transcription ({:.3}s, {:.1}x realtime):", 
                            process_time.as_secs_f32(), 1.0 / rtf);
                    println!("   📝 {}", result.trim());
                    
                    if rtf < 1.0 {
                        println!("   ⚡ Real-time capable!");
                    } else {
                        println!("   ⚠️  Slower than real-time");
                    }
                } else {
                    println!("   🔇 Silent window ({:.3}s)", process_time.as_secs_f32());
                }
            }
            Err(e) => {
                println!("   ❌ Processing failed: {}", e);
            }
        }
        
        println!("   {}", "-".repeat(60));
        
        // Wait for next processing cycle
        thread::sleep(std::time::Duration::from_millis(hop_interval_ms));
    }

    // Wait for streaming to complete
    let _ = stream_handle.join();

    let total_time = start_time.elapsed();

    // Cleanup
    free_whisper(instance_id)?;

    println!();
    println!("🏁 Real-Time Processing Complete!");
    println!("{}", "=".repeat(80));
    println!("📊 Performance Summary:");
    println!("   - Total windows processed: {}", window_count);
    println!("   - Successful transcriptions: {}", successful_transcriptions);
    println!("   - Success rate: {:.1}%", (successful_transcriptions as f32 / window_count as f32) * 100.0);
    println!("   - Total runtime: {:.2}s", total_time.as_secs_f32());
    println!("   - Average processing per window: {:.3}s", total_processing_time / window_count as f32);
    println!("   - Processing interval: {}ms", hop_interval_ms);
    
    let avg_rtf = (total_processing_time / window_count as f32) / window_duration;
    if avg_rtf < 1.0 {
        println!("   ✅ System is real-time capable! (avg {:.1}x faster)", 1.0 / avg_rtf);
    } else {
        println!("   ⚠️  System is slower than real-time (avg {:.1}x)", avg_rtf);
    }

    println!();
    println!("🎤 Real-Time Processing Analysis:");
    println!("   - Window size: {:.1}s", window_duration);
    println!("   - Processing interval: {}ms", hop_interval_ms);
    println!("   - Buffer overlap: Continuous");
    println!("   - Latency: ~{}ms (processing interval)", hop_interval_ms);
    
    println!();
    println!("💡 Real-Time Processing Benefits:");
    println!("   ✅ Live audio processing simulation");
    println!("   ✅ Continuous buffer management");
    println!("   ✅ Low-latency transcription");
    println!("   ✅ Perfect for live murajaah or streaming");
    println!("   ✅ Adaptive to audio input rate");

    Ok(())
}
