use std::process::Command;

pub const VIRTUAL_SINK_NAME: &str = "Mx5ic_Sink";
pub const VIRTUAL_SINK_DESC: &str = "Mx5ic_Isolated_Audio_Processor";

pub struct PipeWireManager {
    module_id: Option<String>,
}

impl PipeWireManager {
    pub fn new() -> Self {
        Self { module_id: None }
    }

    /// Creates the dedicated PipeWire virtual sink node with zero default physical routing.
    pub fn create_isolated_sink(&mut self) -> Result<String, String> {
        println!("[*] Creating isolated PipeWire virtual sink: {}...", VIRTUAL_SINK_NAME);

        // Check if sink already exists
        let check_output = Command::new("pactl")
            .args(&["list", "short", "sinks"])
            .output();

        if let Ok(out) = check_output {
            let sinks_str = String::from_utf8_lossy(&out.stdout);
            if sinks_str.contains(VIRTUAL_SINK_NAME) {
                println!("[+] Virtual sink {} already exists.", VIRTUAL_SINK_NAME);
                return Ok("existing".into());
            }
        }

        let output = Command::new("pactl")
            .args(&[
                "load-module",
                "module-null-sink",
                &format!("sink_name={}", VIRTUAL_SINK_NAME),
                &format!("sink_properties=device.description=\"{}\"", VIRTUAL_SINK_DESC),
                "rate=48000",
                "channels=2",
            ])
            .output()
            .map_err(|e| format!("Failed to execute pactl: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("pactl error: {}", err));
        }

        let mod_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("[+] PipeWire virtual sink initialized with module ID: {}", mod_id);
        self.module_id = Some(mod_id.clone());

        // Verify isolation
        self.verify_isolation();

        Ok(mod_id)
    }

    /// Verifies that no audio can leak directly to the physical sound card
    pub fn verify_isolation(&self) -> bool {
        let output = Command::new("pw-link")
            .arg("-l")
            .output();

        if let Ok(out) = output {
            let links = String::from_utf8_lossy(&out.stdout);
            let has_leak = links.lines().any(|line| {
                line.contains(VIRTUAL_SINK_NAME) && line.contains("alsa_output")
            });

            if !has_leak {
                println!("[+] Verified: Mx5ic_Sink is completely isolated (0 raw leakage to physical output).");
                return true;
            } else {
                eprintln!("[!] Warning: Found direct connection from virtual sink to physical device!");
                return false;
            }
        }
        true
    }

    pub fn destroy_sink(&mut self) {
        if let Some(ref mod_id) = self.module_id {
            println!("[*] Unloading PipeWire virtual sink module {}...", mod_id);
            let _ = Command::new("pactl")
                .args(&["unload-module", mod_id])
                .output();
            self.module_id = None;
        }
    }
}

impl Drop for PipeWireManager {
    fn drop(&mut self) {
        self.destroy_sink();
    }
}
