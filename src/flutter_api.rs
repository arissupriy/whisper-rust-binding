use crate::flutter_transcriber::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use once_cell::sync::Lazy;

// Global transcriber instances management
static TRANSCRIBER_INSTANCES: Lazy<Arc<Mutex<HashMap<String, FlutterTranscriber>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Flutter Rust Bridge API for production-ready real-time transcription
pub struct FlutterTranscriberApi;

// Flutter-compatible structs (must be simple for FRB)
#[derive(Debug, Clone)]
pub struct FrbTranscriptionResult {
    pub text: String,
    pub start_time_ms: u64,
    pub end_time_ms: u64,
    pub confidence: f64,
    pub processing_time_ms: u64,
    pub is_real_time: bool,
    pub word_count: u32,
}

#[derive(Debug, Clone)]
pub struct FrbValidationResult {
    pub transcribed_word: String,
    pub expected_word: String,
    pub is_match: bool,
    pub similarity_score: f64,
    pub suggestion: Option<String>,
    pub validation_type: String,
}

#[derive(Debug, Clone)]
pub struct FrbBufferStatus {
    pub current_duration_ms: u64,
    pub buffer_usage_percent: f64,
    pub is_ready_for_processing: bool,
    pub samples_count: u32,
}

#[derive(Debug, Clone)]
pub struct FrbProcessingStats {
    pub total_processed_windows: u64,
    pub successful_transcriptions: u64,
    pub success_rate_percent: f64,
    pub average_processing_time_ms: f64,
    pub real_time_factor: f64,
    pub buffer_overflows: u64,
}

#[derive(Debug, Clone)]
pub struct FrbTranscriberConfig {
    pub model_path: String,
    pub language: String,
    pub sample_rate: u32,
    pub window_duration_ms: u32,
    pub overlap_duration_ms: u32,
    pub chunk_size_ms: u32,
}

impl Default for FrbTranscriberConfig {
    fn default() -> Self {
        Self {
            model_path: "ggml-tiny.bin".to_string(),
            language: "ar".to_string(),
            sample_rate: 16000,
            window_duration_ms: 2000,
            overlap_duration_ms: 500,
            chunk_size_ms: 50,
        }
    }
}

impl FlutterTranscriberApi {
    /// Initialize a new transcriber instance
    pub fn create_transcriber(
        instance_id: String,
        config: FrbTranscriberConfig,
    ) -> Result<String, String> {
        match FlutterTranscriber::new(
            config.model_path,
            config.language,
            config.sample_rate,
            config.window_duration_ms,
            config.overlap_duration_ms,
            config.chunk_size_ms,
        ) {
            Ok(transcriber) => {
                let mut instances = TRANSCRIBER_INSTANCES.lock().unwrap();
                instances.insert(instance_id.clone(), transcriber);
                Ok(format!("✅ Transcriber '{}' created successfully", instance_id))
            }
            Err(e) => Err(format!("❌ Failed to create transcriber: {}", e)),
        }
    }
    
    /// Add audio chunk from Flutter Record
    pub fn add_audio_chunk(
        instance_id: String,
        audio_data: Vec<f32>,
    ) -> Result<FrbBufferStatus, String> {
        let instances = TRANSCRIBER_INSTANCES.lock().unwrap();
        
        if let Some(transcriber) = instances.get(&instance_id) {
            match transcriber.add_audio_chunk(&audio_data) {
                Ok(status) => Ok(FrbBufferStatus {
                    current_duration_ms: status.current_duration_ms,
                    buffer_usage_percent: status.buffer_usage_percent,
                    is_ready_for_processing: status.is_ready_for_processing,
                    samples_count: status.samples_count as u32,
                }),
                Err(e) => Err(format!("❌ Failed to add audio chunk: {}", e)),
            }
        } else {
            Err(format!("❌ Transcriber instance '{}' not found", instance_id))
        }
    }
    
    /// Process audio if ready and return transcription
    pub fn process_if_ready(instance_id: String) -> Result<Option<FrbTranscriptionResult>, String> {
        let instances = TRANSCRIBER_INSTANCES.lock().unwrap();
        
        if let Some(transcriber) = instances.get(&instance_id) {
            match transcriber.process_if_ready() {
                Ok(Some(result)) => Ok(Some(FrbTranscriptionResult {
                    text: result.text,
                    start_time_ms: result.start_time_ms,
                    end_time_ms: result.end_time_ms,
                    confidence: result.confidence,
                    processing_time_ms: result.processing_time_ms,
                    is_real_time: result.is_real_time,
                    word_count: result.words.len() as u32,
                })),
                Ok(None) => Ok(None),
                Err(e) => Err(format!("❌ Processing failed: {}", e)),
            }
        } else {
            Err(format!("❌ Transcriber instance '{}' not found", instance_id))
        }
    }
    
    /// Validate transcribed text against expected text
    pub fn validate_transcription(
        instance_id: String,
        transcribed_text: String,
        expected_text: String,
    ) -> Result<FrbValidationResult, String> {
        let instances = TRANSCRIBER_INSTANCES.lock().unwrap();
        
        if let Some(transcriber) = instances.get(&instance_id) {
            let result = transcriber.validate_transcription(&transcribed_text, &expected_text);
            
            Ok(FrbValidationResult {
                transcribed_word: result.transcribed_word,
                expected_word: result.expected_word,
                is_match: result.is_match,
                similarity_score: result.similarity_score,
                suggestion: result.suggestion,
                validation_type: format!("{:?}", result.validation_type),
            })
        } else {
            Err(format!("❌ Transcriber instance '{}' not found", instance_id))
        }
    }
    
    /// Get current buffer status
    pub fn get_buffer_status(instance_id: String) -> Result<FrbBufferStatus, String> {
        let instances = TRANSCRIBER_INSTANCES.lock().unwrap();
        
        if let Some(transcriber) = instances.get(&instance_id) {
            let status = transcriber.get_buffer_status();
            
            Ok(FrbBufferStatus {
                current_duration_ms: status.current_duration_ms,
                buffer_usage_percent: status.buffer_usage_percent,
                is_ready_for_processing: status.is_ready_for_processing,
                samples_count: status.samples_count as u32,
            })
        } else {
            Err(format!("❌ Transcriber instance '{}' not found", instance_id))
        }
    }
    
    /// Get processing statistics
    pub fn get_processing_stats(instance_id: String) -> Result<FrbProcessingStats, String> {
        let instances = TRANSCRIBER_INSTANCES.lock().unwrap();
        
        if let Some(transcriber) = instances.get(&instance_id) {
            let stats = transcriber.get_stats();
            let success_rate = if stats.total_processed_windows > 0 {
                (stats.successful_transcriptions as f64 / stats.total_processed_windows as f64) * 100.0
            } else {
                0.0
            };
            
            Ok(FrbProcessingStats {
                total_processed_windows: stats.total_processed_windows,
                successful_transcriptions: stats.successful_transcriptions,
                success_rate_percent: success_rate,
                average_processing_time_ms: stats.average_processing_time_ms,
                real_time_factor: stats.real_time_factor,
                buffer_overflows: stats.buffer_overflows,
            })
        } else {
            Err(format!("❌ Transcriber instance '{}' not found", instance_id))
        }
    }
    
    /// Remove transcriber instance and cleanup
    pub fn destroy_transcriber(instance_id: String) -> Result<String, String> {
        let mut instances = TRANSCRIBER_INSTANCES.lock().unwrap();
        
        if let Some(transcriber) = instances.remove(&instance_id) {
            match transcriber.cleanup() {
                Ok(()) => Ok(format!("✅ Transcriber '{}' destroyed successfully", instance_id)),
                Err(e) => Err(format!("⚠️ Transcriber destroyed but cleanup failed: {}", e)),
            }
        } else {
            Err(format!("❌ Transcriber instance '{}' not found", instance_id))
        }
    }
    
    /// List all active transcriber instances
    pub fn list_transcribers() -> Vec<String> {
        let instances = TRANSCRIBER_INSTANCES.lock().unwrap();
        instances.keys().cloned().collect()
    }
    
    /// Health check for transcriber instance
    pub fn health_check(instance_id: String) -> Result<String, String> {
        let instances = TRANSCRIBER_INSTANCES.lock().unwrap();
        
        if let Some(transcriber) = instances.get(&instance_id) {
            let buffer_status = transcriber.get_buffer_status();
            let stats = transcriber.get_stats();
            
            let health_info = format!(
                "✅ Transcriber '{}' is healthy\n  - Buffer: {:.1}ms ({:.1}% full)\n  - Processed: {} windows\n  - Success rate: {:.1}%\n  - Real-time factor: {:.1}x",
                instance_id,
                buffer_status.current_duration_ms,
                buffer_status.buffer_usage_percent,
                stats.total_processed_windows,
                if stats.total_processed_windows > 0 { 
                    (stats.successful_transcriptions as f64 / stats.total_processed_windows as f64) * 100.0 
                } else { 
                    0.0 
                },
                stats.real_time_factor
            );
            
            Ok(health_info)
        } else {
            Err(format!("❌ Transcriber instance '{}' not found", instance_id))
        }
    }
}

// Convenience functions for common configurations
impl FlutterTranscriberApi {
    /// Create transcriber with default configuration for Arabic
    pub fn create_arabic_transcriber(
        instance_id: String,
        model_path: String,
    ) -> Result<String, String> {
        let config = FrbTranscriberConfig {
            model_path,
            language: "ar".to_string(),
            ..Default::default()
        };
        
        Self::create_transcriber(instance_id, config)
    }
    
    /// Create transcriber optimized for murajaah (longer window, more overlap)
    pub fn create_murajaah_transcriber(
        instance_id: String,
        model_path: String,
    ) -> Result<String, String> {
        let config = FrbTranscriberConfig {
            model_path,
            language: "ar".to_string(),
            sample_rate: 16000,
            window_duration_ms: 3000,  // 3 seconds for better context
            overlap_duration_ms: 1000, // 1 second overlap
            chunk_size_ms: 50,
        };
        
        Self::create_transcriber(instance_id, config)
    }
    
    /// Create transcriber optimized for fast response (shorter window)
    pub fn create_fast_transcriber(
        instance_id: String,
        model_path: String,
    ) -> Result<String, String> {
        let config = FrbTranscriberConfig {
            model_path,
            language: "ar".to_string(),
            sample_rate: 16000,
            window_duration_ms: 1500,  // 1.5 seconds for faster response
            overlap_duration_ms: 300,  // 300ms overlap
            chunk_size_ms: 50,
        };
        
        Self::create_transcriber(instance_id, config)
    }
}

// Global cleanup function
pub fn cleanup_all_transcribers() -> String {
    let mut instances = TRANSCRIBER_INSTANCES.lock().unwrap();
    let count = instances.len();
    
    for (id, transcriber) in instances.drain() {
        let _ = transcriber.cleanup();
        println!("🧹 Cleaned up transcriber: {}", id);
    }
    
    format!("✅ Cleaned up {} transcriber instances", count)
}
