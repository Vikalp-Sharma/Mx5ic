package com.mx5ic.enhancer;

import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.Service;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.media.AudioAttributes;
import android.media.AudioFormat;
import android.media.AudioPlaybackCaptureConfiguration;
import android.media.AudioRecord;
import android.media.audiofx.AudioEffect;
import android.media.projection.MediaProjection;
import android.media.projection.MediaProjectionManager;
import android.os.Build;
import android.os.IBinder;
import android.util.Log;

public class AudioCaptureService extends Service {
    public static final String EXTRA_RESULT_CODE = "RESULT_CODE";
    public static final String EXTRA_DATA = "DATA";

    private static final String TAG = "Mx5icAudioService";
    private static final int NOTIFICATION_ID = 4810;
    private static final String CHANNEL_ID = "mx5ic_local_channel";

    private static OnDeviceAudioEngine audioEngine;
    private static NativeAudioEffects nativeAudioEffects;
    private MediaProjection mediaProjection;
    private AudioRecord audioRecord;
    private volatile boolean isRunning = false;
    private Thread processingThread;

    private static class AudioSessionReceiver extends BroadcastReceiver {
        private final java.lang.ref.WeakReference<AudioCaptureService> serviceRef;

        public AudioSessionReceiver(AudioCaptureService service) {
            this.serviceRef = new java.lang.ref.WeakReference<>(service);
        }

        @Override
        public void onReceive(Context context, Intent intent) {
            AudioCaptureService service = serviceRef.get();
            if (service == null) return;
            String action = intent.getAction();
            if (AudioEffect.ACTION_OPEN_AUDIO_EFFECT_CONTROL_SESSION.equals(action)) {
                int sessionId = intent.getIntExtra(AudioEffect.EXTRA_AUDIO_SESSION, 0);
                if (sessionId != 0 && nativeAudioEffects != null) {
                    Log.i(TAG, "Audio session opened by media player: " + sessionId);
                    nativeAudioEffects.attach(sessionId);
                }
            } else if (AudioEffect.ACTION_CLOSE_AUDIO_EFFECT_CONTROL_SESSION.equals(action)) {
                int sessionId = intent.getIntExtra(AudioEffect.EXTRA_AUDIO_SESSION, 0);
                Log.i(TAG, "Audio session closed by media player: " + sessionId);
            }
        }
    }

    private final AudioSessionReceiver audioSessionReceiver = new AudioSessionReceiver(this);

    private MediaProjection.Callback projectionCallback;

    private static class ProjectionCallback extends MediaProjection.Callback {
        private final java.lang.ref.WeakReference<AudioCaptureService> serviceRef;

        public ProjectionCallback(AudioCaptureService service) {
            this.serviceRef = new java.lang.ref.WeakReference<>(service);
        }

        @Override
        public void onStop() {
            AudioCaptureService service = serviceRef.get();
            if (service != null) {
                Log.i(TAG, "MediaProjection session terminated by system");
                service.stopSelf();
            }
        }
    }

    public static OnDeviceAudioEngine getAudioEngine() {
        return audioEngine;
    }

    public static NativeAudioEffects getNativeAudioEffects() {
        return nativeAudioEffects;
    }

    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }

    @Override
    public void onCreate() {
        super.onCreate();
        try {
            if (audioEngine == null) {
                audioEngine = new OnDeviceAudioEngine(getApplicationContext(), 48000.0f);
            }
            if (nativeAudioEffects == null) {
                nativeAudioEffects = new NativeAudioEffects();
            }
            // Attach in-place system effect chain to Session 0 (Global System Mix)
            nativeAudioEffects.attach(0);

            // Listen for new media sessions from Spotify, YouTube Music, etc.
            IntentFilter filter = new IntentFilter();
            filter.addAction(AudioEffect.ACTION_OPEN_AUDIO_EFFECT_CONTROL_SESSION);
            filter.addAction(AudioEffect.ACTION_CLOSE_AUDIO_EFFECT_CONTROL_SESSION);
            if (Build.VERSION.SDK_INT >= 33) {
                registerReceiver(audioSessionReceiver, filter, Context.RECEIVER_EXPORTED);
            } else {
                registerReceiver(audioSessionReceiver, filter);
            }
        } catch (Throwable t) {
            Log.w(TAG, "Error in AudioCaptureService onCreate: " + t.getMessage(), t);
        }
    }

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        if (intent == null) return START_NOT_STICKY;

        try {
            createNotificationChannel();
            Notification notification = buildNotification("Autonomous In-Place AI Mastering: 3D Spatial Orbit & Virtual Bass (Zero Echo)");
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                startForeground(NOTIFICATION_ID, notification, android.content.pm.ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PROJECTION);
            } else {
                startForeground(NOTIFICATION_ID, notification);
            }

            int resultCode = intent.getIntExtra(EXTRA_RESULT_CODE, 0);
            Intent resultData = intent.getParcelableExtra(EXTRA_DATA);

            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q && resultData != null) {
                MediaProjectionManager manager = (MediaProjectionManager) getSystemService(Context.MEDIA_PROJECTION_SERVICE);
                if (manager != null) {
                    mediaProjection = manager.getMediaProjection(resultCode, resultData);
                    if (mediaProjection != null) {
                        projectionCallback = new ProjectionCallback(this);
                        mediaProjection.registerCallback(projectionCallback, new android.os.Handler(android.os.Looper.getMainLooper()));
                        startOnDeviceProcessing();
                    }
                }
            }
        } catch (Throwable t) {
            Log.e(TAG, "Error in onStartCommand: " + t.getMessage(), t);
        }

        return START_STICKY;
    }

    private void startOnDeviceProcessing() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q || mediaProjection == null) return;

        try {
            AudioPlaybackCaptureConfiguration config = new AudioPlaybackCaptureConfiguration.Builder(mediaProjection)
                    .addMatchingUsage(AudioAttributes.USAGE_MEDIA)
                    .addMatchingUsage(AudioAttributes.USAGE_GAME)
                    .addMatchingUsage(AudioAttributes.USAGE_UNKNOWN)
                    .excludeUid(android.os.Process.myUid())
                    .build();

            boolean isFloatFormat = true;
            int recordBufferSize = AudioRecord.getMinBufferSize(48000, AudioFormat.CHANNEL_IN_STEREO, AudioFormat.ENCODING_PCM_FLOAT) * 2;

            if (recordBufferSize <= 0) {
                isFloatFormat = false;
                recordBufferSize = AudioRecord.getMinBufferSize(48000, AudioFormat.CHANNEL_IN_STEREO, AudioFormat.ENCODING_PCM_16BIT) * 2;
            }

            AudioFormat captureFormat = new AudioFormat.Builder()
                    .setEncoding(isFloatFormat ? AudioFormat.ENCODING_PCM_FLOAT : AudioFormat.ENCODING_PCM_16BIT)
                    .setSampleRate(48000)
                    .setChannelMask(AudioFormat.CHANNEL_IN_STEREO)
                    .build();

            try {
                audioRecord = new AudioRecord.Builder()
                        .setAudioPlaybackCaptureConfig(config)
                        .setAudioFormat(captureFormat)
                        .setBufferSizeInBytes(Math.max(recordBufferSize, 4096))
                        .build();
            } catch (Throwable e) {
                Log.w(TAG, "PCM_FLOAT capture rejected by HAL, falling back to PCM_16BIT: " + e.getMessage());
                isFloatFormat = false;
                recordBufferSize = Math.max(AudioRecord.getMinBufferSize(48000, AudioFormat.CHANNEL_IN_STEREO, AudioFormat.ENCODING_PCM_16BIT) * 2, 4096);
                captureFormat = new AudioFormat.Builder()
                        .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
                        .setSampleRate(48000)
                        .setChannelMask(AudioFormat.CHANNEL_IN_STEREO)
                        .build();
                audioRecord = new AudioRecord.Builder()
                        .setAudioPlaybackCaptureConfig(config)
                        .setAudioFormat(captureFormat)
                        .setBufferSizeInBytes(recordBufferSize)
                        .build();
            }

            if (audioRecord != null && audioRecord.getState() == AudioRecord.STATE_INITIALIZED) {
                audioRecord.startRecording();
                isRunning = true;

                processingThread = new Thread(new AudioProcessingRunner(audioRecord, isFloatFormat, this), "Mx5icOnDeviceAudioThread");
                processingThread.setPriority(Thread.MAX_PRIORITY);
                processingThread.start();
                Log.i(TAG, "AudioPlaybackCapture active (format: " + (isFloatFormat ? "FLOAT" : "PCM_16BIT") + ")");
            } else {
                Log.w(TAG, "AudioRecord could not initialize on this hardware. Native in-place DSP effects remain active!");
            }

        } catch (Throwable t) {
            Log.e(TAG, "Failed to start AudioPlaybackCapture: " + t.getMessage(), t);
        }
    }

    public boolean isStreamRunning() {
        return isRunning;
    }

    private void createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            NotificationChannel channel = new NotificationChannel(
                    CHANNEL_ID,
                    "Mx5ic On-Device Audio Enhancer",
                    NotificationManager.IMPORTANCE_LOW
            );
            NotificationManager manager = getSystemService(NotificationManager.class);
            if (manager != null) {
                manager.createNotificationChannel(channel);
            }
        }
    }

    private Notification buildNotification(String text) {
        Notification.Builder builder;
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            builder = new Notification.Builder(this, CHANNEL_ID);
        } else {
            builder = new Notification.Builder(this);
        }
        return builder.setContentTitle("Mx5ic Autonomous On-Device Mastering")
                .setContentText(text)
                .setSmallIcon(android.R.drawable.ic_media_play)
                .setOngoing(true)
                .build();
    }

    @Override
    public void onDestroy() {
        isRunning = false;
        try {
            unregisterReceiver(audioSessionReceiver);
        } catch (Throwable ignored) {}

        if (processingThread != null) {
            try {
                processingThread.interrupt();
            } catch (Throwable ignored) {}
        }
        if (audioRecord != null) {
            try {
                audioRecord.stop();
                audioRecord.release();
            } catch (Throwable ignored) {}
        }
        if (nativeAudioEffects != null) {
            try {
                nativeAudioEffects.release();
            } catch (Throwable ignored) {}
        }
        if (mediaProjection != null) {
            try {
                if (projectionCallback != null) {
                    mediaProjection.unregisterCallback(projectionCallback);
                }
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
                    mediaProjection.stop();
                }
            } catch (Throwable ignored) {}
        }
        super.onDestroy();
    }

    /**
     * AI Acoustic Analyzer Runner:
     * Reads system audio via AudioPlaybackCapture solely to perform real-time
     * music understanding and autonomous steering of the in-place hardware AudioEffects.
     *
     * DOES NOT output a duplicate AudioTrack stream to the speakers, ensuring:
     * 1. ZERO double playback / echo.
     * 2. ZERO buffer latency.
     * 3. 100% in-place audio enhancement directly in Android AudioFlinger.
     */
    private static class AudioProcessingRunner implements Runnable {
        private final AudioRecord record;
        private final boolean isFloatFormat;
        private final AudioCaptureService service;

        public AudioProcessingRunner(AudioRecord record, boolean isFloatFormat, AudioCaptureService service) {
            this.record = record;
            this.isFloatFormat = isFloatFormat;
            this.service = service;
        }

        @Override
        public void run() {
            float[] inBuffer = new float[256];
            float[] dummyOutBuffer = new float[256];
            short[] shortBuffer = !isFloatFormat ? new short[256] : null;

            while (service.isRunning && !Thread.currentThread().isInterrupted()) {
                try {
                    int read = 0;
                    if (isFloatFormat) {
                        read = record.read(inBuffer, 0, inBuffer.length, AudioRecord.READ_BLOCKING);
                    } else if (shortBuffer != null) {
                        read = record.read(shortBuffer, 0, shortBuffer.length, AudioRecord.READ_BLOCKING);
                        if (read > 0) {
                            for (int i = 0; i < read; i++) {
                                inBuffer[i] = shortBuffer[i] / 32768.0f;
                            }
                        }
                    }

                    if (read > 0) {
                        int frames = read / 2;
                        float peak = 0.0f;
                        for (int i = 0; i < read; i++) {
                            float abs = Math.abs(inBuffer[i]);
                            if (abs > peak) peak = abs;
                        }

                        // Silence Squelch Gate: If input is silent (peak < -66 dBFS), reset
                        if (peak < 0.0005f) {
                            if (audioEngine != null) {
                                audioEngine.reset();
                            }
                        } else {
                            if (audioEngine != null) {
                                audioEngine.process(inBuffer, dummyOutBuffer, frames);

                                // Dynamically update Android's in-place AudioEffects from AI decisions
                                if (nativeAudioEffects != null) {
                                    nativeAudioEffects.updateAiParameters(
                                            audioEngine.getSpatial3dAmount(),
                                            audioEngine.getVirtualBassMix(),
                                            audioEngine.getDialogueBoost()
                                    );
                                }
                            }
                        }
                    }
                } catch (Throwable t) {
                    Log.w(TAG, "AudioProcessingRunner read loop error: " + t.getMessage());
                    break;
                }
            }
        }
    }
}
