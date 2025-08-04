// Example: Integrating whisper-rust-binding with quran_assistant_engine
import 'generated_bindings_whisper.dart' as whisper;
import 'generated_bindings_quran.dart' as quran;

class IntegratedQuranTranscriber {
  late whisper.WhisperRustBinding _whisperBinding;
  late quran.QuranAssistantEngine _quranEngine;
  
  Future<void> initialize() async {
    // Initialize both libraries
    _whisperBinding = whisper.WhisperRustBinding(
      dylib: DynamicLibrary.open('libwhisper_rust_binding.so')
    );
    _quranEngine = quran.QuranAssistantEngine(
      dylib: DynamicLibrary.open('libquran_assistant_engine.so')
    );
    
    // Register validation callback
    await _whisperBinding.whisperRegisterQuranValidator(
      callback: _validateWithQuranEngine,
    );
  }
  
  ValidationResponse _validateWithQuranEngine(
    String transcribedText, 
    int ayahId, 
    int surahId
  ) {
    // Use quran_assistant_engine for validation
    final ayahData = _quranEngine.getAyah(surahId: surahId, ayahId: ayahId);
    final similarity = _quranEngine.calculateSimilarity(
      text1: transcribedText, 
      text2: ayahData.text
    );
    
    return ValidationResponse(
      isValid: similarity > 0.8,
      similarityScore: similarity,
      correctText: ayahData.text,
      wordCountMatch: _countMatchingWords(transcribedText, ayahData.text),
      ayahPosition: ayahData.position,
    );
  }
  
  Future<void> startMurajahSession({
    required int surahId,
    required int startingAyahId,
  }) async {
    await _whisperBinding.startQuranSession(
      instanceId: 'murajaah_session',
      surahId: surahId,
      startingAyahId: startingAyahId,
      sessionConfig: const QuranSessionConfig(),
    );
  }
}
