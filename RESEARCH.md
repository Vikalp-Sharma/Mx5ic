# RESEARCH: Real-Time AI Music Enhancer Architecture & Literature Review

## 1. Commercial State of the Art Analysis

| Product / Technology | Primary Mechanism | Latency / Architecture | Strengths & Failure Modes |
| :--- | :--- | :--- | :--- |
| **Spotify** | ReplayGain / EBU R128 loudness normalization (-14 LUFS standard, linear limiter) + static 6-band graphic EQ presets. | Zero added latency; offline metadata tag processing. | Clean, non-destructive; but completely non-adaptive, cannot restore lost harmonics, zero spatialization or dynamics shaping. |
| **Apple Music** | "Sound Enhancer" (multiband exciter + stereo widener); Dolby Atmos binaural spatial audio via spatial audio renderer and static/head-tracked HRTF profiles. | Sub-5ms CoreAudio DSP; multi-channel discrete bed (7.1.4) downmixed with binaural filters. | Excellent spatialization when native Atmos tracks exist; stereo upmixing ("Spatialize Stereo") suffers from phase smear, comb-filtering, and center-channel vocal hollow-out. |
| **Sony DSEE (HX/Ultimate)** | Edge-AI high-frequency neural bandwidth extension + bit-depth expansion (upscaling 16-bit/44.1kHz to 24-bit/96kHz/192kHz). | Real-time on custom Sony Bluetooth SoC (CXD3277 / Mediatek). | Recovers natural air and cymbal attack above 14 kHz on lossy MP3/AAC/SBC streams; prone to faint high-frequency whistling or pre-ringing if quantization noise floor fluctuates. |
| **Dolby Audio / Dialogue Enhancer** | Center-channel extraction via mid-side phase correlation and energy masking; applies parametric boost (1 kHz–4 kHz) with dynamic EQ. | Low-latency (<10ms) IIR/biquad filter chain. | Excellent speech intelligibility in noisy mixes; tends to pump lead vocal sibilance and alter rhythm guitars/snare body in dense rock/pop arrangements. |

---

## 2. Differentiable DSP (DDSP) & Tone/Dynamics Control

Traditional neural network black boxes (e.g. WaveNet, raw 1D U-Nets) fail real-time constraints (<40ms latency, zero dropouts, low CPU/battery footprint). Differentiable Digital Signal Processing (**DDSP**, Engel et al., 2020) shifts neural networks from generating raw waveforms to predicting time-varying control parameters for closed-form, mathematically stable, analytical DSP blocks:

1. **Parameterizable DSP Chain**:
   - **8-to-12 Band Parametric EQ**: Cascade of second-order biquad filters (peaking, shelving, high-pass) parameterized by center frequency $f_0$, gain $G$, and quality factor $Q$. Formulated with differentiable transfer functions $H(z)$.
   - **Multiband Compressor / Dynamic EQ**: Time-domain envelope followers coupled with soft-knee gain computers and lookahead limiters.
   - **Harmonic Exciter & Virtual Bass (MaxxBass principle)**: Generates synthetic second and third harmonics from sub-bass frequencies (30–80 Hz) using nonlinear saturation functions:
     $$y(t) = \alpha \cdot \tanh(\beta \cdot x_{LP}(t)) + \gamma \cdot x_{LP}^2(t)$$
     Enables small earphone drivers (which physically distort or roll off below 70 Hz) to produce the psychoacoustic pitch perception of deep fundamental bass via the "missing fundamental" phenomenon.
   - **True-Peak Limiter**: Fast lookahead peak detector with oversampled envelope calculation (inter-sample peak prevention) compliant with ITU-R BS.1770-4.

2. **Self-Supervised Learning & Target Matching**:
   Because no clean "pre-enhanced" paired dataset exists, training leverages an inverse degradation framework:
   - Ground truth: high-resolution, professionally mastered tracks (lossless FLAC, 48 kHz).
   - Degradation operator $\mathcal{D}$: random parametric EQ cut/boost, dynamic range squashing, mono-folding, and lossy compression.
   - Controller network: predicts parameter vector $\theta$ to invert $\mathcal{D}$ using multiresolution spectral distance, spectral convergence, and genre-specific spectral centroid/crest factor priors.

---

## 3. Neural Bandwidth Extension (BWE) & Lossy-Source Restoration

Low-bitrate streaming audio (e.g., MP3 at 64–128 kbps, Opus at 32 kbps, Bluetooth SBC) exhibits high-frequency brickwall cutoffs (typically 12–16 kHz), phase smearing, MDCT time-domain aliasing, and rough metallic artifacts:

1. **Architecture**:
   - A causal, streaming temporal convolutional network (TCN) or lightweight complex-valued spectrogram-to-spectrogram U-Net operating on STFT frames (hop size 128–256 samples, 2.6–5.3 ms at 48 kHz).
   - Combines a neural spectral predictor for high frequencies (12–24 kHz) with a differentiable harmonic generator conditioned on lower fundamental frequencies.
2. **Loss Function**:
   - **Multi-Resolution STFT Loss** (Yamamoto et al.):
     $$\mathcal{L}_{MR-STFT} = \sum_{m} \left( \mathcal{L}_{sc}^{(m)}(y, \hat{y}) + \mathcal{L}_{mag}^{(m)}(y, \hat{y}) \right)$$
     with FFT sizes $[512, 1024, 2048]$ to balance temporal sharpness with spectral resolution.
   - **Adversarial Feature Matching Loss**: Uses intermediate layers of a multi-scale discriminator (HiFi-GAN / MelGAN style) to prevent over-smoothed, muffled predictions.

---

## 4. Source Separation & Dialogue/Vocal Enhancement

To selectively elevate vocals or lead instruments without boosting background noise or destructive gain changes:
1. **Lightweight Streaming Separation**:
   - Full models like Demucs v4 (Hybrid Transformer) are too heavy for low-latency live edge inference (~300MB+, hundreds of ms latency).
   - Instead, deploy an ultra-compact causal Conv-TasNet / BSRNN architecture or a sub-band recurrent network operating with fixed 20 ms receptive fields, outputting 4 stem masks (Vocals, Drums, Bass, Other).
2. **Intelligibility & Dialogue Masking**:
   - Speech transmission index (STI) and STOI metrics guide the objective.
   - Lead vocal energy is passed through a bounded dynamic envelope; boost is limited to $+4.5\text{ dB}$ with spectral tilt preservation, ensuring the lead vocal remains glued within the mix rather than sounding like an unmixed karaoke overlay.

---

## 5. Spatial Audio & Mono-to-Immersive Stereo Upmixing

Up-mixing mono or low-coherence stereo to immersive binaural audio for earphones requires resolving phase-cancellation and "in-the-head" localization traps:
1. **The Spatializer Network**:
   - Analyzes stereo mixes across genres folded to mono; learns to predict per-stem spatial positioning (azimuth $\phi$, elevation $\theta$, distance $r$, and decorrelation index $\delta$).
   - **Inviolable Constraints**:
     - Sub-bass (<120 Hz), kick drum, and lead vocal core (mid-band) remain strictly centered ($C$) with high mono-compatibility ($\text{Corr}_{L,R} \approx 1$).
     - Backing vocals, rhythm guitars, synths, reverb tails, and drum cymbals are decorrelated and panned laterally ($\pm 30^\circ$ to $\pm 85^\circ$).
     - **Mono-Compatibility Null Test**: Summing the stereo output back to mono $(L + R)/\sqrt{2}$ must produce minimal comb-filter notches (<3 dB deviation from input mono signal).
2. **Binaural Rendering Engine**:
   - HRIR (Head-Related Impulse Response) direct convolution using open-access measured SOFA databases (e.g., SADIE II dataset, subject dummy heads, CC-BY 4.0).
   - Partitioned convolution using Fast Convolution (overlap-save / FFT) running in chunks of 64–128 samples to keep latency well within the <10ms budget.
   - Early room reflections (image-source method) simulated with low-order reflection rays to pull the stereo image outward from inside the listener's skull into natural room acoustics.

---

## 6. Real-Time Engine & Interception Architecture

To ensure zero dropouts, uncompromised fidelity, and strictly processed-only audio routing:

```
[System Audio / Android App / BT A2DP Sink]
                     │
                     ▼
       PipeWire Virtual Sink ("Enhancer")
                     │
                     ▼ (Loopback Stream / Zero Syscalls on DSP thread)
      Rust Real-Time Audio Engine Core (cdylib)
   ┌─────────────────────────────────────────────────────────┐
   │ Lock-Free SPSC Ring Buffer (heapless / rtrb)           │
   │ Rubato Sinc Resampler (Clock-Drift Compensation)       │
   │ Dynamic Parameter Updates (Atomic Triple Buffering)     │
   │ ONNX Runtime / XNNPACK Causal Inference Buffer         │
   │ Differentiable DSP Chain (Biquads, Exciter, Limiter)    │
   │ Convolution Engine (Partitioned FFT HRTF)               │
   └─────────────────────────────────────────────────────────┘
                     │
                     ▼
       Physical Output Device (Earphones / DAC)
   (Original raw audio is completely isolated & muted)
```

1. **Audio Path Safety**:
   - No memory allocation (`malloc`, `free`, Rust `Vec::resize`), no file I/O, no system calls, no mutex locks on the audio callback thread.
   - Thread communication via lock-free Single-Producer Single-Consumer (SPSC) ring buffers.
2. **Full Interception Guarantee**:
   - The PipeWire virtual sink captures all target audio streams at source.
   - The original physical output route is never unmuted for raw input; audio leaves the system only through the processed master output node.
