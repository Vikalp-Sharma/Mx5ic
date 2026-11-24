package com.mx5ic.enhancer;

import android.media.audiofx.BassBoost;
import android.media.audiofx.Equalizer;
import android.media.audiofx.Virtualizer;
import android.util.Log;

/**
 * Manages native Android system-level audio effects (Virtualizer, BassBoost, Equalizer)
 * attached to the global audio session (Session 0) or specific playback sessions.
 *
 * This provides 100% in-place audio enhancement:
 * - ZERO double sound (audio is modified directly inside AudioFlinger before reaching DAC)
 * - ZERO delay or buffer latency
 * - Authentic 3D binaural spatial widening & psychoacoustic virtual bass
 */
public class NativeAudioEffects {
    private static final String TAG = "Mx5icNativeEffects";

    private Virtualizer virtualizer;
    private BassBoost bassBoost;
    private Equalizer equalizer;
    private boolean isAttached = false;
    private int currentSessionId = 0;

    public synchronized void attach(int sessionId) {
        if (isAttached && currentSessionId == sessionId) {
            return;
        }
        release();

        this.currentSessionId = sessionId;
        try {
            // 1. 3D Spatial Virtualizer (Earbud & Headphone Binaural Widening)
            virtualizer = new Virtualizer(1000, sessionId);
            if (virtualizer.getStrengthSupported()) {
                virtualizer.setStrength((short) 650); // 65% initial 3D immersion
            }
            if (virtualizer.canVirtualize(2, Virtualizer.VIRTUALIZATION_MODE_BINAURAL)) {
                virtualizer.forceVirtualizationMode(Virtualizer.VIRTUALIZATION_MODE_BINAURAL);
            }
            virtualizer.setEnabled(true);
            Log.i(TAG, "Attached 3D Spatial Virtualizer to session " + sessionId);
        } catch (Throwable e) {
            Log.w(TAG, "Virtualizer not supported on session " + sessionId + ": " + e.getMessage());
        }

        try {
            // 2. Psychoacoustic Virtual Bass
            bassBoost = new BassBoost(1000, sessionId);
            if (bassBoost.getStrengthSupported()) {
                bassBoost.setStrength((short) 550); // 55% virtual bass drive
            }
            bassBoost.setEnabled(true);
            Log.i(TAG, "Attached BassBoost to session " + sessionId);
        } catch (Throwable e) {
            Log.w(TAG, "BassBoost not supported on session " + sessionId + ": " + e.getMessage());
        }

        try {
            // 3. Multi-Band Mastering Equalizer
            equalizer = new Equalizer(1000, sessionId);
            equalizer.setEnabled(true);
            applyHarmanTargetEq();
            Log.i(TAG, "Attached Equalizer to session " + sessionId);
        } catch (Throwable e) {
            Log.w(TAG, "Equalizer not supported on session " + sessionId + ": " + e.getMessage());
        }

        isAttached = true;
    }

    public synchronized void updateAiParameters(float spatialAmount, float bassMix, float dialogueBoost) {
        if (!isAttached) return;

        // Dynamic 3D Spatialization
        if (virtualizer != null && virtualizer.getEnabled()) {
            try {
                short strength = (short) (Math.max(0.0f, Math.min(1.0f, spatialAmount)) * 1000);
                virtualizer.setStrength(strength);
            } catch (Throwable ignored) {}
        }

        // Dynamic Virtual Bass
        if (bassBoost != null && bassBoost.getEnabled()) {
            try {
                short strength = (short) (Math.max(0.0f, Math.min(1.0f, bassMix)) * 1000);
                bassBoost.setStrength(strength);
            } catch (Throwable ignored) {}
        }

        // Dialogue Intelligibility & Clarity
        if (equalizer != null && equalizer.getEnabled()) {
            try {
                short numBands = equalizer.getNumberOfBands();
                for (short b = 0; b < numBands; b++) {
                    int centerFreq = equalizer.getCenterFreq(b) / 1000; // in Hz
                    // Speech formant range: 1.5 kHz - 3.5 kHz
                    if (centerFreq >= 1500 && centerFreq <= 3500) {
                        short maxBand = equalizer.getBandLevelRange()[1];
                        short boost = (short) (maxBand * dialogueBoost * 0.45f);
                        equalizer.setBandLevel(b, boost);
                    }
                }
            } catch (Throwable ignored) {}
        }
    }

    public synchronized void setHardwareProfile(int profileIdx) {
        // 1. Update Hardware Virtualizer 3D strength
        if (virtualizer != null && virtualizer.getEnabled()) {
            try {
                short vStr = 600;
                switch (profileIdx) {
                    case 0: vStr = 1000; break; // Cinema 3D Surround
                    case 1: vStr = 850;  break; // Concert Hall Live
                    case 2: vStr = 450;  break; // Vocal Intimacy
                    case 3: vStr = 500;  break; // Club Bass Reflex
                    case 4: vStr = 350;  break; // Night Audiophile
                    case 5: vStr = 150;  break; // Studio Reference
                    case 6: default: vStr = 650; break; // Harman Consumer
                }
                virtualizer.setStrength(vStr);
            } catch (Throwable ignored) {}
        }

        // 2. Update Hardware BassBoost
        if (bassBoost != null && bassBoost.getEnabled()) {
            try {
                short bStr = 400;
                switch (profileIdx) {
                    case 0: bStr = 750; break; // Cinema
                    case 1: bStr = 400; break; // Concert
                    case 2: bStr = 250; break; // Vocal
                    case 3: bStr = 950; break; // Club Bass
                    case 4: bStr = 350; break; // Night
                    case 5: bStr = 0;   break; // Studio Reference (Flat)
                    case 6: default: bStr = 550; break; // Harman
                }
                bassBoost.setStrength(bStr);
            } catch (Throwable ignored) {}
        }

        // 3. Update Hardware Equalizer Curves
        if (equalizer == null || !equalizer.getEnabled()) return;
        try {
            short numBands = equalizer.getNumberOfBands();
            short[] range = equalizer.getBandLevelRange(); // e.g. [-1500, +1500] mB
            short maxLvl = range[1];

            for (short b = 0; b < numBands; b++) {
                int freq = equalizer.getCenterFreq(b) / 1000;
                short targetLevel = 0;

                switch (profileIdx) {
                    case 0: // Cinema 3D Surround
                        if (freq < 150) targetLevel = (short) (maxLvl * 0.35f);
                        else if (freq > 8000) targetLevel = (short) (maxLvl * 0.25f);
                        break;
                    case 1: // Concert Hall Live 3D
                        if (freq > 6000) targetLevel = (short) (maxLvl * 0.40f);
                        else if (freq >= 2000 && freq <= 5000) targetLevel = (short) (maxLvl * 0.15f);
                        break;
                    case 2: // Vocal Intimacy & Center
                        if (freq >= 1800 && freq <= 3500) targetLevel = (short) (maxLvl * 0.45f);
                        else if (freq < 150) targetLevel = (short) (-maxLvl * 0.10f);
                        break;
                    case 3: // Club Bass Reflex
                        if (freq < 120) targetLevel = (short) (maxLvl * 0.55f);
                        else if (freq > 10000) targetLevel = (short) (maxLvl * 0.20f);
                        break;
                    case 4: // Night Audiophile & Dialogue
                        if (freq >= 1500 && freq <= 3000) targetLevel = (short) (maxLvl * 0.25f);
                        else if (freq > 7000) targetLevel = (short) (-maxLvl * 0.25f); // De-hiss damping
                        break;
                    case 5: // Studio Reference Flat
                        targetLevel = 0;
                        break;
                    case 6: // Harman Target (Consumer)
                    default:
                        if (freq < 150) targetLevel = (short) (maxLvl * 0.30f);
                        else if (freq > 8000) targetLevel = (short) (maxLvl * 0.20f);
                        break;
                }

                equalizer.setBandLevel(b, targetLevel);
            }
        } catch (Exception ignored) {}
    }

    private void applyHarmanTargetEq() {
        setHardwareProfile(6);
    }

    public synchronized void release() {
        isAttached = false;
        if (virtualizer != null) {
            try {
                virtualizer.setEnabled(false);
                virtualizer.release();
            } catch (Throwable ignored) {}
            virtualizer = null;
        }
        if (bassBoost != null) {
            try {
                bassBoost.setEnabled(false);
                bassBoost.release();
            } catch (Throwable ignored) {}
            bassBoost = null;
        }
        if (equalizer != null) {
            try {
                equalizer.setEnabled(false);
                equalizer.release();
            } catch (Throwable ignored) {}
            equalizer = null;
        }
    }
}
