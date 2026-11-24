use mx5ic_core::dsp::*;
use std::f32::consts::PI;

#[test]
fn test_biquad_flat_peaking() {
    let mut bq = Biquad::new();
    bq.set_params(FilterType::PeakingEQ, 48000.0, 1000.0, 0.0, 1.0);
    // At 0 dB gain, output must equal input (within float epsilon)
    let out = bq.process(0.5);
    assert!((out - 0.5).abs() < 1e-4);
}

#[test]
fn test_parametric_eq_flat() {
    let mut eq = ParametricEq::new(48000.0);
    // Default config has 0 dB on all 8 bands
    for i in 0..100 {
        let sample = (i as f32 * 0.1).sin();
        let (out_l, out_r) = eq.process_stereo(sample, sample);
        assert!((out_l - sample).abs() < 1e-3, "Mismatch at sample {}: in={}, out={}", i, sample, out_l);
        assert!((out_r - sample).abs() < 1e-3);
    }
}

#[test]
fn test_virtual_bass_harmonics() {
    let mut vb = VirtualBass::new(48000.0);
    vb.drive = 2.5;
    vb.mix = 0.5;

    // Feed a pure 50 Hz sub-bass sine wave
    let mut max_out = 0.0f32;
    for i in 0..1000 {
        let t = i as f32 / 48000.0;
        let s = (2.0 * PI * 50.0 * t).sin();
        let (out_l, _) = vb.process_stereo(s, s);
        if out_l.abs() > max_out {
            max_out = out_l.abs();
        }
    }
    // With harmonics mixed in, peak must be greater than original 1.0
    assert!(max_out > 1.05, "Virtual bass did not generate expected harmonic energy");
}

#[test]
fn test_limiter_brickwall_clamping() {
    let mut limiter = TruePeakLimiter::new(48000.0, -1.0, 2.0); // -1.0 dBTP
    let ceiling = 10.0f32.powf(-1.0 / 20.0); // ~0.89125

    // Feed a massive +12 dBFS signal (amplitude 4.0)
    for i in 0..1000 {
        let t = i as f32 / 48000.0;
        let hot_sample = 4.0 * (2.0 * PI * 1000.0 * t).sin();
        let (out_l, out_r) = limiter.process_stereo(hot_sample, hot_sample);
        assert!(out_l.abs() <= ceiling + 1e-4, "Left channel overshoot: {} > {}", out_l.abs(), ceiling);
        assert!(out_r.abs() <= ceiling + 1e-4, "Right channel overshoot: {} > {}", out_r.abs(), ceiling);
    }
}

#[test]
fn test_crossfade_equal_power() {
    let mut fader = EqualPowerCrossfader::new(48000.0, 10.0); // 10ms = 480 samples
    fader.set_target(false); // Fade from wet to dry

    let mut min_power = 10.0f32;
    let mut max_power = 0.0f32;

    // Crossfading identical coherent signals
    for _ in 0..480 {
        let (out_l, _) = fader.process_stereo(1.0, 1.0, 1.0, 1.0);
        let power = out_l;
        if power < min_power { min_power = power; }
        if power > max_power { max_power = power; }
    }

    // Power must remain within [1.0, 1.414] during coherent crossfade without dip below 1.0
    assert!(min_power >= 0.99, "Crossfade volume dip detected: {}", min_power);
}

#[test]
fn test_spatial_vocal_orbit_zero_echo_and_3d_panning() {
    let mut orbit = SpatialVocalOrbit::new(48000.0);
    orbit.amount = 0.8;
    orbit.orbit_speed_hz = 0.5;

    // 1. Verify zero echo: feed a short burst (500 samples = ~10ms), ensure zero duplicate echo at t > 30ms (i > 1440)
    let total_samples = 48000; // 1 second
    let mut echo_zone_energy = 0.0f32;

    for i in 0..total_samples {
        let input = if i < 500 {
            (2.0 * PI * 1000.0 * (i as f32 / 48000.0)).sin()
        } else {
            0.0
        };

        let (out_l, out_r) = orbit.process_stereo(input, input);

        // At t > 30ms (i > 1440), there must be STRICTLY ZERO echo/double sound
        if i > 1500 {
            echo_zone_energy += out_l.abs() + out_r.abs();
        }
    }

    assert!(echo_zone_energy < 1e-6, "Audible echo or duplicate sound detected at t > 30ms! Energy: {}", echo_zone_energy);

    // 2. Verify 3D earbud movement across LFO cycle
    orbit.reset();
    let mut left_dominant_frames = 0;
    let mut right_dominant_frames = 0;

    for i in 0..48000 {
        let t = i as f32 / 48000.0;
        let s = (2.0 * PI * 440.0 * t).sin();
        let (out_l, out_r) = orbit.process_stereo(s, s);

        if out_l.abs() > out_r.abs() * 1.15 {
            left_dominant_frames += 1;
        } else if out_r.abs() > out_l.abs() * 1.15 {
            right_dominant_frames += 1;
        }
    }

    assert!(left_dominant_frames > 2000, "3D orbit failed to sweep to left earbud: {}", left_dominant_frames);
    assert!(right_dominant_frames > 2000, "3D orbit failed to sweep to right earbud: {}", right_dominant_frames);
}

#[test]
fn test_spatial_orbit_mono_null_compatibility() {
    let mut orbit = SpatialVocalOrbit::new(48000.0);
    orbit.amount = 0.6;

    // Test a continuous musical signal
    let mut max_comb_deviation = 0.0f32;
    for i in 0..24000 {
        let t = i as f32 / 48000.0;
        let s = (2.0 * PI * 800.0 * t).sin();
        let (l, r) = orbit.process_stereo(s, s);

        // Sum to mono: (L + R) / sqrt(2)
        let mono_sum = (l + r) * 0.7071;
        let diff = (mono_sum - s * 0.7071).abs();
        if diff > max_comb_deviation {
            max_comb_deviation = diff;
        }
    }

    // Mono sum must not suffer destructive phase cancellation (< 3.0 dB equivalent)
    assert!(max_comb_deviation < 1.5, "Excessive mono cancellation notch: {}", max_comb_deviation);
}

#[test]
fn test_dialogue_formant_boost_and_de_essing() {
    let mut dia = DialogueEnhancer::new(48000.0);
    dia.set_amount(0.8); // 80% dialogue enhancement

    // Test speech formant frequency (2400 Hz)
    let mut max_formant = 0.0f32;
    for i in 0..1000 {
        let t = i as f32 / 48000.0;
        let s = (2.0 * PI * 2400.0 * t).sin();
        let (out_l, _out_r) = dia.process_stereo(s, s);
        if out_l.abs() > max_formant {
            max_formant = out_l.abs();
        }
    }
    // Formant must be boosted (> 1.25x amplitude)
    assert!(max_formant > 1.25, "Dialogue formant boost did not increase 2.4 kHz speech intelligibility: {}", max_formant);

    // Test harsh sibilance spike clamp (6500 Hz high amplitude burst)
    let mut sibilance_out = 0.0f32;
    for i in 0..1000 {
        let t = i as f32 / 48000.0;
        let s = 1.5 * (2.0 * PI * 6500.0 * t).sin();
        let (out_l, _) = dia.process_stereo(s, s);
        sibilance_out = out_l.abs();
    }
    // De-esser should clamp harsh sibilance
    assert!(sibilance_out < 1.45, "De-esser failed to attenuate high-frequency sibilance burst: {}", sibilance_out);
}

#[test]
fn test_sound_profiles_and_spatial_surround_modes() {
    for profile in SoundProfile::ALL {
        let eq = profile.target_eq_gains();
        assert_eq!(eq.len(), 8);
        let (vb_mix, vb_drive) = profile.virtual_bass_params();
        assert!(vb_mix >= 0.0 && vb_mix <= 1.0);
        assert!(vb_drive >= 1.0 && vb_drive <= 4.0);
        assert!(profile.default_spatial_orbit() >= 0.0 && profile.default_spatial_orbit() <= 1.0);
        assert!(profile.crossfeed_amount() >= 0.0 && profile.crossfeed_amount() <= 1.0);
    }

    // Cinema 3D Surround should have strong orbit and sub-bass lift
    let cinema = SoundProfile::CinemaSurround3D;
    assert!(cinema.default_spatial_orbit() >= 0.80);
    assert!(cinema.target_eq_gains()[0] > 4.0);

    // Studio Reference should have completely flat target EQ
    let studio = SoundProfile::StudioReference;
    for gain in studio.target_eq_gains() {
        assert_eq!(gain, 0.0);
    }
}

#[test]
fn test_adaptive_noise_gate_suppression_and_transparency() {
    let mut gate = AdaptiveNoiseGate::new(48000.0);
    gate.set_threshold_db(-50.0);
    gate.set_reduction_db(18.0);

    // 1. Quiet noise signal (-66 dBFS: amplitude ~0.0005 at 1 kHz)
    let low_amp = 0.0005f32;
    let mut gated_max = 0.0f32;
    for i in 0..4800 { // 100ms
        let t = i as f32 / 48000.0;
        let s = low_amp * (2.0 * PI * 1000.0 * t).sin();
        let (l, _) = gate.process_stereo(s, s);
        if i > 2400 && l.abs() > gated_max {
            gated_max = l.abs();
        }
    }
    // Gated output should be significantly attenuated (< 35% of low_amp = >9 dB suppression, reaching 18dB)
    assert!(gated_max < low_amp * 0.35, "Noise gate failed to suppress low-level noise: in={}, out={}", low_amp, gated_max);

    // 2. Strong musical signal (amplitude 0.5 = -6 dBFS at 1 kHz)
    let signal_amp = 0.5f32;
    let mut opened_max = 0.0f32;
    for i in 0..4800 {
        let t = i as f32 / 48000.0;
        let s = signal_amp * (2.0 * PI * 1000.0 * t).sin();
        let (l, _) = gate.process_stereo(s, s);
        if i > 2400 && l.abs() > opened_max {
            opened_max = l.abs();
        }
    }
    // Gate should open transparently with near-unity gain (within 2%)
    assert!((opened_max - signal_amp).abs() < 0.02, "Noise gate failed to achieve transparency on active signal: in={}, out={}", signal_amp, opened_max);

    // 3. Subsonic rumble rejection (< 22 Hz)
    let mut rumble_max = 0.0f32;
    for i in 0..4800 {
        let t = i as f32 / 48000.0;
        let s = 0.5 * (2.0 * PI * 8.0 * t).sin(); // 8 Hz rumble
        let (l, _) = gate.process_stereo(s, s);
        if i > 2400 && l.abs() > rumble_max {
            rumble_max = l.abs();
        }
    }
    // 8 Hz rumble must be attenuated by at least 15 dB (< 0.09)
    assert!(rumble_max < 0.12, "High-pass failed to block 8Hz subsonic rumble: {}", rumble_max);
}

#[test]
fn test_ai_music_understanding_and_decision() {
    use mx5ic_core::ai::{FeatureExtractor, MusicIntelligenceEngine, MusicGenre};

    let extractor = FeatureExtractor::new(48000.0);
    let engine = MusicIntelligenceEngine::new(48000.0);

    // Generate a synthetic test buffer simulating an EDM/Hip-Hop drop (heavy sub-bass + high air)
    let mut buffer = vec![0.0f32; 1024];
    for i in 0..512 {
        let t = i as f32 / 48000.0;
        let bass = 0.7 * (2.0 * PI * 55.0 * t).sin();
        let air = 0.2 * (2.0 * PI * 12000.0 * t).sin();
        buffer[i * 2] = bass + air;
        buffer[i * 2 + 1] = bass - air;
    }

    let feat = extractor.extract(&buffer);
    let understanding = engine.analyze(&feat);
    let decision = engine.decide(&understanding);

    assert!(
        understanding.primary_genre == MusicGenre::EDM || understanding.primary_genre == MusicGenre::HipHop,
        "AI misclassified heavy bass stream: {:?}", understanding.primary_genre
    );
    assert!(decision.virtual_bass_mix > 0.30, "AI failed to allocate bass mix for electronic track");
    assert!(decision.spatial_orbit_amount > 0.30, "AI failed to allocate spatial orbit for electronic track");
}

#[test]
fn test_silence_is_absolutely_silent() {
    use std::sync::Arc;
    use mx5ic_core::pipeline::{EngineConfig, EngineControls, EnginePipeline};

    let controls = Arc::new(EngineControls::new());
    let mut pipeline = EnginePipeline::new(EngineConfig::default(), controls);

    // Feed 48,000 samples (1 full second @ 48kHz stereo) of pure silence
    let input = vec![0.0f32; 256];
    let mut output = vec![0.0f32; 256];

    for _ in 0..(48000 / 128) {
        pipeline.process_interleaved(&input, &mut output);
        for &s in output.iter() {
            assert_eq!(s, 0.0f32, "Audio pipeline generated noise/explosion during silence: {}", s);
        }
    }
}

#[test]
fn test_model_suite_discovery_and_manifest() {
    use mx5ic_core::ai::ModelSuiteManager;

    let manifest = ModelSuiteManager::discover_and_load();
    assert!(
        manifest.is_some(),
        "Failed to discover Deep AI Model Suite in workspace"
    );

    let m = manifest.unwrap();
    assert!(m.is_complete, "Model suite is missing required neural networks");
    assert!(
        m.total_size_mb >= 100.0,
        "Total model suite size {:.2} MB is smaller than 100 MB minimum",
        m.total_size_mb
    );
    assert!(
        m.loaded_models.len() >= 5,
        "Expected at least 5 neural networks loaded in suite"
    );
}

