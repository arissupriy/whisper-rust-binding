use std::env;
use std::time::Instant;
use whisper_rust_binding::{init_whisper, free_whisper, get_model_info, process_audio};

mod common;
use common::audio_utils::{load_wav_file, normalize_audio};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <model_path> <audio_file> [language] [window_size_sec] [step_size_sec]", args[0]);
        eprintln!("Example: {} ggml-tiny.bin output.wav ar 5.0 2.5", args[0]);
        std::process::exit(1);
    }

    let model_path = &args[1];
    let audio_path = &args[2];
    let language = if args.len() > 3 { Some(args[3].as_str()) } else { None };
    let window_size_sec = if args.len() > 4 { 
        args[4].parse::<f32>().unwrap_or(5.0) 
    } else { 
        5.0 
    };
    let step_size_sec = if args.len() > 5 { 
        args[5].parse::<f32>().unwrap_or(2.5) 
    } else { 
        2.5 
    };

    println!("🎵 Real-Time Sliding Window Transcription");
    println!("{}", "=".repeat(50));
    println!("Model: {}", model_path);
    println!("Audio: {}", audio_path);
    println!("Language: {:?}", language);
    println!("Window size: {:.1}s", window_size_sec);
    println!("Step size: {:.1}s", step_size_sec);
    println!("Overlap: {:.1}s", window_size_sec - step_size_sec);
    println!();

    // Load audio first
    println!("🎧 Loading audio file...");
    let mut audio_data = match load_wav_file(audio_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("❌ Failed to load audio: {}", e);
            std::process::exit(1);
        }
    };

    // Whisper expects 16kHz mono
    let sample_rate = 16000;
    let channels = 1;

    println!("📈 Audio specs: {} channels, {}Hz sample rate", channels, sample_rate);
    println!("📏 Total duration: {:.2}s ({} samples)", audio_data.len() as f32 / sample_rate as f32, audio_data.len());

    // Normalize audio
    normalize_audio(&mut audio_data);
    println!("🔧 Audio normalized");

    // Calculate window parameters
    let samples_per_window = (window_size_sec * sample_rate as f32) as usize;
    let samples_per_step = (step_size_sec * sample_rate as f32) as usize;
    let total_samples = audio_data.len();
    let total_windows = if total_samples <= samples_per_window {
        1
    } else {
        ((total_samples - samples_per_window) / samples_per_step) + 1
    };

    println!("🔢 Window parameters:");
    println!("   - Samples per window: {}", samples_per_window);
    println!("   - Samples per step: {}", samples_per_step);
    println!("   - Total windows: {}", total_windows);
    println!();

    // Process with sliding window - create new instance for each window
    println!("🚀 Starting real-time sliding window transcription...");
    println!("{}", "=".repeat(80));

    let mut total_transcription = String::new();
    let overall_start = Instant::now();
    let mut successful_windows = 0;

    for window_idx in 0..total_windows {
        let start_sample = window_idx * samples_per_step;
        let end_sample = std::cmp::min(start_sample + samples_per_window, total_samples);
        
        if start_sample >= total_samples {
            break;
        }

        let window_audio = &audio_data[start_sample..end_sample];
        let window_duration = window_audio.len() as f32 / sample_rate as f32;
        let start_time_sec = start_sample as f32 / sample_rate as f32;
        let end_time_sec = end_sample as f32 / sample_rate as f32;

        println!("🎬 Window #{}/{}", window_idx + 1, total_windows);
        println!("   ⏰ Time: {:.2}s - {:.2}s ({:.2}s duration)", 
                start_time_sec, end_time_sec, window_duration);
        println!("   📊 Samples: {} - {} ({} samples)", 
                start_sample, end_sample, window_audio.len());

        // Create new instance for each window to avoid state conflicts
        let instance_start = Instant::now();
        let instance_id = match init_whisper(model_path) {
            Ok(id) => id,
            Err(e) => {
                println!("   ❌ Failed to load model: {:?}", e);
                continue;
            }
        };
        let init_time = instance_start.elapsed();

        // Process this window
        let window_start = Instant::now();
        
        match process_audio(instance_id, window_audio, language) {
            Ok(transcription) => {
                let process_time = window_start.elapsed();
                let total_time = instance_start.elapsed();
                let real_time_factor = window_duration / total_time.as_secs_f32();
                
                if !transcription.trim().is_empty() {
                    println!("   ✅ Transcription (init: {:.3}s, process: {:.3}s, total: {:.3}s, {:.1}x realtime):", 
                            init_time.as_secs_f32(), process_time.as_secs_f32(), 
                            total_time.as_secs_f32(), real_time_factor);
                    
                    // Add timestamp to transcription
                    let timestamped_text = format!("[{:.1}s-{:.1}s] {}", 
                                                  start_time_sec, end_time_sec, transcription.trim());
                    println!("   📝 {}", timestamped_text);
                    
                    total_transcription.push_str(&timestamped_text);
                    total_transcription.push('\n');
                    successful_windows += 1;
                    
                    if real_time_factor > 1.0 {
                        println!("   ⚡ Real-time capable!");
                    } else {
                        println!("   ⚠️  Slower than real-time");
                    }
                } else {
                    println!("   ⚠️  No transcription (silent/noise) (total: {:.3}s)", total_time.as_secs_f32());
                }
            }
            Err(e) => {
                println!("   ❌ Error: {:?}", e);
            }
        }
        
        // Free the instance for this window
        if let Err(e) = free_whisper(instance_id) {
            println!("   ⚠️  Warning: Failed to free instance {}: {:?}", instance_id, e);
        }
        
        println!("   {}", "-".repeat(60));
        
        // Simulate processing delay for real-time demonstration
        // std::thread::sleep(std::time::Duration::from_millis(step_size_sec as u64 * 1000));
    }

    let total_time = overall_start.elapsed();
    let audio_duration = total_samples as f32 / sample_rate as f32;
    let overall_rtf = audio_duration / total_time.as_secs_f32();

    println!();
    println!("🏁 Real-Time Sliding Window Transcription Complete!");
    println!("{}", "=".repeat(80));
    println!("📊 Summary:");
    println!("   - Total windows processed: {}/{}", successful_windows, total_windows);
    println!("   - Total processing time: {:.2}s", total_time.as_secs_f32());
    println!("   - Audio duration: {:.2}s", audio_duration);
    println!("   - Overall real-time factor: {:.1}x", overall_rtf);
    println!("   - Success rate: {:.1}%", (successful_windows as f32 / total_windows as f32) * 100.0);
    
    if overall_rtf > 1.0 {
        println!("   ✅ System is real-time capable! ({:.1}x faster than real-time)", overall_rtf);
    } else {
        println!("   ⚠️  System is slower than real-time ({:.1}x)", overall_rtf);
    }

    println!();
    println!("📋 Complete Transcription Timeline:");
    println!("{}", "=".repeat(80));
    if !total_transcription.trim().is_empty() {
        for line in total_transcription.lines() {
            println!("{}", line);
        }
    } else {
        println!("No transcription results.");
    }
    println!("{}", "=".repeat(80));

    println!();
    println!("💡 Use Case: Real-time streaming transcription");
    println!("   - Each window represents a chunk of live audio");
    println!("   - Overlapping windows provide context continuity");
    println!("   - Fresh model instances prevent state contamination");
    println!("   - Perfect for live streaming applications!");
}
