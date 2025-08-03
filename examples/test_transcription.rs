use std::env;
use std::path::Path;
use whisper_rust_binding::{init_whisper, process_audio, get_model_info, free_whisper};

mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure logging
    env_logger::init();

    // Get model path from command line or use default
    let args: Vec<String> = env::args().collect();
    let model_path = if args.len() > 1 {
        &args[1]
    } else {
        "ggml-tiny.bin"
    };

    // Get audio file path from command line or use default
    let audio_path = if args.len() > 2 {
        &args[2]
    } else {
        "output.wav"
    };

    // Get language from command line or use None (auto-detect)
    let language = if args.len() > 3 {
        Some(args[3].as_str())
    } else {
        None
    };

    // Verify files exist
    if !Path::new(model_path).exists() {
        return Err(format!("Model file not found: {}", model_path).into());
    }

    if !Path::new(audio_path).exists() {
        return Err(format!("Audio file not found: {}", audio_path).into());
    }

    println!("Loading model from: {}", model_path);
    println!("Processing audio file: {}", audio_path);
    println!("Language: {}", language.unwrap_or("auto-detect"));

    // Initialize whisper with the model
    let instance_id = init_whisper(model_path)?;
    println!("Model loaded successfully! Instance ID: {}", instance_id);

    // Get model info
    let model_info = get_model_info(instance_id)?;
    println!("Model info: {}", model_info);

    // Load audio data
    let mut audio_data = common::audio_utils::load_wav_file(audio_path)?;
    println!("Loaded audio file with {} samples", audio_data.len());

    // Normalize audio volume
    common::audio_utils::normalize_audio(&mut audio_data);

    // Process audio
    println!("Processing audio...");
    let start = std::time::Instant::now();
    let transcript = process_audio(instance_id, &audio_data, language)?;
    let duration = start.elapsed();

    println!("\nTranscription completed in {:.2?}:", duration);
    println!("-------------------------------------------");
    println!("{}", transcript);
    println!("-------------------------------------------");

    // Free resources
    free_whisper(instance_id)?;
    println!("Resources freed successfully");

    Ok(())
}
