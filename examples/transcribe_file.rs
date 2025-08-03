use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use whisper_rust_binding::{init_whisper, process_audio, get_model_info, free_whisper};

mod audio_utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure logging
    env_logger::init();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} <model_file> <audio_file> [language]", args[0]);
        println!("  model_file: Path to the Whisper model file (e.g., ggml-tiny.bin)");
        println!("  audio_file: Path to the audio file (WAV or MP3)");
        println!("  language: Optional language code (e.g., 'en', 'ar') or omit for auto-detection");
        return Ok(());
    }

    let model_path = &args[1];
    let audio_path = &args[2];
    let language = args.get(3).map(|s| s.as_str());

    // Verify model file exists
    if !Path::new(model_path).exists() {
        return Err(format!("Model file not found: {}", model_path).into());
    }

    // Verify audio file exists
    if !Path::new(audio_path).exists() {
        return Err(format!("Audio file not found: {}", audio_path).into());
    }

    println!("Loading model from: {}", model_path);
    println!("Processing audio file: {}", audio_path);
    println!("Language: {}", language.unwrap_or("auto-detect"));

    // Handle MP3 files by converting to WAV first
    let wav_path = if audio_path.to_lowercase().ends_with(".mp3") {
        convert_mp3_to_wav(audio_path)?
    } else {
        PathBuf::from(audio_path)
    };

    // Initialize whisper with the model
    let instance_id = init_whisper(model_path)?;
    println!("Model loaded successfully! Instance ID: {}", instance_id);

    // Get model info
    let model_info = get_model_info(instance_id)?;
    println!("Model info: {}", model_info);

    // Load audio data
    let mut audio_data = audio_utils::load_wav_file(wav_path.to_str().unwrap())?;
    println!("Loaded audio file with {} samples", audio_data.len());

    // Normalize audio volume
    audio_utils::normalize_audio(&mut audio_data);

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

    // Remove temporary WAV file if we converted from MP3
    if audio_path.to_lowercase().ends_with(".mp3") {
        std::fs::remove_file(wav_path)?;
    }

    Ok(())
}

/// Convert MP3 to WAV format using ffmpeg
fn convert_mp3_to_wav(mp3_path: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let wav_path = PathBuf::from(format!("{}.wav", mp3_path));

    // Check if ffmpeg is available
    if Command::new("ffmpeg").arg("-version").output().is_err() {
        return Err("ffmpeg not found. Please install ffmpeg to process MP3 files.".into());
    }

    println!("Converting MP3 to WAV format...");

    // Convert MP3 to WAV (16kHz, mono)
    let output = Command::new("ffmpeg")
        .args([
            "-y", // Overwrite output files without asking
            "-i", mp3_path,
            "-ar", "16000", // Sample rate: 16kHz
            "-ac", "1",     // Channels: mono
            "-f", "wav",    // Format: WAV
            wav_path.to_str().unwrap()
        ])
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to convert MP3 to WAV: {}", error).into());
    }

    println!("MP3 converted to WAV successfully");
    Ok(wav_path)
}
