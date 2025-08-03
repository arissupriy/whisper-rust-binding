package com.example.whisper;

import android.content.Context;
import android.util.Log;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;

/**
 * Example usage of WhisperEngine in an Android app.
 */
public class WhisperExample {
    private static final String TAG = "WhisperExample";
    private static final String MODEL_FILENAME = "ggml-base.en.bin";

    private final Context context;
    private int instanceId = -1;

    public WhisperExample(Context context) {
        this.context = context;
    }

    /**
     * Initialize the Whisper engine.
     */
    public boolean initialize() {
        // Copy model file from assets to app's files directory if needed
        File modelFile = new File(context.getFilesDir(), MODEL_FILENAME);
        if (!modelFile.exists()) {
            try {
                copyAssetToFile(MODEL_FILENAME, modelFile);
            } catch (IOException e) {
                Log.e(TAG, "Failed to copy model file", e);
                return false;
            }
        }

        // Initialize Whisper engine
        instanceId = WhisperEngine.initWhisper(modelFile.getAbsolutePath());
        return instanceId >= 0;
    }

    /**
     * Process audio data.
     * 
     * @param audioData Float array of audio samples (32-bit PCM, 16kHz mono)
     * @param language Language code (e.g., "ar" for Arabic) or null for auto-detection
     * @return Transcription text
     */
    public String processAudio(float[] audioData, String language) {
        if (instanceId < 0) {
            Log.e(TAG, "Whisper engine not initialized");
            return "";
        }

        return WhisperEngine.processAudio(instanceId, audioData, language);
    }

    /**
     * Process streaming audio with sliding window.
     * 
     * @param audioData Float array of audio samples (32-bit PCM, 16kHz mono)
     * @param windowSizeSec Window size in seconds
     * @param stepSizeSec Step size in seconds
     * @param language Language code (e.g., "ar" for Arabic) or null for auto-detection
     * @return Transcription text
     */
    public String processStreamingAudio(float[] audioData, float windowSizeSec, float stepSizeSec, String language) {
        if (instanceId < 0) {
            Log.e(TAG, "Whisper engine not initialized");
            return "";
        }

        return WhisperEngine.processAudioSlidingWindow(
                instanceId,
                audioData,
                windowSizeSec,
                stepSizeSec,
                16000, // Sample rate (16kHz)
                language);
    }

    /**
     * Release resources.
     */
    public void release() {
        if (instanceId >= 0) {
            WhisperEngine.freeWhisper(instanceId);
            instanceId = -1;
        }
    }

    /**
     * Copy a file from assets to the app's files directory.
     */
    private void copyAssetToFile(String assetName, File outFile) throws IOException {
        try (InputStream in = context.getAssets().open(assetName);
             FileOutputStream out = new FileOutputStream(outFile)) {

            byte[] buffer = new byte[8192];
            int read;
            while ((read = in.read(buffer)) != -1) {
                out.write(buffer, 0, read);
            }
        }
    }
}
