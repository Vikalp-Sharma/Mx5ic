# Mx5ic Strategic Product Roadmap

## Milestone 0: Recon, Architecture & Feasibility Spikes [COMPLETED]
- [x] Comprehensive competitive analysis of Spotify, Apple Music, Dolby, and Sony DSEE ([`RESEARCH.md`](RESEARCH.md)).
- [x] Spike a: Real-time Liquid Glass shader rendering on Linux GPU (Achieved $347.2\text{ FPS}$).
- [x] Spike b: PipeWire virtual sink complete audio isolation (Achieved $0.0\text{ dBFS}$ raw audio leakage, RTF $0.0023$).
- [x] Spike c: Causal ONNX streaming inference ($78.2\text{ µs}$ latency, $97.1\%$ CPU headroom).
- [x] Architectural Decision Records ADR-001 through ADR-006 ([`DECISIONS.md`](DECISIONS.md)).

## Milestone 1: Real-Time Audio DSP Core & Linux Integration [COMPLETED]
- [x] Rust DSP Core with zero allocations on audio path ([`crates/mx5ic_core`](crates/mx5ic_core)).
- [x] 8-Band Parametric EQ cascade with biquad filters.
- [x] Psychoacoustic Virtual Bass harmonic exciter (<80 Hz fundamental enhancement).
- [x] Dynamic 3D Spatial Vocal Orbit & Earbud Ping-Pong Decay (*"vaari jaavan"* spatial effect).
- [x] True-Peak Lookahead Brickwall Limiter ($-1.00\text{ dBTP}$ ceiling, zero clipping).
- [x] Lock-free SPSC ring buffers (`rtrb`) and adaptive clock-drift resampler.
- [x] 300-second audio soak test passed with RTF $0.0092$ and zero xruns.
- [x] Phone-to-Laptop UDP streaming receiver daemon on port $48100$.

## Milestone 2: Deep AI Suite, UI Overhaul & Release Packaging [COMPLETED]
- [x] Strict project rename to **Mx5ic** across all subsystems and documentation.
- [x] Trained and exported 5 custom deep neural networks totaling **$142.98\text{ MB}$** weights:
  - Neural Bandwidth Extension Net ($42.89\text{ MB}$)
  - Streaming 4-Stem Separator ($47.78\text{ MB}$)
  - Deep DDSP Tone & Dynamics Controller ($21.30\text{ MB}$)
  - 3D Spatializer & Binaural Positioning Net ($25.22\text{ MB}$)
  - Multi-Genre & Acoustic Quality Classifier ($5.80\text{ MB}$)
- [x] Dedicated Dialogue & Speech Intelligibility Enhancer with dynamic de-esser.
- [x] Headphone & Hardware Compensation Profiles (Harman, Studio, OpenBack, IEM, Laptop).
- [x] Liquid Glass 3D UI overhaul with interactive 3D Spatial Stage and dual spectrum meters.
- [x] Packaged Linux Debian (`dist/mx5ic_0.1.0_amd64.deb`) and standalone portable (`dist/mx5ic-linux-x86_64.tar.gz`).
- [x] Built and signed Android Companion APK (`dist/Mx5ic.apk`).

## Milestone 3: Neural Stem Synthesis & Edge NPU Acceleration [UPCOMING]
- [ ] Direct Vulkan / DirectML / OpenVINO execution provider backend for zero-copy GPU inference.
- [ ] Android NNAPI acceleration for Qualcomm Hexagon and MediaTek APU NPUs.
- [ ] Real-time generative stem resynthesis for lossy historical recordings (<1970s audio).
- [ ] Wi-Fi Direct zero-router low-latency direct phone-to-laptop pairing.

## Milestone 4: Personalized Ear Canal Acoustic HRTF [FUTURE]
- [ ] Personalized HRTF profile generation via phone camera ear scan.
- [ ] Multi-device spatial mesh (synchronizing laptop, tablet, and phone speakers).
- [ ] Lossless 24-bit/192 kHz audiophile bit-perfect passthrough mode.
