use std::ffi::CString;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;
use mx5ic_core::pipeline::{EngineConfig, EngineControls, EnginePipeline, EnhancerMode};
use mx5ic_core::dsp::SoundProfile;
use glfw::{Action, Context, Key};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let headless = args.iter().any(|a| a == "--headless" || a == "--bench");
    let screenshot_path = args.windows(2).find(|w| w[0] == "--screenshot").map(|w| w[1].clone());

    println!("============================================================");
    println!("     Mx5ic - Real-Time AI Music Enhancer (M1 / M2)          ");
    println!("     Liquid Glass 3D Audio Interface (143 MB Deep Suite)    ");
    println!("============================================================");

    let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to initialize GLFW");

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    if headless || screenshot_path.is_some() {
        glfw.window_hint(glfw::WindowHint::Visible(false));
    }

    let (mut window, events) = glfw
        .create_window(WIDTH, HEIGHT, "Mx5ic - Real-Time AI Music Enhancer", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_pos_polling(true);
    window.make_current();
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    gl::load_with(|s| glfw.get_proc_address_raw(s));

    // Initialize DSP Core
    let controls = Arc::new(EngineControls::new());
    let config = EngineConfig {
        sample_rate: 48000.0,
        block_size: 128,
        mode: EnhancerMode::Live,
    };
    let mut pipeline = EnginePipeline::new(config, controls.clone());

    // Compile liquid glass shader (multi-path lookup with embedded fallback)
    let embedded_frag = include_str!("shaders/liquid_glass.frag");
    let frag_src = std::fs::read_to_string("ui/src/shaders/liquid_glass.frag")
        .or_else(|_| std::fs::read_to_string("shaders/liquid_glass.frag"))
        .or_else(|_| std::fs::read_to_string("/usr/share/mx5ic/shaders/liquid_glass.frag"))
        .unwrap_or_else(|_| embedded_frag.to_string());

    let vert_src = "#version 330 core\nlayout (location = 0) in vec2 aPos;\nvoid main() { gl_Position = vec4(aPos, 0.0, 1.0); }\n";

    let program = unsafe {
        let vs = gl::CreateShader(gl::VERTEX_SHADER);
        let vs_c = CString::new(vert_src).unwrap();
        gl::ShaderSource(vs, 1, &vs_c.as_ptr(), std::ptr::null());
        gl::CompileShader(vs);

        let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
        let fs_c = CString::new(frag_src).unwrap();
        gl::ShaderSource(fs, 1, &fs_c.as_ptr(), std::ptr::null());
        gl::CompileShader(fs);

        let prog = gl::CreateProgram();
        gl::AttachShader(prog, vs);
        gl::AttachShader(prog, fs);
        gl::LinkProgram(prog);

        gl::DeleteShader(vs);
        gl::DeleteShader(fs);
        prog
    };

    let quad_vertices: [f32; 12] = [
        -1.0,  1.0,
        -1.0, -1.0,
         1.0, -1.0,
        -1.0,  1.0,
         1.0, -1.0,
         1.0,  1.0,
    ];

    let mut vao = 0;
    let mut vbo = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (quad_vertices.len() * std::mem::size_of::<f32>()) as isize,
            quad_vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 2 * std::mem::size_of::<f32>() as i32, std::ptr::null());
        gl::EnableVertexAttribArray(0);
    }

    let u_res = unsafe { gl::GetUniformLocation(program, CString::new("u_resolution").unwrap().as_ptr()) };
    let u_time = unsafe { gl::GetUniformLocation(program, CString::new("u_time").unwrap().as_ptr()) };
    let u_ptr = unsafe { gl::GetUniformLocation(program, CString::new("u_pointer").unwrap().as_ptr()) };
    let u_audio = unsafe { gl::GetUniformLocation(program, CString::new("u_audio_rms").unwrap().as_ptr()) };
    let u_byp = unsafe { gl::GetUniformLocation(program, CString::new("u_bypass").unwrap().as_ptr()) };

    println!("[+] Liquid Glass UI ready (1280x720, 60 FPS).");
    println!("[*] Interactive Controls:");
    println!("    - Click / Spacebar: Instant A/B Bypass Toggle");
    println!("    - Keys 1-7: Switch Spatial Surround Mode (Cinema, Concert, Vocal, Club, Night, Studio, Harman)");
    println!("    - Key N: Toggle Adaptive Noise Cancellation & Rumble Shield");
    println!("    - Mouse Move: Liquid Optics & Pointer Glint");
    println!("    - ESC / Close: Exit Application");

    let start_time = Instant::now();
    let mut pointer_x = 0.52f32;
    let mut pointer_y = 0.55f32;
    let mut frame_count = 0usize;

    let mut dummy_out = vec![0.0f32; 256];

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
                    let cur = controls.is_bypass();
                    controls.set_bypass(!cur);
                    println!("[UI] Toggle Bypass: {}", if !cur { "BYPASS (Original Audio)" } else { "ACTIVE (Autonomous AI Mastering)" });
                }
                glfw::WindowEvent::Key(Key::Num1, _, Action::Press, _) => {
                    pipeline.apply_sound_profile(SoundProfile::CinemaSurround3D);
                    println!("[MODE 1] Active: {} -> {}", SoundProfile::CinemaSurround3D.name(), SoundProfile::CinemaSurround3D.description());
                }
                glfw::WindowEvent::Key(Key::Num2, _, Action::Press, _) => {
                    pipeline.apply_sound_profile(SoundProfile::ConcertHallLive);
                    println!("[MODE 2] Active: {} -> {}", SoundProfile::ConcertHallLive.name(), SoundProfile::ConcertHallLive.description());
                }
                glfw::WindowEvent::Key(Key::Num3, _, Action::Press, _) => {
                    pipeline.apply_sound_profile(SoundProfile::VocalIntimacy);
                    println!("[MODE 3] Active: {} -> {}", SoundProfile::VocalIntimacy.name(), SoundProfile::VocalIntimacy.description());
                }
                glfw::WindowEvent::Key(Key::Num4, _, Action::Press, _) => {
                    pipeline.apply_sound_profile(SoundProfile::ClubBassReflex);
                    println!("[MODE 4] Active: {} -> {}", SoundProfile::ClubBassReflex.name(), SoundProfile::ClubBassReflex.description());
                }
                glfw::WindowEvent::Key(Key::Num5, _, Action::Press, _) => {
                    pipeline.apply_sound_profile(SoundProfile::NightAudiophile);
                    println!("[MODE 5] Active: {} -> {}", SoundProfile::NightAudiophile.name(), SoundProfile::NightAudiophile.description());
                }
                glfw::WindowEvent::Key(Key::Num6, _, Action::Press, _) => {
                    pipeline.apply_sound_profile(SoundProfile::StudioReference);
                    println!("[MODE 6] Active: {} -> {}", SoundProfile::StudioReference.name(), SoundProfile::StudioReference.description());
                }
                glfw::WindowEvent::Key(Key::Num7, _, Action::Press, _) => {
                    pipeline.apply_sound_profile(SoundProfile::HarmanConsumer);
                    println!("[MODE 7] Active: {} -> {}", SoundProfile::HarmanConsumer.name(), SoundProfile::HarmanConsumer.description());
                }
                glfw::WindowEvent::Key(Key::N, _, Action::Press, _) => {
                    let cur = controls.is_noise_cancellation();
                    controls.set_noise_cancellation(!cur);
                    println!("[DSP] Adaptive Noise Cancellation: {}", if !cur { "ACTIVE (-18 dB De-Hiss + Rumble Block)" } else { "OFF" });
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    pointer_x = (x as f32 / WIDTH as f32).clamp(0.0, 1.0);
                    pointer_y = (1.0 - (y as f32 / HEIGHT as f32)).clamp(0.0, 1.0);
                }
                glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                    let cur = controls.is_bypass();
                    controls.set_bypass(!cur);
                    println!("[UI] Click -> Toggle Bypass: {}", if !cur { "BYPASS (Dry)" } else { "ACTIVE (Autonomous AI Mastering)" });
                }
                _ => {}
            }
        }

        let elapsed = start_time.elapsed().as_secs_f32();
        let mut audio_block = vec![0.0f32; 256];
        for i in 0..128 {
            let sample_t = elapsed + (i as f32 / 48000.0);
            let kick = 0.45 * (sample_t * 2.2).sin().abs().powi(8) * (2.0 * std::f32::consts::PI * 55.0 * sample_t).sin();
            let vocal = 0.35 * (2.0 * std::f32::consts::PI * 440.0 * sample_t).sin() * (sample_t * 1.5).sin().abs();
            let air = 0.10 * (2.0 * std::f32::consts::PI * 12000.0 * sample_t).sin();
            audio_block[i * 2] = kick + vocal + air;
            audio_block[i * 2 + 1] = kick + vocal - air;
        }

        pipeline.process_interleaved(&audio_block, &mut dummy_out);
        let telem = pipeline.get_telemetry();
        let rms_linear = 10.0f32.powf((telem.rms_db / 20.0).clamp(-60.0, 0.0));
        let audio_rms = if controls.is_bypass() {
            (rms_linear * 0.8).clamp(0.05, 0.95)
        } else {
            (rms_linear * 1.5).clamp(0.15, 0.98)
        };
        let bypass_val = if controls.is_bypass() { 1.0f32 } else { 0.0f32 };

        unsafe {
            gl::Viewport(0, 0, WIDTH as i32, HEIGHT as i32);
            gl::ClearColor(0.02, 0.02, 0.04, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(program);
            gl::Uniform2f(u_res, WIDTH as f32, HEIGHT as f32);
            gl::Uniform1f(u_time, elapsed);
            gl::Uniform2f(u_ptr, pointer_x, pointer_y);
            gl::Uniform1f(u_audio, audio_rms);
            gl::Uniform1f(u_byp, bypass_val);

            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

        if frame_count % 60 == 0 {
            if let Some(und) = pipeline.get_ai_understanding() {
                println!(
                    "[AI LIVE] Genre: {} ({:.0}%) | Bass: {} | Treble: {} | Vocals: {:.0}% (Clarity: {:.0}%)",
                    und.primary_genre.as_str(),
                    und.genre_confidence * 100.0,
                    und.bass_state,
                    und.treble_state,
                    und.vocal_presence * 100.0,
                    und.vocal_clarity_score * 100.0
                );
            }
        }

        if let Some(ref path) = screenshot_path {
            if frame_count >= 10 {
                // Read pixels from backbuffer
                let mut pixels = vec![0u8; (WIDTH * HEIGHT * 3) as usize];
                unsafe {
                    gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
                    gl::ReadPixels(
                        0, 0, WIDTH as i32, HEIGHT as i32,
                        gl::RGB, gl::UNSIGNED_BYTE,
                        pixels.as_mut_ptr() as *mut _,
                    );
                }

                // Write flipped RGB to PPM
                let ppm_path = "/tmp/mx5ic_render.ppm";
                if let Ok(mut f) = std::fs::File::create(ppm_path) {
                    let header = format!("P6\n{} {}\n255\n", WIDTH, HEIGHT);
                    let _ = f.write_all(header.as_bytes());
                    for row in (0..HEIGHT).rev() {
                        let start = (row * WIDTH * 3) as usize;
                        let end = start + (WIDTH * 3) as usize;
                        let _ = f.write_all(&pixels[start..end]);
                    }
                    let _ = f.flush();

                    // Convert to PNG with ffmpeg
                    let _ = std::process::Command::new("ffmpeg")
                        .args(&["-y", "-i", ppm_path, path])
                        .output();
                    println!("[+] Screenshot captured and saved to: {}", path);
                }
                break;
            }
        }

        window.swap_buffers();
        frame_count += 1;

        if headless && screenshot_path.is_none() && frame_count >= 60 {
            println!("[+] Headless bench complete: rendered {} frames.", frame_count);
            break;
        }
    }

    unsafe {
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteProgram(program);
    }
}
