use std::env;
use std::time::Instant;
use std::collections::VecDeque;
use whisper_rust_binding::{init_whisper, process_audio, free_whisper};

mod common;

struct SlidingWindow {
    buffer: VecDeque<f32>,
    window_size: usize,
    hop_size: usize,
    sample_rate: usize,
}

impl SlidingWindow {
    fn new(window_duration_sec: f32, hop_duration_sec: f32, sample_rate: usize) -> Self {
        let window_size = (window_duration_sec * sample_rate as f32) as usize;
        let hop_size = (hop_duration_sec * sample_rate as f32) as usize;
        
        Self {
            buffer: VecDeque::with_capacity(window_size * 2),
            window_size,
            hop_size,
            sample_rate,
        }
    }
    
    fn add_samples(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.buffer.push_back(sample);
            
            // Keep buffer size reasonable
            if self.buffer.len() > self.window_size * 2 {
                self.buffer.pop_front();
            }
        }
    }
    
    fn get_windows(&mut self) -> Vec<Vec<f32>> {
        let mut windows = Vec::new();
        
        while self.buffer.len() >= self.window_size {
            // Extract current window
            let window: Vec<f32> = self.buffer.iter().take(self.window_size).cloned().collect();
            windows.push(window);
            
            // Move window by hop_size
            for _ in 0..self.hop_size.min(self.buffer.len()) {
                self.buffer.pop_front();
            }
            
            // If remaining buffer is too small for next window, break
            if self.buffer.len() < self.window_size {
                break;
            }
        }
        
        windows
    }
    
    fn has_enough_data(&self) -> bool {
        self.buffer.len() >= self.window_size
    }
}

fn load_wav_file(file_path: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::{BufReader, Read};
    
    let mut file = BufReader::new(File::open(file_path)?);
    
    // Skip WAV header (44 bytes)
    let mut header = [0u8; 44];
    file.read_exact(&mut header)?;
    
    // Read PCM data
    let mut pcm_data = Vec::new();
    file.read_to_end(&mut pcm_data)?;
    
    // Convert bytes to i16 samples
    let mut samples = Vec::new();
    for chunk in pcm_data.chunks_exact(2) {
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        samples.push(sample as f32 / 32768.0);
    }
    
    Ok(samples)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <model_path> <audio_file> [language] [window_sec] [hop_sec]", args[0]);
        eprintln!("Example: {} ggml-tiny.bin output.wav ar 2.0 0.5", args[0]);
        eprintln!("  window_sec: Sliding window duration (default: 2.0s)");
        eprintln!("  hop_sec: Step size between windows (default: 0.5s)");
        return Ok(());
    }

    let model_path = &args[1];
    let audio_path = &args[2];
    let language = args.get(3).map(|s| s.as_str());
    let window_duration = args.get(4)
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(2.0);
    let hop_duration = args.get(5)
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.5);

    println!("🔄 Real-Time Sliding Window Transcription");
    println!("=========================================");
    println!("Model: {}", model_path);
    println!("Audio: {}", audio_path);
    println!("Language: {:?}", language);
    println!("Window duration: {:.1}s", window_duration);
    println!("Hop duration: {:.1}s (overlap: {:.1}s)", hop_duration, window_duration - hop_duration);
    println!();

    // Initialize Whisper
    println!("🔧 Initializing Whisper model...");
    let instance_id = init_whisper(model_path)?;
    println!("✅ Model loaded! Instance ID: {}", instance_id);

    // Load audio file
    println!("📁 Loading audio file...");
    let audio_data = load_wav_file(audio_path)?;
    let total_duration = audio_data.len() as f32 / 16000.0;
    println!("✅ Audio loaded: {:.2}s ({} samples)", total_duration, audio_data.len());

    // Initialize sliding window
    let mut sliding_window = SlidingWindow::new(window_duration, hop_duration, 16000);
    
    // Simulate real-time processing by feeding audio in chunks
    let chunk_size = (16000 as f32 * 0.5) as usize; // 500ms chunks for simulation
    let mut chunk_start = 0;
    let mut window_count = 0;
    let mut total_processing_time = 0.0;
    let mut successful_transcriptions = 0;
    
    println!("🚀 Starting sliding window processing...");
    println!("{}", "=".repeat(80));

    let overall_start = Instant::now();

    while chunk_start < audio_data.len() {
        let chunk_end = (chunk_start + chunk_size).min(audio_data.len());
        let chunk = &audio_data[chunk_start..chunk_end];
        
        // Add chunk to sliding window
        sliding_window.add_samples(chunk);
        
        // Process all available windows
        let windows = sliding_window.get_windows();
        
        for window in windows {
            window_count += 1;
            let window_start_time = ((window_count - 1) as f32) * hop_duration;
            
            println!("🎬 Window #{} [{:.1}s - {:.1}s] (samples: {})", 
                    window_count, 
                    window_start_time, 
                    window_start_time + window_duration,
                    window.len());

            let process_start = Instant::now();
            
            // Process window with Whisper - only if we have enough samples
            if window.len() >= 16000 { // At least 1 second of audio
                match process_audio(instance_id, &window, language) {
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
            } else {
                println!("   ⏭️  Skipping short window (only {} samples)", window.len());
            }
            
            println!("   {}", "-".repeat(60));
        }
        
        chunk_start = chunk_end;
        
        // Simulate real-time delay (in real application, this would be natural)
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    let total_time = overall_start.elapsed();
    let overall_rtf = total_duration / total_time.as_secs_f32();

    // Cleanup
    free_whisper(instance_id)?;

    println!();
    println!("🏁 Sliding Window Processing Complete!");
    println!("{}", "=".repeat(80));
    println!("📊 Performance Summary:");
    println!("   - Total windows processed: {}", window_count);
    println!("   - Successful transcriptions: {}", successful_transcriptions);
    println!("   - Success rate: {:.1}%", (successful_transcriptions as f32 / window_count as f32) * 100.0);
    println!("   - Audio duration: {:.2}s", total_duration);
    println!("   - Total processing time: {:.2}s", total_time.as_secs_f32());
    println!("   - Average processing per window: {:.3}s", total_processing_time / window_count as f32);
    println!("   - Overall real-time factor: {:.1}x", overall_rtf);
    
    if overall_rtf > 1.0 {
        println!("   ✅ System is real-time capable! ({:.1}x faster than real-time)", overall_rtf);
    } else {
        println!("   ⚠️  System is slower than real-time ({:.1}x)", overall_rtf);
    }

    println!();
    println!("🔄 Sliding Window Analysis:");
    println!("   - Window size: {:.1}s", window_duration);
    println!("   - Hop size: {:.1}s", hop_duration);
    println!("   - Overlap: {:.1}s ({:.1}%)", 
            window_duration - hop_duration, 
            ((window_duration - hop_duration) / window_duration) * 100.0);
    println!("   - Total windows: {}", window_count);
    println!("   - Expected windows: {:.0}", (total_duration / hop_duration).ceil());
    
    println!();
    println!("💡 Sliding Window Benefits:");
    println!("   ✅ Continuous processing with overlap");
    println!("   ✅ Better context preservation");
    println!("   ✅ Smoother transcription flow");
    println!("   ✅ Real-time capability");
    println!("   ✅ Perfect for live audio or murajaah review");

    Ok(())
}
