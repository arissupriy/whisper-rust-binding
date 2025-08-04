# ⚡ Real-time Transcription
## Live Audio Processing dengan whisper.so

### 🎯 Overview

Implementasi real-time transcription menggunakan whisper-rust-binding.so sebagai transcription engine melalui FRB, dengan sliding window processing untuk hasil yang responsif.

### 🏗️ Real-time Architecture

```
Audio Input → Buffer Management → Sliding Window → whisper.so → FRB → Flutter UI
     ↓              ↓                    ↓             ↓       ↓        ↓
Microphone → Chunk Processing → Window Analysis → .so Call → Bridge → Display
```

### ⚡ Core Real-time Components

#### 1. lib/providers/realtime_provider.dart

```dart
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'dart:async';
import 'dart:typed_data';
import '../services/bridge_service.dart';
import '../models/whisper_model.dart';
import '../models/transcription_result.dart';
import '../models/audio_data.dart';
import '../utils/constants.dart';

part 'realtime_provider.g.dart';

// Real-time Transcription State
@freezed
class RealtimeState with _$RealtimeState {
  const factory RealtimeState({
    @Default(false) bool isActive,
    @Default('') String currentText,
    @Default([]) List<String> textHistory,
    @Default(0.0) double currentConfidence,
    @Default(0) int processedChunks,
    @Default(0.0) double averageProcessingTime,
    String? lastError,
  }) = _RealtimeState;
}

// Real-time Transcription Provider
@riverpod
class RealtimeTranscription extends _$RealtimeTranscription {
  Timer? _processingTimer;
  final List<double> _audioBuffer = [];
  final List<String> _textBuffer = [];
  int _chunkCounter = 0;
  double _totalProcessingTime = 0.0;

  @override
  RealtimeState build() {
    ref.onDispose(() {
      _cleanup();
    });
    
    return const RealtimeState();
  }

  /// Start real-time transcription
  Future<void> startRealtime({
    required int whisperInstanceId,
    WhisperConfig? config,
  }) async {
    try {
      final finalConfig = config ?? const WhisperConfig();
      
      state = state.copyWith(
        isActive: true,
        currentText: '',
        lastError: null,
      );

      // Start audio stream processing
      await _startAudioProcessing(whisperInstanceId, finalConfig);
      
      print('✅ Real-time transcription started');
    } catch (e) {
      state = state.copyWith(
        isActive: false,
        lastError: e.toString(),
      );
      rethrow;
    }
  }

  /// Stop real-time transcription
  Future<void> stopRealtime() async {
    try {
      _cleanup();
      
      state = state.copyWith(
        isActive: false,
        currentText: '',
      );
      
      print('✅ Real-time transcription stopped');
    } catch (e) {
      state = state.copyWith(lastError: e.toString());
    }
  }

  /// Process audio chunk
  void processAudioChunk(Float32List audioChunk) {
    if (!state.isActive) return;

    // Add to buffer
    _audioBuffer.addAll(audioChunk);
    
    // Check if we have enough data for processing
    final config = const WhisperConfig(); // Get from provider
    final windowSamples = (config.windowSizeSec * config.sampleRate).toInt();
    
    if (_audioBuffer.length >= windowSamples) {
      _scheduleProcessing();
    }
  }

  void _scheduleProcessing() {
    // Debounce processing to avoid overwhelming the whisper.so
    _processingTimer?.cancel();
    _processingTimer = Timer(
      const Duration(milliseconds: 300),
      _processBuffer,
    );
  }

  Future<void> _processBuffer() async {
    if (!state.isActive || _audioBuffer.isEmpty) return;

    try {
      final startTime = DateTime.now();
      
      // Get processing configuration
      final config = const WhisperConfig();
      final windowSamples = (config.windowSizeSec * config.sampleRate).toInt();
      
      // Extract audio chunk for processing
      final processingChunk = _audioBuffer.take(windowSamples).toList();
      
      // Call whisper.so through FRB for transcription
      final result = await _transcribeChunk(processingChunk, config);
      
      final endTime = DateTime.now();
      final processingTime = endTime.difference(startTime).inMilliseconds / 1000.0;
      
      // Update statistics
      _chunkCounter++;
      _totalProcessingTime += processingTime;
      final averageTime = _totalProcessingTime / _chunkCounter;
      
      // Update text buffer and current state
      if (result.text.trim().isNotEmpty) {
        _textBuffer.add(result.text.trim());
        
        // Keep only recent text (last 10 chunks)
        if (_textBuffer.length > 10) {
          _textBuffer.removeAt(0);
        }
      }
      
      // Update state
      state = state.copyWith(
        currentText: _textBuffer.join(' '),
        currentConfidence: result.confidence,
        processedChunks: _chunkCounter,
        averageProcessingTime: averageTime,
        lastError: null,
      );
      
      // Remove processed data from buffer (with overlap)
      final stepSamples = (config.stepSizeSec * config.sampleRate).toInt();
      final removeCount = stepSamples.clamp(0, _audioBuffer.length);
      _audioBuffer.removeRange(0, removeCount);
      
      print('📝 Real-time chunk processed: ${result.text.substring(0, min(30, result.text.length))}...');
      
    } catch (e) {
      state = state.copyWith(lastError: e.toString());
      print('❌ Real-time processing error: $e');
      
      // Clear buffer on error to prevent stuck state
      _audioBuffer.clear();
    }
  }

  Future<TranscriptionResult> _transcribeChunk(
    List<double> audioData,
    WhisperConfig config,
  ) async {
    // Get active whisper instance
    final activeInstance = ref.read(activeInstanceProvider).valueOrNull;
    if (activeInstance == null) {
      throw Exception('No active whisper instance for real-time processing');
    }

    // Call whisper.so through FRB
    final api = BridgeService.api;
    final resultText = await api.whisperProcessAudio(
      instanceId: activeInstance.id,
      audioData: audioData,
      language: config.language,
    );

    // Create result object
    return TranscriptionResult(
      id: 'realtime_${DateTime.now().millisecondsSinceEpoch}',
      text: resultText,
      confidence: _estimateConfidence(resultText),
      segments: [], // Real-time doesn't need detailed segments
      timestamp: DateTime.now(),
      language: config.language,
      metadata: TranscriptionMetadata(
        processingTimeSec: 0.0, // Will be calculated by caller
        audioLengthSec: audioData.length / config.sampleRate,
        instanceId: activeInstance.id,
        config: config,
        totalSegments: 1,
        totalWords: resultText.split(' ').length,
      ),
    );
  }

  Future<void> _startAudioProcessing(
    int instanceId,
    WhisperConfig config,
  ) async {
    // Start audio stream from AudioService
    final audioService = ref.read(audioServiceProvider);
    final audioStream = await audioService.getAudioStream();
    
    // Listen to audio stream
    audioStream.listen(
      (audioChunk) {
        processAudioChunk(audioChunk);
      },
      onError: (error) {
        state = state.copyWith(
          isActive: false,
          lastError: 'Audio stream error: $error',
        );
      },
    );
  }

  double _estimateConfidence(String text) {
    // Simple confidence estimation based on text characteristics
    if (text.trim().isEmpty) return 0.0;
    
    final arabicWordCount = RegExp(r'[\u0600-\u06FF]+').allMatches(text).length;
    final totalWords = text.split(' ').where((w) => w.trim().isNotEmpty).length;
    
    if (totalWords == 0) return 0.0;
    
    // Higher confidence for more Arabic words
    final arabicRatio = arabicWordCount / totalWords;
    return (0.5 + (arabicRatio * 0.5)).clamp(0.0, 1.0);
  }

  void _cleanup() {
    _processingTimer?.cancel();
    _processingTimer = null;
    _audioBuffer.clear();
    _textBuffer.clear();
    _chunkCounter = 0;
    _totalProcessingTime = 0.0;
  }

  /// Clear current transcription
  void clearTranscription() {
    _textBuffer.clear();
    state = state.copyWith(
      currentText: '',
      currentConfidence: 0.0,
    );
  }

  /// Get performance metrics
  Map<String, dynamic> getPerformanceMetrics() {
    return {
      'processedChunks': state.processedChunks,
      'averageProcessingTime': state.averageProcessingTime,
      'bufferSize': _audioBuffer.length,
      'textBufferSize': _textBuffer.length,
      'isActive': state.isActive,
    };
  }
}

// Audio Stream Provider for Real-time
@riverpod
class RealtimeAudioStream extends _$RealtimeAudioStream {
  StreamController<Float32List>? _streamController;
  Timer? _simulationTimer;

  @override
  Stream<Float32List>? build() {
    ref.onDispose(() {
      _cleanup();
    });
    return null;
  }

  /// Start real-time audio stream
  Future<void> startStream() async {
    try {
      _streamController = StreamController<Float32List>.broadcast();
      
      // In production, this would connect to actual microphone
      // For now, we simulate audio chunks
      _startSimulatedAudio();
      
      state = _streamController!.stream;
      
    } catch (e) {
      throw Exception('Failed to start audio stream: $e');
    }
  }

  /// Stop audio stream
  Future<void> stopStream() async {
    _cleanup();
    state = null;
  }

  void _startSimulatedAudio() {
    // Simulate 16kHz audio chunks (100ms chunks = 1600 samples)
    const chunkSize = 1600;
    
    _simulationTimer = Timer.periodic(
      const Duration(milliseconds: 100),
      (timer) {
        if (_streamController?.isClosed != false) return;
        
        // Generate simulated audio data
        final audioChunk = Float32List(chunkSize);
        for (int i = 0; i < chunkSize; i++) {
          // Simple sine wave simulation
          audioChunk[i] = 0.1 * sin(2 * pi * 440 * i / 16000);
        }
        
        _streamController?.add(audioChunk);
      },
    );
  }

  void _cleanup() {
    _simulationTimer?.cancel();
    _simulationTimer = null;
    _streamController?.close();
    _streamController = null;
  }
}
```

#### 2. lib/ui/components/realtime_transcription_widget.dart

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import '../../providers/realtime_provider.dart';
import '../../providers/whisper_provider.dart';
import '../../models/whisper_model.dart';

class RealtimeTranscriptionWidget extends HookConsumerWidget {
  final WhisperConfig? config;
  final VoidCallback? onTextUpdate;
  
  const RealtimeTranscriptionWidget({
    super.key,
    this.config,
    this.onTextUpdate,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final realtimeState = ref.watch(realtimeTranscriptionProvider);
    final activeInstance = ref.watch(activeInstanceProvider);
    final scrollController = useScrollController();
    
    // Auto-scroll to bottom when text updates
    useEffect(() {
      if (realtimeState.currentText.isNotEmpty) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (scrollController.hasClients) {
            scrollController.animateTo(
              scrollController.position.maxScrollExtent,
              duration: const Duration(milliseconds: 200),
              curve: Curves.easeOut,
            );
          }
        });
        onTextUpdate?.call();
      }
      return null;
    }, [realtimeState.currentText]);

    return Card(
      elevation: 4,
      child: Container(
        height: 400,
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _buildHeader(context, ref, realtimeState, activeInstance),
            const SizedBox(height: 16),
            _buildStatusIndicator(realtimeState),
            const SizedBox(height: 16),
            _buildTranscriptionArea(context, realtimeState, scrollController),
            const SizedBox(height: 16),
            _buildPerformanceMetrics(context, ref, realtimeState),
          ],
        ),
      ),
    );
  }

  Widget _buildHeader(
    BuildContext context,
    WidgetRef ref,
    RealtimeState realtimeState,
    AsyncValue<WhisperInstance?> activeInstance,
  ) {
    return Row(
      children: [
        Icon(
          realtimeState.isActive ? Icons.mic : Icons.mic_off,
          color: realtimeState.isActive ? Colors.red : Colors.grey,
        ),
        const SizedBox(width: 8),
        Text(
          'تحويل مباشر',
          style: Theme.of(context).textTheme.titleLarge?.copyWith(
            fontWeight: FontWeight.bold,
          ),
        ),
        const Spacer(),
        _buildControlButtons(context, ref, realtimeState, activeInstance),
      ],
    );
  }

  Widget _buildControlButtons(
    BuildContext context,
    WidgetRef ref,
    RealtimeState realtimeState,
    AsyncValue<WhisperInstance?> activeInstance,
  ) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Start/Stop Button
        ElevatedButton.icon(
          onPressed: activeInstance.value != null
              ? () => _toggleRealtime(ref, realtimeState, activeInstance.value!)
              : null,
          icon: Icon(
            realtimeState.isActive ? Icons.stop : Icons.play_arrow,
          ),
          label: Text(realtimeState.isActive ? 'إيقاف' : 'بدء'),
          style: ElevatedButton.styleFrom(
            backgroundColor: realtimeState.isActive ? Colors.red : Colors.green,
            foregroundColor: Colors.white,
          ),
        ),
        const SizedBox(width: 8),
        
        // Clear Button
        IconButton(
          onPressed: realtimeState.currentText.isNotEmpty
              ? () => ref.read(realtimeTranscriptionProvider.notifier)
                  .clearTranscription()
              : null,
          icon: const Icon(Icons.clear),
          tooltip: 'مسح النص',
        ),
      ],
    );
  }

  Widget _buildStatusIndicator(RealtimeState realtimeState) {
    Color statusColor;
    String statusText;
    IconData statusIcon;

    if (realtimeState.lastError != null) {
      statusColor = Colors.red;
      statusText = 'خطأ: ${realtimeState.lastError}';
      statusIcon = Icons.error;
    } else if (realtimeState.isActive) {
      statusColor = Colors.green;
      statusText = 'نشط - جاري المعالجة...';
      statusIcon = Icons.mic;
    } else {
      statusColor = Colors.grey;
      statusText = 'متوقف';
      statusIcon = Icons.mic_off;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: statusColor.withOpacity(0.1),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: statusColor),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(statusIcon, size: 16, color: statusColor),
          const SizedBox(width: 6),
          Text(
            statusText,
            style: TextStyle(
              color: statusColor,
              fontWeight: FontWeight.w500,
              fontSize: 12,
            ),
          ),
          if (realtimeState.isActive) ...[
            const SizedBox(width: 8),
            SizedBox(
              width: 12,
              height: 12,
              child: CircularProgressIndicator(
                strokeWidth: 2,
                valueColor: AlwaysStoppedAnimation<Color>(statusColor),
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildTranscriptionArea(
    BuildContext context,
    RealtimeState realtimeState,
    ScrollController scrollController,
  ) {
    return Expanded(
      child: Container(
        width: double.infinity,
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: Colors.grey[50],
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: Colors.grey[300]!),
        ),
        child: realtimeState.currentText.isEmpty
            ? _buildEmptyState(realtimeState.isActive)
            : _buildTranscriptionContent(
                context,
                realtimeState,
                scrollController,
              ),
      ),
    );
  }

  Widget _buildEmptyState(bool isActive) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            isActive ? Icons.hearing : Icons.hearing_disabled,
            size: 48,
            color: Colors.grey[400],
          ),
          const SizedBox(height: 16),
          Text(
            isActive 
                ? 'جاري الاستماع...'
                : 'اضغط بدء للتحويل المباشر',
            style: TextStyle(
              fontSize: 16,
              color: Colors.grey[600],
            ),
          ),
          if (isActive) ...[
            const SizedBox(height: 8),
            Text(
              'تحدث بوضوح باللغة العربية',
              style: TextStyle(
                fontSize: 14,
                color: Colors.grey[500],
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildTranscriptionContent(
    BuildContext context,
    RealtimeState realtimeState,
    ScrollController scrollController,
  ) {
    return SingleChildScrollView(
      controller: scrollController,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Confidence indicator
          if (realtimeState.currentConfidence > 0) ...[
            _buildConfidenceBar(realtimeState.currentConfidence),
            const SizedBox(height: 12),
          ],
          
          // Transcription text
          SelectableText(
            realtimeState.currentText,
            style: const TextStyle(
              fontSize: 18,
              height: 1.8,
              fontFamily: 'Amiri',
            ),
            textAlign: TextAlign.right,
            textDirection: TextDirection.rtl,
          ),
          
          // Live indicator
          if (realtimeState.isActive) ...[
            const SizedBox(height: 8),
            Row(
              children: [
                Container(
                  width: 8,
                  height: 8,
                  decoration: const BoxDecoration(
                    color: Colors.red,
                    shape: BoxShape.circle,
                  ),
                ),
                const SizedBox(width: 6),
                Text(
                  'مباشر',
                  style: TextStyle(
                    color: Colors.grey[600],
                    fontSize: 12,
                    fontStyle: FontStyle.italic,
                  ),
                ),
              ],
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildConfidenceBar(double confidence) {
    Color color;
    if (confidence >= 0.8) {
      color = Colors.green;
    } else if (confidence >= 0.6) {
      color = Colors.orange;
    } else {
      color = Colors.red;
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Text(
              'جودة التحويل:',
              style: TextStyle(
                fontSize: 12,
                color: Colors.grey[600],
              ),
            ),
            const SizedBox(width: 8),
            Text(
              '${(confidence * 100).toInt()}%',
              style: TextStyle(
                fontSize: 12,
                fontWeight: FontWeight.bold,
                color: color,
              ),
            ),
          ],
        ),
        const SizedBox(height: 4),
        LinearProgressIndicator(
          value: confidence,
          backgroundColor: Colors.grey[300],
          valueColor: AlwaysStoppedAnimation<Color>(color),
        ),
      ],
    );
  }

  Widget _buildPerformanceMetrics(
    BuildContext context,
    WidgetRef ref,
    RealtimeState realtimeState,
  ) {
    if (!realtimeState.isActive && realtimeState.processedChunks == 0) {
      return const SizedBox.shrink();
    }

    return Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: Colors.blue[50],
        borderRadius: BorderRadius.circular(6),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceAround,
        children: [
          _buildMetricItem(
            'المعالجة',
            '${realtimeState.averageProcessingTime.toStringAsFixed(2)}ث',
            Icons.speed,
          ),
          _buildMetricItem(
            'القطع',
            '${realtimeState.processedChunks}',
            Icons.analytics,
          ),
          _buildMetricItem(
            'الحالة',
            realtimeState.isActive ? 'نشط' : 'متوقف',
            realtimeState.isActive ? Icons.play_circle : Icons.pause_circle,
          ),
        ],
      ),
    );
  }

  Widget _buildMetricItem(String label, String value, IconData icon) {
    return Column(
      children: [
        Icon(icon, size: 16, color: Colors.blue[700]),
        const SizedBox(height: 2),
        Text(
          value,
          style: TextStyle(
            fontSize: 12,
            fontWeight: FontWeight.bold,
            color: Colors.blue[700],
          ),
        ),
        Text(
          label,
          style: TextStyle(
            fontSize: 10,
            color: Colors.blue[600],
          ),
        ),
      ],
    );
  }

  void _toggleRealtime(
    WidgetRef ref,
    RealtimeState realtimeState,
    WhisperInstance activeInstance,
  ) async {
    try {
      if (realtimeState.isActive) {
        await ref.read(realtimeTranscriptionProvider.notifier).stopRealtime();
      } else {
        await ref.read(realtimeTranscriptionProvider.notifier).startRealtime(
          whisperInstanceId: activeInstance.id,
          config: config,
        );
      }
    } catch (e) {
      // Handle error
      print('Real-time toggle error: $e');
    }
  }
}
```

### 🎯 Integration Example

#### lib/ui/screens/realtime_screen.dart

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../components/realtime_transcription_widget.dart';
import '../components/model_selector.dart';
import '../../providers/whisper_provider.dart';
import '../../models/whisper_model.dart';

class RealtimeScreen extends ConsumerWidget {
  const RealtimeScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final activeInstance = ref.watch(activeInstanceProvider);
    
    return Scaffold(
      appBar: AppBar(
        title: const Text('التحويل المباشر'),
        backgroundColor: Colors.green[700],
        foregroundColor: Colors.white,
        actions: [
          IconButton(
            onPressed: () => _showSettings(context),
            icon: const Icon(Icons.settings),
          ),
        ],
      ),
      body: Column(
        children: [
          // Model status
          Container(
            padding: const EdgeInsets.all(16),
            color: Colors.green[50],
            child: Row(
              children: [
                Icon(
                  Icons.psychology,
                  color: Colors.green[700],
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: activeInstance.when(
                    data: (instance) => instance != null
                        ? Text(
                            'النموذج النشط: ${instance.modelInfo.name}',
                            style: TextStyle(
                              fontWeight: FontWeight.w500,
                              color: Colors.green[700],
                            ),
                          )
                        : Text(
                            'لا يوجد نموذج نشط',
                            style: TextStyle(color: Colors.red[700]),
                          ),
                    loading: () => const Text('جاري التحميل...'),
                    error: (_, __) => Text(
                      'خطأ في النموذج',
                      style: TextStyle(color: Colors.red[700]),
                    ),
                  ),
                ),
              ],
            ),
          ),
          
          // Real-time transcription widget
          Expanded(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: RealtimeTranscriptionWidget(
                config: const WhisperConfig(
                  language: 'ar',
                  windowSizeSec: 5.0,
                  stepSizeSec: 2.5,
                  confidenceThreshold: 0.6,
                ),
                onTextUpdate: () {
                  // Handle text updates
                },
              ),
            ),
          ),
        ],
      ),
      floatingActionButton: activeInstance.value == null
          ? FloatingActionButton.extended(
              onPressed: () => _showModelSelector(context),
              icon: const Icon(Icons.psychology),
              label: const Text('اختر نموذج'),
              backgroundColor: Colors.green[700],
              foregroundColor: Colors.white,
            )
          : null,
    );
  }

  void _showSettings(BuildContext context) {
    // Show settings dialog
  }

  void _showModelSelector(BuildContext context) {
    showModalBottomSheet(
      context: context,
      builder: (context) => const ModelSelector(),
    );
  }
}
```

### 🔧 Key Features

1. **Sliding Window Processing**: Continuous audio processing dengan overlap
2. **whisper.so Integration**: Direct calls ke transcription engine
3. **Performance Monitoring**: Real-time metrics dan statistics
4. **Buffer Management**: Efficient memory management untuk continuous audio
5. **Arabic Optimization**: RTL text dan confidence estimation untuk Arabic

### 🔄 Next Steps

1. ✅ Real-time selesai → Lanjut ke `08-dual-library.md`
2. Implement dual library integration (whisper + quran)
3. Handle permissions dan error cases
4. Complete examples dengan production setup
