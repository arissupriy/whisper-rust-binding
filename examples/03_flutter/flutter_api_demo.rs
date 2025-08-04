use whisper_rust_binding::flutter_api::*;
use std::fs::File;
use std::io::Read;

/// Example demonstrating Flutter API usage for production-ready real-time transcription
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🚀 Flutter API Demo - Production Ready Transcriber");
    println!("================================================");
    
    // Configuration
    let instance_id = "main_transcriber".to_string();
    let model_path = "ggml-tiny.bin".to_string();
    
    // Create transcriber optimized for murajaah
    println!("\n📱 Creating murajaah-optimized transcriber...");
    match FlutterTranscriberApi::create_murajaah_transcriber(instance_id.clone(), model_path) {
        Ok(msg) => println!("{}", msg),
        Err(e) => {
            eprintln!("❌ Failed to create transcriber: {}", e);
            return Ok(());
        }
    }
    
    // Health check
    println!("\n🔍 Performing health check...");
    match FlutterTranscriberApi::health_check(instance_id.clone()) {
        Ok(health) => println!("{}", health),
        Err(e) => eprintln!("⚠️ Health check failed: {}", e),
    }
    
    // Simulate Flutter Record audio chunks
    println!("\n🎤 Simulating Flutter Record audio stream...");
    
    // Load sample audio file to simulate real-time chunks
    let mut audio_file = File::open("output.wav")?;
    let mut wav_data = Vec::new();
    audio_file.read_to_end(&mut wav_data)?;
    
    // Skip WAV header (44 bytes) and convert to f32
    let raw_audio = &wav_data[44..];
    let audio_samples: Vec<f32> = raw_audio
        .chunks_exact(2)
        .map(|chunk| {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
            sample as f32 / 32768.0
        })
        .collect();
    
    // Simulate real-time streaming in chunks (50ms chunks = 800 samples at 16kHz)
    let chunk_size = 800; // 50ms at 16kHz
    let mut chunk_count = 0;
    let mut transcription_count = 0;
    
    for chunk in audio_samples.chunks(chunk_size) {
        chunk_count += 1;
        
        // Add audio chunk (simulating Flutter Record callback)
        match FlutterTranscriberApi::add_audio_chunk(instance_id.clone(), chunk.to_vec()) {
            Ok(status) => {
                if chunk_count % 20 == 0 { // Log every 1 second (20 chunks)
                    println!("  📊 Buffer: {:.1}ms ({:.1}% full) - {} samples", 
                        status.current_duration_ms,
                        status.buffer_usage_percent,
                        status.samples_count
                    );
                }
                
                // Check if ready for processing
                if status.is_ready_for_processing {
                    // Process and get transcription
                    match FlutterTranscriberApi::process_if_ready(instance_id.clone()) {
                        Ok(Some(result)) => {
                            transcription_count += 1;
                            println!("\n🗣️ Transcription #{}: \"{}\"", transcription_count, result.text);
                            println!("   ⏱️ Processing: {}ms | Confidence: {:.2} | Real-time: {}", 
                                result.processing_time_ms,
                                result.confidence,
                                result.is_real_time
                            );
                            
                            // Simulate validation against expected text
                            let expected_text = "السلام عليكم ورحمة الله وبركاته";
                            match FlutterTranscriberApi::validate_transcription(
                                instance_id.clone(),
                                result.text.clone(),
                                expected_text.to_string(),
                            ) {
                                Ok(validation) => {
                                    println!("   ✅ Validation: {} (similarity: {:.2})", 
                                        if validation.is_match { "MATCH" } else { "NO MATCH" },
                                        validation.similarity_score
                                    );
                                    if let Some(suggestion) = validation.suggestion {
                                        println!("   💡 Suggestion: {}", suggestion);
                                    }
                                }
                                Err(e) => println!("   ❌ Validation error: {}", e),
                            }
                        }
                        Ok(None) => {
                            // Not ready yet, continue
                        }
                        Err(e) => eprintln!("   ❌ Processing error: {}", e),
                    }
                }
            }
            Err(e) => eprintln!("❌ Failed to add chunk: {}", e),
        }
        
        // Simulate real-time delay (50ms)
        std::thread::sleep(std::time::Duration::from_millis(10)); // Faster for demo
    }
    
    // Final statistics
    println!("\n📈 Final Processing Statistics:");
    match FlutterTranscriberApi::get_processing_stats(instance_id.clone()) {
        Ok(stats) => {
            println!("  • Total windows processed: {}", stats.total_processed_windows);
            println!("  • Successful transcriptions: {}", stats.successful_transcriptions);
            println!("  • Success rate: {:.1}%", stats.success_rate_percent);
            println!("  • Average processing time: {:.1}ms", stats.average_processing_time_ms);
            println!("  • Real-time factor: {:.1}x", stats.real_time_factor);
            println!("  • Buffer overflows: {}", stats.buffer_overflows);
        }
        Err(e) => eprintln!("❌ Failed to get stats: {}", e),
    }
    
    // Cleanup
    println!("\n🧹 Cleaning up...");
    match FlutterTranscriberApi::destroy_transcriber(instance_id) {
        Ok(msg) => println!("{}", msg),
        Err(e) => eprintln!("❌ Cleanup failed: {}", e),
    }
    
    println!("\n✅ Flutter API Demo completed successfully!");
    println!("🎯 Ready for Flutter integration with Record dependency");
    
    Ok(())
}

/// Example demonstrating multi-instance transcriber management
#[allow(dead_code)]
fn multi_instance_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Multi-Instance Transcriber Demo");
    println!("==================================");
    
    // Create multiple transcribers for different purposes
    let instances = vec![
        ("murajaah_transcriber", "murajaah"),
        ("fast_transcriber", "fast"),
        ("general_transcriber", "arabic"),
    ];
    
    for (id, config_type) in &instances {
        let result = match *config_type {
            "murajaah" => FlutterTranscriberApi::create_murajaah_transcriber(
                id.to_string(), 
                "ggml-tiny.bin".to_string()
            ),
            "fast" => FlutterTranscriberApi::create_fast_transcriber(
                id.to_string(), 
                "ggml-tiny.bin".to_string()
            ),
            "arabic" => FlutterTranscriberApi::create_arabic_transcriber(
                id.to_string(), 
                "ggml-tiny.bin".to_string()
            ),
            _ => continue,
        };
        
        match result {
            Ok(msg) => println!("✅ {}", msg),
            Err(e) => println!("❌ Failed to create {}: {}", id, e),
        }
    }
    
    // List all transcribers
    let active_transcribers = FlutterTranscriberApi::list_transcribers();
    println!("\n📋 Active transcribers: {:?}", active_transcribers);
    
    // Health check all
    for id in &active_transcribers {
        match FlutterTranscriberApi::health_check(id.clone()) {
            Ok(health) => println!("{}", health),
            Err(e) => println!("❌ Health check failed for {}: {}", id, e),
        }
    }
    
    // Cleanup all
    println!("\n🧹 Cleaning up all transcribers...");
    let cleanup_result = cleanup_all_transcribers();
    println!("{}", cleanup_result);
    
    Ok(())
}
