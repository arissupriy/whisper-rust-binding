# 🔧 Service Layer Implementation
## Business Logic untuk Dual Library Integration

### 🎯 Overview

Service layer yang mengimplementasikan business logic untuk komunikasi dengan native libraries, audio processing, dan data management dengan proper error handling.

### 🏗️ Service Architecture

```
Services Layer
├── BridgeService (FRB Communication)
├── WhisperService (Transcription Logic)
├── AudioService (Recording & Streaming)
├── QuranService (Validation & Search)
├── PermissionService (Platform Permissions)
├── FileService (Model & Audio Files)
└── ConfigService (App Configuration)
```

### 🔧 Core Services

#### 1. lib/services/whisper_service.dart

```dart
import 'dart:async';
import 'dart:typed_data';
import '../generated/bridge_generated.dart';
import '../models/whisper_model.dart';
import '../models/transcription_result.dart';
import '../models/errors.dart';
import '../utils/constants.dart';

class WhisperService {
  final DualLibraryApi _api;
  final Map<int, WhisperInstance> _instances = {};
  
  WhisperService(this._api);

  /// Load whisper model and create instance
  Future<WhisperInstance> loadModel(String modelPath) async {
    try {
      // Validate model file exists
      await _validateModelFile(modelPath);
      
      // Initialize whisper with model
      final instanceId = await _api.whisperInit(modelPath: modelPath);
      
      if (instanceId < 0) {
        throw const AppError.whisperError(
          message: 'Failed to initialize whisper instance',
          type: WhisperErrorType.initializationFailed,
        );
      }

      // Get model information
      final modelInfo = await _getModelInfo(instanceId);
      
      // Create instance object
      final instance = WhisperInstance(
        id: instanceId,
        modelPath: modelPath,
        modelInfo: modelInfo,
        createdAt: DateTime.now(),
        isActive: true,
      );
      
      _instances[instanceId] = instance;
      
      print('✅ Whisper model loaded: ${modelInfo.name} (ID: $instanceId)');
      return instance;
      
    } catch (e) {
      if (e is AppError) rethrow;
      
      throw AppError.whisperError(
        message: 'Model loading failed: $e',
        type: WhisperErrorType.initializationFailed,
        modelPath: modelPath,
      );
    }
  }

  /// Free whisper instance
  Future<void> freeInstance(int instanceId) async {
    try {
      final success = await _api.whisperFree(instanceId: instanceId);
      
      if (!success) {
        throw AppError.whisperError(
          message: 'Failed to free whisper instance',
          type: WhisperErrorType.instanceNotFound,
          instanceId: instanceId,
        );
      }
      
      _instances.remove(instanceId);
      print('✅ Whisper instance freed: $instanceId');
      
    } catch (e) {
      if (e is AppError) rethrow;
      
      throw AppError.whisperError(
        message: 'Instance cleanup failed: $e',
        type: WhisperErrorType.instanceNotFound,
        instanceId: instanceId,
      );
    }
  }

  /// Transcribe audio data
  Future<TranscriptionResult> transcribeAudio({
    required int instanceId,
    required List<double> audioData,
    required WhisperConfig config,
  }) async {
    try {
      final startTime = DateTime.now();
      
      // Validate instance
      final instance = _validateInstance(instanceId);
      
      // Validate audio data
      _validateAudioData(audioData, config);
      
      // Process audio with whisper
      final resultText = await _api.whisperProcessAudio(
        instanceId: instanceId,
        audioData: audioData,
        language: config.language,
      );
      
      final endTime = DateTime.now();
      final processingTime = endTime.difference(startTime).inMilliseconds / 1000.0;
      
      // Parse result and create segments
      final segments = _parseSegments(resultText, audioData.length, config);
      
      // Calculate overall confidence
      final confidence = _calculateOverallConfidence(segments);
      
      // Create transcription result
      final result = TranscriptionResult(
        id: _generateTranscriptionId(),
        text: resultText.trim(),
        confidence: confidence,
        segments: segments,
        timestamp: startTime,
        language: config.language,
        metadata: TranscriptionMetadata(
          processingTimeSec: processingTime,
          audioLengthSec: audioData.length / config.sampleRate,
          instanceId: instanceId,
          config: config,
          totalSegments: segments.length,
          totalWords: _countWords(resultText),
        ),
      );
      
      print('✅ Transcription completed: ${result.text.substring(0, 50)}...');
      return result;
      
    } catch (e) {
      if (e is AppError) rethrow;
      
      throw AppError.whisperError(
        message: 'Transcription failed: $e',
        type: WhisperErrorType.transcriptionFailed,
        instanceId: instanceId,
      );
    }
  }

  /// Transcribe with sliding window
  Future<TranscriptionResult> transcribeWithSlidingWindow({
    required int instanceId,
    required List<double> audioData,
    required WhisperConfig config,
  }) async {
    try {
      final startTime = DateTime.now();
      
      // Validate instance
      final instance = _validateInstance(instanceId);
      
      // Validate audio data
      _validateAudioData(audioData, config);
      
      // Process with sliding window
      final resultText = await _api.whisperProcessAudioSlidingWindow(
        instanceId: instanceId,
        audioData: audioData,
        windowSizeSec: config.windowSizeSec,
        stepSizeSec: config.stepSizeSec,
        sampleRate: config.sampleRate,
        language: config.language,
      );
      
      final endTime = DateTime.now();
      final processingTime = endTime.difference(startTime).inMilliseconds / 1000.0;
      
      // Parse segments with timing information
      final segments = _parseSlidingWindowSegments(
        resultText, 
        audioData.length, 
        config,
      );
      
      // Calculate confidence
      final confidence = _calculateOverallConfidence(segments);
      
      final result = TranscriptionResult(
        id: _generateTranscriptionId(),
        text: resultText.trim(),
        confidence: confidence,
        segments: segments,
        timestamp: startTime,
        language: config.language,
        metadata: TranscriptionMetadata(
          processingTimeSec: processingTime,
          audioLengthSec: audioData.length / config.sampleRate,
          instanceId: instanceId,
          config: config,
          totalSegments: segments.length,
          totalWords: _countWords(resultText),
        ),
      );
      
      print('✅ Sliding window transcription: ${segments.length} segments');
      return result;
      
    } catch (e) {
      if (e is AppError) rethrow;
      
      throw AppError.whisperError(
        message: 'Sliding window transcription failed: $e',
        type: WhisperErrorType.transcriptionFailed,
        instanceId: instanceId,
      );
    }
  }

  /// Get active instances
  Future<List<WhisperInstance>> getActiveInstances() async {
    return _instances.values.where((instance) => instance.isActive).toList();
  }

  /// Validate instance exists
  WhisperInstance _validateInstance(int instanceId) {
    final instance = _instances[instanceId];
    if (instance == null || !instance.isActive) {
      throw AppError.whisperError(
        message: 'Whisper instance not found or inactive',
        type: WhisperErrorType.instanceNotFound,
        instanceId: instanceId,
      );
    }
    return instance;
  }

  /// Validate model file
  Future<void> _validateModelFile(String modelPath) async {
    // Implementation depends on your file system access
    // For now, we'll assume the path is valid
    if (modelPath.isEmpty || !modelPath.endsWith('.bin')) {
      throw const AppError.whisperError(
        message: 'Invalid model file path',
        type: WhisperErrorType.invalidModelFormat,
      );
    }
  }

  /// Get model information
  Future<WhisperModelInfo> _getModelInfo(int instanceId) async {
    try {
      // This would typically call a native function to get model info
      // For now, we'll return default info
      return const WhisperModelInfo(
        name: 'Whisper Model',
        version: '1.0.0',
        vocabularySize: 51864,
        supportedLanguages: ['ar', 'en', 'auto'],
        fileSizeMB: 39.0,
        modelType: 'tiny',
      );
    } catch (e) {
      throw AppError.whisperError(
        message: 'Failed to get model info: $e',
        type: WhisperErrorType.configurationError,
        instanceId: instanceId,
      );
    }
  }

  /// Validate audio data
  void _validateAudioData(List<double> audioData, WhisperConfig config) {
    if (audioData.isEmpty) {
      throw const AppError.audioError(
        message: 'Audio data is empty',
        type: AudioErrorType.invalidFormat,
      );
    }
    
    final minSamples = config.sampleRate * 0.1; // Minimum 0.1 seconds
    if (audioData.length < minSamples) {
      throw AppError.audioError(
        message: 'Audio too short: ${audioData.length} samples',
        type: AudioErrorType.invalidFormat,
      );
    }
    
    final maxSamples = config.sampleRate * 600; // Maximum 10 minutes
    if (audioData.length > maxSamples) {
      throw AppError.audioError(
        message: 'Audio too long: ${audioData.length} samples',
        type: AudioErrorType.invalidFormat,
      );
    }
  }

  /// Parse segments from result text
  List<TranscriptionSegment> _parseSegments(
    String resultText, 
    int audioLength, 
    WhisperConfig config,
  ) {
    // Simple implementation - split by sentences
    // In production, you'd parse actual timing data from whisper
    final sentences = resultText.split('.').where((s) => s.trim().isNotEmpty);
    final segments = <TranscriptionSegment>[];
    
    final audioLengthSec = audioLength / config.sampleRate;
    final segmentDuration = audioLengthSec / sentences.length;
    
    for (int i = 0; i < sentences.length; i++) {
      final sentence = sentences.elementAt(i).trim();
      if (sentence.isNotEmpty) {
        segments.add(TranscriptionSegment(
          id: i,
          text: sentence,
          startTime: i * segmentDuration,
          endTime: (i + 1) * segmentDuration,
          confidence: 0.8 + (0.2 * (sentence.length / 100).clamp(0, 1)),
          words: _parseWords(sentence, i * segmentDuration, segmentDuration),
        ));
      }
    }
    
    return segments;
  }

  /// Parse words with timing
  List<TranscriptionWord> _parseWords(
    String segmentText, 
    double startTime, 
    double segmentDuration,
  ) {
    final words = segmentText.split(' ').where((w) => w.trim().isNotEmpty);
    final wordList = <TranscriptionWord>[];
    
    final wordDuration = segmentDuration / words.length;
    
    for (int i = 0; i < words.length; i++) {
      final word = words.elementAt(i).trim();
      if (word.isNotEmpty) {
        wordList.add(TranscriptionWord(
          text: word,
          startTime: startTime + (i * wordDuration),
          endTime: startTime + ((i + 1) * wordDuration),
          confidence: 0.75 + (0.25 * (word.length / 10).clamp(0, 1)),
          isValidArabic: _isArabicWord(word),
        ));
      }
    }
    
    return wordList;
  }

  /// Parse sliding window segments
  List<TranscriptionSegment> _parseSlidingWindowSegments(
    String resultText,
    int audioLength,
    WhisperConfig config,
  ) {
    // More sophisticated parsing for sliding window results
    // This would include overlap handling and segment merging
    return _parseSegments(resultText, audioLength, config);
  }

  /// Calculate overall confidence
  double _calculateOverallConfidence(List<TranscriptionSegment> segments) {
    if (segments.isEmpty) return 0.0;
    
    final totalConfidence = segments.fold<double>(
      0.0, 
      (sum, segment) => sum + segment.confidence,
    );
    
    return totalConfidence / segments.length;
  }

  /// Check if word is Arabic
  bool _isArabicWord(String word) {
    return RegExp(r'[\u0600-\u06FF]').hasMatch(word);
  }

  /// Count words in text
  int _countWords(String text) {
    return text.trim().split(' ').where((w) => w.isNotEmpty).length;
  }

  /// Generate unique transcription ID
  String _generateTranscriptionId() {
    return 'trans_${DateTime.now().millisecondsSinceEpoch}';
  }

  /// Cleanup all instances
  Future<void> dispose() async {
    final instanceIds = _instances.keys.toList();
    for (final id in instanceIds) {
      try {
        await freeInstance(id);
      } catch (e) {
        print('⚠️ Error freeing instance $id: $e');
      }
    }
    _instances.clear();
  }
}
```

#### 2. lib/services/audio_service.dart

```dart
import 'dart:async';
import 'dart:typed_data';
import 'dart:io';
import 'package:record/record.dart';
import 'package:path_provider/path_provider.dart';
import 'package:path/path.dart' as path;
import '../models/audio_data.dart';
import '../models/errors.dart';
import '../utils/constants.dart';

class AudioService {
  final AudioRecorder _recorder = AudioRecorder();
  StreamController<Float32List>? _audioStreamController;
  Timer? _amplitudeTimer;
  String? _currentRecordingPath;
  DateTime? _recordingStartTime;

  /// Check if recording is supported
  Future<bool> isRecordingSupported() async {
    return await _recorder.hasPermission();
  }

  /// Start recording audio
  Future<void> startRecording() async {
    try {
      // Check permission
      final hasPermission = await _recorder.hasPermission();
      if (!hasPermission) {
        throw const AppError.permissionError(
          message: 'Microphone permission not granted',
          permission: 'microphone',
        );
      }

      // Generate file path
      final directory = await getTemporaryDirectory();
      final fileName = 'recording_${DateTime.now().millisecondsSinceEpoch}.wav';
      _currentRecordingPath = path.join(directory.path, fileName);

      // Configure recording
      const config = RecordConfig(
        encoder: AudioEncoder.wav,
        bitRate: 128000,
        sampleRate: WhisperConstants.sampleRate,
        numChannels: WhisperConstants.channels,
      );

      // Start recording
      await _recorder.start(config, path: _currentRecordingPath!);
      _recordingStartTime = DateTime.now();

      print('✅ Recording started: $_currentRecordingPath');
      
    } catch (e) {
      _currentRecordingPath = null;
      _recordingStartTime = null;
      
      throw AppError.audioError(
        message: 'Failed to start recording: $e',
        type: AudioErrorType.recordingFailed,
      );
    }
  }

  /// Stop recording and return audio data
  Future<AudioData?> stopRecording() async {
    try {
      if (_currentRecordingPath == null) {
        throw const AppError.audioError(
          message: 'No active recording to stop',
          type: AudioErrorType.recordingFailed,
        );
      }

      // Stop recording
      final recordPath = await _recorder.stop();
      
      if (recordPath == null) {
        throw const AppError.audioError(
          message: 'Recording failed to produce output',
          type: AudioErrorType.recordingFailed,
        );
      }

      // Read audio file and convert to audio data
      final audioData = await _convertFileToAudioData(recordPath);
      
      print('✅ Recording stopped: ${audioData.durationSec}s');
      return audioData;
      
    } catch (e) {
      if (e is AppError) rethrow;
      
      throw AppError.audioError(
        message: 'Failed to stop recording: $e',
        type: AudioErrorType.recordingFailed,
      );
    } finally {
      _currentRecordingPath = null;
      _recordingStartTime = null;
    }
  }

  /// Pause recording
  Future<void> pauseRecording() async {
    try {
      await _recorder.pause();
      print('⏸️ Recording paused');
    } catch (e) {
      throw AppError.audioError(
        message: 'Failed to pause recording: $e',
        type: AudioErrorType.recordingFailed,
      );
    }
  }

  /// Resume recording
  Future<void> resumeRecording() async {
    try {
      await _recorder.resume();
      print('▶️ Recording resumed');
    } catch (e) {
      throw AppError.audioError(
        message: 'Failed to resume recording: $e',
        type: AudioErrorType.recordingFailed,
      );
    }
  }

  /// Get current recording amplitude
  Future<double> getCurrentAmplitude() async {
    try {
      final amplitude = await _recorder.getAmplitude();
      return amplitude.current.clamp(0.0, 1.0);
    } catch (e) {
      return 0.0;
    }
  }

  /// Get current recording duration
  Future<double> getCurrentDuration() async {
    if (_recordingStartTime == null) return 0.0;
    
    final now = DateTime.now();
    final duration = now.difference(_recordingStartTime!);
    return duration.inMilliseconds / 1000.0;
  }

  /// Get real-time audio stream
  Future<Stream<Float32List>> getAudioStream() async {
    try {
      // Check permission
      final hasPermission = await _recorder.hasPermission();
      if (!hasPermission) {
        throw const AppError.permissionError(
          message: 'Microphone permission required for streaming',
          permission: 'microphone',
        );
      }

      // Create stream controller
      _audioStreamController = StreamController<Float32List>.broadcast();

      // Start streaming (implementation depends on platform)
      // This is a simplified example - real implementation would use 
      // platform-specific streaming APIs
      
      return _audioStreamController!.stream;
      
    } catch (e) {
      throw AppError.audioError(
        message: 'Failed to start audio stream: $e',
        type: AudioErrorType.deviceNotAvailable,
      );
    }
  }

  /// Stop audio stream
  Future<void> stopAudioStream() async {
    try {
      await _audioStreamController?.close();
      _audioStreamController = null;
      print('✅ Audio stream stopped');
    } catch (e) {
      throw AppError.audioError(
        message: 'Failed to stop audio stream: $e',
        type: AudioErrorType.recordingFailed,
      );
    }
  }

  /// Convert audio file to AudioData
  Future<AudioData> _convertFileToAudioData(String filePath) async {
    try {
      final file = File(filePath);
      
      if (!await file.exists()) {
        throw AppError.audioError(
          message: 'Audio file not found',
          type: AudioErrorType.fileNotFound,
          filePath: filePath,
        );
      }

      // Read file bytes
      final bytes = await file.readAsBytes();
      
      // Parse WAV file (simplified - real implementation would use audio library)
      final samples = _parseWavFile(bytes);
      
      // Calculate duration
      final durationSec = samples.length / WhisperConstants.sampleRate;
      
      return AudioData(
        id: 'audio_${DateTime.now().millisecondsSinceEpoch}',
        samples: Float32List.fromList(samples),
        format: const AudioFormat(
          sampleRate: WhisperConstants.sampleRate,
          channels: WhisperConstants.channels,
          bitDepth: 16,
          encoding: AudioEncoding.wav,
        ),
        recordedAt: DateTime.now(),
        durationSec: durationSec,
        filePath: filePath,
      );
      
    } catch (e) {
      if (e is AppError) rethrow;
      
      throw AppError.audioError(
        message: 'Failed to convert audio file: $e',
        type: AudioErrorType.invalidFormat,
        filePath: filePath,
      );
    }
  }

  /// Parse WAV file to get samples (simplified implementation)
  List<double> _parseWavFile(Uint8List bytes) {
    // This is a very simplified WAV parser
    // In production, use a proper audio library like dart:audio
    
    try {
      // Skip WAV header (44 bytes typically)
      final headerSize = 44;
      if (bytes.length < headerSize) {
        throw const AppError.audioError(
          message: 'Invalid WAV file: too small',
          type: AudioErrorType.invalidFormat,
        );
      }

      // Extract audio data (assuming 16-bit PCM)
      final audioBytes = bytes.sublist(headerSize);
      final samples = <double>[];
      
      for (int i = 0; i < audioBytes.length - 1; i += 2) {
        // Convert 16-bit samples to normalized float
        final sample = (audioBytes[i] | (audioBytes[i + 1] << 8));
        final signedSample = sample > 32767 ? sample - 65536 : sample;
        final normalizedSample = signedSample / 32768.0;
        samples.add(normalizedSample);
      }
      
      return samples;
      
    } catch (e) {
      throw AppError.audioError(
        message: 'WAV parsing failed: $e',
        type: AudioErrorType.invalidFormat,
      );
    }
  }

  /// Cleanup resources
  Future<void> dispose() async {
    try {
      await _recorder.dispose();
      await _audioStreamController?.close();
      _amplitudeTimer?.cancel();
    } catch (e) {
      print('⚠️ Error disposing audio service: $e');
    }
  }
}
```

### 🔄 Next Steps

1. ✅ Services selesai → Lanjut ke `06-ui-components.md`
2. Create UI components yang menggunakan services ini
3. Implement real-time transcription
4. Build complete examples

### 🐛 Common Issues

**Issue**: Recording permission denied
```dart
// Solution: Handle permission properly
final hasPermission = await _recorder.hasPermission();
if (!hasPermission) {
  // Request permission or show error
}
```

**Issue**: Audio file parsing fails
```dart
// Solution: Use proper audio library
// Consider packages like: flutter_audio_waveforms, audioplayers
```

**Issue**: Memory issues with large audio files
```dart
// Solution: Process audio in chunks
final chunkSize = 16000 * 10; // 10 seconds
// Process audio in smaller chunks
```
