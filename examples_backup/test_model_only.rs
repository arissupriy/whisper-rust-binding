use std::env;
use whisper_rust_binding::{init_whisper, process_audio, get_model_info, free_whisper};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <model_file> <audio_file>", args[0]);
        return Ok(());
    }

    let model_path = &args[1];
    let audio_path = &args[2];

    println!("Loading model from: {}", model_path);

    // Test just model loading first
    let instance_id = init_whisper(model_path)?;
    println!("Model loaded successfully! Instance ID: {}", instance_id);

    let model_info = get_model_info(instance_id)?;
    println!("Model info: {}", model_info);

    // Free resources immediately (don't process audio for now)
    free_whisper(instance_id)?;
    println!("Test completed successfully - model loading and freeing works");

    Ok(())
}
