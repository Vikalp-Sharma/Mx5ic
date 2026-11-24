use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    let host = cpal::default_host();
    println!("Default Host: {:?}", host.id());

    println!("\n=== Output Devices ===");
    if let Ok(devices) = host.output_devices() {
        for (i, dev) in devices.enumerate() {
            let name = dev.name().unwrap_or_else(|_| "Unknown".into());
            println!("{}. Output Device: {}", i, name);
        }
    }

    println!("\n=== Input Devices ===");
    if let Ok(devices) = host.input_devices() {
        for (i, dev) in devices.enumerate() {
            let name = dev.name().unwrap_or_else(|_| "Unknown".into());
            println!("{}. Input Device: {}", i, name);
        }
    }
}
