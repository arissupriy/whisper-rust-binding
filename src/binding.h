#ifndef WHISPER_RUST_BINDING_H
#define WHISPER_RUST_BINDING_H

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Initialize a Whisper model.
 * @param model_path Path to the model file (.bin)
 * @return A positive instance ID on success, -1 on failure
 */
int whisper_rust_init(const char* model_path);

/**
 * Free resources associated with a Whisper instance.
 * @param instance_id The instance ID returned from whisper_rust_init
 * @return true on success, false on failure
 */
bool whisper_rust_free(int instance_id);

/**
 * Check if a Whisper model is valid.
 * @param instance_id The instance ID returned from whisper_rust_init
 * @return true if valid, false otherwise
 */
bool whisper_rust_is_valid(int instance_id);

/**
 * Process audio data with Whisper model.
 * @param instance_id The instance ID returned from whisper_rust_init
 * @param audio_data Pointer to audio data (32-bit float PCM, 16kHz mono)
 * @param audio_len Length of audio data in number of samples
 * @param language Language code (e.g., "en", "ar", etc.) or NULL for auto-detection
 * @param result_buffer Buffer to store the transcription result
 * @param result_buffer_size Size of the result buffer
 * @return true on success, false on failure
 */
bool whisper_rust_process_audio(
    int instance_id,
    const float* audio_data,
    int audio_len,
    const char* language,
    char* result_buffer,
    int result_buffer_size
);

/**
 * Process audio data using a sliding window approach.
 * @param instance_id The instance ID returned from whisper_rust_init
 * @param audio_data Pointer to audio data (32-bit float PCM, 16kHz mono)
 * @param audio_len Length of audio data in number of samples
 * @param window_size_sec Window size in seconds
 * @param step_size_sec Step size in seconds (how much to move the window)
 * @param sample_rate Sample rate of the audio (typically 16000)
 * @param language Language code (e.g., "en", "ar", etc.) or NULL for auto-detection
 * @param result_buffer Buffer to store the transcription result
 * @param result_buffer_size Size of the result buffer
 * @return true on success, false on failure
 */
bool whisper_rust_process_audio_sliding_window(
    int instance_id,
    const float* audio_data,
    int audio_len,
    float window_size_sec,
    float step_size_sec,
    int sample_rate,
    const char* language,
    char* result_buffer,
    int result_buffer_size
);

/**
 * Validate if a word exists in the global data words array.
 * @param word The word to validate
 * @param global_data_words Array of strings to validate against
 * @param global_data_words_len Length of the global_data_words array
 * @return true if the word exists in the array, false otherwise
 */
bool whisper_rust_validate_word(
    const char* word,
    const char** global_data_words,
    int global_data_words_len
);

/**
 * Get information about the loaded model.
 * @param instance_id The instance ID returned from whisper_rust_init
 * @param info_buffer Buffer to store the model information
 * @param info_buffer_size Size of the info buffer
 * @return true on success, false on failure
 */
bool whisper_rust_get_model_info(
    int instance_id,
    char* info_buffer,
    int info_buffer_size
);

#ifdef __cplusplus
}
#endif

#endif // WHISPER_RUST_BINDING_H
