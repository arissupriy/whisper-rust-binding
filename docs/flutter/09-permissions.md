# 🔐 Android Permissions & Security
## Complete Permission Management untuk Audio Recording dan Storage

### 🎯 Overview

Dokumentasi lengkap untuk mengelola Android permissions yang diperlukan untuk aplikasi transcription dengan **whisper.so** dan **quran_assistant_engine.so**, termasuk runtime permissions, security policies, dan best practices.

### 📋 Required Permissions

#### 1. android/app/src/main/AndroidManifest.xml

```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    xmlns:tools="http://schemas.android.com/tools"
    package="com.example.whisper_quran_app">

    <!-- Audio Recording Permissions -->
    <uses-permission android:name="android.permission.RECORD_AUDIO" />
    <uses-permission android:name="android.permission.MODIFY_AUDIO_SETTINGS" />
    
    <!-- Storage Permissions -->
    <uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" />
    <uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" 
        android:maxSdkVersion="28" />
    
    <!-- Network Permissions (untuk download models) -->
    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
    
    <!-- Vibration (untuk haptic feedback) -->
    <uses-permission android:name="android.permission.VIBRATE" />
    
    <!-- Wake Lock (untuk mencegah sleep saat recording) -->
    <uses-permission android:name="android.permission.WAKE_LOCK" />
    
    <!-- Foreground Service (untuk background transcription) -->
    <uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
    <uses-permission android:name="android.permission.FOREGROUND_SERVICE_MICROPHONE" />
    
    <!-- Audio Focus -->
    <uses-permission android:name="android.permission.MODIFY_AUDIO_SETTINGS" />

    <application
        android:label="Whisper Quran Assistant"
        android:name="${applicationName}"
        android:icon="@mipmap/ic_launcher"
        android:allowBackup="false"
        android:usesCleartextTraffic="true"
        android:requestLegacyExternalStorage="true"
        android:preserveLegacyExternalStorage="true">
        
        <!-- Main Activity -->
        <activity
            android:name=".MainActivity"
            android:exported="true"
            android:launchMode="singleTop"
            android:theme="@style/LaunchTheme"
            android:configChanges="orientation|keyboardHidden|keyboard|screenSize|smallestScreenSize|locale|layoutDirection|fontScale|screenLayout|density|uiMode"
            android:hardwareAccelerated="true"
            android:windowSoftInputMode="adjustResize">
            
            <!-- Standard Android credentials -->
            <meta-data
                android:name="io.flutter.embedding.android.NormalTheme"
                android:resource="@style/NormalTheme" />
            
            <intent-filter android:autoVerify="true">
                <action android:name="android.intent.action.MAIN"/>
                <category android:name="android.intent.category.LAUNCHER"/>
            </intent-filter>
            
            <!-- Audio file handling -->
            <intent-filter>
                <action android:name="android.intent.action.VIEW" />
                <category android:name="android.intent.category.DEFAULT" />
                <data android:mimeType="audio/*" />
            </intent-filter>
        </activity>
        
        <!-- Background Service untuk transcription -->
        <service
            android:name="com.example.whisper_quran_app.TranscriptionService"
            android:enabled="true"
            android:exported="false"
            android:foregroundServiceType="microphone" />
        
        <!-- File Provider untuk secure file sharing -->
        <provider
            android:name="androidx.core.content.FileProvider"
            android:authorities="${applicationId}.fileprovider"
            android:exported="false"
            android:grantUriPermissions="true">
            <meta-data
                android:name="android.support.FILE_PROVIDER_PATHS"
                android:resource="@xml/file_paths" />
        </provider>

        <!-- Don't delete the meta-data below.
             This is used by the Flutter tool to generate GeneratedPluginRegistrant.java -->
        <meta-data
            android:name="flutterEmbedding"
            android:value="2" />
    </application>

    <!-- Queries for Intent resolution -->
    <queries>
        <!-- File manager apps -->
        <intent>
            <action android:name="android.intent.action.GET_CONTENT" />
            <data android:mimeType="audio/*" />
        </intent>
        
        <!-- Audio players -->
        <intent>
            <action android:name="android.intent.action.VIEW" />
            <data android:mimeType="audio/*" />
        </intent>
    </queries>
</manifest>
```

#### 2. android/app/src/main/res/xml/file_paths.xml

```xml
<?xml version="1.0" encoding="utf-8"?>
<paths xmlns:android="http://schemas.android.com/apk/res/android">
    <!-- Internal app storage -->
    <files-path 
        name="internal_files" 
        path="." />
    
    <!-- Cache directory -->
    <cache-path 
        name="cache_files" 
        path="." />
    
    <!-- External files (recordings) -->
    <external-files-path 
        name="external_files" 
        path="." />
    
    <!-- External cache -->
    <external-cache-path 
        name="external_cache" 
        path="." />
    
    <!-- Downloads folder -->
    <external-path 
        name="downloads" 
        path="Download/" />
        
    <!-- Documents folder -->
    <external-path 
        name="documents" 
        path="Documents/" />
        
    <!-- Whisper models folder -->
    <external-files-path 
        name="whisper_models" 
        path="whisper_models/" />
        
    <!-- Audio recordings folder -->
    <external-files-path 
        name="recordings" 
        path="recordings/" />
</paths>
```

### 🔒 Permission Management Service

#### 1. lib/services/permission_service.dart

```dart
import 'dart:io';
import 'package:permission_handler/permission_handler.dart';
import 'package:device_info_plus/device_info_plus.dart';
import '../models/errors.dart';

/// Service untuk mengelola Android permissions
class PermissionService {
  static final PermissionService _instance = PermissionService._internal();
  factory PermissionService() => _instance;
  PermissionService._internal();

  // Cache untuk SDK version
  int? _sdkVersion;
  
  /// Get Android SDK version
  Future<int> get androidSdkVersion async {
    if (_sdkVersion != null) return _sdkVersion!;
    
    if (Platform.isAndroid) {
      final deviceInfo = DeviceInfoPlugin();
      final androidInfo = await deviceInfo.androidInfo;
      _sdkVersion = androidInfo.version.sdkInt;
      return _sdkVersion!;
    }
    
    return 0; // Not Android
  }

  /// Check all required permissions
  Future<PermissionStatus> checkAllPermissions() async {
    try {
      final permissions = await _getRequiredPermissions();
      
      Map<Permission, PermissionStatus> statuses = {};
      for (final permission in permissions) {
        statuses[permission] = await permission.status;
      }

      // Return worst status
      if (statuses.values.any((status) => status.isDenied)) {
        return PermissionStatus.denied;
      }
      
      if (statuses.values.any((status) => status.isPermanentlyDenied)) {
        return PermissionStatus.permanentlyDenied;
      }
      
      if (statuses.values.every((status) => status.isGranted)) {
        return PermissionStatus.granted;
      }
      
      return PermissionStatus.denied;

    } catch (e) {
      throw AppError.permissionError(
        message: 'Failed to check permissions: $e',
        permission: 'all',
        isTemporary: true,
      );
    }
  }

  /// Request all required permissions
  Future<bool> requestAllPermissions() async {
    try {
      final permissions = await _getRequiredPermissions();
      
      // Request permissions
      Map<Permission, PermissionStatus> statuses = 
          await permissions.request();

      // Check results
      final allGranted = statuses.values.every((status) => status.isGranted);
      
      if (!allGranted) {
        final deniedPerms = statuses.entries
            .where((entry) => !entry.value.isGranted)
            .map((entry) => entry.key.toString())
            .join(', ');
            
        throw AppError.permissionError(
          message: 'Permissions denied: $deniedPerms',
          permission: deniedPerms,
          isTemporary: statuses.values.any((s) => s.isPermanentlyDenied),
        );
      }

      return true;

    } catch (e) {
      if (e is AppError) rethrow;
      
      throw AppError.permissionError(
        message: 'Failed to request permissions: $e',
        permission: 'all',
        isTemporary: true,
      );
    }
  }

  /// Check specific permission
  Future<bool> hasPermission(Permission permission) async {
    try {
      final status = await permission.status;
      return status.isGranted;
    } catch (e) {
      return false;
    }
  }

  /// Request specific permission
  Future<bool> requestPermission(Permission permission) async {
    try {
      if (await hasPermission(permission)) {
        return true;
      }

      final status = await permission.request();
      return status.isGranted;
      
    } catch (e) {
      throw AppError.permissionError(
        message: 'Failed to request ${permission.toString()}: $e',
        permission: permission.toString(),
        isTemporary: true,
      );
    }
  }

  /// Check microphone permission specifically
  Future<bool> hasMicrophonePermission() async {
    return await hasPermission(Permission.microphone);
  }

  /// Request microphone permission
  Future<bool> requestMicrophonePermission() async {
    return await requestPermission(Permission.microphone);
  }

  /// Check storage permissions
  Future<bool> hasStoragePermission() async {
    final sdkVersion = await androidSdkVersion;
    
    if (sdkVersion >= 33) {
      // Android 13+ uses scoped storage
      return true; // App-specific directories don't need permission
    } else if (sdkVersion >= 30) {
      // Android 11-12 uses scoped storage but may need MANAGE_EXTERNAL_STORAGE
      return await hasPermission(Permission.storage);
    } else {
      // Android 10 and below need WRITE_EXTERNAL_STORAGE
      return await hasPermission(Permission.storage);
    }
  }

  /// Request storage permission
  Future<bool> requestStoragePermission() async {
    final sdkVersion = await androidSdkVersion;
    
    if (sdkVersion >= 33) {
      // Android 13+ uses scoped storage
      return true;
    } else {
      return await requestPermission(Permission.storage);
    }
  }

  /// Check if app can show permission rationale
  Future<bool> shouldShowRequestPermissionRationale(Permission permission) async {
    try {
      final status = await permission.status;
      return status.isDenied && !status.isPermanentlyDenied;
    } catch (e) {
      return false;
    }
  }

  /// Open app settings for manual permission grant
  Future<bool> openAppSettings() async {
    try {
      return await openAppSettings();
    } catch (e) {
      return false;
    }
  }

  /// Get permission status details
  Future<PermissionStatusDetails> getPermissionDetails() async {
    final microphone = await Permission.microphone.status;
    final storage = await _getStoragePermissionStatus();
    final internet = PermissionStatus.granted; // Always granted
    
    return PermissionStatusDetails(
      microphone: microphone,
      storage: storage,
      internet: internet,
      notifications: await Permission.notification.status,
      androidSdkVersion: await androidSdkVersion,
    );
  }

  /// Get required permissions based on Android version
  Future<List<Permission>> _getRequiredPermissions() async {
    final sdkVersion = await androidSdkVersion;
    final permissions = <Permission>[
      Permission.microphone, // Always required
    ];

    // Storage permissions based on Android version
    if (sdkVersion < 33) {
      permissions.add(Permission.storage);
    }

    // Notification permission for Android 13+
    if (sdkVersion >= 33) {
      permissions.add(Permission.notification);
    }

    return permissions;
  }

  /// Get storage permission status based on Android version
  Future<PermissionStatus> _getStoragePermissionStatus() async {
    final sdkVersion = await androidSdkVersion;
    
    if (sdkVersion >= 33) {
      // Android 13+ uses scoped storage
      return PermissionStatus.granted;
    } else {
      return await Permission.storage.status;
    }
  }
}

/// Model untuk permission status details
class PermissionStatusDetails {
  final PermissionStatus microphone;
  final PermissionStatus storage;
  final PermissionStatus internet;
  final PermissionStatus notifications;
  final int androidSdkVersion;

  PermissionStatusDetails({
    required this.microphone,
    required this.storage,
    required this.internet,
    required this.notifications,
    required this.androidSdkVersion,
  });

  bool get allRequired => microphone.isGranted && storage.isGranted;
  bool get allOptional => notifications.isGranted;
  bool get hasAnyDenied => 
      microphone.isDenied || storage.isDenied;
  bool get hasAnyPermanentlyDenied => 
      microphone.isPermanentlyDenied || storage.isPermanentlyDenied;

  Map<String, dynamic> toJson() => {
    'microphone': microphone.toString(),
    'storage': storage.toString(),
    'internet': internet.toString(),
    'notifications': notifications.toString(),
    'androidSdkVersion': androidSdkVersion,
    'allRequired': allRequired,
    'allOptional': allOptional,
    'hasAnyDenied': hasAnyDenied,
    'hasAnyPermanentlyDenied': hasAnyPermanentlyDenied,
  };
}
```

#### 2. lib/providers/permission_provider.dart

```dart
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:permission_handler/permission_handler.dart';
import '../services/permission_service.dart';
import '../models/errors.dart';

part 'permission_provider.g.dart';

// Permission Service Provider
@riverpod
PermissionService permissionService(PermissionServiceRef ref) {
  return PermissionService();
}

// Permission Status Provider
@riverpod
class PermissionStatus extends _$PermissionStatus {
  @override
  Future<PermissionStatusDetails> build() async {
    final service = ref.read(permissionServiceProvider);
    return await service.getPermissionDetails();
  }

  /// Refresh permission status
  Future<void> refresh() async {
    state = const AsyncValue.loading();
    
    try {
      final service = ref.read(permissionServiceProvider);
      final details = await service.getPermissionDetails();
      state = AsyncValue.data(details);
    } catch (e, stackTrace) {
      state = AsyncValue.error(e, stackTrace);
    }
  }

  /// Request all permissions
  Future<bool> requestAllPermissions() async {
    try {
      final service = ref.read(permissionServiceProvider);
      final granted = await service.requestAllPermissions();
      
      // Refresh status after request
      await refresh();
      
      return granted;
    } catch (e) {
      // Refresh even on error to get current status
      await refresh();
      rethrow;
    }
  }

  /// Request specific permission
  Future<bool> requestPermission(Permission permission) async {
    try {
      final service = ref.read(permissionServiceProvider);
      final granted = await service.requestPermission(permission);
      
      // Refresh status
      await refresh();
      
      return granted;
    } catch (e) {
      await refresh();
      rethrow;
    }
  }

  /// Open app settings
  Future<void> openAppSettings() async {
    final service = ref.read(permissionServiceProvider);
    await service.openAppSettings();
    
    // Refresh when user returns to app
    Future.delayed(const Duration(seconds: 1), () async {
      await refresh();
    });
  }
}

// Microphone Permission Provider
@riverpod
class MicrophonePermission extends _$MicrophonePermission {
  @override
  Future<bool> build() async {
    final service = ref.read(permissionServiceProvider);
    return await service.hasMicrophonePermission();
  }

  /// Request microphone permission
  Future<bool> request() async {
    state = const AsyncValue.loading();
    
    try {
      final service = ref.read(permissionServiceProvider);
      final granted = await service.requestMicrophonePermission();
      
      state = AsyncValue.data(granted);
      
      // Also refresh main permission status
      ref.read(permissionStatusProvider.notifier).refresh();
      
      return granted;
    } catch (e, stackTrace) {
      state = AsyncValue.error(e, stackTrace);
      rethrow;
    }
  }

  /// Check current status
  Future<void> check() async {
    final service = ref.read(permissionServiceProvider);
    final hasPermission = await service.hasMicrophonePermission();
    state = AsyncValue.data(hasPermission);
  }
}

// Storage Permission Provider
@riverpod
class StoragePermission extends _$StoragePermission {
  @override
  Future<bool> build() async {
    final service = ref.read(permissionServiceProvider);
    return await service.hasStoragePermission();
  }

  /// Request storage permission
  Future<bool> request() async {
    state = const AsyncValue.loading();
    
    try {
      final service = ref.read(permissionServiceProvider);
      final granted = await service.requestStoragePermission();
      
      state = AsyncValue.data(granted);
      
      // Refresh main permission status
      ref.read(permissionStatusProvider.notifier).refresh();
      
      return granted;
    } catch (e, stackTrace) {
      state = AsyncValue.error(e, stackTrace);
      rethrow;
    }
  }

  /// Check current status
  Future<void> check() async {
    final service = ref.read(permissionServiceProvider);
    final hasPermission = await service.hasStoragePermission();
    state = AsyncValue.data(hasPermission);
  }
}
```

### 🎨 Permission UI Components

#### 1. lib/ui/pages/permission_page.dart

```dart
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:permission_handler/permission_handler.dart';
import '../../providers/permission_provider.dart';
import '../../services/permission_service.dart';

class PermissionPage extends ConsumerStatefulWidget {
  final VoidCallback? onPermissionsGranted;
  
  const PermissionPage({
    super.key,
    this.onPermissionsGranted,
  });

  @override
  ConsumerState<PermissionPage> createState() => _PermissionPageState();
}

class _PermissionPageState extends ConsumerState<PermissionPage>
    with TickerProviderStateMixin {
  
  late AnimationController _animationController;
  late Animation<double> _fadeAnimation;
  
  @override
  void initState() {
    super.initState();
    _animationController = AnimationController(
      duration: const Duration(milliseconds: 800),
      vsync: this,
    );
    
    _fadeAnimation = Tween<double>(
      begin: 0.0,
      end: 1.0,
    ).animate(CurvedAnimation(
      parent: _animationController,
      curve: Curves.easeInOut,
    ));
    
    _animationController.forward();
  }
  
  @override
  void dispose() {
    _animationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final permissionStatus = ref.watch(permissionStatusProvider);

    return Scaffold(
      body: permissionStatus.when(
        data: (details) {
          // If all required permissions are granted, call callback
          if (details.allRequired && widget.onPermissionsGranted != null) {
            WidgetsBinding.instance.addPostFrameCallback((_) {
              widget.onPermissionsGranted!();
            });
          }

          return _buildPermissionContent(details);
        },
        loading: () => _buildLoadingState(),
        error: (error, _) => _buildErrorState(error.toString()),
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
          Text('جاري فحص الصلاحيات...'),
        ],
      ),
    );
  }

  Widget _buildErrorState(String error) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(
              Icons.error_outline,
              size: 64,
              color: Colors.red,
            ),
            const SizedBox(height: 16),
            const Text(
              'خطأ في فحص الصلاحيات',
              style: TextStyle(
                fontSize: 20,
                fontWeight: FontWeight.bold,
                color: Colors.red,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              error,
              textAlign: TextAlign.center,
              style: const TextStyle(color: Colors.grey),
            ),
            const SizedBox(height: 24),
            ElevatedButton(
              onPressed: () {
                ref.read(permissionStatusProvider.notifier).refresh();
              },
              child: const Text('إعادة المحاولة'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildPermissionContent(PermissionStatusDetails details) {
    return FadeTransition(
      opacity: _fadeAnimation,
      child: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            children: [
              // Header
              _buildHeader(),
              
              const SizedBox(height: 32),
              
              // Permission list
              Expanded(
                child: SingleChildScrollView(
                  child: Column(
                    children: [
                      _buildPermissionItem(
                        icon: Icons.mic,
                        title: 'تسجيل الصوت',
                        description: 'مطلوب لتحويل الكلام إلى نص باستخدام whisper.so',
                        status: details.microphone,
                        isRequired: true,
                        onTap: () => _requestMicrophonePermission(),
                      ),
                      
                      const SizedBox(height: 16),
                      
                      _buildPermissionItem(
                        icon: Icons.storage,
                        title: 'تخزين الملفات',
                        description: 'مطلوب لحفظ التسجيلات ونماذج whisper.so',
                        status: details.storage,
                        isRequired: true,
                        onTap: () => _requestStoragePermission(),
                      ),
                      
                      const SizedBox(height: 16),
                      
                      _buildPermissionItem(
                        icon: Icons.notifications,
                        title: 'الإشعارات',
                        description: 'لعرض حالة المعالجة في الخلفية',
                        status: details.notifications,
                        isRequired: false,
                        onTap: () => _requestNotificationPermission(),
                      ),
                      
                      const SizedBox(height: 24),
                      
                      // Android version info
                      _buildAndroidVersionInfo(details.androidSdkVersion),
                    ],
                  ),
                ),
              ),
              
              // Action buttons
              _buildActionButtons(details),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildHeader() {
    return Column(
      children: [
        Container(
          width: 120,
          height: 120,
          decoration: BoxDecoration(
            color: Colors.blue.withOpacity(0.1),
            shape: BoxShape.circle,
          ),
          child: const Icon(
            Icons.security,
            size: 60,
            color: Colors.blue,
          ),
        ),
        const SizedBox(height: 24),
        const Text(
          'صلاحيات التطبيق',
          style: TextStyle(
            fontSize: 28,
            fontWeight: FontWeight.bold,
          ),
        ),
        const SizedBox(height: 8),
        const Text(
          'نحتاج إلى بعض الصلاحيات لتشغيل التطبيق بشكل صحيح',
          textAlign: TextAlign.center,
          style: TextStyle(
            fontSize: 16,
            color: Colors.grey,
          ),
        ),
      ],
    );
  }

  Widget _buildPermissionItem({
    required IconData icon,
    required String title,
    required String description,
    required PermissionStatus status,
    required bool isRequired,
    required VoidCallback onTap,
  }) {
    Color statusColor;
    IconData statusIcon;
    String statusText;

    switch (status) {
      case PermissionStatus.granted:
        statusColor = Colors.green;
        statusIcon = Icons.check_circle;
        statusText = 'مُمنوح';
        break;
      case PermissionStatus.denied:
        statusColor = Colors.orange;
        statusIcon = Icons.warning;
        statusText = 'مرفوض';
        break;
      case PermissionStatus.permanentlyDenied:
        statusColor = Colors.red;
        statusIcon = Icons.block;
        statusText = 'مرفوض نهائياً';
        break;
      default:
        statusColor = Colors.grey;
        statusIcon = Icons.help_outline;
        statusText = 'غير محدد';
    }

    return Card(
      elevation: 2,
      child: InkWell(
        onTap: status.isGranted ? null : onTap,
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              // Permission icon
              Container(
                width: 48,
                height: 48,
                decoration: BoxDecoration(
                  color: Colors.blue.withOpacity(0.1),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Icon(icon, color: Colors.blue),
              ),
              
              const SizedBox(width: 16),
              
              // Permission details
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Text(
                          title,
                          style: const TextStyle(
                            fontSize: 16,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                        if (isRequired) ...[
                          const SizedBox(width: 8),
                          Container(
                            padding: const EdgeInsets.symmetric(
                              horizontal: 6,
                              vertical: 2,
                            ),
                            decoration: BoxDecoration(
                              color: Colors.red.withOpacity(0.1),
                              borderRadius: BorderRadius.circular(8),
                            ),
                            child: const Text(
                              'مطلوب',
                              style: TextStyle(
                                fontSize: 10,
                                color: Colors.red,
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                          ),
                        ],
                      ],
                    ),
                    const SizedBox(height: 4),
                    Text(
                      description,
                      style: TextStyle(
                        fontSize: 14,
                        color: Colors.grey[600],
                      ),
                    ),
                  ],
                ),
              ),
              
              const SizedBox(width: 16),
              
              // Status indicator
              Column(
                children: [
                  Icon(statusIcon, color: statusColor),
                  const SizedBox(height: 4),
                  Text(
                    statusText,
                    style: TextStyle(
                      fontSize: 12,
                      color: statusColor,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildAndroidVersionInfo(int sdkVersion) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.grey.withOpacity(0.1),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        children: [
          const Icon(Icons.android, color: Colors.green),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'إصدار Android: $sdkVersion',
                  style: const TextStyle(fontWeight: FontWeight.bold),
                ),
                Text(
                  _getAndroidVersionName(sdkVersion),
                  style: TextStyle(
                    color: Colors.grey[600],
                    fontSize: 12,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildActionButtons(PermissionStatusDetails details) {
    if (details.allRequired) {
      return SizedBox(
        width: double.infinity,
        child: ElevatedButton.icon(
          onPressed: widget.onPermissionsGranted,
          icon: const Icon(Icons.check),
          label: const Text('متابعة'),
          style: ElevatedButton.styleFrom(
            backgroundColor: Colors.green,
            foregroundColor: Colors.white,
            padding: const EdgeInsets.all(16),
          ),
        ),
      );
    }

    return Column(
      children: [
        if (details.hasAnyPermanentlyDenied) ...[
          SizedBox(
            width: double.infinity,
            child: ElevatedButton.icon(
              onPressed: _openAppSettings,
              icon: const Icon(Icons.settings),
              label: const Text('فتح إعدادات التطبيق'),
              style: ElevatedButton.styleFrom(
                backgroundColor: Colors.orange,
                foregroundColor: Colors.white,
                padding: const EdgeInsets.all(16),
              ),
            ),
          ),
          const SizedBox(height: 12),
        ],
        
        SizedBox(
          width: double.infinity,
          child: ElevatedButton.icon(
            onPressed: _requestAllPermissions,
            icon: const Icon(Icons.security),
            label: const Text('طلب جميع الصلاحيات'),
            style: ElevatedButton.styleFrom(
              padding: const EdgeInsets.all(16),
            ),
          ),
        ),
      ],
    );
  }

  // Permission request methods
  Future<void> _requestMicrophonePermission() async {
    try {
      HapticFeedback.lightImpact();
      await ref.read(microphonePermissionProvider.notifier).request();
    } catch (e) {
      _showErrorSnackBar('فشل في طلب صلاحية الميكروفون: $e');
    }
  }

  Future<void> _requestStoragePermission() async {
    try {
      HapticFeedback.lightImpact();
      await ref.read(storagePermissionProvider.notifier).request();
    } catch (e) {
      _showErrorSnackBar('فشل في طلب صلاحية التخزين: $e');
    }
  }

  Future<void> _requestNotificationPermission() async {
    try {
      HapticFeedback.lightImpact();
      await ref.read(permissionStatusProvider.notifier).requestPermission(
        Permission.notification,
      );
    } catch (e) {
      _showErrorSnackBar('فشل في طلب صلاحية الإشعارات: $e');
    }
  }

  Future<void> _requestAllPermissions() async {
    try {
      HapticFeedback.mediumImpact();
      await ref.read(permissionStatusProvider.notifier).requestAllPermissions();
    } catch (e) {
      _showErrorSnackBar('فشل في طلب الصلاحيات: $e');
    }
  }

  Future<void> _openAppSettings() async {
    try {
      HapticFeedback.lightImpact();
      await ref.read(permissionStatusProvider.notifier).openAppSettings();
    } catch (e) {
      _showErrorSnackBar('فشل في فتح إعدادات التطبيق: $e');
    }
  }

  void _showErrorSnackBar(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(message),
        backgroundColor: Colors.red,
        behavior: SnackBarBehavior.floating,
      ),
    );
  }

  String _getAndroidVersionName(int sdkVersion) {
    if (sdkVersion >= 34) return 'Android 14+ (Upside Down Cake)';
    if (sdkVersion >= 33) return 'Android 13 (Tiramisu)';
    if (sdkVersion >= 32) return 'Android 12L (Snow Cone v2)';
    if (sdkVersion >= 31) return 'Android 12 (Snow Cone)';
    if (sdkVersion >= 30) return 'Android 11 (Red Velvet Cake)';
    if (sdkVersion >= 29) return 'Android 10 (Quince Tart)';
    if (sdkVersion >= 28) return 'Android 9 (Pie)';
    return 'Android $sdkVersion';
  }
}
```

### 🔐 Security Best Practices

#### 1. Runtime Permission Checks

```dart
// Always check before using whisper.so atau quran_assistant_engine.so
Future<bool> _ensurePermissionsForTranscription() async {
  final permissionService = PermissionService();
  
  // Check microphone permission
  if (!await permissionService.hasMicrophonePermission()) {
    throw AppError.permissionError(
      message: 'Microphone permission required for whisper.so',
      permission: 'microphone',
    );
  }
  
  // Check storage permission for model files
  if (!await permissionService.hasStoragePermission()) {
    throw AppError.permissionError(
      message: 'Storage permission required for whisper models',
      permission: 'storage',
    );
  }
  
  return true;
}
```

#### 2. Secure File Handling

```dart
// Use app-specific directories untuk .so files dan models
Path getSecureModelsDirectory() {
  return path.join(
    getApplicationDocumentsDirectory(),
    'whisper_models',
  );
}

Path getSecureRecordingsDirectory() {
  return path.join(
    getApplicationDocumentsDirectory(),
    'recordings',
  );
}
```

### 🎯 Integration dengan Whisper Services

```dart
// Dalam WhisperService, selalu check permissions
class WhisperService {
  Future<void> _checkPermissions() async {
    await _ensurePermissionsForTranscription();
  }
  
  Future<TranscriptionResult> transcribeAudio(/*...*/) async {
    await _checkPermissions(); // Check sebelum panggil whisper.so
    
    // Proceed dengan transcription
    return await _whisperApi.transcribe(/*...*/);
  }
}
```

### 🔄 Next Steps

1. ✅ Permissions selesai → Lanjut ke `10-examples.md`
2. Complete working examples
3. Production deployment guide
4. Performance optimization tips
