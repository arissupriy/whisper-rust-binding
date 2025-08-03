use std::env;
use std::time::Instant;
use whisper_rust_binding::{init_whisper, process_audio, free_whisper};

mod common;

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
        eprintln!("Usage: {} <model_path> <audio_file> [language] [window_sec]", args[0]);
        eprintln!("Example: {} ggml-tiny.bin output.wav ar 2.0", args[0]);
        return Ok(());
    }

    let model_path = &args[1];
    let audio_path = &args[2];
    let language = args.get(3).map(|s| s.as_str());
    let window_duration = args.get(4)
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(2.0);

    println!("🔄 Simple Sliding Window Transcription");
    println!("=====================================");
    println!("Model: {}", model_path);
    println!("Audio: {}", audio_path);
    println!("Language: {:?}", language);
    println!("Window duration: {:.1}s", window_duration);
    println!();

    // Initialize Whisper once for testing
    println!("🔧 Testing Whisper model loading...");
    let test_instance_id = init_whisper(model_path)?;
    println!("✅ Model test successful! Freeing test instance...");
    free_whisper(test_instance_id)?;

    // Load audio file
    println!("📁 Loading audio file...");
    let audio_data = load_wav_file(audio_path)?;
    let total_duration = audio_data.len() as f32 / 16000.0;
    println!("✅ Audio loaded: {:.2}s ({} samples)", total_duration, audio_data.len());

    // Process in sliding windows
    let window_size = (16000.0 * window_duration) as usize;
    let hop_size = window_size / 2; // 50% overlap
    let hop_duration = window_duration / 2.0;
    
    println!("📊 Processing configuration:");
    println!("   - Window size: {} samples ({:.1}s)", window_size, window_duration);
    println!("   - Hop size: {} samples ({:.1}s)", hop_size, hop_duration);
    println!("   - Overlap: {:.1}%", 50.0);
    println!();
    
    let mut window_count = 0;
    let mut total_processing_time = 0.0;
    let mut successful_transcriptions = 0;
    
    println!("🚀 Starting sliding window processing...");
    println!("{}", "=".repeat(80));

    let overall_start = Instant::now();
    let mut pos = 0;

    while pos + window_size <= audio_data.len() {
        window_count += 1;
        let window_start_time = pos as f32 / 16000.0;
        
        println!("🎬 Window #{} [{:.1}s - {:.1}s] (samples: {} - {})", 
                window_count, 
                window_start_time, 
                window_start_time + window_duration,
                pos,
                pos + window_size);

        let window = &audio_data[pos..pos + window_size];
        let process_start = Instant::now();
        
        // Create new Whisper instance for each window to avoid memory issues
        println!("   🔧 Creating fresh Whisper instance...");
        let window_instance_id = match init_whisper(model_path) {
            Ok(id) => id,
            Err(e) => {
                println!("   ❌ Failed to initialize Whisper: {}", e);
                pos += hop_size;
                continue;
            }
        };
        
        // Process window with fresh Whisper instance
        match process_audio(window_instance_id, window, language) {
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
        
        // Cleanup window instance
        let _ = free_whisper(window_instance_id);
        
        println!("   {}", "-".repeat(60));
        
        // Move to next window
        pos += hop_size;
        
        // Small delay for system stability
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let total_time = overall_start.elapsed();
    let overall_rtf = total_duration / total_time.as_secs_f32();

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
    println!("   - Expected windows: {:.0}", ((total_duration - window_duration) / hop_duration).max(0.0) + 1.0);
    
    println!();
    println!("💡 Sliding Window vs Chunk-based Comparison:");
    println!("   🔄 Sliding Window (this implementation):");
    println!("     ✅ Overlapping windows for better context");
    println!("     ✅ Smooth transitions between segments");
    println!("     ✅ Better detection of speech boundaries");
    println!("     ✅ More suitable for continuous speech");
    println!("   📦 Chunk-based (murajaah_chunks.rs):");
    println!("     ✅ Clean segment separation");
    println!("     ✅ No overlap - perfect for review");
    println!("     ✅ Easier timestamp management");
    println!("     ✅ Better for discrete phrase learning");
    
    println!();
    println!("🎯 Use Cases:");
    println!("   - Sliding Window: Live transcription, continuous speech");
    println!("   - Chunk-based: Murajaah review, phrase practice");

    Ok(())
}
