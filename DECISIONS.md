# DECISIONS.md: Architectural Decision Records (ADRs)

## ADR-001: Audio Isolation Architecture via Dedicated PipeWire Virtual Sink
- **Context**: The user requires that phone/system audio is captured into the enhancer application, processed, and *only* the processed audio reaches the earphones. Raw audio must never play alongside processed audio.
- **Decision**: Create a dedicated PipeWire virtual sink (`Enhancer_Sink`) using `module-null-sink` / `libpipewire-0.3`. When the enhancer starts, audio playback sources are directed to `Enhancer_Sink`. `Enhancer_Sink` has no physical audio connections. The Rust audio engine connects as a loopback listener to `Enhancer_Sink.monitor`, captures raw PCM into lock-free ring buffers, applies neural inference + DDSP processing, and routes solely to the user's selected physical ALSA/PipeWire sink.
- **Consequences**: Zero raw audio leakage. Bit-exact isolation. No comb-filtering or duplicate playback.

## ADR-002: Dual-Mode Latency Target (Live <= 40ms vs Studio <= 300ms)
- **Context**: Different use cases require different trade-offs: gaming/calls require sub-40ms latency, while dedicated music listening allows up to 300ms for maximal neural depth.
- **Decision**:
  - **Live Mode**: Causal 1D TCN bandwidth extension (hop 256 @ 48kHz = 5.3ms), 8-band parametric biquad EQ, virtual bass exciter, lookahead limiter (2ms), bypassed heavy separation. Total added latency <= 35ms.
  - **Studio Mode**: Full 4-stem sub-band separation, dialogue extraction, high-order HRTF spatialization, multiband dynamic EQ. Total added latency <= 250ms.
- **Consequences**: User can toggle modes seamlessly without pipeline reinitialization.

## ADR-003: Rust Real-Time Audio Core with Lock-Free SPSC Ring Buffers
- **Context**: Audio threads must never experience buffer underruns (xruns). Any heap allocation, lock contention, GC pause, or syscall on the audio thread causes audible glitches.
- **Decision**: Implement the DSP core in Rust as a `cdylib` library. Use `rtrb` lock-free single-producer single-consumer ring buffers for thread communication. Parameter updates from the UI/analyzer use atomic pointer swapping and triple buffering. No Dart or GC code touches the audio callback.
- **Consequences**: Mathematically guaranteed lock-free, zero-allocation real-time safety.

## ADR-004: Dual-Path Phone Audio Ingestion (Android App + Bluetooth A2DP Sink)
- **Context**: Capturing Android phone audio over Wi-Fi/USB via `AudioPlaybackCapture` is subject to app opt-out and DRM flags.
- **Decision**:
  - **Path A (Primary)**: Android foreground service with `AudioPlaybackCapture` streaming Opus packets over low-latency UDP with adaptive jitter buffering and Rubato resampler for clock-drift compensation.
  - **Path B (Universal Fallback)**: PipeWire + BlueZ Bluetooth A2DP Sink on the laptop. The phone connects to the laptop as a standard Bluetooth speaker. This bypasses all app-level capture restrictions and redirects 100% of phone audio into the PipeWire engine.
- **Consequences**: Uncompromised compatibility across all streaming apps and phone operating systems.

## ADR-005: Liquid Glass Shader Rendering & Multi-Tier Fallback
- **Context**: Existing Flutter liquid-glass packages rely heavily on Impeller, which is unsupported or unstable on Linux Wayland/X11.
- **Decision**: Develop custom GLSL fragment shaders (lensing, specular rim highlight, chromatic dispersion) rendered over an in-app animated background. Implement three automatic performance tiers: Full (refraction + dispersion), Medium (blur + highlights), and Low (frosted static) based on frame time monitoring.
- **Consequences**: Guarantees 60+ fps across all Linux GPUs (Intel integrated, AMD, Nvidia) and Android devices.

## ADR-006: Dynamic 3D Spatial Vocal Orbit & Binaural Earbud Ping-Pong Decay
- **Context**: In standard stereo or mono tracks, repeating vocal lines, refrains, or ad-libs (e.g. "vaari jaavan") sound flat and centered. Commercial spatial audio produces an expansive 3D feel by taking repeated vocal phrases and projecting them sequentially from one earbud to the other while naturally fading into the depth field.
- **Decision**: Implement [`SpatialVocalOrbit`](file:///run/media/kairo/S-M-X-F/Documents/Data/Mx5ic/crates/mx5ic_core/src/dsp/spatial_orbit.rs):
  1. **Vocal Core Extraction**: Isolates vocal speech and singing energy (300 Hz–3.5 kHz) from the center channel using Mid-Side decomposition.
  2. **Binaural Ping-Pong Delay**: Dual cross-feedback delay lines (~280 ms musical timing) bounce repeated phrases between left and right channels.
  3. **Head-Shadow Filtering (ILD/ITD)**: Applies high-shelf damping to the ear opposite the sound location to simulate acoustic skull diffraction.
  4. **Atmospheric Distance Damping**: High-frequency absorption and exponential decay simulate sound fading into 3D space.
  5. **Center Anchor Preservation**: Dry lead vocals remain locked in the center for intelligibility and mono compatibility; only spatialized repeats/tails orbit across the earbuds.
- **Consequences**: Flat music becomes vibrant and three-dimensional with audible earbud-to-earbud phrase movement, while maintaining zero audible dropouts and mono null test compatibility.
