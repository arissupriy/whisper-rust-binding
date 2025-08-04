# 🏪 Riverpod Providers
## State Management untuk Dual Library Integration

### 🎯 Overview

Riverpod providers untuk mengelola state whisper transcription, audio recording, dan Quran validation dengan architecture yang scalable dan testable.

### 🏗️ Provider Architecture

```
AppProviders
├── WhisperProviders
│   ├── whisperInstancesProvider
│   ├── activeInstanceProvider
│   └── transcriptionHistoryProvider
├── AudioProviders
│   ├── recordingStateProvider
│   ├── audioStreamProvider
│   └── audioPermissionProvider
├── QuranProviders
│   ├── quranDatabaseProvider
│   ├── validationProvider
│   └── searchProvider
└── UIProviders
    ├── appStateProvider
    ├── processingStateProvider
    └── errorProvider
```

### 🔧 Core Providers

#### 1. lib/providers/whisper_provider.dart

```dart
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../services/bridge_service.dart';
import '../services/whisper_service.dart';
import '../models/whisper_model.dart';
import '../models/transcription_result.dart';
import '../models/errors.dart';

part 'whisper_provider.g.dart';

// Whisper Service Provider
@riverpod
WhisperService whisperService(WhisperServiceRef ref) {
  return WhisperService(BridgeService.api);
}

// Whisper Instances Provider
@riverpod
class WhisperInstances extends _$WhisperInstances {
  @override
  Future<List<WhisperInstance>> build() async {
    return [];
  }

  /// Load model and create instance
  Future<WhisperInstance> loadModel(String modelPath) async {
    state = const AsyncValue.loading();
    
    try {
      final service = ref.read(whisperServiceProvider);
      final instance = await service.loadModel(modelPath);
      
      final currentInstances = state.valueOrNull ?? [];
      final updatedInstances = [...currentInstances, instance];
      
      state = AsyncValue.data(updatedInstances);
      
      // Set as active instance if it's the first one
      if (currentInstances.isEmpty) {
        ref.read(activeInstanceProvider.notifier).setActiveInstance(instance);
      }
      
      return instance;
    } catch (e, stackTrace) {
      final error = AppError.whisperError(
        message: 'Failed to load model: $e',
        type: WhisperErrorType.initializationFailed,
        modelPath: modelPath,
      );
      state = AsyncValue.error(error, stackTrace);
      rethrow;
    }
  }

  /// Remove instance
  Future<void> removeInstance(int instanceId) async {
    try {
      final service = ref.read(whisperServiceProvider);
      await service.freeInstance(instanceId);
      
      final currentInstances = state.valueOrNull ?? [];
      final updatedInstances = currentInstances
          .where((instance) => instance.id != instanceId)
          .toList();
      
      state = AsyncValue.data(updatedInstances);
      
      // Clear active instance if it was removed
      final activeInstance = ref.read(activeInstanceProvider).valueOrNull;
      if (activeInstance?.id == instanceId) {
        ref.read(activeInstanceProvider.notifier).clearActiveInstance();
      }
    } catch (e, stackTrace) {
      final error = AppError.whisperError(
        message: 'Failed to remove instance: $e',
        type: WhisperErrorType.instanceNotFound,
        instanceId: instanceId,
      );
      state = AsyncValue.error(error, stackTrace);
    }
  }

  /// Refresh instances
  Future<void> refresh() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(() async {
      final service = ref.read(whisperServiceProvider);
      return await service.getActiveInstances();
    });
  }
}

// Active Instance Provider
@riverpod
class ActiveInstance extends _$ActiveInstance {
  @override
  Future<WhisperInstance?> build() async {
    return null;
  }

  void setActiveInstance(WhisperInstance instance) {
    state = AsyncValue.data(instance);
  }

  void clearActiveInstance() {
    state = const AsyncValue.data(null);
  }

  /// Get active instance or throw error
  WhisperInstance requireActiveInstance() {
    final instance = state.valueOrNull;
    if (instance == null) {
      throw const AppError.whisperError(
        message: 'No active whisper instance',
        type: WhisperErrorType.instanceNotFound,
      );
    }
    return instance;
  }
}

// Transcription Provider
@riverpod
class TranscriptionController extends _$TranscriptionController {
  @override
  Future<TranscriptionResult?> build() async {
    return null;
  }

  /// Transcribe audio data
  Future<TranscriptionResult> transcribeAudio({
    required List<double> audioData,
    WhisperConfig? config,
  }) async {
    state = const AsyncValue.loading();
    
    try {
      final activeInstance = ref.read(activeInstanceProvider.notifier)
          .requireActiveInstance();
      
      final service = ref.read(whisperServiceProvider);
      final result = await service.transcribeAudio(
        instanceId: activeInstance.id,
        audioData: audioData,
        config: config ?? const WhisperConfig(),
      );
      
      state = AsyncValue.data(result);
      
      // Add to history
      ref.read(transcriptionHistoryProvider.notifier)
          .addTranscription(result);
      
      return result;
    } catch (e, stackTrace) {
      final error = AppError.whisperError(
        message: 'Transcription failed: $e',
        type: WhisperErrorType.transcriptionFailed,
      );
      state = AsyncValue.error(error, stackTrace);
      rethrow;
    }
  }

  /// Transcribe with sliding window
  Future<TranscriptionResult> transcribeWithSlidingWindow({
    required List<double> audioData,
    required WhisperConfig config,
  }) async {
    state = const AsyncValue.loading();
    
    try {
      final activeInstance = ref.read(activeInstanceProvider.notifier)
          .requireActiveInstance();
      
      final service = ref.read(whisperServiceProvider);
      final result = await service.transcribeWithSlidingWindow(
        instanceId: activeInstance.id,
        audioData: audioData,
        config: config,
      );
      
      state = AsyncValue.data(result);
      
      // Add to history
      ref.read(transcriptionHistoryProvider.notifier)
          .addTranscription(result);
      
      return result;
    } catch (e, stackTrace) {
      final error = AppError.whisperError(
        message: 'Sliding window transcription failed: $e',
        type: WhisperErrorType.transcriptionFailed,
      );
      state = AsyncValue.error(error, stackTrace);
      rethrow;
    }
  }

  /// Clear current transcription
  void clearTranscription() {
    state = const AsyncValue.data(null);
  }
}

// Transcription History Provider
@riverpod
class TranscriptionHistory extends _$TranscriptionHistory {
  @override
  Future<List<TranscriptionResult>> build() async {
    return [];
  }

  void addTranscription(TranscriptionResult result) {
    final currentHistory = state.valueOrNull ?? [];
    final updatedHistory = [result, ...currentHistory];
    
    // Keep only last 50 transcriptions
    final limitedHistory = updatedHistory.take(50).toList();
    
    state = AsyncValue.data(limitedHistory);
  }

  void removeTranscription(String transcriptionId) {
    final currentHistory = state.valueOrNull ?? [];
    final updatedHistory = currentHistory
        .where((result) => result.id != transcriptionId)
        .toList();
    
    state = AsyncValue.data(updatedHistory);
  }

  void clearHistory() {
    state = const AsyncValue.data([]);
  }

  /// Search transcriptions
  List<TranscriptionResult> searchTranscriptions(String query) {
    final history = state.valueOrNull ?? [];
    if (query.isEmpty) return history;
    
    return history.where((result) {
      return result.text.toLowerCase().contains(query.toLowerCase()) ||
             result.segments.any((segment) => 
                 segment.text.toLowerCase().contains(query.toLowerCase()));
    }).toList();
  }
}

// Whisper Configuration Provider
@riverpod
class WhisperConfigNotifier extends _$WhisperConfigNotifier {
  @override
  WhisperConfig build() {
    return const WhisperConfig();
  }

  void updateConfig(WhisperConfig newConfig) {
    state = newConfig;
  }

  void updateLanguage(String language) {
    state = state.copyWith(language: language);
  }

  void updateSampleRate(int sampleRate) {
    state = state.copyWith(sampleRate: sampleRate);
  }

  void updateWindowSize(double windowSizeSec) {
    state = state.copyWith(windowSizeSec: windowSizeSec);
  }

  void updateStepSize(double stepSizeSec) {
    state = state.copyWith(stepSizeSec: stepSizeSec);
  }

  void updateConfidenceThreshold(double threshold) {
    state = state.copyWith(confidenceThreshold: threshold);
  }

  void resetToDefaults() {
    state = const WhisperConfig();
  }
}
```

#### 2. lib/providers/audio_provider.dart

```dart
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'dart:async';
import 'dart:typed_data';
import '../services/audio_service.dart';
import '../services/permission_service.dart';
import '../models/audio_data.dart';
import '../models/errors.dart';

part 'audio_provider.g.dart';

// Audio Service Provider
@riverpod
AudioService audioService(AudioServiceRef ref) {
  return AudioService();
}

// Permission Service Provider
@riverpod
PermissionService permissionService(PermissionServiceRef ref) {
  return PermissionService();
}

// Audio Permission Provider
@riverpod
class AudioPermission extends _$AudioPermission {
  @override
  Future<bool> build() async {
    final service = ref.read(permissionServiceProvider);
    return await service.checkMicrophonePermission();
  }

  Future<bool> requestPermission() async {
    final service = ref.read(permissionServiceProvider);
    final granted = await service.requestMicrophonePermission();
    
    state = AsyncValue.data(granted);
    return granted;
  }

  Future<void> openSettings() async {
    final service = ref.read(permissionServiceProvider);
    await service.openPermissionSettings();
  }
}

// Recording State Provider
@riverpod
class RecordingStateNotifier extends _$RecordingStateNotifier {
  Timer? _amplitudeTimer;
  
  @override
  RecordingState build() {
    return const RecordingState();
  }

  /// Start recording
  Future<void> startRecording() async {
    try {
      // Check permission first
      final hasPermission = await ref.read(audioPermissionProvider.future);
      if (!hasPermission) {
        final granted = await ref.read(audioPermissionProvider.notifier)
            .requestPermission();
        if (!granted) {
          throw const AppError.permissionError(
            message: 'Microphone permission denied',
            permission: 'microphone',
          );
        }
      }

      final audioService = ref.read(audioServiceProvider);
      await audioService.startRecording();
      
      state = state.copyWith(
        isRecording: true,
        isPaused: false,
        startTime: DateTime.now(),
        errorMessage: null,
      );
      
      // Start amplitude monitoring
      _startAmplitudeMonitoring();
      
    } catch (e) {
      state = state.copyWith(
        isRecording: false,
        errorMessage: e.toString(),
      );
      rethrow;
    }
  }

  /// Stop recording
  Future<AudioData?> stopRecording() async {
    try {
      _stopAmplitudeMonitoring();
      
      final audioService = ref.read(audioServiceProvider);
      final audioData = await audioService.stopRecording();
      
      state = const RecordingState();
      
      return audioData;
    } catch (e) {
      state = state.copyWith(
        errorMessage: e.toString(),
      );
      rethrow;
    }
  }

  /// Pause recording
  Future<void> pauseRecording() async {
    try {
      final audioService = ref.read(audioServiceProvider);
      await audioService.pauseRecording();
      
      state = state.copyWith(isPaused: true);
      _stopAmplitudeMonitoring();
    } catch (e) {
      state = state.copyWith(errorMessage: e.toString());
      rethrow;
    }
  }

  /// Resume recording
  Future<void> resumeRecording() async {
    try {
      final audioService = ref.read(audioServiceProvider);
      await audioService.resumeRecording();
      
      state = state.copyWith(isPaused: false);
      _startAmplitudeMonitoring();
    } catch (e) {
      state = state.copyWith(errorMessage: e.toString());
      rethrow;
    }
  }

  void _startAmplitudeMonitoring() {
    _amplitudeTimer = Timer.periodic(
      const Duration(milliseconds: 100),
      (timer) async {
        try {
          final audioService = ref.read(audioServiceProvider);
          final amplitude = await audioService.getCurrentAmplitude();
          final duration = await audioService.getCurrentDuration();
          
          state = state.copyWith(
            currentAmplitude: amplitude,
            currentDurationSec: duration,
          );
        } catch (e) {
          // Ignore amplitude errors
        }
      },
    );
  }

  void _stopAmplitudeMonitoring() {
    _amplitudeTimer?.cancel();
    _amplitudeTimer = null;
  }

  @override
  void dispose() {
    _stopAmplitudeMonitoring();
    super.dispose();
  }
}

// Audio Stream Provider for Real-time
@riverpod
class AudioStream extends _$AudioStream {
  StreamSubscription<Float32List>? _audioSubscription;
  
  @override
  Stream<Float32List>? build() {
    return null;
  }

  /// Start real-time audio stream
  Future<void> startStream() async {
    try {
      // Check permission
      final hasPermission = await ref.read(audioPermissionProvider.future);
      if (!hasPermission) {
        throw const AppError.permissionError(
          message: 'Microphone permission required for streaming',
          permission: 'microphone',
        );
      }

      final audioService = ref.read(audioServiceProvider);
      final audioStream = await audioService.getAudioStream();
      
      state = audioStream;
      
      // Listen to stream for real-time transcription
      _audioSubscription = audioStream.listen(
        (audioChunk) {
          // Trigger real-time transcription
          ref.read(realtimeTranscriptionProvider.notifier)
              .processAudioChunk(audioChunk);
        },
        onError: (error) {
          state = null;
          throw AppError.audioError(
            message: 'Audio stream error: $error',
            type: AudioErrorType.recordingFailed,
          );
        },
      );
      
    } catch (e) {
      state = null;
      rethrow;
    }
  }

  /// Stop audio stream
  Future<void> stopStream() async {
    await _audioSubscription?.cancel();
    _audioSubscription = null;
    
    final audioService = ref.read(audioServiceProvider);
    await audioService.stopAudioStream();
    
    state = null;
  }

  @override
  void dispose() {
    _audioSubscription?.cancel();
    super.dispose();
  }
}

// Real-time Transcription Provider
@riverpod
class RealtimeTranscription extends _$RealtimeTranscription {
  final List<double> _audioBuffer = [];
  Timer? _processTimer;
  
  @override
  Future<String?> build() async {
    return null;
  }

  void processAudioChunk(Float32List audioChunk) {
    // Add to buffer
    _audioBuffer.addAll(audioChunk);
    
    // Process when buffer reaches threshold
    final config = ref.read(whisperConfigNotifierProvider);
    final bufferSizeSamples = (config.windowSizeSec * config.sampleRate).toInt();
    
    if (_audioBuffer.length >= bufferSizeSamples) {
      _scheduleProcessing();
    }
  }

  void _scheduleProcessing() {
    _processTimer?.cancel();
    _processTimer = Timer(const Duration(milliseconds: 500), _processBuffer);
  }

  Future<void> _processBuffer() async {
    if (_audioBuffer.isEmpty) return;
    
    try {
      // Take audio chunk for processing
      final config = ref.read(whisperConfigNotifierProvider);
      final chunkSize = (config.windowSizeSec * config.sampleRate).toInt();
      
      final audioChunk = _audioBuffer.take(chunkSize).toList();
      
      // Process with whisper
      final result = await ref.read(transcriptionControllerProvider.notifier)
          .transcribeAudio(audioData: audioChunk, config: config);
      
      state = AsyncValue.data(result.text);
      
      // Remove processed data from buffer (with overlap)
      final stepSize = (config.stepSizeSec * config.sampleRate).toInt();
      _audioBuffer.removeRange(0, stepSize.clamp(0, _audioBuffer.length));
      
    } catch (e) {
      // Continue processing even if transcription fails
      _audioBuffer.clear();
    }
  }

  void clearBuffer() {
    _audioBuffer.clear();
    state = const AsyncValue.data(null);
  }

  @override
  void dispose() {
    _processTimer?.cancel();
    _audioBuffer.clear();
    super.dispose();
  }
}
```

#### 3. lib/providers/quran_provider.dart

```dart
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../services/bridge_service.dart';
import '../services/quran_service.dart';
import '../models/quran_models.dart';
import '../models/errors.dart';

part 'quran_provider.g.dart';

// Quran Service Provider
@riverpod
QuranService quranService(QuranServiceRef ref) {
  return QuranService(BridgeService.api);
}

// Quran Database Provider
@riverpod
class QuranDatabase extends _$QuranDatabase {
  @override
  Future<List<QuranVerse>> build() async {
    final service = ref.read(quranServiceProvider);
    return await service.loadQuranDatabase();
  }

  /// Refresh database
  Future<void> refresh() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(() async {
      final service = ref.read(quranServiceProvider);
      return await service.loadQuranDatabase();
    });
  }
}

// Arabic Word Validation Provider
@riverpod
class ArabicValidation extends _$ArabicValidation {
  @override
  Future<ValidationResult?> build() async {
    return null;
  }

  /// Validate single Arabic word
  Future<ValidationResult> validateWord(String word) async {
    state = const AsyncValue.loading();
    
    try {
      final service = ref.read(quranServiceProvider);
      final result = await service.validateArabicWord(word);
      
      state = AsyncValue.data(result);
      return result;
    } catch (e, stackTrace) {
      final error = AppError.validationError(
        message: 'Word validation failed: $e',
        fieldErrors: {'word': word},
      );
      state = AsyncValue.error(error, stackTrace);
      rethrow;
    }
  }

  /// Validate multiple words
  Future<List<ValidationResult>> validateWords(List<String> words) async {
    final results = <ValidationResult>[];
    
    for (final word in words) {
      try {
        final result = await validateWord(word);
        results.add(result);
      } catch (e) {
        // Add failed validation result
        results.add(ValidationResult(
          originalWord: word,
          isValid: false,
          suggestions: [],
          references: [],
        ));
      }
    }
    
    return results;
  }

  /// Clear validation result
  void clearValidation() {
    state = const AsyncValue.data(null);
  }
}

// Quran Search Provider
@riverpod
class QuranSearch extends _$QuranSearch {
  @override
  Future<List<QuranVerse>> build() async {
    return [];
  }

  /// Search Quran verses
  Future<List<QuranVerse>> searchVerses(String query) async {
    if (query.trim().isEmpty) {
      state = const AsyncValue.data([]);
      return [];
    }

    state = const AsyncValue.loading();
    
    try {
      final service = ref.read(quranServiceProvider);
      final results = await service.searchQuranVerses(query);
      
      state = AsyncValue.data(results);
      return results;
    } catch (e, stackTrace) {
      final error = AppError.unknownError(
        message: 'Search failed: $e',
        originalError: e,
        stackTrace: stackTrace,
      );
      state = AsyncValue.error(error, stackTrace);
      rethrow;
    }
  }

  /// Search by Surah and verse number
  Future<QuranVerse?> getVerse(int surahNumber, int verseNumber) async {
    try {
      final service = ref.read(quranServiceProvider);
      return await service.getVerse(surahNumber, verseNumber);
    } catch (e) {
      return null;
    }
  }

  /// Clear search results
  void clearResults() {
    state = const AsyncValue.data([]);
  }
}

// Recent Verses Provider
@riverpod
class RecentVerses extends _$RecentVerses {
  @override
  Future<List<QuranVerse>> build() async {
    return [];
  }

  void addVerse(QuranVerse verse) {
    final currentVerses = state.valueOrNull ?? [];
    
    // Remove if already exists
    final updatedVerses = currentVerses
        .where((v) => v.id != verse.id)
        .toList();
    
    // Add to beginning
    updatedVerses.insert(0, verse);
    
    // Keep only last 20 verses
    final limitedVerses = updatedVerses.take(20).toList();
    
    state = AsyncValue.data(limitedVerses);
  }

  void removeVerse(int verseId) {
    final currentVerses = state.valueOrNull ?? [];
    final updatedVerses = currentVerses
        .where((verse) => verse.id != verseId)
        .toList();
    
    state = AsyncValue.data(updatedVerses);
  }

  void clearRecentVerses() {
    state = const AsyncValue.data([]);
  }
}
```

### 🎯 Usage Examples

#### Using Providers in Widgets

```dart
// lib/ui/screens/home_screen.dart
class HomeScreen extends ConsumerWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final whisperInstances = ref.watch(whisperInstancesProvider);
    final recordingState = ref.watch(recordingStateNotifierProvider);
    final transcriptionResult = ref.watch(transcriptionControllerProvider);

    return Scaffold(
      body: Column(
        children: [
          // Whisper Status
          whisperInstances.when(
            data: (instances) => Text('${instances.length} models loaded'),
            loading: () => const CircularProgressIndicator(),
            error: (error, _) => Text('Error: $error'),
          ),
          
          // Recording Button
          ElevatedButton(
            onPressed: recordingState.isRecording 
                ? () => ref.read(recordingStateNotifierProvider.notifier)
                    .stopRecording()
                : () => ref.read(recordingStateNotifierProvider.notifier)
                    .startRecording(),
            child: Text(recordingState.isRecording ? 'Stop' : 'Record'),
          ),
          
          // Transcription Result
          transcriptionResult.when(
            data: (result) => result != null 
                ? Text(result.text)
                : const Text('No transcription'),
            loading: () => const CircularProgressIndicator(),
            error: (error, _) => Text('Error: $error'),
          ),
        ],
      ),
    );
  }
}
```

### 🔄 Next Steps

1. ✅ Providers selesai → Lanjut ke `05-services.md`
2. Implement service layer implementation
3. Create UI components
4. Build real-time features
