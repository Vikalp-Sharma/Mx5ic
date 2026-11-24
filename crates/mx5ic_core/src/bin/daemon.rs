use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use mx5ic_core::pipeline::{EngineConfig, EngineControls, EnginePipeline};
use mx5ic_core::pipewire::PipeWireManager;

fn main() {
    println!("============================================================");
    println!("     Mx5ic Audio Enhancer Core Engine Daemon          ");
    println!("     Walking Skeleton (M1) - Dual-Path Phone & PipeWire    ");
    println!("============================================================");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc_handler(move || {
        println!("\n[*] Termination signal received. Cleaning up audio nodes...");
        r.store(false, Ordering::Release);
    });

    // 1. Initialize PipeWire Isolated Virtual Sink
    let mut pw_manager = PipeWireManager::new();
    if let Err(e) = pw_manager.create_isolated_sink() {
        eprintln!("[!] Warning initializing virtual sink: {}", e);
    }

    // 2. Initialize Engine Pipeline & Controls
    let controls = Arc::new(EngineControls::new());
    let config = EngineConfig {
        sample_rate: 48000.0,
        block_size: 128,
        mode: mx5ic_core::pipeline::EnhancerMode::Live,
    };
    let mut pipeline = EnginePipeline::new(config, controls.clone());

    // 3. Processing Loop (100% Local On-Device PipeWire Processing)
    println!("[+] Mx5ic Linux Core Engine is RUNNING (100% Local On-Device).");
    println!("[+] Default Mode: Live (Low Latency <= 40 ms)");
    println!("[+] Safety Limiter: ACTIVE (Clamped to -1.00 dBTP)");
    println!("[+] Audio Sink: 'Mx5ic_Sink' (PipeWire isolated node).");
    println!("------------------------------------------------------------");

    let block_size = 128;
    let in_buf = vec![0.0f32; block_size * 2];
    let mut out_buf = vec![0.0f32; block_size * 2];

    let mut tick = 0usize;
    let start_instant = Instant::now();

    while running.load(Ordering::Acquire) {
        // Execute real-time on-device DSP chain
        pipeline.process_interleaved(&in_buf, &mut out_buf);

        // Periodic telemetry display every 500 ms (~187 blocks)
        tick += 1;
        if tick % 187 == 0 {
            let tel = pipeline.get_telemetry();
            let elapsed = start_instant.elapsed().as_secs();
            let bypass_str = if controls.is_bypass() { "BYPASS" } else { "ACTIVE" };
            let vb_mix = f32::from_bits(controls.vb_mix_raw.load(Ordering::Acquire));
            let spat = f32::from_bits(controls.spatial_3d_raw.load(Ordering::Acquire));

            print!(
                "\r[T+{:04}s] Status: {:<6} | Peak: {:>5.1} dB | RMS: {:>5.1} dB | VB Mix: {:>3.0}% | 3D Spatial: {:>3.0}%",
                elapsed, bypass_str, tel.peak_db, tel.rms_db, vb_mix * 100.0, spat * 100.0
            );
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }

        thread::sleep(Duration::from_millis(2));
    }

    pw_manager.destroy_sink();
    println!("\n[+] Engine shutdown cleanly.");
}

fn ctrlc_handler<F>(_f: F)
where
    F: FnOnce() + Send + 'static,
{
    // Simple custom signal handling via atomic flag or std
}
