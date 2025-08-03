//! Mock implementation for testing without loading real models

use crate::WhisperError;
use log::info;

/// Initialize a mock whisper engine for testing
pub fn init_mock() -> Result<i32, WhisperError> {
    info!("Initializing mock whisper engine");
    Ok(999) // Special instance ID for mock
}

/// Process audio with mock engine (returns predefined text)
pub fn process_audio_mock(audio_data: &[f32], language: Option<&str>) -> Result<String, WhisperError> {
    let lang = language.unwrap_or("auto");
    info!("Mock processing audio with {} samples, language: {}", audio_data.len(), lang);

    // Return different text based on language for testing
    let text = match lang {
        "ar" => "بسم الله الرحمن الرحيم",
        "en" => "This is a mock transcription for testing.",
        _ => "Mock transcription text.",
    };

    Ok(text.to_string())
}

/// Process audio with sliding window (mock version)
pub fn process_audio_sliding_window_mock(
    audio_data: &[f32],
    window_size_sec: f32,
    step_size_sec: f32,
    sample_rate: i32,
    language: Option<&str>
) -> Result<String, WhisperError> {
    let lang = language.unwrap_or("auto");
    info!(
        "Mock processing audio with sliding window: {} samples, window={}, step={}, sample_rate={}, language={}",
        audio_data.len(), window_size_sec, step_size_sec, sample_rate, lang
    );

    // Return different text based on language for testing
    let segments = match lang {
        "ar" => vec![
            "بسم الله الرحمن الرحيم",
            "الحمد لله رب العالمين",
            "الرحمن الرحيم",
            "مالك يوم الدين",
        ],
        "en" => vec![
            "This is a mock transcription.",
            "Using sliding window approach.",
            "For testing purposes only.",
            "Without loading actual models.",
        ],
        _ => vec!["Mock segment 1.", "Mock segment 2.", "Mock segment 3."],
    };

    Ok(segments.join("\n"))
}

/// Check if a word exists in a dictionary (mock version)
pub fn validate_word_mock(word: &str, global_data_words: &[&str]) -> bool {
    info!("Mock validating word: {}", word);

    // Always return true for certain test words
    if word == "test" || word == "mock" || word == "الله" {
        return true;
    }

    global_data_words.contains(&word)
}
