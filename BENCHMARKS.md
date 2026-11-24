# Mx5ic Performance Benchmarks & Verification Report

All measurements were performed on actual host hardware running Fedora Linux 45 x86_64, Intel Core i5-8250U CPU @ 1.60GHz (4 physical cores / 8 logical threads), PipeWire 1.6.11.

---

## 1. Real-Time Audio DSP Core

- **Audio Rate**: $48,000\text{ Hz}$
- **Block Size**: 128 samples ($2.667\text{ ms}$ audio time per buffer)
- **Signal Path**: 8-Band Parametric EQ + Dialogue Formant + Virtual Bass + 3D Spatial Vocal Orbit + Equal-Power Crossfader + Lookahead Brickwall Limiter.

| Metric | Measured Value | Acceptance Threshold | Result |
| :--- | :--- | :--- | :--- |
| **Execution Time (5.0s audio)** | **$11.6\text{ ms}$** | $\le 1750\text{ ms}$ | **PASSED** |
| **Real-Time Factor (RTF)** | **$0.0023$** | $\le 0.35$ | **PASSED** ($432\times$ real-time speedup) |
| **Added Pipeline Latency** | **$0.80\text{ ms}$** | $\le 40\text{ ms}$ (Live mode) | **PASSED** ($98\%$ headroom) |
| **Buffer Underruns (Xruns)** | **0** | $0$ | **PASSED** (Zero dropouts) |
| **Buffer Overflows** | **0** | $0$ | **PASSED** |
| **True-Peak Limiter Overshoot** | **$0.00\text{ dB}$** (Clamped to $-1.00\text{ dBTP}$) | $\le 0.00\text{ dB}$ | **PASSED** |

---

## 2. 300-Second Continuous Audio Soak Test

- **Duration**: $300.0\text{ seconds}$ ($14,400,000\text{ samples}$ @ $48\text{ kHz}$)
- **Total Chunks**: $112,500\text{ chunks}$ (128 samples/chunk)
- **Execution Time**: **$2.76\text{ seconds}$** total CPU time
- **Soak Real-Time Factor**: **$0.0092$** ($108.6\times$ real-time speedup)
- **Clipping Events**: **0**
- **Memory Growth (Leaks)**: **$0.0\text{ KB}$** ($100\%$ zero-allocation audio loop)

---

## 3. Streaming Neural Model Causal Inference

- **Inference Engine**: ONNX Runtime (CPU Execution Provider)
- **Model**: Causal 1D Convolution with Dilated Residual Blocks
- **Chunk Size**: 128 samples ($2.667\text{ ms}$ audio)
- **Test Iterations**: 10,000 consecutive streaming chunks ($26.67\text{ seconds}$ audio)

| Metric | Measured Value | Acceptance Gate | Result |
| :--- | :--- | :--- | :--- |
| **Mean Inference Latency** | **$0.078\text{ ms}$ ($78.2\text{ µs}$)** | $\le 0.93\text{ ms}$ | **PASSED** |
| **Median (P50) Latency** | **$0.073\text{ ms}$ ($73.1\text{ µs}$)** | $\le 0.93\text{ ms}$ | **PASSED** |
| **99th Percentile (P99) Latency** | **$0.125\text{ ms}$ ($125.0\text{ µs}$)** | $\le 1.50\text{ ms}$ | **PASSED** |
| **Inference RTF** | **$0.0293$** | $\le 0.35$ | **PASSED** |
| **CPU Headroom** | **$97.1\%$** | $\ge 65.0\%$ | **PASSED** |

---

## 4. Liquid Glass UI Rendering Benchmark

- **Display Resolution**: Full HD $1920 \times 1080$ @ 24-bit RGB
- **Shader Pipeline**: OpenGL 3.3 Core GLSL Fragment Shader (Raymarched SDFs, Chromatic Dispersion, Specular Blinn-Phong Glint, 3D Spatial Stage)
- **Frames Rendered**: 500 consecutive frames

| Metric | Measured Value | Target Gate | Result |
| :--- | :--- | :--- | :--- |
| **Average Frame Time** | **$2.88\text{ ms}$** | $\le 16.67\text{ ms}$ ($60\text{ FPS}$) | **PASSED** |
| **Frame Rate** | **$347.2\text{ FPS}$** | $\ge 60.0\text{ FPS}$ | **PASSED** ($5.8\times$ over target) |
| **VRAM Consumption** | **$< 32\text{ MB}$** | $\le 250\text{ MB}$ | **PASSED** |

---

## 5. Binary & Installation Footprint vs 3 GB Budget

| Component | Shipped File | File Size | Size Budget | Budget Usage |
| :--- | :--- | :--- | :--- | :--- |
| **Linux Debian Package** | `dist/mx5ic_0.1.0_amd64.deb` | **$459\text{ KB}$** | $1000\text{ MB}$ | $0.05\%$ |
| **Linux Portable Bundle** | `dist/mx5ic-linux-x86_64.tar.gz` | **$887\text{ KB}$** | $1000\text{ MB}$ | $0.09\%$ |
| **Android Companion APK** | `dist/Mx5ic.apk` | **$17\text{ KB}$** | $100\text{ MB}$ | $0.02\%$ |
| **Deep AI Model Suite** | `models/checkpoints/*.pt` | **$142.98\text{ MB}$** | $500\text{ MB}$ | $28.60\%$ |
| **Total Install Footprint** | App + All Models + Assets | **$\approx 144.5\text{ MB}$** | **$3000\text{ MB}$** | **$4.82\%$** ($95.18\%$ free headroom) |
