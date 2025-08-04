#[cfg(feature = "android-jni")]
use jni::objects::{JClass, JObject, JString, JValue};
#[cfg(feature = "android-jni")]
use jni::sys::{jboolean, jfloatArray, jint, jlong, jstring};
#[cfg(feature = "android-jni")]
use jni::JNIEnv;

#[cfg(feature = "android-jni")]
use std::panic::catch_unwind;

#[cfg(feature = "android-jni")]
use crate::{free_whisper, init_whisper, is_valid_model, process_audio, process_audio_sliding_window, WhisperError};

#[cfg(target_os = "android")]
pub fn init_android_logger() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)  // Updated API
            .with_tag("WhisperRust"),
    );
}

// JNI bindings for Android
#[cfg(feature = "android-jni")]
#[no_mangle]
pub extern "system" fn Java_com_example_whisper_WhisperEngine_initWhisper(
    env: JNIEnv,
    _class: JClass,
    model_path: JString,
) -> jint {
    let result = catch_unwind(|| {
        let model_path: String = env.get_string(model_path).expect("Invalid model path").into();
        match init_whisper(&model_path) {
            Ok(id) => id,
            Err(_) => -1,
        }
    });

    match result {
        Ok(id) => id,
        Err(_) => -1,
    }
}

#[cfg(feature = "android-jni")]
#[no_mangle]
pub extern "system" fn Java_com_example_whisper_WhisperEngine_freeWhisper(
    _env: JNIEnv,
    _class: JClass,
    instance_id: jint,
) -> jboolean {
    let result = catch_unwind(|| {
        match free_whisper(instance_id) {
            Ok(_) => true,
            Err(_) => false,
        }
    });

    match result {
        Ok(success) => if success { 1 } else { 0 },
        Err(_) => 0,
    }
}

#[cfg(feature = "android-jni")]
#[no_mangle]
pub extern "system" fn Java_com_example_whisper_WhisperEngine_isValidModel(
    _env: JNIEnv,
    _class: JClass,
    instance_id: jint,
) -> jboolean {
    let result = catch_unwind(|| {
        is_valid_model(instance_id)
    });

    match result {
        Ok(valid) => if valid { 1 } else { 0 },
        Err(_) => 0,
    }
}

#[cfg(feature = "android-jni")]
#[no_mangle]
pub extern "system" fn Java_com_example_whisper_WhisperEngine_processAudio(
    env: JNIEnv,
    _class: JClass,
    instance_id: jint,
    audio_data: jfloatArray,
    language: JString,
) -> jstring {
    let result = catch_unwind(|| {
        // Convert jfloatArray to Rust Vec<f32>
        let length = env.get_array_length(audio_data).unwrap_or(0) as usize;
        let mut buffer = vec![0.0f32; length];
        env.get_float_array_region(audio_data, 0, &mut buffer).expect("Failed to get audio data");

        // Get language string (or null)
        let language: Option<String> = if env.is_null_object(language.into()) {
            None
        } else {
            Some(env.get_string(language).expect("Invalid language string").into())
        };

        // Process audio
        match process_audio(instance_id, &buffer, language.as_deref()) {
            Ok(transcript) => env.new_string(transcript).expect("Failed to create result string").into_raw(),
            Err(_) => env.new_string("").expect("Failed to create empty string").into_raw(),
        }
    });

    match result {
        Ok(string) => string,
        Err(_) => env.new_string("").expect("Failed to create empty string").into_raw(),
    }
}

#[cfg(feature = "android-jni")]
#[no_mangle]
pub extern "system" fn Java_com_example_whisper_WhisperEngine_processAudioSlidingWindow(
    env: JNIEnv,
    _class: JClass,
    instance_id: jint,
    audio_data: jfloatArray,
    window_size_sec: jfloat,
    step_size_sec: jfloat,
    sample_rate: jint,
    language: JString,
) -> jstring {
    let result = catch_unwind(|| {
        // Convert jfloatArray to Rust Vec<f32>
        let length = env.get_array_length(audio_data).unwrap_or(0) as usize;
        let mut buffer = vec![0.0f32; length];
        env.get_float_array_region(audio_data, 0, &mut buffer).expect("Failed to get audio data");

        // Get language string (or null)
        let language: Option<String> = if env.is_null_object(language.into()) {
            None
        } else {
            Some(env.get_string(language).expect("Invalid language string").into())
        };

        // Process audio with sliding window
        match process_audio_sliding_window(
            instance_id, 
            &buffer, 
            window_size_sec as f32, 
            step_size_sec as f32, 
            sample_rate, 
            language.as_deref()
        ) {
            Ok(transcript) => env.new_string(transcript).expect("Failed to create result string").into_raw(),
            Err(_) => env.new_string("").expect("Failed to create empty string").into_raw(),
        }
    });

    match result {
        Ok(string) => string,
        Err(_) => env.new_string("").expect("Failed to create empty string").into_raw(),
    }
}
