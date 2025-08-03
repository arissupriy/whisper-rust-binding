# Audio Processing Guide

Comprehensive guide for audio processing with Whisper Rust Binding.

## 🎵 Overview

This guide covers:
- 🎤 **Audio Input** - Recording and file handling
- 🔄 **Format Conversion** - Converting between audio formats
- ⚡ **Preprocessing** - Optimizing audio for transcription
- ⏱️ **Real-time Processing** - Live audio transcription
- 📊 **Quality Analysis** - Audio quality assessment
- 🔧 **Advanced Techniques** - Professional audio processing

## 🎤 Audio Input Sources

### Microphone Recording

#### Linux (ALSA/PulseAudio)

```rust
use std::process::Command;

fn record_microphone(duration_seconds: u32, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Record using arecord (ALSA)
    let output = Command::new("arecord")
        .args(&[
            "-f", "S16_LE",           // 16-bit little endian
            "-c", "1",               // Mono
            "-r", "16000",           // 16kHz sample rate
            "-t", "wav",             // WAV format
            "-d", &duration_seconds.to_string(),
            output_file
        ])
        .output()?;
    
    if !output.status.success() {
        eprintln!("Recording failed: {}", String::from_utf8_lossy(&output.stderr));
        return Err("Recording failed".into());
    }
    
    println!("Recorded audio to: {}", output_file);
    Ok(())
}

// Alternative using PulseAudio
fn record_pulseaudio(duration_seconds: u32, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("parecord")
        .args(&[
            "--format=s16le",
            "--channels=1",
            "--rate=16000",
            "--file-format=wav",
            &format!("--duration={}", duration_seconds),
            output_file
        ])
        .output()?;
    
    if !output.status.success() {
        return Err("PulseAudio recording failed".into());
    }
    
    Ok(())
}
```

#### Android Audio Recording

```java
public class AndroidAudioRecorder {
    private static final int SAMPLE_RATE = 16000;
    private static final int CHANNEL_CONFIG = AudioFormat.CHANNEL_IN_MONO;
    private static final int AUDIO_FORMAT = AudioFormat.ENCODING_PCM_16BIT;
    
    private AudioRecord audioRecord;
    private boolean isRecording = false;
    
    public float[] recordAudio(int durationMs) {
        int bufferSize = AudioRecord.getMinBufferSize(SAMPLE_RATE, CHANNEL_CONFIG, AUDIO_FORMAT);
        
        audioRecord = new AudioRecord(
            MediaRecorder.AudioSource.MIC,
            SAMPLE_RATE,
            CHANNEL_CONFIG,
            AUDIO_FORMAT,
            bufferSize
        );
        
        if (audioRecord.getState() != AudioRecord.STATE_INITIALIZED) {
            throw new RuntimeException("AudioRecord initialization failed");
        }
        
        audioRecord.startRecording();
        
        int totalSamples = (SAMPLE_RATE * durationMs) / 1000;
        short[] pcmData = new short[totalSamples];
        int samplesRead = 0;
        
        while (samplesRead < totalSamples) {
            int result = audioRecord.read(pcmData, samplesRead, totalSamples - samplesRead);
            if (result > 0) {
                samplesRead += result;
            }
        }
        
        audioRecord.stop();
        audioRecord.release();
        
        // Convert to float
        float[] floatData = new float[totalSamples];
        for (int i = 0; i < totalSamples; i++) {
            floatData[i] = pcmData[i] / 32768.0f;
        }
        
        return floatData;
    }
}
```

### File Input Processing

#### WAV File Processing

```rust
use std::fs::File;
use std::io::{BufReader, Read};

pub struct WavHeader {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
    pub data_size: u32,
}

pub fn read_wav_file(file_path: &str) -> Result<(WavHeader, Vec<f32>), Box<dyn std::error::Error>> {
    let mut file = BufReader::new(File::open(file_path)?);
    
    // Read WAV header
    let mut header = [0u8; 44];
    file.read_exact(&mut header)?;
    
    // Parse header
    let wav_header = WavHeader {
        sample_rate: u32::from_le_bytes([header[24], header[25], header[26], header[27]]),
        channels: u16::from_le_bytes([header[22], header[23]]),
        bits_per_sample: u16::from_le_bytes([header[34], header[35]]),
        data_size: u32::from_le_bytes([header[40], header[41], header[42], header[43]]),
    };
    
    // Validate WAV format
    if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
        return Err("Invalid WAV file format".into());
    }
    
    if wav_header.bits_per_sample != 16 {
        return Err("Only 16-bit WAV files are supported".into());
    }
    
    // Read audio data
    let num_samples = (wav_header.data_size / 2) as usize; // 16-bit = 2 bytes per sample
    let mut pcm_data = vec![0i16; num_samples];
    
    for i in 0..num_samples {
        let mut sample_bytes = [0u8; 2];
        file.read_exact(&mut sample_bytes)?;
        pcm_data[i] = i16::from_le_bytes(sample_bytes);
    }
    
    // Convert to float
    let mut float_data = Vec::with_capacity(num_samples);
    for sample in pcm_data {
        float_data.push(sample as f32 / 32768.0);
    }
    
    // Convert stereo to mono if needed
    if wav_header.channels == 2 {
        float_data = stereo_to_mono(&float_data);
    }
    
    // Resample to 16kHz if needed
    if wav_header.sample_rate != 16000 {
        float_data = resample_to_16khz(&float_data, wav_header.sample_rate);
    }
    
    Ok((wav_header, float_data))
}

fn stereo_to_mono(stereo_data: &[f32]) -> Vec<f32> {
    stereo_data.chunks(2)
        .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
        .collect()
}

fn resample_to_16khz(input: &[f32], input_sample_rate: u32) -> Vec<f32> {
    if input_sample_rate == 16000 {
        return input.to_vec();
    }
    
    let ratio = input_sample_rate as f64 / 16000.0;
    let output_length = (input.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(output_length);
    
    for i in 0..output_length {
        let src_index = (i as f64 * ratio) as usize;
        
        if src_index + 1 < input.len() {
            let frac = (i as f64 * ratio) - src_index as f64;
            let interpolated = input[src_index] * (1.0 - frac as f32) + 
                             input[src_index + 1] * frac as f32;
            output.push(interpolated);
        } else if src_index < input.len() {
            output.push(input[src_index]);
        }
    }
    
    output
}
```

#### MP3/OGG Processing with FFmpeg

```rust
use std::process::Command;
use std::fs;

pub fn convert_audio_to_wav(input_file: &str, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("ffmpeg")
        .args(&[
            "-i", input_file,
            "-ar", "16000",          // 16kHz sample rate
            "-ac", "1",              // Mono
            "-sample_fmt", "s16",    // 16-bit
            "-y",                    // Overwrite output
            output_file
        ])
        .output()?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg conversion failed: {}", error_msg).into());
    }
    
    println!("Converted {} to {}", input_file, output_file);
    Ok(())
}

pub fn process_any_audio_file(input_file: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let temp_wav = "/tmp/temp_audio.wav";
    
    // Convert to WAV format
    convert_audio_to_wav(input_file, temp_wav)?;
    
    // Read WAV data
    let (_, audio_data) = read_wav_file(temp_wav)?;
    
    // Clean up temporary file
    fs::remove_file(temp_wav).ok();
    
    Ok(audio_data)
}
```

## 🔄 Format Conversion

### Sample Rate Conversion

```rust
pub struct AudioResampler {
    input_rate: u32,
    output_rate: u32,
}

impl AudioResampler {
    pub fn new(input_rate: u32, output_rate: u32) -> Self {
        Self { input_rate, output_rate }
    }
    
    pub fn resample(&self, input: &[f32]) -> Vec<f32> {
        if self.input_rate == self.output_rate {
            return input.to_vec();
        }
        
        let ratio = self.input_rate as f64 / self.output_rate as f64;
        let output_length = (input.len() as f64 / ratio) as usize;
        let mut output = Vec::with_capacity(output_length);
        
        for i in 0..output_length {
            let src_pos = i as f64 * ratio;
            let src_index = src_pos as usize;
            let frac = src_pos - src_index as f64;
            
            let sample = if src_index + 1 < input.len() {
                // Linear interpolation
                input[src_index] * (1.0 - frac as f32) + 
                input[src_index + 1] * frac as f32
            } else if src_index < input.len() {
                input[src_index]
            } else {
                0.0
            };
            
            output.push(sample);
        }
        
        output
    }
    
    // Higher quality resampling using sinc interpolation
    pub fn resample_sinc(&self, input: &[f32]) -> Vec<f32> {
        if self.input_rate == self.output_rate {
            return input.to_vec();
        }
        
        let ratio = self.input_rate as f64 / self.output_rate as f64;
        let output_length = (input.len() as f64 / ratio) as usize;
        let mut output = Vec::with_capacity(output_length);
        
        let filter_width = 8; // Sinc filter width
        
        for i in 0..output_length {
            let src_pos = i as f64 * ratio;
            let center = src_pos as isize;
            let mut sum = 0.0f32;
            let mut weight_sum = 0.0f32;
            
            for j in (center - filter_width)..(center + filter_width + 1) {
                if j >= 0 && j < input.len() as isize {
                    let x = src_pos - j as f64;
                    let weight = if x.abs() < 1e-10 {
                        1.0
                    } else {
                        let pi_x = std::f64::consts::PI * x;
                        (pi_x.sin() / pi_x) as f32
                    };
                    
                    sum += input[j as usize] * weight;
                    weight_sum += weight;
                }
            }
            
            output.push(if weight_sum != 0.0 { sum / weight_sum } else { 0.0 });
        }
        
        output
    }
}
```

### Channel Conversion

```rust
pub fn stereo_to_mono(stereo: &[f32]) -> Vec<f32> {
    stereo.chunks(2)
        .map(|chunk| {
            if chunk.len() == 2 {
                (chunk[0] + chunk[1]) / 2.0
            } else {
                chunk[0]
            }
        })
        .collect()
}

pub fn mono_to_stereo(mono: &[f32]) -> Vec<f32> {
    let mut stereo = Vec::with_capacity(mono.len() * 2);
    for &sample in mono {
        stereo.push(sample);
        stereo.push(sample);
    }
    stereo
}

pub fn extract_channel(multichannel: &[f32], num_channels: usize, channel_index: usize) -> Vec<f32> {
    multichannel.iter()
        .skip(channel_index)
        .step_by(num_channels)
        .copied()
        .collect()
}
```

## ⚡ Audio Preprocessing

### Noise Reduction

```rust
pub struct NoiseGate {
    threshold: f32,
    ratio: f32,
    attack_samples: usize,
    release_samples: usize,
    envelope: f32,
}

impl NoiseGate {
    pub fn new(threshold: f32, ratio: f32, attack_ms: f32, release_ms: f32, sample_rate: u32) -> Self {
        Self {
            threshold,
            ratio,
            attack_samples: ((attack_ms / 1000.0) * sample_rate as f32) as usize,
            release_samples: ((release_ms / 1000.0) * sample_rate as f32) as usize,
            envelope: 0.0,
        }
    }
    
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output = Vec::with_capacity(input.len());
        
        for &sample in input {
            let level = sample.abs();
            
            // Update envelope
            if level > self.envelope {
                // Attack
                self.envelope += (level - self.envelope) / self.attack_samples as f32;
            } else {
                // Release
                self.envelope -= (self.envelope - level) / self.release_samples as f32;
            }
            
            // Apply gate
            let gain = if self.envelope > self.threshold {
                1.0
            } else {
                (self.envelope / self.threshold).powf(self.ratio - 1.0)
            };
            
            output.push(sample * gain);
        }
        
        output
    }
}

// Simple high-pass filter to remove low frequency noise
pub fn high_pass_filter(input: &[f32], cutoff_hz: f32, sample_rate: u32) -> Vec<f32> {
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz);
    let dt = 1.0 / sample_rate as f32;
    let alpha = rc / (rc + dt);
    
    let mut output = Vec::with_capacity(input.len());
    let mut prev_input = 0.0;
    let mut prev_output = 0.0;
    
    for &sample in input {
        let filtered = alpha * (prev_output + sample - prev_input);
        output.push(filtered);
        
        prev_input = sample;
        prev_output = filtered;
    }
    
    output
}
```

### Normalization and AGC

```rust
pub fn normalize_audio(input: &[f32]) -> Vec<f32> {
    let max_amplitude = input.iter()
        .map(|&x| x.abs())
        .fold(0.0f32, f32::max);
    
    if max_amplitude == 0.0 {
        return input.to_vec();
    }
    
    let scale = 0.95 / max_amplitude; // Leave some headroom
    input.iter().map(|&x| x * scale).collect()
}

pub fn rms_normalize(input: &[f32], target_rms: f32) -> Vec<f32> {
    let rms = (input.iter().map(|&x| x * x).sum::<f32>() / input.len() as f32).sqrt();
    
    if rms == 0.0 {
        return input.to_vec();
    }
    
    let scale = target_rms / rms;
    input.iter().map(|&x| x * scale).collect()
}

pub struct AutomaticGainControl {
    target_level: f32,
    max_gain: f32,
    attack_time: f32,
    release_time: f32,
    current_gain: f32,
    sample_rate: u32,
}

impl AutomaticGainControl {
    pub fn new(target_level: f32, max_gain: f32, attack_ms: f32, release_ms: f32, sample_rate: u32) -> Self {
        Self {
            target_level,
            max_gain,
            attack_time: (-1000.0 / (attack_ms * sample_rate as f32)).exp(),
            release_time: (-1000.0 / (release_ms * sample_rate as f32)).exp(),
            current_gain: 1.0,
            sample_rate,
        }
    }
    
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output = Vec::with_capacity(input.len());
        
        for &sample in input {
            let level = sample.abs();
            let desired_gain = if level > 0.0 {
                (self.target_level / level).min(self.max_gain)
            } else {
                self.max_gain
            };
            
            // Smooth gain changes
            let time_constant = if desired_gain > self.current_gain {
                self.attack_time
            } else {
                self.release_time
            };
            
            self.current_gain = self.current_gain * time_constant + 
                               desired_gain * (1.0 - time_constant);
            
            output.push(sample * self.current_gain);
        }
        
        output
    }
}
```

### Silence Detection and Trimming

```rust
pub fn detect_silence_regions(
    audio: &[f32], 
    sample_rate: u32, 
    threshold: f32, 
    min_silence_duration_ms: u32
) -> Vec<(usize, usize)> {
    let min_silence_samples = (sample_rate * min_silence_duration_ms / 1000) as usize;
    let mut silence_regions = Vec::new();
    let mut silence_start = None;
    
    for (i, &sample) in audio.iter().enumerate() {
        if sample.abs() <= threshold {
            if silence_start.is_none() {
                silence_start = Some(i);
            }
        } else {
            if let Some(start) = silence_start {
                if i - start >= min_silence_samples {
                    silence_regions.push((start, i));
                }
                silence_start = None;
            }
        }
    }
    
    // Handle silence at the end
    if let Some(start) = silence_start {
        if audio.len() - start >= min_silence_samples {
            silence_regions.push((start, audio.len()));
        }
    }
    
    silence_regions
}

pub fn trim_silence(audio: &[f32], threshold: f32) -> Vec<f32> {
    let start = audio.iter()
        .position(|&x| x.abs() > threshold)
        .unwrap_or(0);
    
    let end = audio.iter()
        .rposition(|&x| x.abs() > threshold)
        .unwrap_or(audio.len());
    
    if start >= end {
        return vec![0.0; (16000.0 * 0.1) as usize]; // Return 100ms of silence
    }
    
    audio[start..=end].to_vec()
}

pub fn split_by_silence(
    audio: &[f32], 
    sample_rate: u32, 
    silence_threshold: f32, 
    min_silence_duration_ms: u32
) -> Vec<Vec<f32>> {
    let silence_regions = detect_silence_regions(audio, sample_rate, silence_threshold, min_silence_duration_ms);
    let mut segments = Vec::new();
    let mut last_end = 0;
    
    for (silence_start, silence_end) in silence_regions {
        if silence_start > last_end {
            segments.push(audio[last_end..silence_start].to_vec());
        }
        last_end = silence_end;
    }
    
    // Add final segment
    if last_end < audio.len() {
        segments.push(audio[last_end..].to_vec());
    }
    
    segments
}
```

## ⏱️ Real-time Processing

### Circular Buffer for Streaming

```rust
pub struct CircularBuffer {
    buffer: Vec<f32>,
    write_pos: usize,
    read_pos: usize,
    size: usize,
}

impl CircularBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            write_pos: 0,
            read_pos: 0,
            size,
        }
    }
    
    pub fn write(&mut self, data: &[f32]) {
        for &sample in data {
            self.buffer[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.size;
        }
    }
    
    pub fn read(&mut self, output: &mut [f32]) -> usize {
        let available = self.available_samples();
        let to_read = output.len().min(available);
        
        for i in 0..to_read {
            output[i] = self.buffer[self.read_pos];
            self.read_pos = (self.read_pos + 1) % self.size;
        }
        
        to_read
    }
    
    pub fn available_samples(&self) -> usize {
        if self.write_pos >= self.read_pos {
            self.write_pos - self.read_pos
        } else {
            self.size - self.read_pos + self.write_pos
        }
    }
    
    pub fn get_latest(&self, length: usize) -> Vec<f32> {
        let available = self.available_samples();
        let to_get = length.min(available);
        let mut result = Vec::with_capacity(to_get);
        
        let start_pos = if self.write_pos >= to_get {
            self.write_pos - to_get
        } else {
            self.size - (to_get - self.write_pos)
        };
        
        for i in 0..to_get {
            let pos = (start_pos + i) % self.size;
            result.push(self.buffer[pos]);
        }
        
        result
    }
}
```

### Real-time Transcription Handler

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct RealTimeTranscriber {
    buffer: Arc<Mutex<CircularBuffer>>,
    whisper_instance_id: i32,
    chunk_size: usize,
    overlap_size: usize,
}

impl RealTimeTranscriber {
    pub fn new(whisper_instance_id: i32, chunk_duration_ms: u32, overlap_ms: u32) -> Self {
        let sample_rate = 16000;
        let chunk_size = (sample_rate * chunk_duration_ms / 1000) as usize;
        let overlap_size = (sample_rate * overlap_ms / 1000) as usize;
        let buffer_size = chunk_size * 4; // Buffer for 4 chunks
        
        Self {
            buffer: Arc::new(Mutex::new(CircularBuffer::new(buffer_size))),
            whisper_instance_id,
            chunk_size,
            overlap_size,
        }
    }
    
    pub fn add_audio(&self, audio_data: &[f32]) {
        if let Ok(mut buffer) = self.buffer.lock() {
            buffer.write(audio_data);
        }
    }
    
    pub fn start_processing<F>(&self, callback: F) -> thread::JoinHandle<()>
    where
        F: Fn(String) + Send + 'static,
    {
        let buffer = Arc::clone(&self.buffer);
        let whisper_id = self.whisper_instance_id;
        let chunk_size = self.chunk_size;
        let overlap_size = self.overlap_size;
        
        thread::spawn(move || {
            let mut last_chunk = Vec::new();
            
            loop {
                thread::sleep(Duration::from_millis(100)); // Process every 100ms
                
                let chunk = if let Ok(buffer) = buffer.lock() {
                    if buffer.available_samples() >= chunk_size {
                        buffer.get_latest(chunk_size)
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };
                
                // Skip if chunk is too similar to last processed chunk
                if !last_chunk.is_empty() && chunks_similar(&chunk, &last_chunk, overlap_size) {
                    continue;
                }
                
                // Process with Whisper
                let mut result_buffer = [0i8; 2048];
                let success = unsafe {
                    whisper_rust_process_audio(
                        whisper_id,
                        chunk.as_ptr(),
                        chunk.len() as i32,
                        std::ptr::null(), // Auto-detect language
                        result_buffer.as_mut_ptr(),
                        result_buffer.len() as i32,
                    )
                };
                
                if success {
                    let result = unsafe {
                        std::ffi::CStr::from_ptr(result_buffer.as_ptr())
                    };
                    
                    if let Ok(text) = result.to_str() {
                        if !text.trim().is_empty() {
                            callback(text.to_string());
                        }
                    }
                }
                
                last_chunk = chunk;
            }
        })
    }
}

fn chunks_similar(chunk1: &[f32], chunk2: &[f32], overlap_size: usize) -> bool {
    if chunk1.len() < overlap_size || chunk2.len() < overlap_size {
        return false;
    }
    
    let start1 = chunk1.len() - overlap_size;
    let start2 = chunk2.len() - overlap_size;
    
    let correlation = chunk1[start1..].iter()
        .zip(&chunk2[start2..])
        .map(|(a, b)| a * b)
        .sum::<f32>();
    
    correlation > 0.8 // 80% similarity threshold
}

// External C function declaration
extern "C" {
    fn whisper_rust_process_audio(
        instance_id: i32,
        audio_data: *const f32,
        audio_len: i32,
        language: *const i8,
        result_buffer: *mut i8,
        result_buffer_size: i32,
    ) -> bool;
}
```

## 📊 Quality Analysis

### Audio Quality Metrics

```rust
pub struct AudioQualityAnalyzer {
    sample_rate: u32,
}

impl AudioQualityAnalyzer {
    pub fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }
    
    pub fn analyze(&self, audio: &[f32]) -> AudioQualityReport {
        let peak_amplitude = audio.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
        let rms = (audio.iter().map(|&x| x * x).sum::<f32>() / audio.len() as f32).sqrt();
        let snr = self.estimate_snr(audio);
        let dynamic_range = self.calculate_dynamic_range(audio);
        let clipping_percentage = self.detect_clipping(audio);
        let frequency_response = self.analyze_frequency_response(audio);
        
        AudioQualityReport {
            peak_amplitude,
            rms_level: rms,
            snr_db: snr,
            dynamic_range_db: dynamic_range,
            clipping_percentage,
            frequency_response,
            duration_seconds: audio.len() as f32 / self.sample_rate as f32,
            quality_score: self.calculate_quality_score(peak_amplitude, rms, snr, clipping_percentage),
        }
    }
    
    fn estimate_snr(&self, audio: &[f32]) -> f32 {
        // Simple SNR estimation using signal variance vs noise floor
        let sorted_levels: Vec<f32> = {
            let mut levels: Vec<f32> = audio.iter().map(|&x| x.abs()).collect();
            levels.sort_by(|a, b| a.partial_cmp(b).unwrap());
            levels
        };
        
        let noise_floor = sorted_levels[sorted_levels.len() / 10]; // Bottom 10% as noise
        let signal_level = sorted_levels[sorted_levels.len() * 9 / 10]; // Top 10% as signal
        
        if noise_floor > 0.0 {
            20.0 * (signal_level / noise_floor).log10()
        } else {
            60.0 // Very clean signal
        }
    }
    
    fn calculate_dynamic_range(&self, audio: &[f32]) -> f32 {
        let max_level = audio.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
        let min_level = audio.iter()
            .map(|&x| x.abs())
            .filter(|&x| x > 0.0)
            .fold(f32::INFINITY, f32::min);
        
        if min_level > 0.0 && min_level != f32::INFINITY {
            20.0 * (max_level / min_level).log10()
        } else {
            0.0
        }
    }
    
    fn detect_clipping(&self, audio: &[f32]) -> f32 {
        let clipped_samples = audio.iter()
            .filter(|&&x| x.abs() >= 0.99)
            .count();
        
        (clipped_samples as f32 / audio.len() as f32) * 100.0
    }
    
    fn analyze_frequency_response(&self, audio: &[f32]) -> FrequencyResponse {
        // Simple frequency analysis using energy in different bands
        let low_energy = self.band_energy(audio, 0.0, 300.0);
        let mid_energy = self.band_energy(audio, 300.0, 3000.0);
        let high_energy = self.band_energy(audio, 3000.0, 8000.0);
        
        FrequencyResponse {
            low_freq_energy: low_energy,
            mid_freq_energy: mid_energy,
            high_freq_energy: high_energy,
        }
    }
    
    fn band_energy(&self, audio: &[f32], low_freq: f32, high_freq: f32) -> f32 {
        // Simplified band energy calculation
        // In a real implementation, you would use FFT
        audio.iter().map(|&x| x * x).sum::<f32>() / audio.len() as f32
    }
    
    fn calculate_quality_score(&self, peak: f32, rms: f32, snr: f32, clipping: f32) -> f32 {
        let mut score = 100.0;
        
        // Penalize low signal levels
        if peak < 0.1 {
            score -= 30.0;
        } else if peak < 0.3 {
            score -= 15.0;
        }
        
        // Penalize low SNR
        if snr < 20.0 {
            score -= (20.0 - snr) * 2.0;
        }
        
        // Penalize clipping
        score -= clipping * 5.0;
        
        // Penalize low RMS (too quiet)
        if rms < 0.05 {
            score -= 20.0;
        }
        
        score.max(0.0).min(100.0)
    }
}

#[derive(Debug)]
pub struct AudioQualityReport {
    pub peak_amplitude: f32,
    pub rms_level: f32,
    pub snr_db: f32,
    pub dynamic_range_db: f32,
    pub clipping_percentage: f32,
    pub frequency_response: FrequencyResponse,
    pub duration_seconds: f32,
    pub quality_score: f32,
}

#[derive(Debug)]
pub struct FrequencyResponse {
    pub low_freq_energy: f32,
    pub mid_freq_energy: f32,
    pub high_freq_energy: f32,
}

impl AudioQualityReport {
    pub fn print_analysis(&self) {
        println!("🔊 Audio Quality Analysis");
        println!("========================");
        println!("Duration: {:.2}s", self.duration_seconds);
        println!("Peak Amplitude: {:.3} ({:.1} dB)", self.peak_amplitude, 20.0 * self.peak_amplitude.log10());
        println!("RMS Level: {:.3} ({:.1} dB)", self.rms_level, 20.0 * self.rms_level.log10());
        println!("SNR: {:.1} dB", self.snr_db);
        println!("Dynamic Range: {:.1} dB", self.dynamic_range_db);
        println!("Clipping: {:.1}%", self.clipping_percentage);
        println!("Quality Score: {:.1}/100", self.quality_score);
        
        // Recommendations
        if self.quality_score < 50.0 {
            println!("\n⚠️  Recommendations:");
            if self.peak_amplitude < 0.1 {
                println!("• Signal level too low - increase input gain");
            }
            if self.snr_db < 20.0 {
                println!("• High noise level - consider noise reduction");
            }
            if self.clipping_percentage > 1.0 {
                println!("• Audio clipping detected - reduce input gain");
            }
        } else if self.quality_score > 80.0 {
            println!("\n✅ Audio quality is excellent for transcription");
        }
    }
}
```

## 🔧 Advanced Techniques

### Voice Activity Detection (VAD)

```rust
pub struct VoiceActivityDetector {
    energy_threshold: f32,
    frequency_threshold: f32,
    min_speech_duration: usize,
    min_silence_duration: usize,
    state: VadState,
    speech_counter: usize,
    silence_counter: usize,
}

#[derive(Debug, PartialEq)]
enum VadState {
    Silence,
    Speech,
}

impl VoiceActivityDetector {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            energy_threshold: 0.01,
            frequency_threshold: 0.5,
            min_speech_duration: sample_rate as usize / 10, // 100ms
            min_silence_duration: sample_rate as usize / 5, // 200ms
            state: VadState::Silence,
            speech_counter: 0,
            silence_counter: 0,
        }
    }
    
    pub fn process_frame(&mut self, frame: &[f32]) -> bool {
        let energy = self.calculate_energy(frame);
        let spectral_centroid = self.calculate_spectral_centroid(frame);
        
        let is_speech = energy > self.energy_threshold && 
                       spectral_centroid > self.frequency_threshold;
        
        match self.state {
            VadState::Silence => {
                if is_speech {
                    self.speech_counter += frame.len();
                    if self.speech_counter >= self.min_speech_duration {
                        self.state = VadState::Speech;
                        self.silence_counter = 0;
                        return true;
                    }
                } else {
                    self.speech_counter = 0;
                }
            }
            VadState::Speech => {
                if !is_speech {
                    self.silence_counter += frame.len();
                    if self.silence_counter >= self.min_silence_duration {
                        self.state = VadState::Silence;
                        self.speech_counter = 0;
                        return false;
                    }
                } else {
                    self.silence_counter = 0;
                }
                return true;
            }
        }
        
        false
    }
    
    fn calculate_energy(&self, frame: &[f32]) -> f32 {
        frame.iter().map(|&x| x * x).sum::<f32>() / frame.len() as f32
    }
    
    fn calculate_spectral_centroid(&self, frame: &[f32]) -> f32 {
        // Simplified spectral centroid calculation
        // In a real implementation, you would use FFT
        let mut weighted_sum = 0.0;
        let mut magnitude_sum = 0.0;
        
        for (i, &sample) in frame.iter().enumerate() {
            let magnitude = sample.abs();
            weighted_sum += (i as f32) * magnitude;
            magnitude_sum += magnitude;
        }
        
        if magnitude_sum > 0.0 {
            weighted_sum / magnitude_sum
        } else {
            0.0
        }
    }
}
```

### Audio Enhancement Pipeline

```rust
pub struct AudioEnhancementPipeline {
    noise_gate: NoiseGate,
    agc: AutomaticGainControl,
    high_pass: HighPassFilter,
    vad: VoiceActivityDetector,
}

impl AudioEnhancementPipeline {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            noise_gate: NoiseGate::new(0.02, 2.0, 10.0, 100.0, sample_rate),
            agc: AutomaticGainControl::new(0.3, 10.0, 50.0, 200.0, sample_rate),
            high_pass: HighPassFilter::new(80.0, sample_rate),
            vad: VoiceActivityDetector::new(sample_rate),
        }
    }
    
    pub fn process(&mut self, input: &[f32]) -> ProcessedAudio {
        // Step 1: High-pass filter to remove low frequency noise
        let filtered = self.high_pass.process(input);
        
        // Step 2: Noise gate to reduce background noise
        let gated = self.noise_gate.process(&filtered);
        
        // Step 3: Automatic gain control for consistent levels
        let normalized = self.agc.process(&gated);
        
        // Step 4: Voice activity detection
        let has_speech = self.vad.process_frame(&normalized);
        
        ProcessedAudio {
            audio: normalized,
            has_speech,
            processing_info: ProcessingInfo {
                input_peak: input.iter().map(|&x| x.abs()).fold(0.0f32, f32::max),
                output_peak: normalized.iter().map(|&x| x.abs()).fold(0.0f32, f32::max),
                gain_applied: 1.0, // Would be calculated by AGC
            },
        }
    }
}

pub struct ProcessedAudio {
    pub audio: Vec<f32>,
    pub has_speech: bool,
    pub processing_info: ProcessingInfo,
}

pub struct ProcessingInfo {
    pub input_peak: f32,
    pub output_peak: f32,
    pub gain_applied: f32,
}

struct HighPassFilter {
    alpha: f32,
    prev_input: f32,
    prev_output: f32,
}

impl HighPassFilter {
    fn new(cutoff_hz: f32, sample_rate: u32) -> Self {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz);
        let dt = 1.0 / sample_rate as f32;
        let alpha = rc / (rc + dt);
        
        Self {
            alpha,
            prev_input: 0.0,
            prev_output: 0.0,
        }
    }
    
    fn process(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output = Vec::with_capacity(input.len());
        
        for &sample in input {
            let filtered = self.alpha * (self.prev_output + sample - self.prev_input);
            output.push(filtered);
            
            self.prev_input = sample;
            self.prev_output = filtered;
        }
        
        output
    }
}
```

This comprehensive audio processing guide provides all the tools needed to handle audio input, processing, and optimization for the Whisper Rust Binding library.
