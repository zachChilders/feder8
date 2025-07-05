use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Default to wokwi simulation
    let script = if args.len() > 1 && args[1] == "qemu" {
        "qemu_esp32.sh"
    } else {
        "qemu_simple.sh"
    };

    let firmware_dir = env::current_dir()
        .expect("Failed to get current directory")
        .join("src/firmware");

    let script_path = firmware_dir.join(script);

    println!("Running ESP32 simulation with: {}", script);
    println!("Script path: {}", script_path.display());

    if !script_path.exists() {
        eprintln!("Error: Script not found at {}", script_path.display());
        std::process::exit(1);
    }

    let mut cmd = Command::new("bash");
    cmd.arg(script_path);
    cmd.current_dir(firmware_dir);

    match cmd.spawn() {
        Ok(mut child) => match child.wait() {
            Ok(status) => {
                if !status.success() {
                    eprintln!("Simulation exited with error: {}", status);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Failed to wait for simulation: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Failed to start simulation: {}", e);
            std::process::exit(1);
        }
    }
}
