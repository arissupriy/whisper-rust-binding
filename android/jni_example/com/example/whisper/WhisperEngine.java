package com.example.whisper;

/**
 * JNI wrapper for Whisper speech recognition engine.
 */
public class WhisperEngine {
    static {
        System.loadLibrary("whisper_rust");
    }

    /**
     * Initialize a Whisper model.
     * 
     * @param modelPath Path to the model file (.bin)
     * @return A positive instance ID on success, -1 on failure
     */
    public static native int initWhisper(String modelPath);

    /**
     * Free resources associated with a Whisper instance.
     * 
     * @param instanceId The instance ID returned from initWhisper
     * @return true on success, false on failure
     */
    public static native boolean freeWhisper(int instanceId);

    /**
     * Check if a Whisper model is valid.
     * 
     * @param instanceId The instance ID returned from initWhisper
     * @return true if valid, false otherwise
     */
    public static native boolean isValidModel(int instanceId);

    /**
     * Process audio data with Whisper model.
     * 
     * @param instanceId The instance ID returned from initWhisper
     * @param audioData Float array of audio data (32-bit PCM, 16kHz mono)
     * @param language Language code (e.g., "en", "ar", etc.) or null for auto-detection
     * @return Transcription text, or empty string on failure
     */
    public static native String processAudio(int instanceId, float[] audioData, String language);

    /**
     * Process audio data using a sliding window approach.
     * 
     * @param instanceId The instance ID returned from initWhisper
     * @param audioData Float array of audio data (32-bit PCM, 16kHz mono)
     * @param windowSizeSec Window size in seconds
     * @param stepSizeSec Step size in seconds (how much to move the window)
     * @param sampleRate Sample rate of the audio (typically 16000)
     * @param language Language code (e.g., "en", "ar", etc.) or null for auto-detection
     * @return Transcription text, or empty string on failure
     */
    public static native String processAudioSlidingWindow(
            int instanceId,
            float[] audioData,
            float windowSizeSec,
            float stepSizeSec,
            int sampleRate,
            String language);
}
