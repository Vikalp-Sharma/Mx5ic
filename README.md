# Mx5ic Audio — Music, Reimagined by Intelligence

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Platforms](https://img.shields.io/badge/Platform-Linux%20%7C%20Android-green.svg)](#platform-support)
[![Engine](https://img.shields.io/badge/Audio%20Thread-Lock--Free%20RT-orange.svg)](#real-time-dsp-engine)
[![AI Suite](https://img.shields.io/badge/Model%20Suite-143%20MB-purple.svg)](#deep-music-intelligence)

**Mx5ic** is a real-time, on-device AI music understanding and intelligent audio enhancement ecosystem designed for Linux desktop and Android.

Unlike static equalizers or blind post-processing filters, Mx5ic continuously analyzes what it hears: identifying genres, instrument layers, vocal presence, masking resonances, and stereo balance to generate dynamic, content-aware enhancements in real time with zero audible dropouts.

---

## Key Features

- **Custom-Trained Deep AI Suite (143 MB)**:
  5 custom causal neural networks running locally on-device:
  1. *Neural Bandwidth Extension (BWE)*: Restores 12–24 kHz high-frequency air and repairs MP3/AAC/Opus lossy codec artifacts.
  2. *Streaming 4-Stem Separator*: Sub-band causal isolation of Vocals, Bass, Drums, and Other with dialogue masking.
  3. *Deep DDSP Tone & Dynamics Controller*: Predicts continuous 8-band parametric EQ and harmonic exciter settings.
  4. *3D Spatializer & Binaural Positioning Net*: Head-shadow ITD/ILD filtering and depth positioning.
  5. *Multi-Genre & Quality Classifier*: Hierarchical recognition across 42+ world genres.
- **Dynamic 3D Spatial Vocal Orbit & Earbud Ping-Pong**:
  Sweeps repeated vocal phrases and ad-libs between left and right earbuds before fading into ambient room depth (inspired by vocal tracks like *"vaari jaavan"*), while preserving dry lead vocal intelligibility and mono compatibility.
- **Zero-Dropout Real-Time Audio Engine**:
  Rust-powered real-time audio thread: zero heap allocations, lock-free ring buffers (`rtrb`), system-call free. Added latency $\le 0.8\text{ ms}$, real-time factor (RTF) $0.009$ ($108\times$ real-time speedup).
- **PipeWire Audio Isolation Guarantee**:
  `Mx5ic_Sink` virtual null sink isolates input streams from applications (Spotify, YouTube, VLC, Steam); raw audio never leaks alongside processed output.
- **Dedicated Dialogue & Speech Intelligibility Enhancer**:
  Formant peak intelligibility (+6 dB @ 2.4 kHz) with dynamic high-frequency sibilance de-essing and low-frequency rumble masking reduction.
- **Liquid Glass Interface (iOS 26 Aesthetic)**:
  Custom GLSL fragment shader rendering at 340+ FPS: smooth-minimum (`smin`) fluid SDF surfaces, chromatic dispersion ($IOR_R=0.73, IOR_G=0.70, IOR_B=0.67$), pointer specular reflections, and an interactive 3D Spatial Stage.
- **Phone-to-Laptop Real-Time Audio Streaming**:
  Low-latency UDP streaming over Wi-Fi/USB (`48100`) with Android foreground `AudioPlaybackCapture` (API 29+) and zero-configuration Bluetooth A2DP Sink fallback.
- **Strict Size Budget**:
  Entire installation package is under 2 MB, and total deep model weights are 143 MB, well within the 3 GB release budget.

---

## Architecture Overview

```
                      +---------------------------------------+
                      |       Phone / Laptop Audio Source     |
                      +---------------------------------------+
                                          |
                                          v
                      +---------------------------------------+
                      | PipeWire Virtual Sink: Mx5ic_Sink     |
                      | (Completely Isolated from ALSA Out)   |
                      +---------------------------------------+
                                          |
                                          v
                      +---------------------------------------+
                      |         Real-Time Audio Loop          |
                      |   - Lock-free Ring Buffer             |
                      |   - Zero Heap Allocation              |
                      +---------------------------------------+
                       /                  |                 \
                      v                   v                  v
         +--------------------+  +------------------+  +-------------------+
         | 8-Band Parametric  |  | Virtual Bass     |  | 3D Spatial Vocal  |
         | Biquad EQ Cascade  |  | Harmonic Exciter |  | Orbit Ping-Pong   |
         +--------------------+  +------------------+  +-------------------+
                      \                   |                  /
                       +------------------+-----------------+
                                          |
                                          v
                      +---------------------------------------+
                      | Click-Free Equal-Power Crossfader     |
                      +---------------------------------------+
                                          |
                                          v
                      +---------------------------------------+
                      | True-Peak Lookahead Brickwall Limiter |
                      | (-1.00 dBTP Ceiling, Zero Clipping)   |
                      +---------------------------------------+
                                          |
                                          v
                      +---------------------------------------+
                      | Physical Output (Headphones / DAC)    |
                      +---------------------------------------+
```

---

## Quickstart & Installation

### Option 1: Debian / Ubuntu / Pop!_OS (.deb)
```bash
sudo dpkg -i dist/mx5ic_0.1.0_amd64.deb
mx5ic
```

### Option 2: Linux Standalone Portable Bundle
```bash
tar -xzf dist/mx5ic-linux-x86_64.tar.gz
cd mx5ic
./run.sh
```

### Option 3: Android Companion App
```bash
adb install dist/Mx5ic.apk
```

### Option 4: Unified Build from Source
```bash
./setup.sh
```

---

## Project Documentation

- [`ARCHITECTURE.md`](ARCHITECTURE.md): Deep software and DSP architecture specification.
- [`PROJECT_STATUS.md`](PROJECT_STATUS.md): Milestones, test passes, and release checklist.
- [`MODEL_CARD.md`](MODEL_CARD.md): Complete specs of the 143 MB custom AI neural suite.
- [`BENCHMARKS.md`](BENCHMARKS.md): Measured latency, CPU headroom, and frame rates.
- [`AUDIO_QUALITY_REPORT.md`](AUDIO_QUALITY_REPORT.md): Objective and perceptual audio evaluation.
- [`PLATFORM_COMPATIBILITY.md`](PLATFORM_COMPATIBILITY.md): Linux distributions and Android API tiers.
- [`KNOWN_LIMITATIONS.md`](KNOWN_LIMITATIONS.md): Transparent technical constraints and workarounds.
- [`RESEARCH.md`](RESEARCH.md): Literature review of state-of-the-art commercial & neural audio systems.
- [`DECISIONS.md`](DECISIONS.md): Architectural Decision Records (ADRs).
- [`EXPERIMENTS.md`](EXPERIMENTS.md): Experimental logs, loss curves, and validation data.
