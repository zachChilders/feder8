use std::path::Path;
use std::process::Command;

fn main() {
    println!("ESP32 Firmware Test Runner");
    println!("==========================");

    let mut tests_passed = 0;
    let mut tests_failed = 0;

    // Test 1: Project structure
    print!("Testing project structure... ");
    if test_project_structure() {
        println!("✓ PASSED");
        tests_passed += 1;
    } else {
        println!("✗ FAILED");
        tests_failed += 1;
    }

    // Test 2: Script permissions
    print!("Testing script permissions... ");
    if test_scripts_executable() {
        println!("✓ PASSED");
        tests_passed += 1;
    } else {
        println!("✗ FAILED");
        tests_failed += 1;
    }

    // Test 3: Cargo.toml structure
    print!("Testing Cargo.toml structure... ");
    if test_cargo_toml_structure() {
        println!("✓ PASSED");
        tests_passed += 1;
    } else {
        println!("✗ FAILED");
        tests_failed += 1;
    }

    // Test 4: Source code syntax
    print!("Testing source code syntax... ");
    if test_source_code_syntax() {
        println!("✓ PASSED");
        tests_passed += 1;
    } else {
        println!("✗ FAILED");
        tests_failed += 1;
    }

    // Test 5: QEMU simulation
    print!("Testing QEMU simulation... ");
    if test_qemu_simulation() {
        println!("✓ PASSED");
        tests_passed += 1;
    } else {
        println!("✗ FAILED");
        tests_failed += 1;
    }

    // Summary
    println!("\n==========================");
    println!("Test Results:");
    println!("Tests Passed: {}", tests_passed);
    println!("Tests Failed: {}", tests_failed);

    if tests_failed == 0 {
        println!("🎉 All tests passed!");
        std::process::exit(0);
    } else {
        println!("❌ Some tests failed!");
        std::process::exit(1);
    }
}

fn test_project_structure() -> bool {
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
        "test_setup.sh",
    ];

    for file in required_files {
        if !Path::new(file).exists() {
            eprintln!("Required file missing: {}", file);
            return false;
        }
    }

    true
}

fn test_scripts_executable() -> bool {
    let scripts = vec![
        "qemu_esp32.sh",
        "qemu_simple.sh",
        "qemu_test.sh",
        "run_tests.sh",
        "test_setup.sh",
    ];

    for script in scripts {
        let path = Path::new(script);
        if !path.exists() {
            eprintln!("Script missing: {}", script);
            return false;
        }

        // Check if file is executable (on Unix systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(path) {
                let permissions = metadata.permissions();
                if permissions.mode() & 0o111 == 0 {
                    eprintln!("Script not executable: {}", script);
                    return false;
                }
            } else {
                eprintln!("Failed to get metadata for: {}", script);
                return false;
            }
        }
    }

    true
}

fn test_cargo_toml_structure() -> bool {
    let cargo_content = match std::fs::read_to_string("Cargo.toml") {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Failed to read Cargo.toml");
            return false;
        }
    };

    // Check for required sections
    let required_sections = vec![
        ("[package]", "Missing [package] section"),
        ("name = \"feder8-firmware\"", "Missing package name"),
        ("[dependencies]", "Missing [dependencies] section"),
        ("esp-idf-sys", "Missing esp-idf-sys dependency"),
        ("esp-idf-hal", "Missing esp-idf-hal dependency"),
    ];

    for (section, error_msg) in required_sections {
        if !cargo_content.contains(section) {
            eprintln!("{}", error_msg);
            return false;
        }
    }

    true
}

fn test_source_code_syntax() -> bool {
    // Check main.rs
    let main_content = match std::fs::read_to_string("src/main.rs") {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Failed to read main.rs");
            return false;
        }
    };

    let main_checks = vec![
        ("fn main()", "Missing main function"),
        ("esp_idf_sys", "Missing ESP-IDF imports"),
        ("println!", "Missing println macro"),
    ];

    for (check, error_msg) in main_checks {
        if !main_content.contains(check) {
            eprintln!("{}", error_msg);
            return false;
        }
    }

    // Check sim_runner.rs
    let sim_content = match std::fs::read_to_string("src/sim_runner.rs") {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Failed to read sim_runner.rs");
            return false;
        }
    };

    let sim_checks = vec![
        ("fn main()", "Missing main function in sim_runner"),
        ("Command", "Missing Command import in sim_runner"),
    ];

    for (check, error_msg) in sim_checks {
        if !sim_content.contains(check) {
            eprintln!("{}", error_msg);
            return false;
        }
    }

    true
}

fn test_qemu_simulation() -> bool {
    // Run the test simulation
    let output = match Command::new("./qemu_test.sh").output() {
        Ok(output) => output,
        Err(_) => {
            eprintln!("Failed to run QEMU test");
            return false;
        }
    };

    if !output.status.success() {
        eprintln!("QEMU test script failed");
        return false;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check for expected output
    let expected_messages = vec![
        ("Hello, ESP32 World!", "Missing hello world message"),
        (
            "LED pin configured successfully",
            "Missing LED config message",
        ),
    ];

    for (message, error_msg) in expected_messages {
        if !stdout.contains(message) {
            eprintln!("{}", error_msg);
            return false;
        }
    }

    // Check for LED toggle messages
    if !stdout.contains("LED ON") && !stdout.contains("LED OFF") {
        eprintln!("Missing LED toggle message");
        return false;
    }

    true
}
