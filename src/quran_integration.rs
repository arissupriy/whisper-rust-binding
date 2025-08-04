use crate::flutter_api::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// External validation interface to communicate with quran_assistant_engine
/// This allows validation using Quran data from the other library
pub struct ExternalValidationInterface;

/// Callback function type for external validation
pub type ExternalValidationCallback = extern "C" fn(
    transcribed_text: *const c_char,
    ayah_id: i32,
    surah_id: i32,
) -> ValidationResponse;

/// C-compatible validation response from quran_assistant_engine
#[repr(C)]
pub struct ValidationResponse {
    pub is_valid: bool,
    pub similarity_score: f64,
    pub correct_text: *const c_char,
    pub word_count_match: i32,
    pub ayah_position: i32,
}

/// Enhanced Flutter API with external validation support
pub struct IntegratedFlutterApi {
    external_validator: Option<ExternalValidationCallback>,
}

static mut INTEGRATED_API: IntegratedFlutterApi = IntegratedFlutterApi {
    external_validator: None,
};

impl IntegratedFlutterApi {
    /// Register external validation callback from quran_assistant_engine
    pub fn register_external_validator(callback: ExternalValidationCallback) -> Result<String, String> {
        unsafe {
            INTEGRATED_API.external_validator = Some(callback);
        }
        Ok("✅ External validator registered successfully".to_string())
    }
    
    /// Enhanced transcription with Quran validation
    pub fn transcribe_with_quran_validation(
        instance_id: String,
        expected_ayah_id: i32,
        expected_surah_id: i32,
    ) -> Result<Option<FrbTranscriptionWithQuranValidation>, String> {
        // Get transcription from whisper
        let transcription_result = FlutterTranscriberApi::process_if_ready(instance_id.clone())?;
        
        if let Some(transcription) = transcription_result {
            // Validate using external Quran engine if available
            let quran_validation = unsafe {
                if let Some(validator) = INTEGRATED_API.external_validator {
                    let text_cstr = CString::new(transcription.text.clone())
                        .map_err(|e| format!("Failed to convert text: {}", e))?;
                    
                    let response = validator(text_cstr.as_ptr(), expected_ayah_id, expected_surah_id);
                    
                    Some(FrbQuranValidation {
                        is_valid: response.is_valid,
                        similarity_score: response.similarity_score,
                        correct_text: if response.correct_text.is_null() {
                            String::new()
                        } else {
                            CStr::from_ptr(response.correct_text)
                                .to_string_lossy()
                                .to_string()
                        },
                        word_count_match: response.word_count_match,
                        ayah_position: response.ayah_position,
                    })
                } else {
                    None
                }
            };
            
            Ok(Some(FrbTranscriptionWithQuranValidation {
                transcription: transcription,
                quran_validation: quran_validation,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Start real-time transcription with Quran context
    pub fn start_quran_session(
        instance_id: String,
        surah_id: i32,
        starting_ayah_id: i32,
        session_config: FrbQuranSessionConfig,
    ) -> Result<String, String> {
        // Create transcriber with Quran-optimized settings
        let config = FrbTranscriberConfig {
            model_path: session_config.model_path,
            language: "ar".to_string(),
            sample_rate: 16000,
            window_duration_ms: session_config.window_duration_ms,
            overlap_duration_ms: session_config.overlap_duration_ms,
            chunk_size_ms: 50,
        };
        
        FlutterTranscriberApi::create_transcriber(instance_id.clone(), config)?;
        
        // Store session context for validation
        // In a real implementation, you'd store this in a session manager
        println!("📖 Quran session started: Surah {} from Ayah {}", surah_id, starting_ayah_id);
        
        Ok(format!("✅ Quran session '{}' started for Surah {} from Ayah {}", 
            instance_id, surah_id, starting_ayah_id))
    }
    
    /// Get next expected ayah for progressive reading
    pub fn get_next_expected_ayah(
        current_surah_id: i32,
        current_ayah_id: i32,
    ) -> Result<FrbNextAyahInfo, String> {
        // This would typically call the quran_assistant_engine
        // For now, return mock data
        Ok(FrbNextAyahInfo {
            surah_id: current_surah_id,
            ayah_id: current_ayah_id + 1,
            expected_text: "بسم الله الرحمن الرحيم".to_string(),
            ayah_length: 19,
            estimated_duration_ms: 5000,
        })
    }
}

/// Flutter-compatible structs for Quran integration
#[derive(Debug, Clone)]
pub struct FrbTranscriptionWithQuranValidation {
    pub transcription: FrbTranscriptionResult,
    pub quran_validation: Option<FrbQuranValidation>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct FrbQuranValidation {
    pub is_valid: bool,
    pub similarity_score: f64,
    pub correct_text: String,
    pub word_count_match: i32,
    pub ayah_position: i32,
}

#[derive(Debug, Clone)]
pub struct FrbQuranSessionConfig {
    pub model_path: String,
    pub window_duration_ms: u32,
    pub overlap_duration_ms: u32,
    pub reading_speed_wpm: u32,
    pub strictness_level: u32, // 1=lenient, 5=strict
}

#[derive(Debug, Clone)]
pub struct FrbNextAyahInfo {
    pub surah_id: i32,
    pub ayah_id: i32,
    pub expected_text: String,
    pub ayah_length: i32,
    pub estimated_duration_ms: u32,
}

impl Default for FrbQuranSessionConfig {
    fn default() -> Self {
        Self {
            model_path: "ggml-tiny.bin".to_string(),
            window_duration_ms: 3000,
            overlap_duration_ms: 1000,
            reading_speed_wpm: 80, // Average Arabic reading speed
            strictness_level: 3,   // Medium strictness
        }
    }
}

/// C-compatible functions for external library communication
#[no_mangle]
pub extern "C" fn whisper_register_quran_validator(
    callback: ExternalValidationCallback
) -> *const c_char {
    match IntegratedFlutterApi::register_external_validator(callback) {
        Ok(msg) => {
            let c_str = CString::new(msg).unwrap();
            c_str.into_raw()
        }
        Err(e) => {
            let c_str = CString::new(format!("❌ Error: {}", e)).unwrap();
            c_str.into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn whisper_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

/// Export integrated API for Flutter
pub use IntegratedFlutterApi as QuranWhisperApi;
