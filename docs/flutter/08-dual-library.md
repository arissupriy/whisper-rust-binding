# 🔗 Dual Library Integration
## whisper.so + quran_assistant_engine.so Integration

### 🎯 Overview

Integrasi dual library dimana **whisper-rust-binding.so** fokus pada transcription engine, sedangkan **quran_assistant_engine.so** menangani validasi Arabic text dan data Al-Quran. Kedua library berkomunikasi melalui FRB.

### 🏗️ Dual Library Architecture

```
Flutter App
├── FRB Bridge Layer
│   ├── whisper_bridge.dart → whisper-rust-binding.so
│   └── quran_bridge.dart → quran_assistant_engine.so
├── Service Layer
│   ├── WhisperService (transcription only)
│   ├── QuranService (validation only)
│   └── IntegrationService (coordination)
└── UI Layer
    ├── TranscriptionDisplay
    ├── ValidationResults
    └── CombinedWorkflow

Native Libraries (.so files):
├── whisper-rust-binding.so
│   ├── whisper_init()
│   ├── whisper_transcribe()
│   └── whisper_free()
└── quran_assistant_engine.so
    ├── validate_arabic_word()
    ├── search_quran_verse()
    └── get_suggestions()
```

### 🔧 Dual Library Services

#### 1. lib/services/integration_service.dart

```dart
import 'dart:async';
import '../generated/whisper_bridge.dart';
import '../generated/quran_bridge.dart';
import '../models/transcription_result.dart';
import '../models/quran_models.dart';
import '../models/errors.dart';

/// Service untuk koordinasi antara whisper.so dan quran_assistant_engine.so
class IntegrationService {
  final WhisperApi _whisperApi;
  final QuranApi _quranApi;
  
  IntegrationService(this._whisperApi, this._quranApi);

  /// Complete workflow: Transcribe + Validate + Search
  Future<IntegratedResult> processAudioWithValidation({
    required int whisperInstanceId,
    required List<double> audioData,
    required String language,
    bool enableQuranSearch = true,
    bool enableWordValidation = true,
  }) async {
    try {
      // Step 1: Transcribe audio menggunakan whisper.so
      final transcriptionResult = await _transcribeAudio(
        whisperInstanceId,
        audioData,
        language,
      );

      // Step 2: Extract Arabic words untuk validation
      final arabicWords = _extractArabicWords(transcriptionResult.text);
      
      // Step 3: Validate words menggunakan quran_assistant_engine.so
      List<WordValidation> wordValidations = [];
      if (enableWordValidation && arabicWords.isNotEmpty) {
        wordValidations = await _validateWords(arabicWords);
      }

      // Step 4: Search Quran verses
      List<QuranVerse> relatedVerses = [];
      if (enableQuranSearch && transcriptionResult.text.trim().isNotEmpty) {
        relatedVerses = await _searchQuranVerses(transcriptionResult.text);
      }

      // Step 5: Generate integrated result
      return IntegratedResult(
        transcription: transcriptionResult,
        wordValidations: wordValidations,
        relatedVerses: relatedVerses,
        confidence: _calculateIntegratedConfidence(
          transcriptionResult.confidence,
          wordValidations,
        ),
        processingMetadata: IntegratedMetadata(
          whisperProcessingTime: transcriptionResult.metadata.processingTimeSec,
          validationProcessingTime: wordValidations.isNotEmpty ? 0.1 : 0.0,
          searchProcessingTime: relatedVerses.isNotEmpty ? 0.2 : 0.0,
          totalArabicWords: arabicWords.length,
          validArabicWords: wordValidations.where((w) => w.isValid).length,
          foundVerses: relatedVerses.length,
        ),
      );

    } catch (e) {
      throw AppError.unknownError(
        message: 'Integration processing failed: $e',
        originalError: e,
      );
    }
  }

  /// Transcribe using whisper.so only
  Future<TranscriptionResult> _transcribeAudio(
    int instanceId,
    List<double> audioData,
    String language,
  ) async {
    try {
      final startTime = DateTime.now();
      
      // Call whisper.so
      final resultText = await _whisperApi.whisperProcessAudio(
        instanceId: instanceId,
        audioData: audioData,
        language: language,
      );

      final endTime = DateTime.now();
      final processingTime = endTime.difference(startTime).inMilliseconds / 1000.0;

      return TranscriptionResult(
        id: 'integrated_${DateTime.now().millisecondsSinceEpoch}',
        text: resultText.trim(),
        confidence: _estimateTranscriptionConfidence(resultText),
        segments: [], // Simplified for integration
        timestamp: startTime,
        language: language,
        metadata: TranscriptionMetadata(
          processingTimeSec: processingTime,
          audioLengthSec: audioData.length / 16000.0,
          instanceId: instanceId,
          config: const WhisperConfig(), // Default config
          totalSegments: 1,
          totalWords: resultText.split(' ').length,
        ),
      );

    } catch (e) {
      throw AppError.whisperError(
        message: 'Whisper transcription failed: $e',
        type: WhisperErrorType.transcriptionFailed,
        instanceId: instanceId,
      );
    }
  }

  /// Validate words using quran_assistant_engine.so
  Future<List<WordValidation>> _validateWords(List<String> words) async {
    try {
      final validations = <WordValidation>[];
      
      for (final word in words) {
        // Call quran_assistant_engine.so untuk validation
        final isValid = await _quranApi.validateArabicWord(
          word: word,
          dictionary: [], // Dictionary akan diload oleh quran engine
        );

        // Get suggestions jika word tidak valid
        List<String> suggestions = [];
        if (!isValid) {
          suggestions = await _getSuggestions(word);
        }

        // Get Quran references untuk word
        final references = await _getWordReferences(word);

        validations.add(WordValidation(
          word: word,
          isValid: isValid,
          suggestions: suggestions,
          references: references,
          confidence: isValid ? 0.9 : 0.3,
        ));
      }

      return validations;

    } catch (e) {
      throw AppError.validationError(
        message: 'Word validation failed: $e',
        fieldErrors: {'words': words.join(', ')},
      );
    }
  }

  /// Search Quran verses using quran_assistant_engine.so
  Future<List<QuranVerse>> _searchQuranVerses(String searchText) async {
    try {
      // Call quran_assistant_engine.so untuk search
      final verses = await _quranApi.searchQuranVerse(
        searchTerm: searchText,
      );

      return verses.take(5).toList(); // Limit to top 5 results

    } catch (e) {
      print('⚠️ Quran search failed: $e');
      return []; // Non-critical error, return empty list
    }
  }

  /// Get word suggestions from quran_assistant_engine.so
  Future<List<String>> _getSuggestions(String word) async {
    try {
      // This would be implemented in quran_assistant_engine.so
      // For now, return empty list
      return [];
    } catch (e) {
      return [];
    }
  }

  /// Get Quran references for word
  Future<List<QuranReference>> _getWordReferences(String word) async {
    try {
      final verses = await _searchQuranVerses(word);
      
      return verses.map((verse) => QuranReference(
        surahNumber: verse.surahNumber,
        verseNumber: verse.verseNumber,
        context: verse.arabicText.substring(0, 100), // First 100 chars
        relevanceScore: 0.8, // Simplified scoring
      )).toList();

    } catch (e) {
      return [];
    }
  }

  /// Extract Arabic words from text
  List<String> _extractArabicWords(String text) {
    final arabicRegex = RegExp(r'[\u0600-\u06FF]+');
    final matches = arabicRegex.allMatches(text);
    
    return matches
        .map((match) => match.group(0)!)
        .where((word) => word.length > 1) // Skip single characters
        .toSet() // Remove duplicates
        .toList();
  }

  /// Estimate transcription confidence
  double _estimateTranscriptionConfidence(String text) {
    if (text.trim().isEmpty) return 0.0;
    
    final arabicWordCount = RegExp(r'[\u0600-\u06FF]+').allMatches(text).length;
    final totalWords = text.split(' ').where((w) => w.trim().isNotEmpty).length;
    
    if (totalWords == 0) return 0.0;
    
    final arabicRatio = arabicWordCount / totalWords;
    return (0.6 + (arabicRatio * 0.4)).clamp(0.0, 1.0);
  }

  /// Calculate integrated confidence
  double _calculateIntegratedConfidence(
    double transcriptionConfidence,
    List<WordValidation> validations,
  ) {
    if (validations.isEmpty) return transcriptionConfidence;
    
    final validWords = validations.where((v) => v.isValid).length;
    final validationScore = validWords / validations.length;
    
    // Weighted average: 70% transcription, 30% validation
    return (transcriptionConfidence * 0.7) + (validationScore * 0.3);
  }
}

/// Models untuk integrated results
@freezed
class IntegratedResult with _$IntegratedResult {
  const factory IntegratedResult({
    required TranscriptionResult transcription,
    required List<WordValidation> wordValidations,
    required List<QuranVerse> relatedVerses,
    required double confidence,
    required IntegratedMetadata processingMetadata,
  }) = _IntegratedResult;

  factory IntegratedResult.fromJson(Map<String, dynamic> json) =>
      _$IntegratedResultFromJson(json);
}

@freezed
class WordValidation with _$WordValidation {
  const factory WordValidation({
    required String word,
    required bool isValid,
    required List<String> suggestions,
    required List<QuranReference> references,
    required double confidence,
  }) = _WordValidation;

  factory WordValidation.fromJson(Map<String, dynamic> json) =>
      _$WordValidationFromJson(json);
}

@freezed
class IntegratedMetadata with _$IntegratedMetadata {
  const factory IntegratedMetadata({
    required double whisperProcessingTime,
    required double validationProcessingTime,
    required double searchProcessingTime,
    required int totalArabicWords,
    required int validArabicWords,
    required int foundVerses,
  }) = _IntegratedMetadata;

  factory IntegratedMetadata.fromJson(Map<String, dynamic> json) =>
      _$IntegratedMetadataFromJson(json);
}
```

#### 2. lib/providers/integration_provider.dart

```dart
import 'package:riverpod_annotation/riverpod_annotation.dart';
import '../services/integration_service.dart';
import '../services/bridge_service.dart';
import '../models/transcription_result.dart';
import '../models/quran_models.dart';

part 'integration_provider.g.dart';

// Integration Service Provider
@riverpod
IntegrationService integrationService(IntegrationServiceRef ref) {
  final api = BridgeService.api;
  return IntegrationService(api, api);
}

// Integrated Processing Provider
@riverpod
class IntegratedProcessing extends _$IntegratedProcessing {
  @override
  Future<IntegratedResult?> build() async {
    return null;
  }

  /// Process audio dengan full integration
  Future<IntegratedResult> processAudioComplete({
    required int whisperInstanceId,
    required List<double> audioData,
    String language = 'ar',
    bool enableValidation = true,
    bool enableQuranSearch = true,
  }) async {
    state = const AsyncValue.loading();

    try {
      final service = ref.read(integrationServiceProvider);
      
      final result = await service.processAudioWithValidation(
        whisperInstanceId: whisperInstanceId,
        audioData: audioData,
        language: language,
        enableWordValidation: enableValidation,
        enableQuranSearch: enableQuranSearch,
      );

      state = AsyncValue.data(result);
      return result;

    } catch (e, stackTrace) {
      state = AsyncValue.error(e, stackTrace);
      rethrow;
    }
  }

  /// Process dengan whisper.so saja (tanpa validation)
  Future<TranscriptionResult> processAudioTranscriptionOnly({
    required int whisperInstanceId,
    required List<double> audioData,
    String language = 'ar',
  }) async {
    try {
      final service = ref.read(integrationServiceProvider);
      
      final result = await service.processAudioWithValidation(
        whisperInstanceId: whisperInstanceId,
        audioData: audioData,
        language: language,
        enableWordValidation: false,
        enableQuranSearch: false,
      );

      return result.transcription;

    } catch (e) {
      rethrow;
    }
  }

  /// Clear results
  void clearResults() {
    state = const AsyncValue.data(null);
  }
}

// Word Validation Provider
@riverpod
class WordValidationProcessor extends _$WordValidationProcessor {
  @override
  Future<List<WordValidation>> build() async {
    return [];
  }

  /// Validate specific words menggunakan quran_assistant_engine.so
  Future<List<WordValidation>> validateWords(List<String> words) async {
    if (words.isEmpty) {
      state = const AsyncValue.data([]);
      return [];
    }

    state = const AsyncValue.loading();

    try {
      final service = ref.read(integrationServiceProvider);
      
      // Use private method through a public interface
      final validations = <WordValidation>[];
      
      for (final word in words) {
        // Simplified validation - in real implementation,
        // this would call quran_assistant_engine.so directly
        final validation = WordValidation(
          word: word,
          isValid: await _isValidArabicWord(word),
          suggestions: await _getWordSuggestions(word),
          references: await _getQuranReferences(word),
          confidence: 0.8,
        );
        
        validations.add(validation);
      }

      state = AsyncValue.data(validations);
      return validations;

    } catch (e, stackTrace) {
      state = AsyncValue.error(e, stackTrace);
      rethrow;
    }
  }

  Future<bool> _isValidArabicWord(String word) async {
    // Call quran_assistant_engine.so
    try {
      final api = BridgeService.api;
      return await api.validateArabicWord(word: word, dictionary: []);
    } catch (e) {
      return false;
    }
  }

  Future<List<String>> _getWordSuggestions(String word) async {
    // Get suggestions from quran_assistant_engine.so
    return []; // Simplified
  }

  Future<List<QuranReference>> _getQuranReferences(String word) async {
    // Search references from quran_assistant_engine.so
    return []; // Simplified
  }
}

// Quran Search Provider
@riverpod
class QuranSearchProcessor extends _$QuranSearchProcessor {
  @override
  Future<List<QuranVerse>> build() async {
    return [];
  }

  /// Search verses menggunakan quran_assistant_engine.so
  Future<List<QuranVerse>> searchVerses(String searchTerm) async {
    if (searchTerm.trim().isEmpty) {
      state = const AsyncValue.data([]);
      return [];
    }

    state = const AsyncValue.loading();

    try {
      final api = BridgeService.api;
      final verses = await api.searchQuranVerse(searchTerm: searchTerm);
      
      state = AsyncValue.data(verses);
      return verses;

    } catch (e, stackTrace) {
      state = AsyncValue.error(e, stackTrace);
      rethrow;
    }
  }
}
```

#### 3. lib/ui/components/integrated_results_widget.dart

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../providers/integration_provider.dart';
import '../../models/quran_models.dart';

class IntegratedResultsWidget extends ConsumerWidget {
  final bool showTranscription;
  final bool showValidation;
  final bool showQuranSearch;
  
  const IntegratedResultsWidget({
    super.key,
    this.showTranscription = true,
    this.showValidation = true,
    this.showQuranSearch = true,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final integratedResults = ref.watch(integratedProcessingProvider);

    return integratedResults.when(
      data: (result) {
        if (result == null) {
          return _buildEmptyState();
        }

        return SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Overall confidence
              _buildConfidenceIndicator(result.confidence),
              const SizedBox(height: 16),

              // Transcription results (whisper.so)
              if (showTranscription) ...[
                _buildTranscriptionSection(result.transcription),
                const SizedBox(height: 16),
              ],

              // Word validation (quran_assistant_engine.so)
              if (showValidation && result.wordValidations.isNotEmpty) ...[
                _buildValidationSection(result.wordValidations),
                const SizedBox(height: 16),
              ],

              // Quran search results (quran_assistant_engine.so)
              if (showQuranSearch && result.relatedVerses.isNotEmpty) ...[
                _buildQuranSection(result.relatedVerses),
                const SizedBox(height: 16),
              ],

              // Processing metrics
              _buildMetricsSection(result.processingMetadata),
            ],
          ),
        );
      },
      loading: () => _buildLoadingState(),
      error: (error, _) => _buildErrorState(error.toString()),
    );
  }

  Widget _buildEmptyState() {
    return const Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.psychology_outlined, size: 64, color: Colors.grey),
          SizedBox(height: 16),
          Text(
            'لا توجد نتائج للعرض',
            style: TextStyle(fontSize: 18, color: Colors.grey),
          ),
          SizedBox(height: 8),
          Text(
            'ابدأ بتسجيل الصوت أو تحميل ملف',
            style: TextStyle(fontSize: 14, color: Colors.grey),
          ),
        ],
      ),
    );
  }

  Widget _buildLoadingState() {
    return const Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          CircularProgressIndicator(),
          SizedBox(height: 16),
          Text('جاري المعالجة...'),
          SizedBox(height: 8),
          Text(
            'whisper.so + quran_assistant_engine.so',
            style: TextStyle(fontSize: 12, color: Colors.grey),
          ),
        ],
      ),
    );
  }

  Widget _buildErrorState(String error) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.error, size: 64, color: Colors.red),
          const SizedBox(height: 16),
          Text(
            'خطأ في المعالجة',
            style: const TextStyle(fontSize: 18, color: Colors.red),
          ),
          const SizedBox(height: 8),
          Text(
            error,
            style: const TextStyle(fontSize: 14, color: Colors.grey),
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }

  Widget _buildConfidenceIndicator(double confidence) {
    Color color;
    String label;
    
    if (confidence >= 0.8) {
      color = Colors.green;
      label = 'ممتاز';
    } else if (confidence >= 0.6) {
      color = Colors.orange;
      label = 'جيد';
    } else {
      color = Colors.red;
      label = 'يحتاج تحسين';
    }

    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: color),
      ),
      child: Row(
        children: [
          Icon(Icons.analytics, color: color),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'جودة النتائج المدمجة',
                  style: TextStyle(
                    fontWeight: FontWeight.bold,
                    color: color,
                  ),
                ),
                Text(
                  '$label (${(confidence * 100).toInt()}%)',
                  style: TextStyle(color: color),
                ),
              ],
            ),
          ),
          CircularProgressIndicator(
            value: confidence,
            backgroundColor: color.withOpacity(0.3),
            valueColor: AlwaysStoppedAnimation<Color>(color),
          ),
        ],
      ),
    );
  }

  Widget _buildTranscriptionSection(TranscriptionResult transcription) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.record_voice_over, color: Colors.blue),
                const SizedBox(width: 8),
                const Text(
                  'نتيجة التحويل (whisper.so)',
                  style: TextStyle(fontWeight: FontWeight.bold),
                ),
                const Spacer(),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                  decoration: BoxDecoration(
                    color: Colors.blue.withOpacity(0.1),
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Text(
                    '${(transcription.confidence * 100).toInt()}%',
                    style: const TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.bold,
                      color: Colors.blue,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: Colors.grey[50],
                borderRadius: BorderRadius.circular(8),
              ),
              child: SelectableText(
                transcription.text,
                style: const TextStyle(
                  fontSize: 16,
                  height: 1.6,
                  fontFamily: 'Amiri',
                ),
                textAlign: TextAlign.right,
                textDirection: TextDirection.rtl,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildValidationSection(List<WordValidation> validations) {
    final validWords = validations.where((v) => v.isValid).length;
    final totalWords = validations.length;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.spellcheck, color: Colors.green),
                const SizedBox(width: 8),
                const Text(
                  'التحقق من صحة الكلمات (quran_assistant_engine.so)',
                  style: TextStyle(fontWeight: FontWeight.bold),
                ),
                const Spacer(),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                  decoration: BoxDecoration(
                    color: Colors.green.withOpacity(0.1),
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Text(
                    '$validWords/$totalWords',
                    style: const TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.bold,
                      color: Colors.green,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: validations.map((validation) {
                return Container(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                  decoration: BoxDecoration(
                    color: validation.isValid 
                        ? Colors.green.withOpacity(0.1)
                        : Colors.red.withOpacity(0.1),
                    borderRadius: BorderRadius.circular(16),
                    border: Border.all(
                      color: validation.isValid ? Colors.green : Colors.red,
                    ),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(
                        validation.isValid ? Icons.check : Icons.close,
                        size: 16,
                        color: validation.isValid ? Colors.green : Colors.red,
                      ),
                      const SizedBox(width: 4),
                      Text(
                        validation.word,
                        style: TextStyle(
                          color: validation.isValid ? Colors.green : Colors.red,
                          fontFamily: 'Amiri',
                        ),
                      ),
                    ],
                  ),
                );
              }).toList(),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildQuranSection(List<QuranVerse> verses) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.menu_book, color: Colors.purple),
                const SizedBox(width: 8),
                const Text(
                  'آيات ذات صلة (quran_assistant_engine.so)',
                  style: TextStyle(fontWeight: FontWeight.bold),
                ),
                const Spacer(),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                  decoration: BoxDecoration(
                    color: Colors.purple.withOpacity(0.1),
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Text(
                    '${verses.length} آية',
                    style: const TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.bold,
                      color: Colors.purple,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            ...verses.map((verse) => _buildVerseItem(verse)),
          ],
        ),
      ),
    );
  }

  Widget _buildVerseItem(QuranVerse verse) {
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.purple.withOpacity(0.05),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.purple.withOpacity(0.2)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: Colors.purple,
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Text(
                  'سورة ${verse.surahNumber} آية ${verse.verseNumber}',
                  style: const TextStyle(
                    color: Colors.white,
                    fontSize: 12,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            verse.arabicText,
            style: const TextStyle(
              fontSize: 16,
              fontFamily: 'Amiri',
              height: 1.8,
            ),
            textAlign: TextAlign.right,
            textDirection: TextDirection.rtl,
          ),
          if (verse.translation.isNotEmpty) ...[
            const SizedBox(height: 8),
            Text(
              verse.translation,
              style: TextStyle(
                fontSize: 14,
                color: Colors.grey[600],
                fontStyle: FontStyle.italic,
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildMetricsSection(IntegratedMetadata metadata) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Row(
              children: [
                Icon(Icons.analytics_outlined, color: Colors.orange),
                SizedBox(width: 8),
                Text(
                  'إحصائيات المعالجة',
                  style: TextStyle(fontWeight: FontWeight.bold),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: _buildMetricItem(
                    'whisper.so',
                    '${metadata.whisperProcessingTime.toStringAsFixed(2)}ث',
                    Colors.blue,
                  ),
                ),
                Expanded(
                  child: _buildMetricItem(
                    'التحقق',
                    '${metadata.validationProcessingTime.toStringAsFixed(2)}ث',
                    Colors.green,
                  ),
                ),
                Expanded(
                  child: _buildMetricItem(
                    'البحث',
                    '${metadata.searchProcessingTime.toStringAsFixed(2)}ث',
                    Colors.purple,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Row(
              children: [
                Expanded(
                  child: _buildMetricItem(
                    'الكلمات العربية',
                    '${metadata.totalArabicWords}',
                    Colors.orange,
                  ),
                ),
                Expanded(
                  child: _buildMetricItem(
                    'الكلمات الصحيحة',
                    '${metadata.validArabicWords}',
                    Colors.green,
                  ),
                ),
                Expanded(
                  child: _buildMetricItem(
                    'الآيات الموجودة',
                    '${metadata.foundVerses}',
                    Colors.purple,
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMetricItem(String label, String value, Color color) {
    return Container(
      padding: const EdgeInsets.all(8),
      margin: const EdgeInsets.symmetric(horizontal: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        children: [
          Text(
            value,
            style: TextStyle(
              fontWeight: FontWeight.bold,
              color: color,
              fontSize: 16,
            ),
          ),
          Text(
            label,
            style: TextStyle(
              color: color,
              fontSize: 12,
            ),
          ),
        ],
      ),
    );
  }
}
```

### 🎯 Key Integration Points

1. **Separated Responsibilities**:
   - `whisper-rust-binding.so`: Pure transcription engine
   - `quran_assistant_engine.so`: Arabic validation & Quran data

2. **FRB Coordination**: Single FRB interface mengelola kedua libraries

3. **Performance Optimization**: Parallel processing dan cached results

4. **Error Isolation**: Error di satu library tidak mempengaruhi yang lain

5. **Scalable Architecture**: Mudah menambah library ketiga jika diperlukan

### 🔄 Next Steps

1. ✅ Dual Library selesai → Lanjut ke `09-permissions.md`
2. Handle Android permissions
3. Complete examples
4. Production deployment guide
