import os
import math
import wave
import struct

def make_wav_stereo(filename, sample_rate, duration_sec, sample_generator):
    os.makedirs(os.path.dirname(filename), exist_ok=True)
    num_frames = int(sample_rate * duration_sec)
    with wave.open(filename, 'w') as wf:
        wf.setnchannels(2)
        wf.setsampwidth(2) # 16-bit PCM
        wf.setframerate(sample_rate)
        
        frames = bytearray()
        for i in range(num_frames):
            t = i / sample_rate
            l, r = sample_generator(t, i, sample_rate)
            l_clamped = max(-1.0, min(1.0, l))
            r_clamped = max(-1.0, min(1.0, r))
            l_int = int(l_clamped * 32767.0)
            r_int = int(r_clamped * 32767.0)
            frames.extend(struct.pack('<hh', l_int, r_int))
        wf.writeframes(frames)
    size_kb = os.path.getsize(filename) / 1024
    print(f"[+] Generated {filename} ({duration_sec}s, {size_kb:.1f} KB)")

def generate_vocal_tune(t, i, sr):
    # Simulated vocal melody with rhythmic repeated vocal hook phrases (e.g. "vaari jaavan" style)
    # Lead melody uses formant synthesis (vocal formant resonances at 800 Hz, 1200 Hz, 2400 Hz)
    bpm = 100.0
    beat_dur = 60.0 / bpm
    beat_pos = (t % (beat_dur * 4)) / beat_dur # 0 to 4 beats per bar
    
    # Rhythmic kick and percussion backdrop
    kick = 0.0
    if (t % beat_dur) < 0.15:
        kick_env = math.exp(-(t % beat_dur) * 30.0)
        kick_freq = 120.0 * math.exp(-(t % beat_dur) * 25.0) + 45.0
        kick = 0.5 * kick_env * math.sin(2.0 * math.pi * kick_freq * t)

    # Vocal Hook: occurs on beats 1 and 3 of every measure
    # Synthesized formant singing phrase (A4 = 440 Hz / E4 = 330 Hz / F#4 = 370 Hz)
    vocal = 0.0
    phrase_window = (t % (beat_dur * 2))
    if phrase_window < 0.65:
        # Vocal syllable envelope (smooth attack, sustained singing, gentle release)
        env = math.sin(math.pi * (phrase_window / 0.65))
        pitch = 370.0 if (t % (beat_dur * 4)) < (beat_dur * 2) else 440.0
        # Formant harmonics (1st, 2nd, 3rd, 5th, 6th) to simulate vocal timbre
        f0 = math.sin(2.0 * math.pi * pitch * t)
        f1 = 0.6 * math.sin(2.0 * math.pi * pitch * 2.0 * t)
        f2 = 0.4 * math.sin(2.0 * math.pi * pitch * 3.0 * t)
        formant_boost = 0.5 * math.sin(2.0 * math.pi * 2400.0 * t) # formant ring
        vocal = 0.55 * env * (f0 + f1 + f2 + formant_boost * env)

    # Ambient pad synth in background (warm stereo spread)
    pad_l = 0.12 * math.sin(2.0 * math.pi * 220.0 * t) + 0.08 * math.sin(2.0 * math.pi * 330.0 * t)
    pad_r = 0.12 * math.sin(2.0 * math.pi * 220.5 * t) + 0.08 * math.sin(2.0 * math.pi * 329.5 * t)

    out_l = kick + vocal + pad_l
    out_r = kick + vocal + pad_r
    return out_l, out_r

def generate_edm_tune(t, i, sr):
    # Heavy EDM drop with 45 Hz sub-bass, 128 BPM kick, and bright synths
    bpm = 128.0
    beat_dur = 60.0 / bpm
    beat_idx = int(t / beat_dur)
    t_in_beat = t % beat_dur

    # Heavy punch kick + 45 Hz sub-bass rumble
    kick_env = math.exp(-t_in_beat * 18.0)
    kick = 0.6 * kick_env * math.sin(2.0 * math.pi * 50.0 * t)
    sub = 0.35 * math.sin(2.0 * math.pi * 42.0 * t)

    # Hi-hat on offbeats
    hat = 0.0
    if 0.2 < t_in_beat < 0.3:
        hat_env = math.exp(-(t_in_beat - 0.2) * 50.0)
        # White noise burst
        hat = 0.15 * hat_env * (math.sin(2.0 * math.pi * 11000.0 * t) + math.sin(2.0 * math.pi * 14500.0 * t))

    # Sawtooth synth lead
    lead = 0.0
    notes = [261.6, 329.6, 392.0, 523.2] # C major arpeggio
    note_freq = notes[int(t * 4) % len(notes)]
    lead_saw = 2.0 * ((t * note_freq) % 1.0) - 1.0
    lead = 0.20 * lead_saw

    out_l = kick + sub + hat + lead * 0.8
    out_r = kick + sub - hat + lead * 0.8
    return out_l, out_r

def generate_acoustic_tune(t, i, sr):
    # Acoustic guitar arpeggio and natural warm strings
    chord_freqs = [
        [196.0, 246.9, 293.7, 392.0], # G major
        [164.8, 220.0, 261.6, 329.6], # E minor
        [130.8, 196.0, 261.6, 329.6], # C major
        [146.8, 220.0, 293.7, 370.0], # D major
    ]
    bar_dur = 2.4 # seconds per chord
    chord_idx = int(t / bar_dur) % len(chord_freqs)
    current_chord = chord_freqs[chord_idx]

    # Plucked string arpeggio (decay envelope)
    pluck_interval = bar_dur / len(current_chord)
    string_idx = int((t % bar_dur) / pluck_interval)
    t_pluck = (t % bar_dur) % pluck_interval
    pluck_freq = current_chord[string_idx]
    pluck_env = math.exp(-t_pluck * 5.0)
    pluck = 0.4 * pluck_env * math.sin(2.0 * math.pi * pluck_freq * t)

    # Warm body resonance
    body = 0.1 * math.sin(2.0 * math.pi * (pluck_freq * 0.5) * t)

    return pluck + body, pluck * 0.95 + body * 1.05

def generate_silence(t, i, sr):
    return 0.0, 0.0

if __name__ == '__main__':
    sr = 48000
    os.makedirs("test_audio", exist_ok=True)
    print("[*] Generating high-fidelity test audio files in test_audio/...")
    make_wav_stereo("test_audio/vocal_tune.wav", sr, 6.0, generate_vocal_tune)
    make_wav_stereo("test_audio/edm_tune.wav", sr, 6.0, generate_edm_tune)
    make_wav_stereo("test_audio/acoustic_tune.wav", sr, 6.0, generate_acoustic_tune)
    make_wav_stereo("test_audio/silence.wav", sr, 3.0, generate_silence)
    print("[+] All test audio files generated successfully!")
