use std::env;
use std::time::Instant;
use std::process::Command;
use std::fs;
use std::path::Path;

mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <model_path> <audio_file> [language] [window_sec] [overlap_sec]", args[0]);
        eprintln!("Example: {} ggml-tiny.bin output.wav ar 2.0 0.5", args[0]);
        eprintln!("  window_sec: Window duration (default: 2.0s)");
        eprintln!("  overlap_sec: Overlap duration (default: 0.5s)");
        return Ok(());
    }

    let model_path = &args[1];
    let audio_path = &args[2];
    let language = args.get(3).map(|s| s.as_str()).unwrap_or("ar");
    let window_duration = args.get(4)
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(2.0);
    let overlap_duration = args.get(5)
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.5);

    if overlap_duration >= window_duration {
        eprintln!("❌ Overlap duration must be less than window duration");
        return Ok(());
    }

    let hop_duration = window_duration - overlap_duration;

    println!("🔄 Hybrid Sliding Window Transcription");
    println!("=====================================");
    println!("Model: {}", model_path);
    println!("Audio: {}", audio_path);
    println!("Language: {}", language);
    println!("Window duration: {:.1}s", window_duration);
    println!("Overlap duration: {:.1}s", overlap_duration);
    println!("Hop duration: {:.1}s", hop_duration);
    println!("Overlap percentage: {:.1}%", (overlap_duration / window_duration) * 100.0);
    println!();

    // Get audio duration using ffprobe
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "quiet",
            "-show_entries", "format=duration",
            "-of", "csv=p=0",
            audio_path
        ])
        .output();

    let total_duration = match output {
        Ok(output) => {
            let duration_str = String::from_utf8_lossy(&output.stdout);
            duration_str.trim().parse::<f32>().unwrap_or(0.0)
        }
        Err(_) => {
            eprintln!("❌ Could not get audio duration. Make sure ffprobe is installed.");
            std::process::exit(1);
        }
    };

    if total_duration <= 0.0 {
        eprintln!("❌ Invalid audio duration: {}", total_duration);
        std::process::exit(1);
    }

    println!("📏 Total audio duration: {:.2}s", total_duration);

    // Calculate number of windows
    let total_windows = ((total_duration - window_duration) / hop_duration).max(0.0) as u32 + 1;
    println!("🔢 Total windows: {} (overlap: {:.1}s)", total_windows, overlap_duration);
    println!();

    // Create windows directory
    let windows_dir = "temp_windows";
    if Path::new(windows_dir).exists() {
        fs::remove_dir_all(windows_dir).unwrap_or_default();
    }
    fs::create_dir(windows_dir).expect("Failed to create windows directory");

    println!("🚀 Starting hybrid sliding window processing...");
    println!("{}", "=".repeat(80));

    let mut all_transcriptions = Vec::new();
    let overall_start = Instant::now();
    let mut successful_windows = 0;

    for window_idx in 0..total_windows {
        let start_time = window_idx as f32 * hop_duration;
        let end_time = (start_time + window_duration).min(total_duration);
        let actual_duration = end_time - start_time;
        
        if actual_duration < 0.5 {
            // Skip windows that are too short
            continue;
        }
        
        let window_filename = format!("{}/window_{:03}.wav", windows_dir, window_idx);
        
        println!("🎬 Window #{}/{}", window_idx + 1, total_windows);
        println!("   ⏰ Time: {:.1}s - {:.1}s ({:.1}s duration)", 
                start_time, end_time, actual_duration);

        // Extract window using ffmpeg with overlap
        let extract_start = Instant::now();
        let extract_result = Command::new("ffmpeg")
            .args(&[
                "-v", "quiet",
                "-y",
                "-i", audio_path,
                "-ss", &start_time.to_string(),
                "-t", &actual_duration.to_string(),
                "-ar", "16000",
                "-ac", "1",
                "-f", "wav",
                &window_filename
            ])
            .output();

        if extract_result.is_err() {
            println!("   ❌ Failed to extract window");
            continue;
        }

        let extract_time = extract_start.elapsed();
        println!("   📄 Window extracted ({:.3}s)", extract_time.as_secs_f32());

        // Transcribe window using external transcribe_file process
        let transcribe_start = Instant::now();
        let transcribe_result = Command::new("./target/debug/examples/transcribe_file")
            .args(&[model_path, &window_filename, language])
            .output();

        match transcribe_result {
            Ok(output) => {
                let transcribe_time = transcribe_start.elapsed();
                let output_str = String::from_utf8_lossy(&output.stdout);
                
                // Parse transcription from output
                if let Some(start_marker) = output_str.find("-------------------------------------------") {
                    if let Some(content_start) = output_str[start_marker..].find('\n') {
                        let content_section = &output_str[start_marker + content_start + 1..];
                        if let Some(end_marker) = content_section.find("-------------------------------------------") {
                            let transcription = content_section[..end_marker].trim();
                            
                            if !transcription.is_empty() {
                                let real_time_factor = actual_duration / transcribe_time.as_secs_f32();
                                println!("   ✅ Transcription ({:.3}s, {:.1}x realtime):", 
                                        transcribe_time.as_secs_f32(), real_time_factor);
                                
                                let timestamped_text = format!("[{:.1}s-{:.1}s] {}", 
                                                              start_time, end_time, transcription);
                                println!("   📝 {}", timestamped_text);
                                
                                all_transcriptions.push((start_time, end_time, transcription.to_string()));
                                successful_windows += 1;
                                
                                if real_time_factor > 1.0 {
                                    println!("   ⚡ Real-time capable!");
                                } else {
                                    println!("   ⚠️  Slower than real-time");
                                }
                            } else {
                                println!("   🔇 Silent window ({:.3}s)", transcribe_time.as_secs_f32());
                            }
                        } else {
                            println!("   ⚠️  Could not find end marker in output ({:.3}s)", transcribe_time.as_secs_f32());
                        }
                    } else {
                        println!("   ⚠️  Could not find content start ({:.3}s)", transcribe_time.as_secs_f32());
                    }
                } else {
                    println!("   ❌ Could not find transcription start marker in output");
                }
            }
            Err(e) => {
                println!("   ❌ Failed to transcribe window: {}", e);
            }
        }

        // Clean up window file
        fs::remove_file(&window_filename).unwrap_or_default();
        
        println!("   {}", "-".repeat(60));
        
        // Small delay between windows for system stability
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Cleanup windows directory
    fs::remove_dir(windows_dir).unwrap_or_default();

    let total_time = overall_start.elapsed();
    let overall_rtf = total_duration / total_time.as_secs_f32();

    println!();
    println!("🏁 Hybrid Sliding Window Processing Complete!");
    println!("{}", "=".repeat(80));
    println!("📊 Performance Summary:");
    println!("   - Total windows processed: {}", total_windows);
    println!("   - Successful transcriptions: {}", successful_windows);
    println!("   - Success rate: {:.1}%", (successful_windows as f32 / total_windows as f32) * 100.0);
    println!("   - Audio duration: {:.2}s", total_duration);
    println!("   - Total processing time: {:.2}s", total_time.as_secs_f32());
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
    println!("   - Overlap: {:.1}s ({:.1}%)", overlap_duration, (overlap_duration / window_duration) * 100.0);
    println!("   - Total windows: {}", total_windows);
    
    println!();
    println!("📝 Transcription Results with Overlap:");
    println!("{}", "=".repeat(60));
    
    if all_transcriptions.is_empty() {
        println!("   🔇 No transcriptions found");
    } else {
        for (start, end, text) in &all_transcriptions {
            println!("   [{:.1}s-{:.1}s] {}", start, end, text);
        }
        
        println!();
        println!("🔍 Overlap Analysis:");
        for i in 1..all_transcriptions.len() {
            let prev_end = all_transcriptions[i-1].1;
            let curr_start = all_transcriptions[i].0;
            let actual_overlap = prev_end - curr_start;
            if actual_overlap > 0.0 {
                println!("   🔗 Windows {} & {}: {:.1}s overlap", i, i+1, actual_overlap);
            }
        }
    }

    println!();
    println!("💡 Hybrid Sliding Window Benefits:");
    println!("   ✅ True overlapping windows for better context");
    println!("   ✅ Stable processing (each window is independent)");
    println!("   ✅ Configurable overlap amount");
    println!("   ✅ Better speech boundary detection");
    println!("   ✅ Suitable for continuous speech analysis");
    println!("   ✅ Perfect for murajaah with context preservation");

    Ok(())
}
