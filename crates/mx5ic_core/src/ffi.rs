use std::sync::Arc;
use crate::pipeline::{EngineConfig, EngineControls, EnginePipeline};
use crate::dsp::NUM_SPECTRUM_BANDS;

pub struct Mx5icContext {
    pipeline: EnginePipeline,
    controls: Arc<EngineControls>,
}

#[no_mangle]
pub extern "C" fn mx5ic_init(sample_rate: f32, block_size: usize) -> *mut Mx5icContext {
    let controls = Arc::new(EngineControls::new());
    let config = EngineConfig {
        sample_rate: if sample_rate > 8000.0 { sample_rate } else { 48000.0 },
        block_size: if block_size > 0 { block_size } else { 128 },
        mode: crate::pipeline::EnhancerMode::Live,
    };
    let pipeline = EnginePipeline::new(config, controls.clone());
    let ctx = Box::new(Mx5icContext { pipeline, controls });
    Box::into_raw(ctx)
}

#[no_mangle]
pub extern "C" fn mx5ic_set_bypass(ctx: *mut Mx5icContext, bypass: bool) {
    if !ctx.is_null() {
        let context = unsafe { &*ctx };
        context.controls.set_bypass(bypass);
    }
}

#[no_mangle]
pub extern "C" fn mx5ic_set_master_gain(ctx: *mut Mx5icContext, gain_linear: f32) {
    if !ctx.is_null() {
        let context = unsafe { &*ctx };
        context.controls.set_master_gain(gain_linear);
    }
}

#[no_mangle]
pub extern "C" fn mx5ic_set_virtual_bass(ctx: *mut Mx5icContext, mix: f32, drive: f32) {
    if !ctx.is_null() {
        let context = unsafe { &*ctx };
        context.controls.set_virtual_bass(mix, drive);
    }
}

#[no_mangle]
pub extern "C" fn mx5ic_set_eq_band(ctx: *mut Mx5icContext, band: usize, gain_db: f32) {
    if !ctx.is_null() {
        let context = unsafe { &mut *ctx };
        context.pipeline.set_eq_band(band, gain_db);
    }
}

#[no_mangle]
pub extern "C" fn mx5ic_process(
    ctx: *mut Mx5icContext,
    input: *const f32,
    output: *mut f32,
    num_samples: usize,
) {
    if ctx.is_null() || input.is_null() || output.is_null() || num_samples == 0 {
        return;
    }
    let context = unsafe { &mut *ctx };
    let in_slice = unsafe { std::slice::from_raw_parts(input, num_samples) };
    let out_slice = unsafe { std::slice::from_raw_parts_mut(output, num_samples) };
    context.pipeline.process_interleaved(in_slice, out_slice);
}

#[no_mangle]
pub extern "C" fn mx5ic_get_telemetry(
    ctx: *mut Mx5icContext,
    out_rms_db: *mut f32,
    out_peak_db: *mut f32,
    out_spectrum: *mut f32,
) {
    if ctx.is_null() {
        return;
    }
    let context = unsafe { &*ctx };
    let tel = context.pipeline.get_telemetry();

    if !out_rms_db.is_null() {
        unsafe { *out_rms_db = tel.rms_db; }
    }
    if !out_peak_db.is_null() {
        unsafe { *out_peak_db = tel.peak_db; }
    }
    if !out_spectrum.is_null() {
        unsafe {
            for i in 0..NUM_SPECTRUM_BANDS {
                *out_spectrum.add(i) = tel.spectrum[i];
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn mx5ic_destroy(ctx: *mut Mx5icContext) {
    if !ctx.is_null() {
        unsafe {
            let _ = Box::from_raw(ctx);
        }
    }
}

#[no_mangle]
pub extern "C" fn mx5ic_set_spatial_3d(ctx: *mut Mx5icContext, amount: f32, speed_hz: f32) {
    if !ctx.is_null() {
        let context = unsafe { &*ctx };
        context.controls.set_spatial_3d(amount, speed_hz);
    }
}
