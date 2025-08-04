use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::fs;
use std::path::Path;
use std::process::Command;
use crate::{init_whisper, free_whisper, WhisperError};

/// Production-ready real-time transcriber for Flutter integration
#[derive(Debug)]
pub struct FlutterTranscriber {
    // Audio buffer management
    audio_buffer: Arc<Mutex<VecDeque<f32>>>,
    
    // Configuration
    sample_rate: u32,
    window_duration_ms: u32,
    overlap_duration_ms: u32,
    chunk_size_ms: u32,
    max_buffer_duration_ms: u32,
    
    // Processing state
    last_processed_samples: Arc<Mutex<usize>>,
    is_processing: Arc<Mutex<bool>>,
    
    // Model configuration
    model_path: String,
    language: String,
    
    // Temporary files management
    temp_dir: String,
    
    // Performance monitoring
    processing_stats: Arc<Mutex<ProcessingStats>>,
}

#[derive(Debug, Clone)]
pub struct ProcessingStats {
    pub total_processed_windows: u64,
    pub successful_transcriptions: u64,
    pub average_processing_time_ms: f64,
    pub real_time_factor: f64,
    pub buffer_overflows: u64,
    pub last_processing_time: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    pub text: String,
    pub start_time_ms: u64,
    pub end_time_ms: u64,
    pub confidence: f64,
    pub words: Vec<WordResult>,
    pub processing_time_ms: u64,
    pub is_real_time: bool,
}

#[derive(Debug, Clone)]
pub struct WordResult {
    pub word: String,
    pub start_time_ms: u64,
    pub end_time_ms: u64,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub transcribed_word: String,
    pub expected_word: String,
    pub is_match: bool,
    pub similarity_score: f64,
    pub suggestion: Option<String>,
    pub validation_type: ValidationType,
}

#[derive(Debug, Clone)]
pub enum ValidationType {
    ExactMatch,
    FuzzyMatch,
    PhoneticMatch,
    NoMatch,
}

#[derive(Debug, Clone)]
pub struct BufferStatus {
    pub current_duration_ms: u64,
    pub buffer_usage_percent: f64,
    pub is_ready_for_processing: bool,
    pub samples_count: usize,
    pub last_chunk_time: Option<SystemTime>,
}

impl Default for ProcessingStats {
    fn default() -> Self {
        Self {
            total_processed_windows: 0,
            successful_transcriptions: 0,
            average_processing_time_ms: 0.0,
            real_time_factor: 0.0,
            buffer_overflows: 0,
            last_processing_time: None,
        }
    }
}

impl FlutterTranscriber {
    /// Create new production-ready transcriber
    pub fn new(
        model_path: String,
        language: String,
        sample_rate: u32,
        window_duration_ms: u32,
        overlap_duration_ms: u32,
        chunk_size_ms: u32,
    ) -> Result<Self, WhisperError> {
        // Validate parameters
        if overlap_duration_ms >= window_duration_ms {
            return Err(WhisperError::InvalidParameter(
                "Overlap duration must be less than window duration".to_string()
            ));
        }
        
        if chunk_size_ms > window_duration_ms / 4 {
            return Err(WhisperError::InvalidParameter(
                "Chunk size should be at most 1/4 of window duration".to_string()
            ));
        }
        
        // Validate model file exists
        if !Path::new(&model_path).exists() {
            return Err(WhisperError::ModelInitError(
                format!("Model file not found: {}", model_path)
            ));
        }
        
        // Test model loading
        let test_instance = init_whisper(&model_path)?;
        free_whisper(test_instance)?;
        
        let max_buffer_duration_ms = window_duration_ms * 5; // 5x window size
        let max_buffer_samples = (sample_rate as u64 * max_buffer_duration_ms as u64 / 1000) as usize;
        
        // Create temporary directory
        let temp_dir = format!("/tmp/flutter_transcriber_{}", 
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
        fs::create_dir_all(&temp_dir).map_err(|e| 
            WhisperError::ProcessingError(format!("Failed to create temp dir: {}", e)))?;
        
        println!("🎤 Flutter Transcriber initialized:");
        println!("   - Model: {}", model_path);
        println!("   - Language: {}", language);
        println!("   - Sample rate: {}Hz", sample_rate);
        println!("   - Window: {}ms (overlap: {}ms)", window_duration_ms, overlap_duration_ms);
        println!("   - Chunk size: {}ms", chunk_size_ms);
        println!("   - Max buffer: {}ms", max_buffer_duration_ms);
        println!("   - Temp dir: {}", temp_dir);
        
        Ok(FlutterTranscriber {
            audio_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(max_buffer_samples))),
            sample_rate,
            window_duration_ms,
            overlap_duration_ms,
            chunk_size_ms,
            max_buffer_duration_ms,
            last_processed_samples: Arc::new(Mutex::new(0)),
            is_processing: Arc::new(Mutex::new(false)),
            model_path,
            language,
            temp_dir,
            processing_stats: Arc::new(Mutex::new(ProcessingStats::default())),
        })
    }
    
    /// Add audio chunk from Flutter Record (call this every ~50ms)
    pub fn add_audio_chunk(&self, audio_data: &[f32]) -> Result<BufferStatus, WhisperError> {
        let mut buffer = self.audio_buffer.lock().map_err(|_| 
            WhisperError::ProcessingError("Buffer lock failed".to_string()))?;
        
        // Add new samples
        for &sample in audio_data {
            buffer.push_back(sample);
        }
        
        // Prevent buffer overflow
        let max_samples = (self.sample_rate as u64 * self.max_buffer_duration_ms as u64 / 1000) as usize;
        let mut overflow_count = 0;
        while buffer.len() > max_samples {
            buffer.pop_front();
            overflow_count += 1;
        }
        
        if overflow_count > 0 {
            let mut stats = self.processing_stats.lock().unwrap();
            stats.buffer_overflows += 1;
        }
        
        let current_duration_ms = (buffer.len() as u64 * 1000) / self.sample_rate as u64;
        let buffer_usage = buffer.len() as f64 / max_samples as f64;
        let is_ready = current_duration_ms >= self.window_duration_ms as u64;
        
        Ok(BufferStatus {
            current_duration_ms,
            buffer_usage_percent: buffer_usage * 100.0,
            is_ready_for_processing: is_ready,
            samples_count: buffer.len(),
            last_chunk_time: Some(SystemTime::now()),
        })
    }
    
    /// Process audio if ready (call this regularly from Flutter)
    pub fn process_if_ready(&self) -> Result<Option<TranscriptionResult>, WhisperError> {
        // Check if processing is already in progress
        {
            let is_processing = self.is_processing.lock().map_err(|_| 
                WhisperError::ProcessingError("Processing lock failed".to_string()))?;
            if *is_processing {
                return Ok(None); // Still processing previous window
            }
        }
        
        // Check if we have enough data
        let buffer_status = {
            let buffer = self.audio_buffer.lock().map_err(|_| 
                WhisperError::ProcessingError("Buffer lock failed".to_string()))?;
            
            let current_samples = buffer.len();
            let required_samples = (self.sample_rate as u64 * self.window_duration_ms as u64 / 1000) as usize;
            
            current_samples >= required_samples
        };
        
        if !buffer_status {
            return Ok(None); // Not enough data yet
        }
        
        // Check if it's time to process (hop duration)
        let should_process = {
            let last_processed = *self.last_processed_samples.lock().unwrap();
            let current_samples = {
                let buffer = self.audio_buffer.lock().unwrap();
                buffer.len()
            };
            
            let hop_samples = (self.sample_rate as u64 * (self.window_duration_ms - self.overlap_duration_ms) as u64 / 1000) as usize;
            
            current_samples >= last_processed + hop_samples
        };
        
        if !should_process {
            return Ok(None); // Not time for next window yet
        }
        
        // Mark as processing
        {
            let mut is_processing = self.is_processing.lock().unwrap();
            *is_processing = true;
        }
        
        // Process the window
        let result = self.process_current_window();
        
        // Mark as not processing
        {
            let mut is_processing = self.is_processing.lock().unwrap();
            *is_processing = false;
        }
        
        result
    }
    
    /// Internal method to process current window
    fn process_current_window(&self) -> Result<Option<TranscriptionResult>, WhisperError> {
        let process_start = Instant::now();
        
        // Extract window from buffer
        let window_samples = {
            let buffer = self.audio_buffer.lock().unwrap();
            let window_size = (self.sample_rate as u64 * self.window_duration_ms as u64 / 1000) as usize;
            
            if buffer.len() < window_size {
                return Ok(None);
            }
            
            // Extract latest window
            let start_idx = buffer.len() - window_size;
            buffer.iter().skip(start_idx).cloned().collect::<Vec<f32>>()
        };
        
        // Create temporary WAV file
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let temp_file = format!("{}/window_{}.wav", self.temp_dir, timestamp);
        
        // Write audio to temporary file
        self.write_wav_file(&temp_file, &window_samples)?;
        
        // Process with external transcribe_file (most stable approach)
        let transcription_result = self.transcribe_file(&temp_file)?;
        
        // Clean up temporary file
        let _ = fs::remove_file(&temp_file);
        
        // Update processing stats
        let processing_time = process_start.elapsed();
        self.update_stats(processing_time, transcription_result.is_some());
        
        // Update last processed samples
        {
            let mut last_processed = self.last_processed_samples.lock().unwrap();
            let buffer = self.audio_buffer.lock().unwrap();
            *last_processed = buffer.len();
        }
        
        if let Some(mut result) = transcription_result {
            result.processing_time_ms = processing_time.as_millis() as u64;
            result.is_real_time = processing_time.as_millis() < self.window_duration_ms as u128;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
    
    /// Write audio samples to WAV file
    fn write_wav_file(&self, file_path: &str, samples: &[f32]) -> Result<(), WhisperError> {
        use std::fs::File;
        use std::io::{Write, BufWriter};
        
        let mut file = BufWriter::new(File::create(file_path).map_err(|e| 
            WhisperError::ProcessingError(format!("Failed to create WAV file: {}", e)))?);
        
        // WAV header
        let num_samples = samples.len() as u32;
        let byte_rate = self.sample_rate * 2; // 16-bit mono
        let data_size = num_samples * 2;
        let file_size = data_size + 36;
        
        // RIFF header
        file.write_all(b"RIFF").unwrap();
        file.write_all(&file_size.to_le_bytes()).unwrap();
        file.write_all(b"WAVE").unwrap();
        
        // fmt chunk
        file.write_all(b"fmt ").unwrap();
        file.write_all(&16u32.to_le_bytes()).unwrap(); // chunk size
        file.write_all(&1u16.to_le_bytes()).unwrap(); // audio format (PCM)
        file.write_all(&1u16.to_le_bytes()).unwrap(); // num channels
        file.write_all(&self.sample_rate.to_le_bytes()).unwrap(); // sample rate
        file.write_all(&byte_rate.to_le_bytes()).unwrap(); // byte rate
        file.write_all(&2u16.to_le_bytes()).unwrap(); // block align
        file.write_all(&16u16.to_le_bytes()).unwrap(); // bits per sample
        
        // data chunk
        file.write_all(b"data").unwrap();
        file.write_all(&data_size.to_le_bytes()).unwrap();
        
        // Convert f32 samples to i16 and write
        for sample in samples {
            let i16_sample = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
            file.write_all(&i16_sample.to_le_bytes()).unwrap();
        }
        
        Ok(())
    }
    
    /// Transcribe audio file using external process (most stable)
    fn transcribe_file(&self, file_path: &str) -> Result<Option<TranscriptionResult>, WhisperError> {
        let output = Command::new("./target/debug/examples/transcribe_file")
            .args(&[&self.model_path, file_path, &self.language])
            .output()
            .map_err(|e| WhisperError::ProcessingError(format!("Transcription failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(WhisperError::ProcessingError(
                format!("Transcription process failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        self.parse_transcription_output(&output_str)
    }
    
    /// Parse transcription output
    fn parse_transcription_output(&self, output: &str) -> Result<Option<TranscriptionResult>, WhisperError> {
        // Find transcription markers
        if let Some(start_marker) = output.find("-------------------------------------------") {
            if let Some(content_start) = output[start_marker..].find('\n') {
                let content_section = &output[start_marker + content_start + 1..];
                if let Some(end_marker) = content_section.find("-------------------------------------------") {
                    let transcription = content_section[..end_marker].trim();
                    
                    if !transcription.is_empty() {
                        // Extract words (simple implementation, can be enhanced)
                        let words: Vec<WordResult> = transcription
                            .split_whitespace()
                            .enumerate()
                            .map(|(i, word)| WordResult {
                                word: word.to_string(),
                                start_time_ms: (i as u64 * 500), // Rough estimate
                                end_time_ms: ((i + 1) as u64 * 500),
                                confidence: 0.95, // Default confidence
                            })
                            .collect();
                        
                        return Ok(Some(TranscriptionResult {
                            text: transcription.to_string(),
                            start_time_ms: 0,
                            end_time_ms: self.window_duration_ms as u64,
                            confidence: 0.95,
                            words,
                            processing_time_ms: 0, // Will be set by caller
                            is_real_time: true, // Will be set by caller
                        }));
                    }
                }
            }
        }
        
        Ok(None) // No transcription found
    }
    
    /// Validate transcribed text against expected content
    pub fn validate_transcription(&self, transcribed: &str, expected: &str) -> ValidationResult {
        let transcribed_clean = self.clean_arabic_text(transcribed);
        let expected_clean = self.clean_arabic_text(expected);
        
        // Exact match
        if transcribed_clean == expected_clean {
            return ValidationResult {
                transcribed_word: transcribed.to_string(),
                expected_word: expected.to_string(),
                is_match: true,
                similarity_score: 1.0,
                suggestion: None,
                validation_type: ValidationType::ExactMatch,
            };
        }
        
        // Fuzzy match (can be enhanced with proper Arabic fuzzy matching)
        let similarity = self.calculate_similarity(&transcribed_clean, &expected_clean);
        
        let validation_type = if similarity > 0.8 {
            ValidationType::FuzzyMatch
        } else if similarity > 0.6 {
            ValidationType::PhoneticMatch
        } else {
            ValidationType::NoMatch
        };
        
        ValidationResult {
            transcribed_word: transcribed.to_string(),
            expected_word: expected.to_string(),
            is_match: similarity > 0.8,
            similarity_score: similarity,
            suggestion: if similarity < 0.8 { Some(expected.to_string()) } else { None },
            validation_type,
        }
    }
    
    /// Get current processing statistics
    pub fn get_stats(&self) -> ProcessingStats {
        self.processing_stats.lock().unwrap().clone()
    }
    
    /// Get current buffer status
    pub fn get_buffer_status(&self) -> BufferStatus {
        let buffer = self.audio_buffer.lock().unwrap();
        let current_duration_ms = (buffer.len() as u64 * 1000) / self.sample_rate as u64;
        let max_samples = (self.sample_rate as u64 * self.max_buffer_duration_ms as u64 / 1000) as usize;
        let buffer_usage = buffer.len() as f64 / max_samples as f64;
        
        BufferStatus {
            current_duration_ms,
            buffer_usage_percent: buffer_usage * 100.0,
            is_ready_for_processing: current_duration_ms >= self.window_duration_ms as u64,
            samples_count: buffer.len(),
            last_chunk_time: Some(SystemTime::now()),
        }
    }
    
    /// Clean up resources
    pub fn cleanup(&self) -> Result<(), WhisperError> {
        // Remove temporary directory
        if Path::new(&self.temp_dir).exists() {
            fs::remove_dir_all(&self.temp_dir).map_err(|e| 
                WhisperError::ProcessingError(format!("Cleanup failed: {}", e)))?;
        }
        
        println!("🧹 Flutter Transcriber cleaned up");
        Ok(())
    }
    
    // Helper methods
    fn clean_arabic_text(&self, text: &str) -> String {
        // Remove diacritics and normalize Arabic text
        text.chars()
            .filter(|c| !matches!(*c, '\u{064B}'..='\u{065F}' | '\u{0670}' | '\u{06D6}'..='\u{06ED}'))
            .collect::<String>()
            .trim()
            .to_lowercase()
    }
    
    fn calculate_similarity(&self, text1: &str, text2: &str) -> f64 {
        // Simple Levenshtein distance (can be enhanced)
        let len1 = text1.chars().count();
        let len2 = text2.chars().count();
        
        if len1 == 0 && len2 == 0 {
            return 1.0;
        }
        
        if len1 == 0 || len2 == 0 {
            return 0.0;
        }
        
        let max_len = len1.max(len2);
        let distance = self.levenshtein_distance(text1, text2);
        
        1.0 - (distance as f64 / max_len as f64)
    }
    
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }
        
        matrix[len1][len2]
    }
    
    fn update_stats(&self, processing_time: Duration, success: bool) {
        let mut stats = self.processing_stats.lock().unwrap();
        stats.total_processed_windows += 1;
        
        if success {
            stats.successful_transcriptions += 1;
        }
        
        let processing_time_ms = processing_time.as_millis() as f64;
        stats.average_processing_time_ms = (stats.average_processing_time_ms * (stats.total_processed_windows - 1) as f64 + processing_time_ms) / stats.total_processed_windows as f64;
        
        let window_duration_ms = self.window_duration_ms as f64;
        stats.real_time_factor = window_duration_ms / processing_time_ms;
        stats.last_processing_time = Some(Instant::now());
    }
}

impl Drop for FlutterTranscriber {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
