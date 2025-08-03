use std::env;
use std::time::Instant;
use std::process::Command;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <model_path> <audio_file> [language] [chunk_duration_sec]", args[0]);
        eprintln!("Example: {} ggml-tiny.bin output.wav ar 2", args[0]);
        std::process::exit(1);
    }

    let model_path = &args[1];
    let audio_path = &args[2];
    let language = if args.len() > 3 { Some(args[3].as_str()) } else { None };
    let chunk_duration = if args.len() > 4 { 
        args[4].parse::<u32>().unwrap_or(2) 
    } else { 
        2 
    };

    println!("🎵 Murajaah (Review) Chunk-Based Transcription");
    println!("==============================================");
    println!("Model: {}", model_path);
    println!("Audio: {}", audio_path);
    println!("Language: {:?}", language);
    println!("Chunk duration: {}s (perfect for murajaah)", chunk_duration);
    println!();

    // Check if audio file exists
    if !Path::new(audio_path).exists() {
        eprintln!("❌ Audio file not found: {}", audio_path);
        std::process::exit(1);
    }

    // Check if model exists
    if !Path::new(model_path).exists() {
        eprintln!("❌ Model file not found: {}", model_path);
        std::process::exit(1);
    }

    // Get audio duration using ffmpeg
    println!("📊 Getting audio information...");
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "quiet",
            "-show_entries", "format=duration",
            "-of", "csv=p=0",
            audio_path
        ])
        .output();

    let duration = match output {
        Ok(output) => {
            let duration_str = String::from_utf8_lossy(&output.stdout);
            duration_str.trim().parse::<f32>().unwrap_or(0.0)
        }
        Err(_) => {
            eprintln!("❌ Could not get audio duration. Make sure ffprobe is installed.");
            std::process::exit(1);
        }
    };

    if duration <= 0.0 {
        eprintln!("❌ Invalid audio duration: {}", duration);
        std::process::exit(1);
    }

    println!("📏 Total audio duration: {:.2}s", duration);

    // Calculate number of chunks
    let total_chunks = (duration / chunk_duration as f32).ceil() as u32;
    println!("🔢 Total chunks: {} ({}s each)", total_chunks, chunk_duration);
    println!();

    // Create chunks directory
    let chunks_dir = "temp_chunks";
    if Path::new(chunks_dir).exists() {
        fs::remove_dir_all(chunks_dir).unwrap_or_default();
    }
    fs::create_dir(chunks_dir).expect("Failed to create chunks directory");

    println!("🚀 Starting chunk-based transcription for murajaah...");
    println!("{}", "=".repeat(80));

    let mut all_transcriptions = Vec::new();
    let overall_start = Instant::now();
    let mut successful_chunks = 0;

    for chunk_idx in 0..total_chunks {
        let start_time = chunk_idx * chunk_duration;
        let chunk_filename = format!("{}/chunk_{:03}.wav", chunks_dir, chunk_idx);
        
        println!("🎬 Chunk #{}/{}", chunk_idx + 1, total_chunks);
        println!("   ⏰ Time: {}s - {}s ({}s duration)", 
                start_time, start_time + chunk_duration, chunk_duration);

        // Extract chunk using ffmpeg
        let extract_start = Instant::now();
        let ffmpeg_result = Command::new("ffmpeg")
            .args(&[
                "-i", audio_path,
                "-ss", &start_time.to_string(),
                "-t", &chunk_duration.to_string(),
                "-ar", "16000",
                "-ac", "1",
                "-y",
                &chunk_filename
            ])
            .output();

        match ffmpeg_result {
            Ok(_) => {
                let extract_time = extract_start.elapsed();
                println!("   ✅ Chunk extracted ({:.3}s)", extract_time.as_secs_f32());
            }
            Err(e) => {
                println!("   ❌ Failed to extract chunk: {}", e);
                continue;
            }
        }

        // Transcribe chunk
        let transcribe_start = Instant::now();
        
        let mut transcribe_cmd = Command::new("./target/debug/examples/transcribe_file");
        transcribe_cmd.args(&[model_path, &chunk_filename]);
        
        if let Some(lang) = language {
            transcribe_cmd.arg(lang);
        }

        let transcribe_result = transcribe_cmd.output();

        match transcribe_result {
            Ok(output) => {
                let transcribe_time = transcribe_start.elapsed();
                let stdout = String::from_utf8_lossy(&output.stdout);
                
                // Extract transcription from output
                if let Some(start_marker) = stdout.find("-------------------------------------------") {
                    if let Some(end_marker) = stdout.rfind("-------------------------------------------") {
                        if start_marker != end_marker {
                            let transcription = stdout[start_marker + 43..end_marker]
                                .trim()
                                .to_string();
                            
                            if !transcription.is_empty() {
                                let real_time_factor = chunk_duration as f32 / transcribe_time.as_secs_f32();
                                println!("   ✅ Transcription ({:.3}s, {:.1}x realtime):", 
                                        transcribe_time.as_secs_f32(), real_time_factor);
                                
                                let timestamped_text = format!("[{}s-{}s] {}", 
                                                              start_time, start_time + chunk_duration, transcription);
                                println!("   📝 {}", timestamped_text);
                                
                                all_transcriptions.push(timestamped_text);
                                successful_chunks += 1;
                                
                                if real_time_factor > 1.0 {
                                    println!("   ⚡ Real-time capable!");
                                } else {
                                    println!("   ⚠️  Slower than real-time");
                                }
                            } else {
                                println!("   ⚠️  No transcription (silent/noise) ({:.3}s)", transcribe_time.as_secs_f32());
                            }
                        } else {
                            println!("   ⚠️  No transcription found in output ({:.3}s)", transcribe_time.as_secs_f32());
                        }
                    } else {
                        println!("   ❌ Could not parse transcription output");
                    }
                } else {
                    println!("   ❌ Could not find transcription markers in output");
                }
            }
            Err(e) => {
                println!("   ❌ Failed to transcribe chunk: {}", e);
            }
        }

        // Clean up chunk file
        fs::remove_file(&chunk_filename).unwrap_or_default();
        
        println!("   {}", "-".repeat(60));
        
        // Small delay between chunks for system stability
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    // Cleanup chunks directory
    fs::remove_dir(chunks_dir).unwrap_or_default();

    let total_time = overall_start.elapsed();
    let overall_rtf = duration / total_time.as_secs_f32();

    println!();
    println!("🏁 Murajaah Chunk-Based Transcription Complete!");
    println!("{}", "=".repeat(80));
    println!("📊 Summary:");
    println!("   - Total chunks processed: {}/{}", successful_chunks, total_chunks);
    println!("   - Total processing time: {:.2}s", total_time.as_secs_f32());
    println!("   - Audio duration: {:.2}s", duration);
    println!("   - Overall real-time factor: {:.1}x", overall_rtf);
    println!("   - Success rate: {:.1}%", (successful_chunks as f32 / total_chunks as f32) * 100.0);
    
    if overall_rtf > 1.0 {
        println!("   ✅ System is real-time capable! ({:.1}x faster than real-time)", overall_rtf);
    } else {
        println!("   ⚠️  System is slower than real-time ({:.1}x)", overall_rtf);
    }

    println!();
    println!("📋 Complete Murajaah Transcription Timeline:");
    println!("{}", "=".repeat(80));
    if !all_transcriptions.is_empty() {
        for transcription in &all_transcriptions {
            println!("{}", transcription);
        }
    } else {
        println!("No transcription results.");
    }
    println!("{}", "=".repeat(80));

    println!();
    println!("💡 Perfect for Murajaah (Review):");
    println!("   - Each {}s chunk shows clear time segments", chunk_duration);
    println!("   - Easy to review specific parts of the recitation");
    println!("   - No overlapping content - clean segmentation");
    println!("   - Ideal for study and memorization review");
}
