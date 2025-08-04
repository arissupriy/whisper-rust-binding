#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "android")]
pub use android::init_android_logger;

// Mock implementation for testing without real models
pub mod mock;

// Real-time transcriber for Flutter integration
pub mod realtime_transcriber;

// Production-ready Flutter transcriber
pub mod flutter_transcriber;

// Flutter Rust Bridge API
pub mod flutter_api;

// Quran integration for dual-project setup
pub mod quran_integration;

use std::ffi::{c_char, c_float, c_int, c_void, CStr, CString};
use std::sync::{Arc, Mutex};
use std::ptr::null_mut;
use std::path::Path;
use std::slice;
use std::collections::HashMap;
use anyhow::Result;
use once_cell::sync::Lazy;
use log::error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WhisperError {
    #[error("Failed to initialize model: {0}")]
    ModelInitError(String),

    #[error("Invalid model: {0}")]
    InvalidModel(String),

    #[error("Failed to process audio: {0}")]
    ProcessingError(String),
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Invalid audio data")]
    InvalidAudioData,

    #[error("Internal error: {0}")]
    InternalError(String),
}

// Global static instance manager
static INSTANCES: Lazy<Mutex<HashMap<i32, Arc<Mutex<WhisperContext>>>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

static NEXT_INSTANCE_ID: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0));

#[repr(C)]
pub struct WhisperContext {
    ctx: *mut c_void,
    state: *mut c_void,
    is_loaded: bool,
    model_path: String,
}

// FFI bindings to whisper.cpp
mod ffi {
    use super::*;

    unsafe extern "C" {
        // Context management
        pub fn whisper_init_from_file_with_params(path: *const c_char, params: WhisperContextParams) -> *mut c_void;
        pub fn whisper_free(ctx: *mut c_void);
        pub fn whisper_init_state(ctx: *mut c_void) -> *mut c_void;
        pub fn whisper_free_state(state: *mut c_void);

        // Model info
        pub fn whisper_lang_id(ctx: *mut c_void, lang: *const c_char) -> c_int;
        pub fn whisper_version() -> *const c_char;
        pub fn whisper_lang_str(lang_id: c_int) -> *const c_char;

        // Audio processing
        pub fn whisper_pcm_to_mel_with_state(
            ctx: *mut c_void,
            state: *mut c_void,
            samples: *const c_float,
            n_samples: c_int,
            n_threads: c_int
        ) -> c_int;

        pub fn whisper_full_with_state(
            ctx: *mut c_void,
            state: *mut c_void,
            params: WhisperFullParams,
            samples: *const c_float,
            n_samples: c_int
        ) -> c_int;

        // Results
        pub fn whisper_full_n_segments(ctx: *mut c_void) -> c_int;
        pub fn whisper_full_n_segments_from_state(state: *mut c_void) -> c_int;
        pub fn whisper_full_get_segment_text(ctx: *mut c_void, segment_id: c_int) -> *const c_char;
        pub fn whisper_full_get_segment_text_from_state(state: *mut c_void, segment_id: c_int) -> *const c_char;
        pub fn whisper_full_n_tokens(ctx: *mut c_void, segment_id: c_int) -> c_int;
        pub fn whisper_full_get_token_text(ctx: *mut c_void, token_id: c_int) -> *const c_char;
        pub fn whisper_full_get_token_data(ctx: *mut c_void, segment_id: c_int, token_id: c_int) -> WhisperTokenData;

        // Default params
        pub fn whisper_full_default_params(strategy: c_int) -> WhisperFullParams;
        pub fn whisper_context_default_params() -> WhisperContextParams;
    }

    #[repr(C)]
    pub struct WhisperTokenData {
        pub id: i32,
        pub tid: i32,
        pub p: f32,
        pub plog: f32,
        pub pt: f32,
        pub ptsum: f32,
        pub t0: i64,
        pub t1: i64,
        pub t_dtw: i64,
        pub vlen: f32,
    }

    #[repr(C)]
    pub struct WhisperContextParams {
        pub use_gpu: bool,
        pub flash_attn: bool,
        pub gpu_device: c_int,
        pub dtw_token_timestamps: bool,
        pub dtw_aheads_preset: c_int,
        pub dtw_n_top: c_int,
        pub dtw_aheads: WhisperAheads,
        pub dtw_mem_size: usize,
    }

    #[repr(C)]
    pub struct WhisperAheads {
        pub n_heads: usize,
        pub heads: *const WhisperAhead,
    }

    #[repr(C)]
    pub struct WhisperAhead {
        pub n_text_layer: c_int,
        pub n_head: c_int,
    }

    #[repr(C)]
    pub struct WhisperFullParams {
        pub strategy: c_int, // enum whisper_sampling_strategy

        pub n_threads: c_int,
        pub n_max_text_ctx: c_int,
        pub offset_ms: c_int,
        pub duration_ms: c_int,

        pub translate: bool,
        pub no_context: bool,
        pub no_timestamps: bool,
        pub single_segment: bool,
        pub print_special: bool,
        pub print_progress: bool,
        pub print_realtime: bool,
        pub print_timestamps: bool,

        // [EXPERIMENTAL] token-level timestamps
        pub token_timestamps: bool,
        pub thold_pt: f32,
        pub thold_ptsum: f32,
        pub max_len: c_int,
        pub split_on_word: bool,
        pub max_tokens: c_int,

        // [EXPERIMENTAL] speed-up techniques
        pub debug_mode: bool,
        pub audio_ctx: c_int,

        // [EXPERIMENTAL] [TDRZ] tinydiarize
        pub tdrz_enable: bool,

        // A regular expression that matches tokens to suppress
        pub suppress_regex: *const c_char,

        // tokens to provide to the whisper decoder as initial prompt
        pub initial_prompt: *const c_char,
        pub prompt_tokens: *const c_int,
        pub prompt_n_tokens: c_int,

        // for auto-detection, set to nullptr, "" or "auto"
        pub language: *const c_char,
        pub detect_language: bool,

        // common decoding parameters
        pub suppress_blank: bool,
        pub suppress_nst: bool, // non-speech tokens

        pub temperature: f32,
        pub max_initial_ts: f32,
        pub length_penalty: f32,

        // fallback parameters
        pub temperature_inc: f32,
        pub entropy_thold: f32,
        pub logprob_thold: f32,
        pub no_speech_thold: f32,

        // Greedy strategy parameters
        pub greedy: WhisperGreedyParams,

        // Beam search strategy parameters  
        pub beam_search: WhisperBeamSearchParams,

        // callbacks
        pub new_segment_callback: *mut c_void,
        pub new_segment_callback_user_data: *mut c_void,
        pub progress_callback: *mut c_void,
        pub progress_callback_user_data: *mut c_void,
        pub encoder_begin_callback: *mut c_void,
        pub encoder_begin_callback_user_data: *mut c_void,

        // Additional callbacks missing from our struct
        pub abort_callback: *mut c_void,
        pub abort_callback_user_data: *mut c_void,
        pub logits_filter_callback: *mut c_void,
        pub logits_filter_callback_user_data: *mut c_void,

        // Grammar parameters
        pub grammar_rules: *const *const c_void,
        pub n_grammar_rules: usize,
        pub i_start_rule: usize,
        pub grammar_penalty: f32,

        // VAD parameters
        pub vad: bool,
        pub vad_model_path: *const c_char,
        pub vad_params: WhisperVadParams,
    }

    #[repr(C)]
    pub struct WhisperGreedyParams {
        pub best_of: c_int,
    }

    #[repr(C)]
    pub struct WhisperBeamSearchParams {
        pub beam_size: c_int,
        pub patience: f32,
    }

    #[repr(C)]
    pub struct WhisperVadParams {
        pub threshold: f32,
        pub min_speech_duration_ms: c_int,
        pub min_silence_duration_ms: c_int,
        pub max_speech_duration_s: f32,
        pub speech_pad_ms: c_int,
        pub samples_overlap: f32,
    }
}

unsafe impl Send for WhisperContext {}
unsafe impl Sync for WhisperContext {}

impl Drop for WhisperContext {
    fn drop(&mut self) {
        unsafe {
            if !self.ctx.is_null() {
                if !self.state.is_null() {
                    ffi::whisper_free_state(self.state);
                    self.state = null_mut();
                }
                ffi::whisper_free(self.ctx);
                self.ctx = null_mut();
            }
        }
        self.is_loaded = false;
    }
}

impl WhisperContext {
    fn new(model_path: &str) -> Result<Self, WhisperError> {
        let model_path_c = CString::new(model_path)
            .map_err(|_| WhisperError::ModelInitError("Invalid model path".to_string()))?;

        let params = unsafe { ffi::whisper_context_default_params() };

        let ctx = unsafe { ffi::whisper_init_from_file_with_params(model_path_c.as_ptr(), params) };

        if ctx.is_null() {
            return Err(WhisperError::ModelInitError(format!("Failed to load model from {}", model_path)));
        }

        let state = unsafe { ffi::whisper_init_state(ctx) };

        if state.is_null() {
            unsafe { ffi::whisper_free(ctx) };
            return Err(WhisperError::ModelInitError("Failed to initialize state".to_string()));
        }

        Ok(WhisperContext {
            ctx,
            state,
            is_loaded: true,
            model_path: model_path.to_string(),
        })
    }

    fn process_audio(&mut self, audio_data: &[f32], language: Option<&str>) -> Result<Vec<String>, WhisperError> {
        if !self.is_loaded || self.ctx.is_null() || self.state.is_null() {
            return Err(WhisperError::InvalidModel("Model not loaded".to_string()));
        }

        println!("DEBUG: Starting process_audio with {} samples", audio_data.len());

        // Set up parameters with safer defaults
        let mut params = unsafe { ffi::whisper_full_default_params(0) }; // 0 = WHISPER_SAMPLING_GREEDY

        // Essential parameters only
        params.print_realtime = false;
        params.print_progress = false;
        params.print_timestamps = true;
        params.translate = false;
        params.single_segment = false;
        params.max_tokens = 0;
        params.n_threads = 4;
        
        // Initialize all pointer fields to null for safety
        params.language = null_mut();
        params.initial_prompt = null_mut();
        params.prompt_tokens = null_mut();
        params.suppress_regex = null_mut();
        params.new_segment_callback = null_mut();
        params.new_segment_callback_user_data = null_mut();
        params.progress_callback = null_mut();
        params.progress_callback_user_data = null_mut();
        params.encoder_begin_callback = null_mut();
        params.encoder_begin_callback_user_data = null_mut();
        
        // Initialize new fields that were missing
        params.abort_callback = null_mut();
        params.abort_callback_user_data = null_mut();
        params.logits_filter_callback = null_mut();
        params.logits_filter_callback_user_data = null_mut();
        params.grammar_rules = null_mut();
        params.n_grammar_rules = 0;
        params.i_start_rule = 0;
        params.grammar_penalty = 0.0;
        params.vad = false;
        params.vad_model_path = null_mut();
        
        // Initialize VAD params to default values
        params.vad_params = ffi::WhisperVadParams {
            threshold: 0.5,
            min_speech_duration_ms: 250,
            min_silence_duration_ms: 2000,
            max_speech_duration_s: 30.0,
            speech_pad_ms: 30,
            samples_overlap: 0.0,
        };

        // Set language if provided
        let lang_c_string: Option<CString> = language.map(|lang| CString::new(lang).unwrap_or_default());
        if let Some(lang_ptr) = lang_c_string.as_ref() {
            params.language = lang_ptr.as_ptr();
            println!("DEBUG: Language set to: {:?}", language);
        }

        println!("DEBUG: Calling whisper_full_with_state...");

        // Process audio
        let result = unsafe {
            ffi::whisper_full_with_state(
                self.ctx,
                self.state,
                params,
                audio_data.as_ptr(),
                audio_data.len() as c_int
            )
        };

        println!("DEBUG: whisper_full_with_state returned: {}", result);

        if result != 0 {
            return Err(WhisperError::ProcessingError(format!("Failed to process audio: {}", result)));
        }

        println!("DEBUG: Getting number of segments...");

        // Extract results using state-based functions
        let n_segments = unsafe { ffi::whisper_full_n_segments_from_state(self.state) };
        println!("DEBUG: Found {} segments", n_segments);
        
        let mut segments = Vec::with_capacity(n_segments as usize);

        for i in 0..n_segments {
            let text_ptr = unsafe { ffi::whisper_full_get_segment_text_from_state(self.state, i) };
            if !text_ptr.is_null() {
                let text = unsafe { CStr::from_ptr(text_ptr) }
                    .to_string_lossy()
                    .to_string();
                segments.push(text);
            }
        }

        println!("DEBUG: process_audio completed successfully");
        Ok(segments)
    }

    fn get_model_info(&self) -> Result<String, WhisperError> {
        if !self.is_loaded || self.ctx.is_null() {
            return Err(WhisperError::InvalidModel("Model not loaded".to_string()));
        }

        let info_ptr = unsafe { ffi::whisper_version() };
        if info_ptr.is_null() {
            return Err(WhisperError::InternalError("Failed to get model info".to_string()));
        }

        let info = unsafe { CStr::from_ptr(info_ptr) }
            .to_string_lossy()
            .to_string();

        Ok(info)
    }

    fn is_valid(&self) -> bool {
        self.is_loaded && !self.ctx.is_null() && !self.state.is_null()
    }

    fn process_audio_sliding_window(
        &mut self, 
        audio_data: &[f32], 
        window_size_sec: f32,
        step_size_sec: f32,
        sample_rate: i32,
        language: Option<&str>
    ) -> Result<Vec<String>, WhisperError> {
        if !self.is_loaded || self.ctx.is_null() || self.state.is_null() {
            return Err(WhisperError::InvalidModel("Model not loaded".to_string()));
        }

        if window_size_sec <= 0.0 || step_size_sec <= 0.0 || step_size_sec > window_size_sec {
            return Err(WhisperError::ProcessingError("Invalid window or step size".to_string()));
        }

        if sample_rate <= 0 {
            return Err(WhisperError::ProcessingError("Invalid sample rate".to_string()));
        }

        let window_samples = (window_size_sec * sample_rate as f32) as usize;
        let step_samples = (step_size_sec * sample_rate as f32) as usize;

        if window_samples >= audio_data.len() {
            // If audio is shorter than one window, process the entire audio
            return self.process_audio(audio_data, language);
        }

        let mut all_segments = Vec::new();
        let mut position = 0;

        while position + window_samples <= audio_data.len() {
            let window = &audio_data[position..position + window_samples];
            let segments = self.process_audio(window, language)?;

            for segment in segments {
                all_segments.push(segment);
            }

            position += step_samples;
        }

        // Process the last window if there's remaining audio
        if position < audio_data.len() && audio_data.len() - position > step_samples / 2 {
            let window = &audio_data[audio_data.len() - window_samples.min(audio_data.len())..audio_data.len()];
            let segments = self.process_audio(window, language)?;

            for segment in segments {
                all_segments.push(segment);
            }
        }

        Ok(all_segments)
    }
}

// Exported C API functions

#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisper_rust_init(model_path: *const c_char) -> i32 {
    let model_path_str = match unsafe { CStr::from_ptr(model_path) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match WhisperContext::new(model_path_str) {
        Ok(context) => {
            let mut next_id = NEXT_INSTANCE_ID.lock().unwrap();
            let instance_id = *next_id;
            *next_id += 1;

            let mut instances = INSTANCES.lock().unwrap();
            instances.insert(instance_id, Arc::new(Mutex::new(context)));

            instance_id
        },
        Err(_) => -1,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisper_rust_free(instance_id: i32) -> bool {
    let mut instances = INSTANCES.lock().unwrap();
    instances.remove(&instance_id).is_some()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisper_rust_is_valid(instance_id: i32) -> bool {
    let instances = INSTANCES.lock().unwrap();

    if let Some(context) = instances.get(&instance_id) {
        let context = context.lock().unwrap();
        context.is_valid()
    } else {
        false
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisper_rust_process_audio(
    instance_id: i32,
    audio_data: *const c_float,
    audio_len: i32,
    language: *const c_char,
    result_buffer: *mut c_char,
    result_buffer_size: i32
) -> bool {
    if audio_data.is_null() || audio_len <= 0 || result_buffer.is_null() || result_buffer_size <= 0 {
        return false;
    }

    let instances = INSTANCES.lock().unwrap();

    let context = match instances.get(&instance_id) {
        Some(c) => c,
        None => return false,
    };

    let audio_slice = unsafe { slice::from_raw_parts(audio_data, audio_len as usize) };

    let language_str = if language.is_null() {
        None
    } else {
        match unsafe { CStr::from_ptr(language) }.to_str() {
            Ok(s) => Some(s),
            Err(_) => return false,
        }
    };

    let mut context = context.lock().unwrap();

    match context.process_audio(audio_slice, language_str) {
        Ok(segments) => {
            let result = segments.join("\n");
            let result_c = match CString::new(result) {
                Ok(s) => s,
                Err(_) => return false,
            };

            let result_bytes = result_c.as_bytes_with_nul();
            if result_bytes.len() > result_buffer_size as usize {
                return false;
            }

            unsafe {
                std::ptr::copy_nonoverlapping(
                    result_bytes.as_ptr(),
                    result_buffer as *mut u8,
                    result_bytes.len()
                );
            }

            true
        },
        Err(_) => false,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisper_rust_process_audio_sliding_window(
    instance_id: i32,
    audio_data: *const c_float,
    audio_len: i32,
    window_size_sec: f32,
    step_size_sec: f32,
    sample_rate: i32,
    language: *const c_char,
    result_buffer: *mut c_char,
    result_buffer_size: i32
) -> bool {
    if audio_data.is_null() || audio_len <= 0 || result_buffer.is_null() || result_buffer_size <= 0 {
        return false;
    }

    let instances = INSTANCES.lock().unwrap();

    let context = match instances.get(&instance_id) {
        Some(c) => c,
        None => return false,
    };

    let audio_slice = unsafe { slice::from_raw_parts(audio_data, audio_len as usize) };

    let language_str = if language.is_null() {
        None
    } else {
        match unsafe { CStr::from_ptr(language) }.to_str() {
            Ok(s) => Some(s),
            Err(_) => return false,
        }
    };

    let mut context = context.lock().unwrap();

    match context.process_audio_sliding_window(audio_slice, window_size_sec, step_size_sec, sample_rate, language_str) {
        Ok(segments) => {
            let result = segments.join("\n");
            let result_c = match CString::new(result) {
                Ok(s) => s,
                Err(_) => return false,
            };

            let result_bytes = result_c.as_bytes_with_nul();
            if result_bytes.len() > result_buffer_size as usize {
                return false;
            }

            unsafe {
                std::ptr::copy_nonoverlapping(
                    result_bytes.as_ptr(),
                    result_buffer as *mut u8,
                    result_bytes.len()
                );
            }

            true
        },
        Err(_) => false,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisper_rust_validate_word(
    word: *const c_char,
    global_data_words: *const *const c_char,
    global_data_words_len: i32
) -> bool {
    if word.is_null() || global_data_words.is_null() || global_data_words_len <= 0 {
        return false;
    }

    let word_str = match unsafe { CStr::from_ptr(word) }.to_str() {
        Ok(s) => s.to_lowercase(),
        Err(_) => return false,
    };

    // Convert the array of C strings to Rust strings
    let mut words = Vec::with_capacity(global_data_words_len as usize);
    for i in 0..global_data_words_len {
        let word_ptr = unsafe { *global_data_words.offset(i as isize) };
        if word_ptr.is_null() {
            continue;
        }

        match unsafe { CStr::from_ptr(word_ptr) }.to_str() {
            Ok(s) => words.push(s.to_lowercase()),
            Err(_) => continue,
        }
    }

    words.contains(&word_str)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisper_rust_get_model_info(
    instance_id: i32,
    info_buffer: *mut c_char,
    info_buffer_size: i32
) -> bool {
    if info_buffer.is_null() || info_buffer_size <= 0 {
        return false;
    }

    let instances = INSTANCES.lock().unwrap();

    let context = match instances.get(&instance_id) {
        Some(c) => c,
        None => return false,
    };

    let context = context.lock().unwrap();

    match context.get_model_info() {
        Ok(info) => {
            let info_c = match CString::new(info) {
                Ok(s) => s,
                Err(_) => return false,
            };

            let info_bytes = info_c.as_bytes_with_nul();
            if info_bytes.len() > info_buffer_size as usize {
                return false;
            }

            unsafe {
                std::ptr::copy_nonoverlapping(
                    info_bytes.as_ptr(),
                    info_buffer as *mut u8,
                    info_bytes.len()
                );
            }

            true
        },
        Err(_) => false,
    }
}

// Public Rust API (when used as a Rust library)

pub fn init_whisper(model_path: &str) -> Result<i32, WhisperError> {
    let model_path = Path::new(model_path);
    if !model_path.exists() {
        return Err(WhisperError::ModelInitError(format!("Model file not found: {}", model_path.display())));
    }

    let model_path_c = CString::new(model_path.to_string_lossy().as_bytes())
        .map_err(|_| WhisperError::ModelInitError("Invalid model path".to_string()))?;

    let instance_id = unsafe { whisper_rust_init(model_path_c.as_ptr()) };

    if instance_id < 0 {
        Err(WhisperError::ModelInitError("Failed to initialize whisper model".to_string()))
    } else {
        Ok(instance_id)
    }
}

pub fn free_whisper(instance_id: i32) -> Result<(), WhisperError> {
    if unsafe { whisper_rust_free(instance_id) } {
        Ok(())
    } else {
        Err(WhisperError::InternalError(format!("Failed to free instance {}", instance_id)))
    }
}

pub fn is_valid_model(instance_id: i32) -> bool {
    unsafe { whisper_rust_is_valid(instance_id) }
}

pub fn process_audio(instance_id: i32, audio: &[f32], language: Option<&str>) -> Result<String, WhisperError> {
    let buffer_size = 10240; // Adjust as needed
    let mut result_buffer = vec![0u8; buffer_size];

    let language_c = language.map(|l| CString::new(l).unwrap());
    let language_ptr = language_c.as_ref().map_or(std::ptr::null(), |c| c.as_ptr());

    let success = unsafe { whisper_rust_process_audio(
        instance_id,
        audio.as_ptr(),
        audio.len() as i32,
        language_ptr,
        result_buffer.as_mut_ptr() as *mut c_char,
        buffer_size as i32
    ) };

    if success {
        let result = unsafe { CStr::from_ptr(result_buffer.as_ptr() as *const c_char) }
            .to_string_lossy()
            .to_string();
        Ok(result)
    } else {
        Err(WhisperError::ProcessingError("Failed to process audio".to_string()))
    }
}

pub fn process_audio_sliding_window(
    instance_id: i32,
    audio: &[f32],
    window_size_sec: f32,
    step_size_sec: f32,
    sample_rate: i32,
    language: Option<&str>
) -> Result<String, WhisperError> {
    let buffer_size = 102400; // Larger buffer for sliding window results
    let mut result_buffer = vec![0u8; buffer_size];

    let language_c = language.map(|l| CString::new(l).unwrap());
    let language_ptr = language_c.as_ref().map_or(std::ptr::null(), |c| c.as_ptr());

    let success = unsafe { whisper_rust_process_audio_sliding_window(
        instance_id,
        audio.as_ptr(),
        audio.len() as i32,
        window_size_sec,
        step_size_sec,
        sample_rate,
        language_ptr,
        result_buffer.as_mut_ptr() as *mut c_char,
        buffer_size as i32
    ) };

    if success {
        let result = unsafe { CStr::from_ptr(result_buffer.as_ptr() as *const c_char) }
            .to_string_lossy()
            .to_string();
        Ok(result)
    } else {
        Err(WhisperError::ProcessingError("Failed to process audio with sliding window".to_string()))
    }
}

pub fn get_model_info(instance_id: i32) -> Result<String, WhisperError> {
    let buffer_size = 1024;
    let mut info_buffer = vec![0u8; buffer_size];

    let success = unsafe { whisper_rust_get_model_info(
        instance_id,
        info_buffer.as_mut_ptr() as *mut c_char,
        buffer_size as i32
    ) };

    if success {
        let info = unsafe { CStr::from_ptr(info_buffer.as_ptr() as *const c_char) }
            .to_string_lossy()
            .to_string();
        Ok(info)
    } else {
        Err(WhisperError::InternalError("Failed to get model info".to_string()))
    }
}

pub fn validate_word(word: &str, global_data_words: &[&str]) -> bool {
    let word_c = CString::new(word).unwrap();

    // Convert the Rust string slice to an array of C strings
    let c_words: Vec<CString> = global_data_words.iter()
        .map(|w| CString::new(*w).unwrap())
        .collect();

    let c_word_ptrs: Vec<*const c_char> = c_words.iter()
        .map(|c| c.as_ptr())
        .collect();

    unsafe { whisper_rust_validate_word(
        word_c.as_ptr(),
        c_word_ptrs.as_ptr(),
        c_word_ptrs.len() as i32
    ) }
}

// Export Flutter API
pub use flutter_api::*;

// Export Quran integration API
pub use quran_integration::*;

#[cfg(test)]
mod tests {
    #[test]
    fn test_api() {
        // This test is just a placeholder. Real tests would need a model file and audio data.
        assert!(true);
    }
}

