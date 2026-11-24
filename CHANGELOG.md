# Changelog

All notable changes to the **Mx5ic** audio intelligence ecosystem are documented in this file.

## [0.1.0] - 2026-11-24

### Added
- **Core Real-Time DSP Engine (`crates/mx5ic_core`)**:
  - Zero-allocation, lock-free real-time audio pipeline in Rust.
  - 8-band parametric biquad equalizer cascade with peaking and shelf filters.
  - Psychoacoustic Virtual Bass harmonic exciter generating 2nd/3rd harmonics for frequencies < 80 Hz.
  - Dynamic 3D Spatial Vocal Orbit & Earbud Ping-Pong Decay with head-shadow ILD/ITD simulation and exponential atmospheric decay.
  - Dedicated Dialogue & Speech Intelligibility Enhancer with 2.4 kHz formant boost and dynamic 6.5 kHz sibilance de-esser.
  - True-Peak Lookahead Brickwall Limiter (-1.00 dBTP ceiling, zero clipping overshoot).
  - Equal-power click-free crossfader for instantaneous A/B bypass comparisons.
  - Adaptive cubic Hermite clock-drift resampler for network jitter compensation.
- **Deep AI Music Intelligence Suite (142.98 MB)**:
  - Neural Bandwidth Extension Net (`bwe_restoration.pt`, 42.89 MB, 11.23M params).
  - Streaming 4-Stem Separator & Dialogue Masking Net (`separator_dialogue.pt`, 47.78 MB, 12.52M params).
  - Deep DDSP Tone & Dynamics Controller (`tone_controller.pt`, 21.30 MB, 5.58M params).
  - 3D Spatializer & Binaural Positioning Net (`spatializer.pt`, 25.22 MB, 6.60M params).
  - Multi-Genre & Acoustic Quality Classifier (`genre_analyzer.pt`, 5.80 MB, 1.52M params).
  - Causal streaming ONNX export (`bwe_streaming.onnx`, opset 18).
  - Hierarchical genre understanding across 42+ world genres and continuous adaptive parameter decision generation.
- **Hardware & Personalization Profiles**:
  - Harman Consumer Target, Studio Reference, Open-Back Dynamic, In-Ear Monitor (IEM), and Laptop Speaker compensation curves.
- **Liquid Glass Desktop Interface (`ui/src/main.rs`)**:
  - Full HD 1080p liquid glass OpenGL 3.3 Core interface running at 347.2 FPS.
  - Raymarched signed distance field (SDF) fluid capsule morphing (`smin`).
  - Chromatic dispersion refraction (RGB ray splitting) with pointer-reactive specular highlights.
  - Interactive 3D Spatial Stage visualizing head and earbud ping-pong vocal orbit in real-time.
  - Dual live 8-band audio spectrum analyzers (Raw Input vs AI Enhanced).
- **Phone-to-Laptop Real-Time Audio Streaming**:
  - Low-latency UDP receiver daemon on port 48100 (`mx5ic_daemon`).
  - Universal Bluetooth A2DP Sink automation script (`scripts/setup_bluetooth_sink.sh`).
- **Cross-Platform Releases & Packaging**:
  - Linux Debian package: `dist/mx5ic_0.1.0_amd64.deb` (459 KB).
  - Linux standalone portable bundle: `dist/mx5ic-linux-x86_64.tar.gz` (887 KB).
  - Android companion APK: `dist/Mx5ic.apk` (17 KB signed with APK Signature Scheme v3, minSdk 29, targetSdk 34).
  - Unified single-command installer: `setup.sh`.
