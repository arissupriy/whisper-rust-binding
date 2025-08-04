use std::collections::VecDeque;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use crate::{init_whisper, process_audio, free_whisper};

#[derive(Debug, Clone)]
pub struct TranscriptionSegment {
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: f64,
    pub words: Vec<WordSegment>,
}

#[derive(Debug, Clone)]
pub struct WordSegment {
    pub word: String,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: f64,
    pub validated: bool,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub original_word: String,
    pub expected_word: String,
    pub is_match: bool,
    pub confidence: f64,
    pub suggestion: Option<String>,
}

pub struct RealTimeTranscriber {
    // Audio buffer with overlap management
    audio_buffer: Arc<Mutex<VecDeque<f32>>>,
    
    // Whisper instance
    whisper_instance: Option<i32>,
    
    // Configuration
    sample_rate: usize,
    window_duration: f64,
    overlap_duration: f64,
    
    // Processing state
    last_processed_time: Arc<Mutex<f64>>,
    
    // Channels for communication
    transcription_sender: Option<mpsc::Sender<TranscriptionSegment>>,
    validation_sender: Option<mpsc::Sender<ValidationResult>>,
    
    // Processing thread handles
    processing_handle: Option<thread::JoinHandle<()>>,
    
    // Buffer management
    max_buffer_duration: f64,
}

impl RealTimeTranscriber {
    pub fn new(
        model_path: &str,
        sample_rate: usize,
        window_duration: f64,
        overlap_duration: f64,
        max_buffer_duration: f64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        println!("🎤 Initializing Real-Time Transcriber");
        println!("   - Sample rate: {}Hz", sample_rate);
        println!("   - Window: {:.1}s", window_duration);
        println!("   - Overlap: {:.1}s", overlap_duration);
        println!("   - Max buffer: {:.1}s", max_buffer_duration);
        
        // Initialize Whisper
        let whisper_instance = init_whisper(model_path)?;
        println!("   ✅ Whisper model loaded (ID: {})", whisper_instance);
        
        let max_buffer_samples = (sample_rate as f64 * max_buffer_duration) as usize;
        
        Ok(RealTimeTranscriber {
            audio_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(max_buffer_samples))),
            whisper_instance: Some(whisper_instance),
            sample_rate,
            window_duration,
            overlap_duration,
            last_processed_time: Arc::new(Mutex::new(0.0)),
            transcription_sender: None,
            validation_sender: None,
            processing_handle: None,
            max_buffer_duration,
        })
    }
    
    /// Add audio data from Flutter Record (called continuously)
    pub fn add_audio_chunk(&mut self, audio_data: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = self.audio_buffer.lock().unwrap();
        
        // Add new samples
        for &sample in audio_data {
            buffer.push_back(sample);
        }
        
        // Maintain buffer size - remove old samples if buffer is too long
        let max_samples = (self.sample_rate as f64 * self.max_buffer_duration) as usize;
        while buffer.len() > max_samples {
            buffer.pop_front();
        }
        
        Ok(())
    }
    
    /// Get current buffer duration in seconds
    pub fn get_buffer_duration(&self) -> f64 {
        let buffer = self.audio_buffer.lock().unwrap();
        buffer.len() as f64 / self.sample_rate as f64
    }
    
    /// Start real-time processing with callbacks
    pub fn start_processing<F, V>(
        &mut self,
        mut transcription_callback: F,
        mut validation_callback: V,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(TranscriptionSegment) + Send + 'static,
        V: FnMut(ValidationResult) + Send + 'static,
    {
        if self.processing_handle.is_some() {
            return Err("Processing already started".into());
        }
        
        let buffer_clone = Arc::clone(&self.audio_buffer);
        let last_processed_clone = Arc::clone(&self.last_processed_time);
        let whisper_instance = self.whisper_instance.unwrap();
        let sample_rate = self.sample_rate;
        let window_duration = self.window_duration;
        let overlap_duration = self.overlap_duration;
        
        // Create communication channels
        let (tx_transcription, rx_transcription) = mpsc::channel();
        let (tx_validation, rx_validation) = mpsc::channel();
        
        self.transcription_sender = Some(tx_transcription);
        self.validation_sender = Some(tx_validation);
        
        // Start processing thread
        let processing_handle = thread::spawn(move || {
            Self::processing_loop(
                buffer_clone,
                last_processed_clone,
                whisper_instance,
                sample_rate,
                window_duration,
                overlap_duration,
            );
        });
        
        // Start callback threads
        thread::spawn(move || {
            while let Ok(segment) = rx_transcription.recv() {
                transcription_callback(segment);
            }
        });
        
        thread::spawn(move || {
            while let Ok(result) = rx_validation.recv() {
                validation_callback(result);
            }
        });
        
        self.processing_handle = Some(processing_handle);
        
        println!("🚀 Real-time processing started!");
        Ok(())
    }
    
    /// Main processing loop
    fn processing_loop(
        buffer: Arc<Mutex<VecDeque<f32>>>,
        last_processed_time: Arc<Mutex<f64>>,
        whisper_instance: i32,
        sample_rate: usize,
        window_duration: f64,
        overlap_duration: f64,
    ) {
        let hop_duration = window_duration - overlap_duration;
        let window_samples = (sample_rate as f64 * window_duration) as usize;
        let hop_samples = (sample_rate as f64 * hop_duration) as usize;
        
        println!("📊 Processing configuration:");
        println!("   - Window samples: {}", window_samples);
        println!("   - Hop samples: {}", hop_samples);
        println!("   - Hop duration: {:.1}s", hop_duration);
        
        loop {
            let current_time = {
                let buffer_guard = buffer.lock().unwrap();
                buffer_guard.len() as f64 / sample_rate as f64
            };
            
            let last_processed = {
                let last_guard = last_processed_time.lock().unwrap();
                *last_guard
            };
            
            // Check if we have enough data for next window
            if current_time - last_processed >= hop_duration && current_time >= window_duration {
                let audio_window = {
                    let buffer_guard = buffer.lock().unwrap();
                    if buffer_guard.len() >= window_samples {
                        // Extract latest window
                        let start_idx = buffer_guard.len() - window_samples;
                        buffer_guard.iter().skip(start_idx).cloned().collect::<Vec<f32>>()
                    } else {
                        continue;
                    }
                };
                
                let window_start_time = current_time - window_duration;
                let window_end_time = current_time;
                
                println!("🎬 Processing window [{:.1}s - {:.1}s]", window_start_time, window_end_time);
                
                // Process with Whisper
                let process_start = Instant::now();
                match process_audio(whisper_instance, &audio_window, Some("ar")) {
                    Ok(result) => {
                        let process_time = process_start.elapsed();
                        let rtf = process_time.as_secs_f64() / window_duration;
                        
                        if !result.trim().is_empty() {
                            let combined_text = result.trim();
                            
                            println!("   ✅ Transcribed: '{}' ({:.3}s, {:.1}x RT)", 
                                    combined_text, process_time.as_secs_f64(), 1.0 / rtf);
                            
                            // TODO: Send to transcription callback
                            // TODO: Send individual words for validation
                            
                        } else {
                            println!("   🔇 Silent window");
                        }
                    }
                    Err(e) => {
                        println!("   ❌ Transcription failed: {}", e);
                    }
                }
                
                // Update last processed time
                {
                    let mut last_guard = last_processed_time.lock().unwrap();
                    *last_guard = current_time - overlap_duration;
                }
            }
            
            // Sleep briefly to avoid busy waiting
            thread::sleep(Duration::from_millis(50));
        }
    }
    
    /// Validate transcribed text against expected content
    pub fn validate_text(&self, transcribed: &str, expected: &str) -> ValidationResult {
        // Simple word-level validation (can be enhanced with fuzzy matching)
        let _transcribed_words: Vec<&str> = transcribed.split_whitespace().collect();
        let _expected_words: Vec<&str> = expected.split_whitespace().collect();
        
        // For now, simple exact match
        let is_match = transcribed.trim() == expected.trim();
        let confidence = if is_match { 1.0 } else { 0.0 };
        
        ValidationResult {
            original_word: transcribed.to_string(),
            expected_word: expected.to_string(),
            is_match,
            confidence,
            suggestion: if !is_match { Some(expected.to_string()) } else { None },
        }
    }
    
    /// Stop processing
    pub fn stop_processing(&mut self) {
        if let Some(_handle) = self.processing_handle.take() {
            // TODO: Implement graceful shutdown
            println!("⏹️ Stopping real-time processing...");
        }
    }
}

impl Drop for RealTimeTranscriber {
    fn drop(&mut self) {
        if let Some(instance_id) = self.whisper_instance.take() {
            let _ = free_whisper(instance_id);
            println!("🧹 Whisper instance cleaned up");
        }
    }
}

// Flutter-Rust Bridge compatible functions
pub fn create_realtime_transcriber(
    model_path: String,
    sample_rate: u32,
    window_duration: f64,
    overlap_duration: f64,
    max_buffer_duration: f64,
) -> Result<Box<RealTimeTranscriber>, String> {
    match RealTimeTranscriber::new(
        &model_path,
        sample_rate as usize,
        window_duration,
        overlap_duration,
        max_buffer_duration,
    ) {
        Ok(transcriber) => Ok(Box::new(transcriber)),
        Err(e) => Err(e.to_string()),
    }
}

pub fn add_audio_samples(
    transcriber: &mut RealTimeTranscriber,
    samples: Vec<f32>,
) -> Result<(), String> {
    transcriber.add_audio_chunk(&samples).map_err(|e| e.to_string())
}

pub fn get_buffer_duration_seconds(transcriber: &RealTimeTranscriber) -> f64 {
    transcriber.get_buffer_duration()
}

pub fn validate_transcription(
    transcriber: &RealTimeTranscriber,
    transcribed_text: String,
    expected_text: String,
) -> ValidationResult {
    transcriber.validate_text(&transcribed_text, &expected_text)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_buffer_management() {
        // Test audio buffer management
        let mut transcriber = RealTimeTranscriber::new(
            "test_model.bin",
            16000,
            2.0,
            0.5,
            5.0,
        ).unwrap();
        
        // Add some test data
        let test_samples = vec![0.0; 8000]; // 0.5 seconds at 16kHz
        transcriber.add_audio_chunk(&test_samples).unwrap();
        
        assert_eq!(transcriber.get_buffer_duration(), 0.5);
    }
    
    #[test]
    fn test_validation() {
        let transcriber = RealTimeTranscriber::new(
            "test_model.bin",
            16000,
            2.0,
            0.5,
            5.0,
        ).unwrap();
        
        let result = transcriber.validate_text("hello world", "hello world");
        assert!(result.is_match);
        assert_eq!(result.confidence, 1.0);
        
        let result2 = transcriber.validate_text("hello word", "hello world");
        assert!(!result2.is_match);
        assert_eq!(result2.confidence, 0.0);
    }
}
