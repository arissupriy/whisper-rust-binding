# 🎨 UI Components
## Flutter Components untuk Whisper Integration

### 🎯 Overview

UI components yang menggunakan whisper-rust-binding.so melalui FRB untuk transcription. Arsitektur terpisah dimana whisper-rust-binding purely sebagai transcription engine.

### 🏗️ Architecture Separation

```
Flutter App (FRB)                    Native Libraries
├── UI Components                     ├── whisper-rust-binding.so (transcription engine)
├── Riverpod Providers               └── quran_assistant_engine.so (validation)
├── FRB Bridge Layer                  
└── Business Logic                    

Communication Flow:
Flutter → FRB → whisper.so → Results → FRB → Flutter UI
```

### 🎨 Core UI Components

#### 1. lib/ui/components/recording_button.dart

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import '../../providers/audio_provider.dart';
import '../../providers/whisper_provider.dart';
import '../../models/audio_data.dart';

class RecordingButton extends HookConsumerWidget {
  final VoidCallback? onTranscriptionComplete;
  final String language;
  
  const RecordingButton({
    super.key,
    this.onTranscriptionComplete,
    this.language = 'ar',
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final recordingState = ref.watch(recordingStateNotifierProvider);
    final amplitudeAnimation = useAnimationController(
      duration: const Duration(milliseconds: 100),
    );

    // Animate based on amplitude
    useEffect(() {
      if (recordingState.isRecording) {
        amplitudeAnimation.animateTo(recordingState.currentAmplitude);
      } else {
        amplitudeAnimation.reset();
      }
      return null;
    }, [recordingState.currentAmplitude, recordingState.isRecording]);

    return GestureDetector(
      onTap: recordingState.isRecording ? _stopRecording : _startRecording,
      child: AnimatedBuilder(
        animation: amplitudeAnimation,
        builder: (context, child) {
          final scale = 1.0 + (amplitudeAnimation.value * 0.3);
          
          return Transform.scale(
            scale: scale,
            child: Container(
              width: 120,
              height: 120,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: recordingState.isRecording 
                    ? Colors.red.withOpacity(0.8)
                    : Colors.blue,
                boxShadow: [
                  BoxShadow(
                    color: (recordingState.isRecording ? Colors.red : Colors.blue)
                        .withOpacity(0.3),
                    blurRadius: 20,
                    spreadRadius: amplitudeAnimation.value * 10,
                  ),
                ],
              ),
              child: Icon(
                recordingState.isRecording ? Icons.stop : Icons.mic,
                size: 48,
                color: Colors.white,
              ),
            ),
          );
        },
      ),
    );

    void _startRecording() async {
      try {
        await ref.read(recordingStateNotifierProvider.notifier).startRecording();
      } catch (e) {
        if (context.mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('خطأ في بدء التسجيل: $e'),
              backgroundColor: Colors.red,
            ),
          );
        }
      }
    }

    void _stopRecording() async {
      try {
        final audioData = await ref.read(recordingStateNotifierProvider.notifier)
            .stopRecording();
        
        if (audioData != null) {
          // Send to whisper.so for transcription
          await _transcribeAudio(audioData);
        }
      } catch (e) {
        if (context.mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('خطأ في إيقاف التسجيل: $e'),
              backgroundColor: Colors.red,
            ),
          );
        }
      }
    }

    Future<void> _transcribeAudio(AudioData audioData) async {
      try {
        // Use whisper.so through FRB for transcription
        final config = ref.read(whisperConfigNotifierProvider);
        await ref.read(transcriptionControllerProvider.notifier)
            .transcribeAudio(
              audioData: audioData.samples.toList(),
              config: config.copyWith(language: language),
            );
        
        onTranscriptionComplete?.call();
      } catch (e) {
        if (context.mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('خطأ في التحويل: $e'),
              backgroundColor: Colors.red,
            ),
          );
        }
      }
    }
  }
}
```

#### 2. lib/ui/components/transcription_display.dart

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../providers/whisper_provider.dart';
import '../../providers/quran_provider.dart';
import '../../models/transcription_result.dart';
import '../../models/quran_models.dart';

class TranscriptionDisplay extends ConsumerWidget {
  final bool showValidation;
  final VoidCallback? onWordTap;
  
  const TranscriptionDisplay({
    super.key,
    this.showValidation = true,
    this.onWordTap,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final transcriptionAsync = ref.watch(transcriptionControllerProvider);
    final validationAsync = ref.watch(arabicValidationProvider);

    return Card(
      elevation: 4,
      margin: const EdgeInsets.all(16),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _buildHeader(context),
            const SizedBox(height: 16),
            _buildTranscriptionContent(context, transcriptionAsync, validationAsync),
            if (showValidation) ...[
              const SizedBox(height: 16),
              _buildValidationResults(context, validationAsync),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildHeader(BuildContext context) {
    return Row(
      children: [
        const Icon(Icons.record_voice_over, color: Colors.blue),
        const SizedBox(width: 8),
        Text(
          'نتيجة التحويل',
          style: Theme.of(context).textTheme.titleLarge?.copyWith(
            fontWeight: FontWeight.bold,
          ),
        ),
        const Spacer(),
        IconButton(
          onPressed: () {
            // Clear transcription
          },
          icon: const Icon(Icons.clear),
        ),
      ],
    );
  }

  Widget _buildTranscriptionContent(
    BuildContext context,
    AsyncValue<TranscriptionResult?> transcriptionAsync,
    AsyncValue<ValidationResult?> validationAsync,
  ) {
    return transcriptionAsync.when(
      data: (result) {
        if (result == null) {
          return const Center(
            child: Text(
              'اضغط على زر التسجيل لبدء التحويل',
              style: TextStyle(
                fontSize: 16,
                color: Colors.grey,
              ),
            ),
          );
        }

        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _buildConfidenceIndicator(result.confidence),
            const SizedBox(height: 12),
            _buildTranscriptionText(context, result),
            const SizedBox(height: 12),
            _buildMetadata(context, result.metadata),
          ],
        );
      },
      loading: () => const Center(
        child: Column(
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 8),
            Text('جاري التحويل باستخدام whisper.so...'),
          ],
        ),
      ),
      error: (error, _) => Center(
        child: Text(
          'خطأ في التحويل: $error',
          style: const TextStyle(color: Colors.red),
        ),
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
      label = 'ضعيف';
    }

    return Row(
      children: [
        Text('جودة التحويل: '),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          decoration: BoxDecoration(
            color: color.withOpacity(0.2),
            borderRadius: BorderRadius.circular(12),
            border: Border.all(color: color),
          ),
          child: Text(
            '$label (${(confidence * 100).toInt()}%)',
            style: TextStyle(
              color: color,
              fontWeight: FontWeight.bold,
              fontSize: 12,
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildTranscriptionText(BuildContext context, TranscriptionResult result) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.grey[50],
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.grey[300]!),
      ),
      child: SelectableText(
        result.text,
        style: const TextStyle(
          fontSize: 18,
          height: 1.6,
          fontFamily: 'Amiri', // Arabic font
        ),
        textAlign: TextAlign.right,
        textDirection: TextDirection.rtl,
      ),
    );
  }

  Widget _buildMetadata(BuildContext context, TranscriptionMetadata metadata) {
    return Wrap(
      spacing: 16,
      runSpacing: 8,
      children: [
        _buildMetadataChip(
          icon: Icons.timer,
          label: 'المدة',
          value: '${metadata.audioLengthSec.toStringAsFixed(1)}ث',
        ),
        _buildMetadataChip(
          icon: Icons.speed,
          label: 'المعالجة',
          value: '${metadata.processingTimeSec.toStringAsFixed(2)}ث',
        ),
        _buildMetadataChip(
          icon: Icons.text_fields,
          label: 'الكلمات',
          value: '${metadata.totalWords}',
        ),
        _buildMetadataChip(
          icon: Icons.segment,
          label: 'المقاطع',
          value: '${metadata.totalSegments}',
        ),
      ],
    );
  }

  Widget _buildMetadataChip({
    required IconData icon,
    required String label,
    required String value,
  }) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: Colors.blue[50],
        borderRadius: BorderRadius.circular(16),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 16, color: Colors.blue[700]),
          const SizedBox(width: 4),
          Text(
            '$label: $value',
            style: TextStyle(
              fontSize: 12,
              color: Colors.blue[700],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildValidationResults(
    BuildContext context,
    AsyncValue<ValidationResult?> validationAsync,
  ) {
    return validationAsync.when(
      data: (validation) {
        if (validation == null) return const SizedBox.shrink();
        
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Divider(),
            Text(
              'التحقق من صحة النص العربي',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            _buildValidationChip(validation),
            if (validation.suggestions.isNotEmpty) ...[
              const SizedBox(height: 8),
              _buildSuggestions(validation.suggestions),
            ],
          ],
        );
      },
      loading: () => const LinearProgressIndicator(),
      error: (error, _) => Text(
        'خطأ في التحقق: $error',
        style: const TextStyle(color: Colors.red, fontSize: 12),
      ),
    );
  }

  Widget _buildValidationChip(ValidationResult validation) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: validation.isValid ? Colors.green[50] : Colors.red[50],
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: validation.isValid ? Colors.green : Colors.red,
        ),
      ),
      child: Row(
        children: [
          Icon(
            validation.isValid ? Icons.check_circle : Icons.error,
            color: validation.isValid ? Colors.green : Colors.red,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              validation.isValid 
                  ? 'النص صحيح وموجود في القرآن الكريم'
                  : 'النص قد يحتوي على أخطاء',
              style: TextStyle(
                color: validation.isValid ? Colors.green[700] : Colors.red[700],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSuggestions(List<String> suggestions) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('اقتراحات التصحيح:'),
        const SizedBox(height: 4),
        Wrap(
          spacing: 8,
          runSpacing: 4,
          children: suggestions.map((suggestion) {
            return GestureDetector(
              onTap: () {
                // Apply suggestion
                onWordTap?.call();
              },
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: Colors.blue[100],
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Text(
                  suggestion,
                  style: TextStyle(
                    color: Colors.blue[700],
                    fontSize: 14,
                  ),
                ),
              ),
            );
          }).toList(),
        ),
      ],
    );
  }
}
```

#### 3. lib/ui/components/model_selector.dart

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../providers/whisper_provider.dart';
import '../../models/whisper_model.dart';

class ModelSelector extends ConsumerWidget {
  final Function(WhisperInstance)? onModelSelected;
  
  const ModelSelector({
    super.key,
    this.onModelSelected,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final instancesAsync = ref.watch(whisperInstancesProvider);
    final activeInstance = ref.watch(activeInstanceProvider);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.psychology, color: Colors.purple),
                const SizedBox(width: 8),
                Text(
                  'نماذج whisper.so',
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const Spacer(),
                IconButton(
                  onPressed: () => _showLoadModelDialog(context, ref),
                  icon: const Icon(Icons.add),
                  tooltip: 'تحميل نموذج جديد',
                ),
              ],
            ),
            const SizedBox(height: 16),
            instancesAsync.when(
              data: (instances) {
                if (instances.isEmpty) {
                  return _buildEmptyState(context);
                }
                
                return Column(
                  children: instances.map((instance) {
                    final isActive = activeInstance.valueOrNull?.id == instance.id;
                    return _buildInstanceTile(context, ref, instance, isActive);
                  }).toList(),
                );
              },
              loading: () => const Center(
                child: CircularProgressIndicator(),
              ),
              error: (error, _) => _buildErrorState(context, error.toString()),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(24),
      child: Column(
        children: [
          Icon(
            Icons.psychology_outlined,
            size: 64,
            color: Colors.grey[400],
          ),
          const SizedBox(height: 16),
          Text(
            'لا توجد نماذج محملة',
            style: TextStyle(
              fontSize: 16,
              color: Colors.grey[600],
            ),
          ),
          const SizedBox(height: 8),
          Text(
            'اضغط على + لتحميل نموذج whisper',
            style: TextStyle(
              fontSize: 14,
              color: Colors.grey[500],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildInstanceTile(
    BuildContext context,
    WidgetRef ref,
    WhisperInstance instance,
    bool isActive,
  ) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      decoration: BoxDecoration(
        color: isActive ? Colors.blue[50] : Colors.transparent,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: isActive ? Colors.blue : Colors.grey[300]!,
          width: isActive ? 2 : 1,
        ),
      ),
      child: ListTile(
        leading: CircleAvatar(
          backgroundColor: isActive ? Colors.blue : Colors.grey[400],
          child: Text(
            instance.modelInfo.modelType.substring(0, 1).toUpperCase(),
            style: const TextStyle(
              color: Colors.white,
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
        title: Text(
          instance.modelInfo.name,
          style: TextStyle(
            fontWeight: isActive ? FontWeight.bold : FontWeight.normal,
          ),
        ),
        subtitle: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('المسار: ${instance.modelPath.split('/').last}'),
            Text('الحجم: ${instance.modelInfo.fileSizeMB.toStringAsFixed(1)} MB'),
            Text('اللغات: ${instance.modelInfo.supportedLanguages.join(', ')}'),
          ],
        ),
        trailing: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (isActive)
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: Colors.green,
                  borderRadius: BorderRadius.circular(12),
                ),
                child: const Text(
                  'نشط',
                  style: TextStyle(
                    color: Colors.white,
                    fontSize: 12,
                  ),
                ),
              ),
            IconButton(
              onPressed: () => _removeInstance(ref, instance.id),
              icon: const Icon(Icons.delete, color: Colors.red),
              tooltip: 'حذف النموذج',
            ),
          ],
        ),
        onTap: () => _selectInstance(ref, instance),
      ),
    );
  }

  Widget _buildErrorState(BuildContext context, String error) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.red[50],
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.red[300]!),
      ),
      child: Row(
        children: [
          const Icon(Icons.error, color: Colors.red),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              'خطأ في تحميل النماذج: $error',
              style: const TextStyle(color: Colors.red),
            ),
          ),
        ],
      ),
    );
  }

  void _showLoadModelDialog(BuildContext context, WidgetRef ref) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('تحميل نموذج whisper'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Text('اختر نموذج للتحميل:'),
            const SizedBox(height: 16),
            _buildModelOption(
              context,
              ref,
              'ggml-tiny.bin',
              'صغير (39 MB)',
              'سريع، دقة متوسطة',
            ),
            _buildModelOption(
              context,
              ref,
              'ggml-base.bin',
              'أساسي (142 MB)',
              'متوازن السرعة والدقة',
            ),
            _buildModelOption(
              context,
              ref,
              'ggml-small.bin',
              'صغير متقدم (466 MB)',
              'دقة عالية، بطيء نسبياً',
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('إلغاء'),
          ),
        ],
      ),
    );
  }

  Widget _buildModelOption(
    BuildContext context,
    WidgetRef ref,
    String fileName,
    String size,
    String description,
  ) {
    return Card(
      child: ListTile(
        title: Text(fileName),
        subtitle: Text('$size - $description'),
        trailing: const Icon(Icons.download),
        onTap: () {
          Navigator.of(context).pop();
          _loadModel(ref, fileName);
        },
      ),
    );
  }

  void _selectInstance(WidgetRef ref, WhisperInstance instance) {
    ref.read(activeInstanceProvider.notifier).setActiveInstance(instance);
    onModelSelected?.call(instance);
  }

  void _removeInstance(WidgetRef ref, int instanceId) {
    ref.read(whisperInstancesProvider.notifier).removeInstance(instanceId);
  }

  void _loadModel(WidgetRef ref, String fileName) async {
    try {
      // In production, you would download the model file first
      final modelPath = 'assets/models/$fileName';
      await ref.read(whisperInstancesProvider.notifier).loadModel(modelPath);
    } catch (e) {
      // Handle error
    }
  }
}
```

### 🔄 Complete Usage Example

#### lib/ui/screens/transcription_screen.dart

```dart
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../components/recording_button.dart';
import '../components/transcription_display.dart';
import '../components/model_selector.dart';
import '../../providers/whisper_provider.dart';

class TranscriptionScreen extends ConsumerWidget {
  const TranscriptionScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('تحويل الصوت إلى نص'),
        backgroundColor: Colors.blue[700],
        foregroundColor: Colors.white,
      ),
      body: Column(
        children: [
          // Model Selector
          const ModelSelector(),
          
          // Recording Button
          Expanded(
            child: Center(
              child: RecordingButton(
                onTranscriptionComplete: () {
                  // Handle completion
                },
              ),
            ),
          ),
          
          // Transcription Results
          const Expanded(
            flex: 2,
            child: TranscriptionDisplay(
              showValidation: true,
            ),
          ),
        ],
      ),
    );
  }
}
```

### 🎯 Key Points

1. **Pure Transcription Engine**: whisper-rust-binding.so hanya untuk transcription
2. **FRB Integration**: Flutter berkomunikasi dengan .so melalui FRB
3. **Separated Concerns**: UI terpisah dari transcription logic
4. **Real-time Feedback**: Visual feedback untuk recording dan processing
5. **Arabic Support**: RTL text dan font Arab

### 🔄 Next Steps

1. ✅ UI Components selesai → Lanjut ke `07-realtime-transcription.md`
2. Implement real-time transcription features
3. Handle dual library integration
4. Complete examples
