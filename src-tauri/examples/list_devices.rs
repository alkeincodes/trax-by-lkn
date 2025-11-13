use cpal::traits::{HostTrait, DeviceTrait};

fn main() {
    let host = cpal::default_host();
    
    println!("=== Default Output Device ===");
    if let Some(device) = host.default_output_device() {
        if let Ok(name) = device.name() {
            println!("Default: {}", name);
        }
    }
    
    println!("\n=== All Output Devices ===");
    if let Ok(devices) = host.output_devices() {
        for (i, device) in devices.enumerate() {
            if let Ok(name) = device.name() {
                println!("{}: {}", i + 1, name);
            }
        }
    }
}
