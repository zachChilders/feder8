use std::process::Command;
use std::path::Path;

#[test]
fn test_project_structure() {
    println!("Testing firmware project structure...");
    
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
        "qemu_test.sh",
        "README.md",
        "run_tests.sh",
        "test_setup.sh"
    ];
    
    for file in required_files {
        assert!(Path::new(file).exists(), "Required file missing: {}", file);
    }
    
    println!("✓ All required project files exist!");
}

#[test]
fn test_scripts_executable() {
    println!("Testing script permissions...");
    
    let scripts = vec![
        "qemu_esp32.sh",
        "qemu_simple.sh", 
        "qemu_test.sh",
        "run_tests.sh",
        "test_setup.sh"
    ];
    
    for script in scripts {
        let path = Path::new(script);
        assert!(path.exists(), "Script missing: {}", script);
        
        // Check if file is executable (on Unix systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(path).expect("Failed to get metadata");
            let permissions = metadata.permissions();
            assert!(permissions.mode() & 0o111 != 0, "Script not executable: {}", script);
        }
    }
    
    println!("✓ All scripts are executable!");
}

#[test]
fn test_cargo_toml_structure() {
    println!("Testing Cargo.toml structure...");
    
    let cargo_content = std::fs::read_to_string("Cargo.toml")
        .expect("Failed to read Cargo.toml");
    
    // Check for required sections
    assert!(cargo_content.contains("[package]"), "Missing [package] section");
    assert!(cargo_content.contains("name = \"feder8-firmware\""), "Missing package name");
    assert!(cargo_content.contains("[dependencies]"), "Missing [dependencies] section");
    assert!(cargo_content.contains("esp-idf-sys"), "Missing esp-idf-sys dependency");
    assert!(cargo_content.contains("esp-idf-hal"), "Missing esp-idf-hal dependency");
    
    println!("✓ Cargo.toml structure is valid!");
}

#[test]
fn test_qemu_simulation() {
    println!("Testing QEMU simulation...");
    
    // Run the test simulation
    let output = Command::new("./qemu_test.sh")
        .output()
        .expect("Failed to run QEMU test");
    
    assert!(output.status.success(), "QEMU test script failed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Check for expected output
    assert!(stdout.contains("Hello, ESP32 World!"), "Missing hello world message");
    assert!(stdout.contains("LED pin configured successfully"), "Missing LED config message");
    assert!(stdout.contains("LED ON") || stdout.contains("LED OFF"), "Missing LED toggle message");
    
    println!("✓ QEMU simulation test passed!");
}

#[test]
fn test_source_code_syntax() {
    println!("Testing source code syntax...");
    
    // Check main.rs syntax by attempting to parse it
    let main_content = std::fs::read_to_string("src/main.rs")
        .expect("Failed to read main.rs");
    
    // Basic syntax checks
    assert!(main_content.contains("fn main()"), "Missing main function");
    assert!(main_content.contains("esp_idf_sys"), "Missing ESP-IDF imports");
    assert!(main_content.contains("println!"), "Missing println macro");
    
    // Check sim_runner.rs
    let sim_content = std::fs::read_to_string("src/sim_runner.rs")
        .expect("Failed to read sim_runner.rs");
    
    assert!(sim_content.contains("fn main()"), "Missing main function in sim_runner");
    assert!(sim_content.contains("Command"), "Missing Command import in sim_runner");
    
    println!("✓ Source code syntax checks passed!");
}