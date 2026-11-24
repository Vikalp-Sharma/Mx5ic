import time
import json
import numpy as np
import onnxruntime as ort
import os

def main():
    print("============================================================")
    print("   Spike c: ONNX Causal Audio Model Real-Time Benchmark    ")
    print("   Target: 128 samples (2.67ms) @ 48kHz, RTF <= 0.35      ")
    print("============================================================")

    model_path = "spikes/spike_c_dummy_onnx/dummy_causal_enhancer.onnx"
    assert os.path.exists(model_path), f"Model not found: {model_path}"

    opts = ort.SessionOptions()
    opts.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
    opts.intra_op_num_threads = 2
    opts.inter_op_num_threads = 1
    opts.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL

    print(f"[*] Loading model into ONNX Runtime (Provider: CPUExecutionProvider)...")
    session = ort.InferenceSession(model_path, sess_options=opts, providers=["CPUExecutionProvider"])

    input_name = session.get_inputs()[0].name
    output_name = session.get_outputs()[0].name

    block_samples = 128
    sample_rate = 48000.0
    chunk_duration_ms = (block_samples / sample_rate) * 1000.0 # 2.6667 ms
    total_iterations = 10000

    print(f"[*] Chunk size        : {block_samples} samples ({chunk_duration_ms:.2f} ms @ {sample_rate:.0f} Hz)")
    print(f"[*] Benchmark runs    : {total_iterations} chunks ({total_iterations * chunk_duration_ms / 1000.0:.2f} s audio)")

    # Warmup 100 iterations
    for _ in range(100):
        dummy_chunk = np.random.randn(1, 1, block_samples).astype(np.float32)
        _ = session.run([output_name], {input_name: dummy_chunk})

    latencies_us = np.zeros(total_iterations, dtype=np.float64)

    print("[*] Running 10,000 streaming inference iterations...")
    total_start = time.perf_counter()

    for i in range(total_iterations):
        chunk = np.random.randn(1, 1, block_samples).astype(np.float32)

        t0 = time.perf_counter_ns()
        _ = session.run([output_name], {input_name: chunk})
        t1 = time.perf_counter_ns()

        latencies_us[i] = (t1 - t0) / 1000.0 # microseconds

    total_end = time.perf_counter()
    total_time_sec = total_end - total_start

    latencies_ms = latencies_us / 1000.0
    mean_lat_ms = float(np.mean(latencies_ms))
    p50_lat_ms = float(np.percentile(latencies_ms, 50))
    p95_lat_ms = float(np.percentile(latencies_ms, 95))
    p99_lat_ms = float(np.percentile(latencies_ms, 99))
    max_lat_ms = float(np.max(latencies_ms))
    min_lat_ms = float(np.min(latencies_ms))

    # Real-Time Factor (RTF) = inference_time / audio_duration
    rtf = mean_lat_ms / chunk_duration_ms
    effective_speedup = 1.0 / rtf

    print("\n=== ONNX Causal Audio Model Benchmark Results ===")
    print(f"Iterations Evaluated : {total_iterations}")
    print(f"Chunk Audio Duration : {chunk_duration_ms:.3f} ms")
    print(f"Mean Inference Time  : {mean_lat_ms:.3f} ms ({mean_lat_ms * 1000.0:.1f} µs)")
    print(f"Median (P50) Latency : {p50_lat_ms:.3f} ms")
    print(f"95th Percentile (P95): {p95_lat_ms:.3f} ms")
    print(f"99th Percentile (P99): {p99_lat_ms:.3f} ms")
    print(f"Min Inference Time   : {min_lat_ms:.3f} ms")
    print(f"Max Inference Time   : {max_lat_ms:.3f} ms")
    print(f"Real-Time Factor(RTF): {rtf:.4f} (Budget: <= 0.35)")
    print(f"Effective Speedup    : {effective_speedup:.1f}x real-time")
    print("-------------------------------------------------")

    passed_rtf = (rtf <= 0.35)
    passed_latency = (mean_lat_ms < chunk_duration_ms)

    if passed_rtf and passed_latency:
        print(f"VERDICT: [PASS] Model meets real-time budget with {(1.0 - rtf)*100.0:.1f}% CPU headroom!")
    else:
        print(f"VERDICT: [FAIL] RTF exceeds 0.35 budget.")

    res = {
        "model_path": model_path,
        "block_samples": block_samples,
        "sample_rate": sample_rate,
        "chunk_duration_ms": chunk_duration_ms,
        "total_iterations": total_iterations,
        "mean_latency_ms": mean_lat_ms,
        "p50_latency_ms": p50_lat_ms,
        "p95_latency_ms": p95_lat_ms,
        "p99_latency_ms": p99_lat_ms,
        "min_latency_ms": min_lat_ms,
        "max_latency_ms": max_lat_ms,
        "rtf": rtf,
        "effective_speedup": effective_speedup,
        "passed_rtf_gate": passed_rtf
    }

    with open("spikes/spike_c_dummy_onnx/benchmark_result.json", "w") as f:
        json.dump(res, f, indent=2)

if __name__ == "__main__":
    main()
