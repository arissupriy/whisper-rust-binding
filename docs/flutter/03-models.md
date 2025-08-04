# 📊 Data Models dan Types
## Type-Safe Models untuk Flutter Integration

### 🎯 Overview

Data models yang type-safe dan immutable untuk menghandle data dari native libraries dengan proper error handling dan serialization.

### 🏗️ Core Models

#### 1. lib/models/whisper_model.dart

```dart
import 'package:freezed_annotation/freezed_annotation.dart';
import 'dart:typed_data';

part 'whisper_model.freezed.dart';
part 'whisper_model.g.dart';

@freezed
class WhisperInstance with _$WhisperInstance {
  const factory WhisperInstance({
    required int id,
    required String modelPath,
    required WhisperModelInfo modelInfo,
    required DateTime createdAt,
    @Default(true) bool isActive,
  }) = _WhisperInstance;

  factory WhisperInstance.fromJson(Map<String, dynamic> json) =>
      _$WhisperInstanceFromJson(json);
}

@freezed
class WhisperModelInfo with _$WhisperModelInfo {
  const factory WhisperModelInfo({
    required String name,
    required String version,
    required int vocabularySize,
    required List<String> supportedLanguages,
    required double fileSizeMB,
    @Default('unknown') String modelType,
  }) = _WhisperModelInfo;

  factory WhisperModelInfo.fromJson(Map<String, dynamic> json) =>
      _$WhisperModelInfoFromJson(json);
}

@freezed
class WhisperConfig with _$WhisperConfig {
  const factory WhisperConfig({
    @Default('ar') String language,
    @Default(16000) int sampleRate,
    @Default(1) int channels,
    @Default(10.0) double windowSizeSec,
    @Default(5.0) double stepSizeSec,
    @Default(true) bool enableTimestamps,
    @Default(0.5) double confidenceThreshold,
  }) = _WhisperConfig;

  factory WhisperConfig.fromJson(Map<String, dynamic> json) =>
      _$WhisperConfigFromJson(json);
}
```

#### 2. lib/models/transcription_result.dart

```dart
import 'package:freezed_annotation/freezed_annotation.dart';

part 'transcription_result.freezed.dart';
part 'transcription_result.g.dart';

@freezed
class TranscriptionResult with _$TranscriptionResult {
  const factory TranscriptionResult({
    required String id,
    required String text,
    required double confidence,
    required List<TranscriptionSegment> segments,
    required DateTime timestamp,
    required TranscriptionMetadata metadata,
    String? language,
    @Default([]) List<String> warnings,
  }) = _TranscriptionResult;

  factory TranscriptionResult.fromJson(Map<String, dynamic> json) =>
      _$TranscriptionResultFromJson(json);
}

@freezed
class TranscriptionSegment with _$TranscriptionSegment {
  const factory TranscriptionSegment({
    required int id,
    required String text,
    required double startTime,
    required double endTime,
    required double confidence,
    @Default([]) List<TranscriptionWord> words,
    String? language,
  }) = _TranscriptionSegment;

  factory TranscriptionSegment.fromJson(Map<String, dynamic> json) =>
      _$TranscriptionSegmentFromJson(json);
}

@freezed
class TranscriptionWord with _$TranscriptionWord {
  const factory TranscriptionWord({
    required String text,
    required double startTime,
    required double endTime,
    required double confidence,
    @Default(false) bool isValidArabic,
    @Default([]) List<String> suggestions,
  }) = _TranscriptionWord;

  factory TranscriptionWord.fromJson(Map<String, dynamic> json) =>
      _$TranscriptionWordFromJson(json);
}

@freezed
class TranscriptionMetadata with _$TranscriptionMetadata {
  const factory TranscriptionMetadata({
    required double processingTimeSec,
    required double audioLengthSec,
    required int instanceId,
    required WhisperConfig config,
    @Default(0) int totalSegments,
    @Default(0) int totalWords,
  }) = _TranscriptionMetadata;

  factory TranscriptionMetadata.fromJson(Map<String, dynamic> json) =>
      _$TranscriptionMetadataFromJson(json);
}
```

#### 3. lib/models/audio_data.dart

```dart
import 'package:freezed_annotation/freezed_annotation.dart';
import 'dart:typed_data';

part 'audio_data.freezed.dart';
part 'audio_data.g.dart';

@freezed
class AudioData with _$AudioData {
  const factory AudioData({
    required String id,
    required Float32List samples,
    required AudioFormat format,
    required DateTime recordedAt,
    required double durationSec,
    String? filePath,
    String? description,
  }) = _AudioData;

  factory AudioData.fromJson(Map<String, dynamic> json) =>
      _$AudioDataFromJson(json);
}

@freezed
class AudioFormat with _$AudioFormat {
  const factory AudioFormat({
    @Default(16000) int sampleRate,
    @Default(1) int channels,
    @Default(16) int bitDepth,
    @Default(AudioEncoding.pcm) AudioEncoding encoding,
  }) = _AudioFormat;

  factory AudioFormat.fromJson(Map<String, dynamic> json) =>
      _$AudioFormatFromJson(json);
}

enum AudioEncoding {
  @JsonValue('pcm')
  pcm,
  @JsonValue('wav')
  wav,
  @JsonValue('mp3')
  mp3,
  @JsonValue('aac')
  aac,
}

@freezed
class RecordingState with _$RecordingState {
  const factory RecordingState({
    @Default(false) bool isRecording,
    @Default(false) bool isPaused,
    @Default(0.0) double currentDurationSec,
    @Default(0.0) double currentAmplitude,
    String? currentFilePath,
    DateTime? startTime,
    String? errorMessage,
  }) = _RecordingState;

  factory RecordingState.fromJson(Map<String, dynamic> json) =>
      _$RecordingStateFromJson(json);
}
```

#### 4. lib/models/quran_models.dart

```dart
import 'package:freezed_annotation/freezed_annotation.dart';

part 'quran_models.freezed.dart';
part 'quran_models.g.dart';

@freezed
class QuranVerse with _$QuranVerse {
  const factory QuranVerse({
    required int id,
    required int surahNumber,
    required int verseNumber,
    required String arabicText,
    required String transliteration,
    required String translation,
    @Default([]) List<String> keywords,
    @Default([]) List<String> topics,
  }) = _QuranVerse;

  factory QuranVerse.fromJson(Map<String, dynamic> json) =>
      _$QuranVerseFromJson(json);
}

@freezed
class QuranSurah with _$QuranSurah {
  const factory QuranSurah({
    required int number,
    required String arabicName,
    required String englishName,
    required String transliteration,
    required int versesCount,
    required SurahType type,
    required int revelationOrder,
  }) = _QuranSurah;

  factory QuranSurah.fromJson(Map<String, dynamic> json) =>
      _$QuranSurahFromJson(json);
}

enum SurahType {
  @JsonValue('meccan')
  meccan,
  @JsonValue('medinan')
  medinan,
}

@freezed
class ValidationResult with _$ValidationResult {
  const factory ValidationResult({
    required String originalWord,
    required bool isValid,
    @Default([]) List<String> suggestions,
    @Default([]) List<QuranReference> references,
    String? correctedWord,
    double? confidence,
  }) = _ValidationResult;

  factory ValidationResult.fromJson(Map<String, dynamic> json) =>
      _$ValidationResultFromJson(json);
}

@freezed
class QuranReference with _$QuranReference {
  const factory QuranReference({
    required int surahNumber,
    required int verseNumber,
    required String context,
    required double relevanceScore,
  }) = _QuranReference;

  factory QuranReference.fromJson(Map<String, dynamic> json) =>
      _$QuranReferenceFromJson(json);
}
```

### 🔄 State Models

#### 5. lib/models/app_state.dart

```dart
import 'package:freezed_annotation/freezed_annotation.dart';
import 'whisper_model.dart';
import 'transcription_result.dart';
import 'audio_data.dart';
import 'quran_models.dart';

part 'app_state.freezed.dart';
part 'app_state.g.dart';

@freezed
class AppState with _$AppState {
  const factory AppState({
    @Default(AppStatus.initial) AppStatus status,
    @Default([]) List<WhisperInstance> whisperInstances,
    @Default([]) List<TranscriptionResult> transcriptionHistory,
    @Default([]) List<QuranVerse> recentVerses,
    WhisperInstance? activeInstance,
    TranscriptionResult? currentTranscription,
    RecordingState? recordingState,
    String? errorMessage,
  }) = _AppState;

  factory AppState.fromJson(Map<String, dynamic> json) =>
      _$AppStateFromJson(json);
}

enum AppStatus {
  @JsonValue('initial')
  initial,
  @JsonValue('loading')
  loading,
  @JsonValue('ready')
  ready,
  @JsonValue('recording')
  recording,
  @JsonValue('transcribing')
  transcribing,
  @JsonValue('error')
  error,
}

@freezed
class ProcessingState with _$ProcessingState {
  const factory ProcessingState({
    @Default(ProcessingStatus.idle) ProcessingStatus status,
    @Default(0.0) double progress,
    String? currentOperation,
    DateTime? startTime,
    String? errorMessage,
  }) = _ProcessingState;

  factory ProcessingState.fromJson(Map<String, dynamic> json) =>
      _$ProcessingStateFromJson(json);
}

enum ProcessingStatus {
  @JsonValue('idle')
  idle,
  @JsonValue('initializing')
  initializing,
  @JsonValue('processing')
  processing,
  @JsonValue('validating')
  validating,
  @JsonValue('completed')
  completed,
  @JsonValue('error')
  error,
}
```

### 🛡️ Error Models

#### 6. lib/models/errors.dart

```dart
import 'package:freezed_annotation/freezed_annotation.dart';

part 'errors.freezed.dart';

@freezed
class AppError with _$AppError {
  const factory AppError.whisperError({
    required String message,
    required WhisperErrorType type,
    int? instanceId,
    String? modelPath,
  }) = WhisperError;

  const factory AppError.audioError({
    required String message,
    required AudioErrorType type,
    String? filePath,
  }) = AudioError;

  const factory AppError.permissionError({
    required String message,
    required String permission,
  }) = PermissionError;

  const factory AppError.networkError({
    required String message,
    int? statusCode,
    String? url,
  }) = NetworkError;

  const factory AppError.validationError({
    required String message,
    required Map<String, String> fieldErrors,
  }) = ValidationError;

  const factory AppError.unknownError({
    required String message,
    Object? originalError,
    StackTrace? stackTrace,
  }) = UnknownError;
}

enum WhisperErrorType {
  initializationFailed,
  modelNotFound,
  invalidModelFormat,
  transcriptionFailed,
  instanceNotFound,
  memoryError,
  configurationError,
}

enum AudioErrorType {
  recordingFailed,
  playbackFailed,
  fileNotFound,
  invalidFormat,
  permissionDenied,
  deviceNotAvailable,
}
```

### 🔧 Extension Methods

#### 7. lib/models/extensions.dart

```dart
import 'package:flutter/material.dart';
import 'transcription_result.dart';
import 'quran_models.dart';
import 'errors.dart';

extension TranscriptionResultX on TranscriptionResult {
  /// Get formatted display text
  String get displayText {
    return text.trim().isEmpty ? 'نص غير واضح' : text;
  }
  
  /// Get confidence color
  Color get confidenceColor {
    if (confidence >= 0.8) return Colors.green;
    if (confidence >= 0.6) return Colors.orange;
    return Colors.red;
  }
  
  /// Get Arabic words only
  List<String> get arabicWords {
    return text.split(' ')
        .where((word) => _isArabic(word))
        .toList();
  }
  
  /// Calculate reading time estimate
  Duration get estimatedReadingTime {
    final wordsCount = text.split(' ').length;
    final secondsPerWord = 0.5; // Average for Arabic
    return Duration(seconds: (wordsCount * secondsPerWord).round());
  }
  
  bool _isArabic(String text) {
    return RegExp(r'[\u0600-\u06FF]').hasMatch(text);
  }
}

extension QuranVerseX on QuranVerse {
  /// Get verse reference (Surah:Verse)
  String get reference => '$surahNumber:$verseNumber';
  
  /// Get formatted Arabic text with diacritics
  String get formattedArabicText {
    return arabicText.replaceAll(RegExp(r'\s+'), ' ').trim();
  }
  
  /// Check if verse contains specific word
  bool containsWord(String word) {
    return arabicText.contains(word) || 
           transliteration.toLowerCase().contains(word.toLowerCase()) ||
           translation.toLowerCase().contains(word.toLowerCase());
  }
}

extension AppErrorX on AppError {
  /// Get user-friendly error message
  String get userMessage {
    return when(
      whisperError: (message, type, instanceId, modelPath) {
        switch (type) {
          case WhisperErrorType.modelNotFound:
            return 'ملف النموذج غير موجود. يرجى التحقق من المسار.';
          case WhisperErrorType.initializationFailed:
            return 'فشل في تهيئة نموذج التعرف على الصوت.';
          case WhisperErrorType.transcriptionFailed:
            return 'فشل في تحويل الصوت إلى نص.';
          default:
            return 'خطأ في نظام التعرف على الصوت: $message';
        }
      },
      audioError: (message, type, filePath) {
        switch (type) {
          case AudioErrorType.permissionDenied:
            return 'لا يوجد إذن للوصول إلى الميكروفون.';
          case AudioErrorType.recordingFailed:
            return 'فشل في تسجيل الصوت.';
          case AudioErrorType.deviceNotAvailable:
            return 'جهاز التسجيل غير متاح.';
          default:
            return 'خطأ في الصوت: $message';
        }
      },
      permissionError: (message, permission) {
        return 'يرجى منح الإذن للوصول إلى $permission.';
      },
      networkError: (message, statusCode, url) {
        return 'خطأ في الشبكة. يرجى التحقق من الاتصال.';
      },
      validationError: (message, fieldErrors) {
        return 'خطأ في التحقق من صحة البيانات.';
      },
      unknownError: (message, originalError, stackTrace) {
        return 'حدث خطأ غير متوقع. يرجى المحاولة مرة أخرى.';
      },
    );
  }
  
  /// Get error icon
  IconData get icon {
    return when(
      whisperError: (_, __, ___, ____) => Icons.record_voice_over,
      audioError: (_, __, ___) => Icons.mic_off,
      permissionError: (_, __) => Icons.security,
      networkError: (_, __, ___) => Icons.wifi_off,
      validationError: (_, __) => Icons.error_outline,
      unknownError: (_, __, ___) => Icons.bug_report,
    );
  }
}
```

### 🔧 Build Configuration

#### 8. build.yaml

```yaml
targets:
  $default:
    builders:
      freezed:
        options:
          # Generate copyWith, toString, operator==, hashCode
          # and serialization methods
          generate_to_string: true
          generate_equal: true
          generate_copy_with: true
        enabled: true
      json_serializable:
        options:
          # Generate JSON serialization
          generate_to_json: true
          generate_from_json: true
          # Handle null values gracefully
          include_if_null: false
          # Use explicit types
          explicit_to_json: true
        enabled: true
```

### 📋 Usage Examples

```dart
// Creating models
final whisperConfig = WhisperConfig(
  language: 'ar',
  sampleRate: 16000,
  confidenceThreshold: 0.7,
);

final transcriptionResult = TranscriptionResult(
  id: 'trans_001',
  text: 'بسم الله الرحمن الرحيم',
  confidence: 0.95,
  segments: [],
  timestamp: DateTime.now(),
  metadata: TranscriptionMetadata(
    processingTimeSec: 1.2,
    audioLengthSec: 5.0,
    instanceId: 1,
    config: whisperConfig,
  ),
);

// Using extensions
print(transcriptionResult.displayText);
print(transcriptionResult.confidenceColor);
print(transcriptionResult.arabicWords);
```

### 🔄 Next Steps

1. ✅ Models selesai → Lanjut ke `04-providers.md`
2. Implement Riverpod providers dengan models ini
3. Create service layer
4. Build UI components
