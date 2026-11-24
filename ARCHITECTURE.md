# Mx5ic System Architecture & Technical Specification

## 1. High-Level Design Principles

Mx5ic separates concerns into decoupled execution tiers to guarantee strict real-time safety, zero audible dropouts, and offline on-device execution:

1. **Audio Callback Path (Hard Real-Time)**:
   - Written in Rust (`crates/mx5ic_core`).
   - Strictly zero heap allocations, zero system calls, zero mutexes, zero locks.
   - SPSC lock-free ring buffers (`rtrb`) connect audio threads with control/inference workers.
   - Clock drift management: adaptive cubic Hermite resampler.
2. **AI Inference & Decision Worker (Soft Real-Time)**:
   - Causal neural networks executing asynchronously.
   - Reads sliding audio analysis windows (100–250 ms context).
   - Generates continuous, click-free parameter trajectories for the DSP core.
   - If inference experiences load spikes, the DSP core falls back safely to the last valid parameters with zero audio interruptions.
3. **Desktop Presentation (OpenGL Core)**:
   - High-performance GLFW + OpenGL 3.3 Core shader pipeline (`ui/src/main.rs`).
   - Custom raymarched signed distance field (SDF) liquid glass shader rendering at 340+ FPS.

---

## 2. Audio Signal Flow & Isolation Guarantee

```
[System Audio Stream: Spotify / YouTube / Browser]
                     |
                     v
+-------------------------------------------------------+
| PipeWire Virtual Sink: "Mx5ic_Sink"                  |
| - Module: module-null-sink                            |
| - Rate: 48,000 Hz, 2 Channels, 32-bit Float           |
| - Strict Isolation: ZERO links to alsa_output         |
+-------------------------------------------------------+
                     |
                     | (Loopback Capture)
                     v
+-------------------------------------------------------+
| Mx5ic Real-Time Audio Processing Pipeline             |
|                                                       |
| 1. 8-Band Parametric EQ Cascade (Biquad Peaking/Shelf)|
| 2. Dialogue & Formant Intelligibility Enhancer        |
| 3. Psychoacoustic Virtual Bass Harmonic Generator    |
| 4. 3D Spatial Vocal Orbit & Earbud Ping-Pong Echo     |
| 5. Equal-Power Click-Free Bypass Crossfader           |
| 6. True-Peak Lookahead Brickwall Limiter (-1.00 dBTP) |
+-------------------------------------------------------+
                     |
                     v
[Physical ALSA Output: Physical Earphones / DAC]
```

### Complete Audio Isolation Guarantee (ADR-001)
Raw audio routed to `Mx5ic_Sink` is isolated inside PipeWire. The engine captures PCM from `Mx5ic_Sink.monitor` and writes exclusively to the physical hardware output sink (`alsa_output.*`). Raw unenhanced audio is never mixed or leaked into the physical earphones.

---

## 3. Deep AI Neural Architecture Suite (142.98 MB)

The music intelligence is distributed across 5 specialized causal deep models designed for on-device edge execution:

```
                                 [Input Stereo Audio]
                                          |
          +-------------------------------+-------------------------------+
          |                               |                               |
          v                               v                               v
+--------------------+          +--------------------+          +--------------------+
| Model 1: BWE &     |          | Model 2: 4-Stem    |          | Model 5: Multi-    |
| Bandwidth Ext Net  |          | Separator & Mask   |          | Genre & Quality    |
| (42.89 MB, 11.23M) |          | (47.78 MB, 12.52M) |          | (5.80 MB, 1.52M)   |
+--------------------+          +--------------------+          +--------------------+
          |                               |                               |
          | (High Freq Air)               | (Stems & Mask)                | (Genre & Dynamics)
          |                               v                               v
          |                     +--------------------+          +--------------------+
          |                     | Model 4: 3D        |          | Model 3: Deep DDSP |
          |                     | Spatializer & Orbit|          | Tone & EQ Engine   |
          |                     | (25.22 MB, 6.60M)  |          | (21.30 MB, 5.58M)  |
          |                     +--------------------+          +--------------------+
          |                               |                               |
          \                               |                              /
           +------------------------------+-----------------------------+
                                          |
                                          v
                            [Adaptive Decision Controller]
                                          |
                                          v
                              [Lock-Free DSP Controls]
```

---

## 4. Phone-to-Laptop Audio Transport

```
+---------------------------------------+
| Android Device (Android 10+, API 29+) |
| - Foreground AudioCaptureService      |
| - AudioPlaybackCapture API            |
| - 48 kHz 16-bit PCM Stereo            |
+---------------------------------------+
                   |
                   | UDP Datagrams (Port 48100) / USB Tether
                   v
+---------------------------------------+
| Linux Laptop Host                     |
| - UDP Receiver Daemon (receiver.rs)   |
| - Adaptive Jitter Resampler           |
| - Injected directly into Mx5ic DSP    |
+---------------------------------------+
```

---

## 5. Technology Stack Summary

| Subsystem | Technologies Selected | Rationale |
| :--- | :--- | :--- |
| **DSP Core** | Rust 2021, `rtrb`, SIMD-friendly loops | Zero garbage collection, deterministic audio latency, memory safety. |
| **Linux Audio** | PipeWire 0.3+, PulseAudio client libraries | Native low-latency audio graph management, universal Linux compatibility. |
| **AI Modeling** | PyTorch 2.12+, ONNX Runtime, ONNX Opset 18 | High-throughput causal temporal convolutional networks, optimized CPU/NPU runtime. |
| **Desktop UI** | OpenGL 3.3 Core, GLFW, GLSL SDF Shaders | Liquid glass physical refraction without heavy UI framework bloat; 340+ FPS. |
| **Mobile App** | Kotlin, Android SDK 34 (minSdk 29), D8, AAPT2 | Lightweight native companion, official `AudioPlaybackCapture` integration. |
