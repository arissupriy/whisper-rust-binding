use std::env;
use std::time::Duration;
use std::thread;
use whisper_rust_binding::realtime_transcriber::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <model_path> [audio_file]", args[0]);
        eprintln!("Example: {} ggml-tiny.bin output.wav", args[0]);
        return Ok(());
    }

    let model_path = &args[1];
    let audio_file = args.get(2);

    println!("🎤 Real-Time Transcription Demo for Flutter Integration");
    println!("=====================================================");
    println!("Model: {}", model_path);
    if let Some(file) = audio_file {
        println!("Audio file: {}", file);
    } else {
        println!("Mode: Live simulation");
    }
    println!();

    // Create real-time transcriber
    let mut transcriber = RealTimeTranscriber::new(
        model_path,
        16000,       // 16kHz sample rate
        2.0,         // 2 second windows
        0.5,         // 0.5 second overlap
        10.0,        // 10 second max buffer
    )?;

    println!("✅ Real-time transcriber created");

    if let Some(audio_path) = audio_file {
        // Simulate real-time audio streaming from file
        simulate_flutter_audio_stream(&mut transcriber, audio_path)?;
    } else {
        // Simulate live audio capture
        simulate_live_audio_capture(&mut transcriber)?;
    }

    Ok(())
}

fn simulate_flutter_audio_stream(
    transcriber: &mut RealTimeTranscriber,
    audio_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::{BufReader, Read};
    
    println!("📁 Loading audio file for simulation...");
    
    // Load WAV file
    let mut file = BufReader::new(File::open(audio_path)?);
    
    // Skip WAV header
    let mut header = [0u8; 44];
    file.read_exact(&mut header)?;
    
    // Read PCM data
    let mut pcm_data = Vec::new();
    file.read_to_end(&mut pcm_data)?;
    
    // Convert to f32 samples
    let mut all_samples = Vec::new();
    for chunk in pcm_data.chunks_exact(2) {
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        all_samples.push(sample as f32 / 32768.0);
    }
    
    let total_duration = all_samples.len() as f64 / 16000.0;
    println!("✅ Audio loaded: {:.2}s ({} samples)", total_duration, all_samples.len());
    
    // Start processing with callbacks
    let expected_text = "أَوْ قَصَيِّبٍ مِنَ السَّمَاءِ فِيهِ ظُلُمَاتٌ وَرَحْضٌ غَبَرْقٌ يَجْعَلُونَ أَصَابِعَهُمْ فِي آذَانِهِمْ مِنَ الصَّيَاةِ ادْحَبَرَ الْمَوْتِ وَاللَّهُ يُؤْخِيكُمْ بِالْكَافِرِينَ";
    let expected_clone = expected_text.to_string();
    
    transcriber.start_processing(
        // Transcription callback (send to Flutter)
        move |segment: TranscriptionSegment| {
            println!("📝 [Flutter] New transcription: '{}'", segment.text);
            println!("   Time: {:.1}s - {:.1}s", segment.start_time, segment.end_time);
            println!("   Confidence: {:.2}", segment.confidence);
            
            // Here you would send to Flutter via FRB
            // flutter_callback_transcription(segment);
        },
        
        // Validation callback (send validation results to Flutter)
        move |result: ValidationResult| {
            if result.is_match {
                println!("✅ [Flutter] Word validated: '{}'", result.original_word);
            } else {
                println!("❌ [Flutter] Word mismatch: '{}' (expected: '{}')", 
                        result.original_word, result.expected_word);
                if let Some(suggestion) = &result.suggestion {
                    println!("💡 [Flutter] Suggestion: '{}'", suggestion);
                }
            }
            
            // Here you would send to Flutter via FRB
            // flutter_callback_validation(result);
        },
    )?;
    
    // Simulate Flutter Record chunks (50ms chunks)
    let chunk_size = 800; // 50ms at 16kHz
    let chunk_duration = Duration::from_millis(50);
    
    println!("🚀 Starting audio stream simulation (50ms chunks)...");
    println!("{}", "=".repeat(60));
    
    for chunk in all_samples.chunks(chunk_size) {
        // Simulate Flutter Record sending audio chunks
        transcriber.add_audio_chunk(chunk)?;
        
        let buffer_duration = transcriber.get_buffer_duration();
        print!("\r📊 Buffer: {:.1}s", buffer_duration);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        
        // Simulate real-time delay (50ms chunks)
        thread::sleep(chunk_duration);
    }
    
    println!("\n📡 Audio stream completed!");
    
    // Let processing continue for a bit to catch up
    thread::sleep(Duration::from_secs(3));
    
    transcriber.stop_processing();
    
    Ok(())
}

fn simulate_live_audio_capture(
    transcriber: &mut RealTimeTranscriber,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎙️ Simulating live audio capture...");
    println!("   (In real Flutter app, this would be Flutter Record)");
    
    // Start processing
    transcriber.start_processing(
        |segment: TranscriptionSegment| {
            println!("📝 [Flutter] Live transcription: '{}'", segment.text);
        },
        |result: ValidationResult| {
            if result.is_match {
                println!("✅ [Flutter] Live validation: OK");
            } else {
                println!("❌ [Flutter] Live validation: FAIL");
            }
        },
    )?;
    
    // Simulate live audio chunks (silent for demo)
    for i in 0..100 {
        // Generate some test audio (sine wave)
        let frequency = 440.0; // A4 note
        let sample_rate = 16000.0;
        let chunk_duration = 0.05; // 50ms
        let chunk_samples = (sample_rate * chunk_duration) as usize;
        
        let mut chunk = Vec::with_capacity(chunk_samples);
        for j in 0..chunk_samples {
            let time = (i * chunk_samples + j) as f64 / sample_rate;
            let sample = 0.1 * (2.0 * std::f64::consts::PI * frequency * time).sin();
            chunk.push(sample as f32);
        }
        
        transcriber.add_audio_chunk(&chunk)?;
        
        let buffer_duration = transcriber.get_buffer_duration();
        print!("\r🎙️ Live capture: {:.1}s buffer", buffer_duration);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        
        thread::sleep(Duration::from_millis(50));
    }
    
    println!("\n🛑 Live capture stopped");
    transcriber.stop_processing();
    
    Ok(())
}

// Example Flutter-Rust Bridge functions that would be called from Flutter
#[allow(dead_code)]
pub fn flutter_add_audio_chunk(
    transcriber: &mut RealTimeTranscriber,
    audio_data: Vec<f32>,
) -> Result<String, String> {
    match transcriber.add_audio_chunk(&audio_data) {
        Ok(()) => {
            let duration = transcriber.get_buffer_duration();
            Ok(format!("Buffer: {:.1}s", duration))
        }
        Err(e) => Err(e.to_string()),
    }
}

#[allow(dead_code)]
pub fn flutter_validate_word(
    transcriber: &RealTimeTranscriber,
    spoken_word: String,
    expected_word: String,
) -> ValidationResult {
    transcriber.validate_text(&spoken_word, &expected_word)
}

#[allow(dead_code)]
pub fn flutter_get_buffer_status(transcriber: &RealTimeTranscriber) -> String {
    format!("Buffer duration: {:.1}s", transcriber.get_buffer_duration())
}
