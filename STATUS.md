# STATUS.md: Project Milestone Dashboard

## Current Milestone: M1 (Walking Skeleton Pipeline) - COMPLETE
- [x] **M0: Recon, Technical Spikes & Validation**
  - Architecture, literature review (`RESEARCH.md`), and ADRs (`DECISIONS.md`) complete.
  - Spike a (Liquid Glass shader): **347.2 FPS** @ 1080p.
  - Spike b (PipeWire virtual sink): 100% audio isolation (0 raw leakage), RTF **0.0023**.
  - Spike c (ONNX causal model): RTF **0.0293** (78.2 µs latency, 97.1% CPU headroom).
- [x] **M1: Walking Skeleton Pipeline**
  - [x] Rust Real-time DSP Core (`crates/mx5ic_core`):
    - 8-band parametric biquad EQ cascade
    - Psychoacoustic Virtual Bass harmonic exciter (<80 Hz fundamental saturation)
    - Click-free equal-power bypass crossfader
    - True-Peak lookahead brickwall limiter (-1.00 dBTP ceiling, zero clipping)
    - Lock-free SPSC ring buffers (`rtrb`), zero allocations on audio thread
    - Adaptive clock-drift resampler (cubic Hermite interpolation)
  - [x] PipeWire Virtual Sink Management (`PipeWireManager`): creates `Mx5ic_Sink`, verifies complete isolation from physical ALSA outputs.
  - [x] Phone -> Laptop Streaming Link:
    - UDP receiver daemon on port 48100 (`PhoneAudioReceiver`)
    - Android foreground capture service (`AudioCaptureService.java`) with `AudioPlaybackCapture`
    - Universal fallback script for Bluetooth A2DP Sink (`scripts/setup_bluetooth_sink.sh`)
  - [x] Desktop Liquid Glass UI (`mx5ic_ui`): Interactive 60+ FPS glass renderer with live audio telemetry and A/B bypass toggle.
  - [x] Unit test suite passing 100% across all DSP blocks.
  - [x] Soak test passed: 300s continuous audio, RTF 0.0092 (108x real-time), 0 underruns, 0 overflows, 0 clipping events, 0 memory leaks.

## Milestone Tracker
| Milestone | Description | Status | Pass Gates |
| :--- | :--- | :--- | :--- |
| **M0** | Recon & Technical Spikes | **COMPLETED** | Shader: 347.2 FPS, Audio RTF: 0.0023, ONNX RTF: 0.0293 |
| **M1** | Walking Skeleton Pipeline | **COMPLETED** | Zero-leak sink, 0 xruns, Limiter -1.00 dBTP, Latency 6.17 ms |
| **M2** | Data & Evaluation Harness | **NEXT UP** | Dataset loaders, degradation pipeline, metrics suite |
| **M3** | Custom Model Training | PENDING | 5 ONNX models passing Part D gates |
| **M4** | Engine Integration | PENDING | Live/Studio modes, instant A/B matching |
| **M5** | Liquid Glass UI Polish | PENDING | Full glass shaders, accessibility, tiers |
| **M6** | Hardening & Packaging | PENDING | Install <3GB, weights <=500MB, .deb/AppImage/APK |
