//! Audio utilities for examples

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Load and convert a WAV file to the format expected by Whisper (f32, 16kHz, mono)
pub fn load_wav_file(path: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    if !Path::new(path).exists() {
        return Err(format!("Audio file not found: {}", path).into());
    }

    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Parse WAV file
    let mut reader = hound::WavReader::new(std::io::Cursor::new(buffer))?;
    let spec = reader.spec();

    println!("Audio specs: {} channels, {}Hz sample rate", spec.channels, spec.sample_rate);

    // Convert to f32 mono at 16kHz (Whisper expects this format)
    let mut audio_data = Vec::new();

    match spec.sample_format {
        hound::SampleFormat::Int => {
            for sample in reader.samples::<i32>() {
                let sample = sample? as f32 / std::i32::MAX as f32;
                audio_data.push(sample);
            }
        },
        hound::SampleFormat::Float => {
            for sample in reader.samples::<f32>() {
                audio_data.push(sample?);
            }
        },
    }

    // Convert stereo to mono if needed
    if spec.channels == 2 {
        let mut mono_data = Vec::with_capacity(audio_data.len() / 2);
        for i in (0..audio_data.len()).step_by(2) {
            if i + 1 < audio_data.len() {
                mono_data.push((audio_data[i] + audio_data[i + 1]) / 2.0);
            } else {
                mono_data.push(audio_data[i]);
            }
        }
        audio_data = mono_data;
        println!("Converted stereo to mono");
    }

    // Resample to 16kHz if needed
    if spec.sample_rate != 16000 {
        // Simple resampling (for better quality, use a dedicated resampling library)
        let ratio = 16000.0 / spec.sample_rate as f32;
        let new_len = (audio_data.len() as f32 * ratio) as usize;
        let mut resampled = Vec::with_capacity(new_len);

        for i in 0..new_len {
            let src_idx = (i as f32 / ratio) as usize;
            if src_idx < audio_data.len() {
                resampled.push(audio_data[src_idx]);
            } else {
                break;
            }
        }

        audio_data = resampled;
        println!("Resampled audio from {}Hz to 16000Hz", spec.sample_rate);
    }

    Ok(audio_data)
}

/// Normalize audio volume
pub fn normalize_audio(audio: &mut [f32]) {
    if audio.is_empty() {
        return;
    }

    // Find the maximum absolute amplitude
    let max_amplitude = audio.iter().fold(0.0f32, |max, &sample| {
        max.max(sample.abs())
    });

    if max_amplitude > 0.0 && max_amplitude != 1.0 {
        // Scale all samples
        let scale = 1.0 / max_amplitude;
        for sample in audio.iter_mut() {
            *sample *= scale;
        }
        println!("Normalized audio (max amplitude: {})", max_amplitude);
    }
}

/// Trim silence from the beginning and end of audio
pub fn trim_silence(audio: &[f32], threshold: f32) -> Vec<f32> {
    if audio.is_empty() {
        return Vec::new();
    }

    // Find start index (first sample above threshold)
    let mut start_idx = 0;
    while start_idx < audio.len() && audio[start_idx].abs() < threshold {
        start_idx += 1;
    }

    // Find end index (last sample above threshold)
    let mut end_idx = audio.len() - 1;
    while end_idx > start_idx && audio[end_idx].abs() < threshold {
        end_idx -= 1;
    }

    // Extract the non-silent portion
    let trimmed = audio[start_idx..=end_idx].to_vec();
    println!("Trimmed silence: {} -> {} samples", audio.len(), trimmed.len());

    trimmed
}
