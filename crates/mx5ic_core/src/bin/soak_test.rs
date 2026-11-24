use std::env;
use std::f32::consts::PI;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;
use mx5ic_core::pipeline::{EngineConfig, EngineControls, EnginePipeline, EnhancerMode};
use rtrb::RingBuffer;

fn get_rss_kb() -> usize {
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<usize>() {
                        return kb;
                    }
                }
            }
        }
    }
    0
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut duration_secs = 60usize; // Default 60s for test run, or 3600s for 1-hour soak

    for i in 1..args.len() {
        if args[i] == "--duration" && i + 1 < args.len() {
            if let Ok(d) = args[i + 1].parse::<usize>() {
                duration_secs = d;
            }
        }
    }

    println!("============================================================");
    println!("     Mx5ic Audio Core - Real-Time Soak Test Harness   ");
    println!("     Duration: {} seconds ({} blocks @ 128 samples/block)    ", duration_secs, duration_secs * 375);
    println!("============================================================");

    let sample_rate = 48000.0f32;
    let block_size = 128usize;
    let total_blocks = duration_secs * 375; // 48000 / 128 = 375 blocks/sec

    let controls = Arc::new(EngineControls::new());
    let config = EngineConfig {
        sample_rate,
        block_size,
        mode: EnhancerMode::Live,
    };
    let mut pipeline = EnginePipeline::new(config, controls.clone());

    let (mut producer, mut consumer) = RingBuffer::<f32>::new(8192);

    let initial_rss = get_rss_kb();
    println!("[*] Initial Memory Footprint (VmRSS): {} KB", initial_rss);

    let mut in_block = vec![0.0f32; block_size * 2];
    let mut out_block = vec![0.0f32; block_size * 2];

    let mut max_input_peak = 0.0f32;
    let mut max_output_peak = 0.0f32;
    let mut underrun_count = 0usize;
    let mut overflow_count = 0usize;
    let mut limiter_clipping_events = 0usize;
    let limiter_ceiling = 10.0f32.powf(-1.0 / 20.0); // 0.89125 (-1.0 dBTP)

    let test_start = Instant::now();
    let mut sample_index = 0usize;

    for block_idx in 0..total_blocks {
        // 1. Generate realistic dynamic music test signal
        // Dynamic chord (440Hz + 554.37Hz + 659.25Hz) + deep sub-bass modulation (45Hz) + transients
        for i in 0..block_size {
            let t = (sample_index + i) as f32 / sample_rate;
            let sub = 0.45 * (2.0 * PI * 45.0 * t).sin();
            let chord = 0.30 * (2.0 * PI * 440.0 * t).sin()
                      + 0.25 * (2.0 * PI * 554.37 * t).sin()
                      + 0.20 * (2.0 * PI * 659.25 * t).sin();
            let transient = if (sample_index + i) % 24000 < 200 { 0.5 } else { 0.0 };
            let raw_sample = (sub + chord + transient) * 0.95;

            if raw_sample.abs() > max_input_peak {
                max_input_peak = raw_sample.abs();
            }

            // Push stereo samples to ring buffer
            if let Err(_) = producer.push(raw_sample) {
                overflow_count += 1;
            }
            if let Err(_) = producer.push(raw_sample * 0.98) {
                overflow_count += 1;
            }
        }
        sample_index += block_size;

        // 2. Read from ring buffer into audio thread block
        for i in 0..(block_size * 2) {
            match consumer.pop() {
                Ok(s) => in_block[i] = s,
                Err(_) => {
                    in_block[i] = 0.0;
                    underrun_count += 1;
                }
            }
        }

        // 3. Process through Mx5ic DSP Pipeline
        pipeline.process_interleaved(&in_block, &mut out_block);

        // 4. Verify output peak and limiter ceiling
        for &sample in &out_block {
            let abs_s = sample.abs();
            if abs_s > max_output_peak {
                max_output_peak = abs_s;
            }
            if abs_s > limiter_ceiling + 0.0001 {
                limiter_clipping_events += 1;
            }
        }

        // Progress update every 10 seconds
        if block_idx > 0 && block_idx % (375 * 10) == 0 {
            let elapsed_sec = test_start.elapsed().as_secs();
            let cur_rss = get_rss_kb();
            println!(
                "[*] Progress: {:>4}s / {:>4}s | Output Peak: {:.4} | RSS: {} KB | Underruns: {}",
                elapsed_sec, duration_secs, max_output_peak, cur_rss, underrun_count
            );
        }
    }

    let total_elapsed = test_start.elapsed();
    let total_elapsed_sec = total_elapsed.as_secs_f64();
    let audio_processed_sec = (total_blocks * block_size) as f64 / sample_rate as f64;
    let rtf = total_elapsed_sec / audio_processed_sec;
    let final_rss = get_rss_kb();
    let memory_delta_kb = final_rss as isize - initial_rss as isize;

    println!("\n============================================================");
    println!("                 SOAK TEST HARNESS RESULTS                  ");
    println!("============================================================");
    println!("Audio Duration Tested : {:.2} seconds", audio_processed_sec);
    println!("Wall Time Elapsed     : {:.4} seconds", total_elapsed_sec);
    println!("Real-Time Factor (RTF): {:.6} (Budget: <= 0.10)", rtf);
    println!("Processing Speedup    : {:.1}x real-time", 1.0 / rtf);
    println!("Max Raw Input Peak    : {:.4} ({:.2} dBFS)", max_input_peak, 20.0 * max_input_peak.log10());
    println!("Max Processed Peak    : {:.4} ({:.2} dBFS)", max_output_peak, 20.0 * max_output_peak.log10());
    println!("Limiter Ceiling Target: {:.4} (-1.00 dBTP)", limiter_ceiling);
    println!("Clipping Overshoots   : {}", limiter_clipping_events);
    println!("Buffer Underruns      : {}", underrun_count);
    println!("Buffer Overflows      : {}", overflow_count);
    println!("Initial VmRSS Memory  : {} KB", initial_rss);
    println!("Final VmRSS Memory    : {} KB", final_rss);
    println!("Memory Delta (Leak?)  : {} KB", memory_delta_kb);
    println!("------------------------------------------------------------");

    let rtf_pass = rtf < 0.05;
    let clipping_pass = limiter_clipping_events == 0;
    let dropouts_pass = underrun_count == 0 && overflow_count == 0;
    let mem_pass = memory_delta_kb.abs() < 1024; // Less than 1MB variation
    let overall_pass = rtf_pass && clipping_pass && dropouts_pass && mem_pass;

    if overall_pass {
        println!("VERDICT: [PASS] Soak test fully passed all Part D real-time & stability gates!");
    } else {
        println!("VERDICT: [FAIL] Check failing gate above.");
    }

    let report_json = format!(
        "{{\n  \"duration_seconds\": {},\n  \"wall_time_seconds\": {:.4},\n  \"rtf\": {:.6},\n  \"speedup\": {:.1},\n  \"max_input_peak_db\": {:.2},\n  \"max_output_peak_db\": {:.2},\n  \"clipping_events\": {},\n  \"underruns\": {},\n  \"overflows\": {},\n  \"initial_rss_kb\": {},\n  \"final_rss_kb\": {},\n  \"memory_delta_kb\": {},\n  \"passed\": {}\n}}\n",
        audio_processed_sec, total_elapsed_sec, rtf, 1.0 / rtf,
        20.0 * max_input_peak.log10(), 20.0 * max_output_peak.log10(),
        limiter_clipping_events, underrun_count, overflow_count,
        initial_rss, final_rss, memory_delta_kb, overall_pass
    );

    let mut f = File::create("soak_test_report.json").unwrap();
    f.write_all(report_json.as_bytes()).unwrap();
    println!("[+] Detailed report written to soak_test_report.json");
}
