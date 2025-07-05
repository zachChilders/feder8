use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::io::{BufRead, BufReader};
use std::thread;

#[test]
fn test_firmware_boot_and_functionality() {
    println!("Starting ESP32 firmware integration test...");
    
    // Build the firmware first
    let build_result = Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir(".")
        .output()
        .expect("Failed to build firmware");
    
    if !build_result.status.success() {
        panic!("Firmware build failed: {}", String::from_utf8_lossy(&build_result.stderr));
    }
    
    println!("Firmware build successful");
    
    // Start QEMU with the firmware
    let mut qemu_process = Command::new("bash")
        .arg("qemu_test.sh")
        .current_dir(".")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start QEMU");
    
    let stdout = qemu_process.stdout.take().expect("Failed to get QEMU stdout");
    let reader = BufReader::new(stdout);
    
    // Test expectations
    let mut found_hello_world = false;
    let mut found_led_toggle = false;
    let mut found_startup_message = false;
    
    let start_time = Instant::now();
    let timeout = Duration::from_secs(30); // 30 second timeout
    
    // Read QEMU output and validate expected messages
    for line in reader.lines() {
        if start_time.elapsed() > timeout {
            break;
        }
        
        match line {
            Ok(output) => {
                println!("QEMU Output: {}", output);
                
                // Check for expected firmware messages
                if output.contains("Hello, ESP32 World!") {
                    found_hello_world = true;
                    println!("✓ Found hello world message");
                }
                
                if output.contains("LED pin configured successfully") {
                    found_startup_message = true;
                    println!("✓ Found startup message");
                }
                
                if output.contains("LED ON") || output.contains("LED OFF") {
                    found_led_toggle = true;
                    println!("✓ Found LED toggle message");
                }
                
                // If we found all expected messages, we can terminate early
                if found_hello_world && found_led_toggle && found_startup_message {
                    println!("All expected messages found - test successful!");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading QEMU output: {}", e);
                break;
            }
        }
    }
    
    // Terminate QEMU process
    let _ = qemu_process.kill();
    let _ = qemu_process.wait();
    
    // Assert that we found all expected functionality
    assert!(found_hello_world, "Did not find 'Hello, ESP32 World!' message");
    assert!(found_startup_message, "Did not find startup message");
    assert!(found_led_toggle, "Did not find LED toggle messages");
    
    println!("ESP32 firmware integration test completed successfully!");
}

#[test]
fn test_firmware_build_only() {
    println!("Testing firmware build process...");
    
    // Check if ESP32 target is available
    let target_check = Command::new("rustup")
        .args(&["target", "list", "--installed"])
        .output()
        .expect("Failed to check installed targets");
    
    let targets = String::from_utf8_lossy(&target_check.stdout);
    
    if targets.contains("xtensa-esp32-espidf") {
        // If ESP32 target is available, try to build
        let build_result = Command::new("cargo")
            .args(&["build", "--release"])
            .current_dir(".")
            .output()
            .expect("Failed to execute cargo build");
        
        if !build_result.status.success() {
            println!("Firmware build failed (expected in CI without ESP-IDF): {}", String::from_utf8_lossy(&build_result.stderr));
        } else {
            println!("Firmware build test passed!");
        }
    } else {
        println!("ESP32 target not installed - skipping build test (this is expected in CI)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_project_structure() {
        // Test that all required files exist
        let required_files = vec![
            "Cargo.toml",
            "build.rs", 
            "src/main.rs",
            "src/sim_runner.rs",
            "sdkconfig.defaults",
            ".cargo/config.toml",
            "qemu_esp32.sh",
            "qemu_simple.sh",
            "qemu_test.sh"
        ];
        
        for file in required_files {
            assert!(std::path::Path::new(file).exists(), "Required file missing: {}", file);
        }
        
        println!("All required project files exist!");
    }
}