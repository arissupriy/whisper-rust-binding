use std::env;
use std::ffi::{CString, CStr};
use std::os::raw::{c_int, c_char, c_void, c_float};

// Direct FFI bindings for testing
extern "C" {
    fn whisper_init_from_file(path_model: *const c_char) -> *mut c_void;
    fn whisper_free(ctx: *mut c_void);
    fn whisper_full_default_params(strategy: c_int) -> WhisperFullParams;
    fn whisper_full(
        ctx: *mut c_void,
        params: WhisperFullParams,
        samples: *const c_float,
        n_samples: c_int,
    ) -> c_int;
    fn whisper_full_n_segments(ctx: *mut c_void) -> c_int;
    fn whisper_full_get_segment_text(ctx: *mut c_void, i_segment: c_int) -> *const c_char;
}

// Simplified struct - just the essential fields
#[repr(C)]
#[derive(Debug, Clone)]
struct WhisperFullParams {
    strategy: c_int,
    n_threads: c_int,
    n_max_text_ctx: c_int,
    offset_ms: c_int,
    duration_ms: c_int,
    translate: bool,
    no_context: bool,
    no_timestamps: bool,
    single_segment: bool,
    print_special: bool,
    print_progress: bool,
    print_realtime: bool,
    print_timestamps: bool,
    // Add padding to match C struct
    _padding: [u8; 256], // Large padding to handle any missing fields
}

mod audio_utils {
    use hound::WavReader;
    use std::fs::File;

    pub fn load_wav_file(file_path: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let mut reader = WavReader::new(file)?;
        
        let samples: Result<Vec<f32>, _> = reader
            .samples::<i16>()
            .map(|s| s.map(|sample| sample as f32 / 32768.0))
            .collect();
        
        Ok(samples?)
    }
}

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

    // Load model using simple whisper_init_from_file
    let model_path_c = CString::new(model_path.as_str())?;
    let ctx = unsafe { whisper_init_from_file(model_path_c.as_ptr()) };

    if ctx.is_null() {
        return Err("Failed to load model".into());
    }

    println!("Model loaded successfully!");

    // Load audio
    println!("Loading audio from: {}", audio_path);
    let audio_data = audio_utils::load_wav_file(audio_path)?;
    println!("Loaded {} samples", audio_data.len());

    // Get default parameters
    let mut params = unsafe { whisper_full_default_params(0) }; // WHISPER_SAMPLING_GREEDY = 0
    
    // Set simple parameters
    params.print_realtime = false;
    params.print_progress = false;
    params.print_timestamps = true;
    params.translate = false;
    params.n_threads = 4;

    println!("Processing audio...");

    // Process audio
    let result = unsafe {
        whisper_full(
            ctx,
            params,
            audio_data.as_ptr(),
            audio_data.len() as c_int,
        )
    };

    if result != 0 {
        unsafe { whisper_free(ctx) };
        return Err(format!("Failed to process audio: {}", result).into());
    }

    // Get results
    let n_segments = unsafe { whisper_full_n_segments(ctx) };
    println!("Found {} segments", n_segments);

    for i in 0..n_segments {
        let text_ptr = unsafe { whisper_full_get_segment_text(ctx, i) };
        if !text_ptr.is_null() {
            let text = unsafe { CStr::from_ptr(text_ptr) }.to_string_lossy();
            println!("Segment {}: {}", i, text);
        }
    }

    // Cleanup
    unsafe { whisper_free(ctx) };
    println!("Done!");

    Ok(())
}
