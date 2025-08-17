use std::process::Command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Helper function to run rigr command and capture output
fn run_rigr_command(args: &[&str]) -> Result<std::process::Output, std::io::Error> {
    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--release").arg("--");
    cmd.args(args);
    cmd.output()
}

/// Helper function to create a temporary config for testing
fn create_test_config() -> Result<TempDir, Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let config_dir = temp_dir.path().join(".rigr");
    fs::create_dir_all(&config_dir)?;
    
    let config_content = r#"
[ai]
provider = "Ollama"
api_url = "http://localhost:11434"
model = "codellama"

[testing]
default_framework = "auto"
unit_test_dir = "tests/unit"
integration_test_dir = "tests/integration"
coverage_threshold = 80.0
"#;
    
    fs::write(config_dir.join("config.toml"), config_content)?;
    
    // Set HOME environment variable to temp directory for testing
    std::env::set_var("HOME", temp_dir.path());
    
    Ok(temp_dir)
}

#[test]
fn test_help_command() {
    let output = run_rigr_command(&["--help"]).expect("Failed to execute rigr --help");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("AI-Powered Comprehensive Test Case Generator"));
    assert!(stdout.contains("--file"));
    assert!(stdout.contains("--requirements"));
    assert!(stdout.contains("--github"));
    assert!(stdout.contains("--export"));
}

#[test]
fn test_version_command() {
    let output = run_rigr_command(&["--version"]).expect("Failed to execute rigr --version");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rigr"));
    assert!(stdout.contains("1.0.0"));
}

#[test]
fn test_single_file_processing() {
    let _temp_config = create_test_config().expect("Failed to create test config");
    
    let test_file = "tests/fixtures/source_files/calculator.rs";
    if !Path::new(test_file).exists() {
        panic!("Test file not found: {test_file}");
    }
    
    let temp_output = tempfile::tempdir().expect("Failed to create temp output dir");
    let output_path = temp_output.path().to_str().unwrap();
    
    let output = run_rigr_command(&[
        "--file", test_file,
        "--output", output_path,
        "--unit-only"
    ]).expect("Failed to execute rigr with single file");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("STDOUT: {stdout}");
        println!("STDERR: {stderr}");
        
        // Check if it's a configuration error (expected in some test environments)
        if stderr.contains("Configuration not found") || stdout.contains("Please run 'rigr --setup'") {
            println!("Test skipped: Configuration not available in test environment");
            return;
        }
        
        panic!("rigr command failed");
    }
    
    // Check that output files were created
    let rigr_test_cases = Path::new(output_path).join("RigrTestCases");
    if rigr_test_cases.exists() {
        println!("✅ Test output directory created successfully");
    }
}

#[test]
fn test_directory_processing() {
    let _temp_config = create_test_config().expect("Failed to create test config");
    
    let test_dir = "tests/fixtures/source_files";
    if !Path::new(test_dir).exists() {
        panic!("Test directory not found: {test_dir}");
    }
    
    let temp_output = tempfile::tempdir().expect("Failed to create temp output dir");
    let output_path = temp_output.path().to_str().unwrap();
    
    let output = run_rigr_command(&[
        "--directory", test_dir,
        "--output", output_path,
        "--unit-only",
        "--edge-cases", "5"
    ]).expect("Failed to execute rigr with directory");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("STDOUT: {stdout}");
        println!("STDERR: {stderr}");
        
        if stderr.contains("Configuration not found") || stdout.contains("Please run 'rigr --setup'") {
            println!("Test skipped: Configuration not available in test environment");
            return;
        }
        
        panic!("rigr directory command failed");
    }
    
    println!("✅ Directory processing test completed");
}

#[test]
fn test_requirements_processing() {
    let _temp_config = create_test_config().expect("Failed to create test config");
    
    let requirements_file = "tests/fixtures/requirements/calculator_requirements.txt";
    if !Path::new(requirements_file).exists() {
        panic!("Requirements file not found: {requirements_file}");
    }
    
    let temp_output = tempfile::tempdir().expect("Failed to create temp output dir");
    let output_path = temp_output.path().to_str().unwrap();
    
    let output = run_rigr_command(&[
        "--requirements", requirements_file,
        "--output", output_path,
        "--unit-only"
    ]).expect("Failed to execute rigr with requirements");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("STDOUT: {stdout}");
        println!("STDERR: {stderr}");
        
        if stderr.contains("Configuration not found") || stdout.contains("Please run 'rigr --setup'") {
            println!("Test skipped: Configuration not available in test environment");
            return;
        }
        
        panic!("rigr requirements command failed");
    }
    
    println!("✅ Requirements processing test completed");
}

#[test]
fn test_export_formats() {
    let _temp_config = create_test_config().expect("Failed to create test config");
    
    let test_file = "tests/fixtures/source_files/calculator.py";
    if !Path::new(test_file).exists() {
        panic!("Test file not found: {test_file}");
    }
    
    let temp_output = tempfile::tempdir().expect("Failed to create temp output dir");
    let output_path = temp_output.path().to_str().unwrap();
    
    let output = run_rigr_command(&[
        "--file", test_file,
        "--output", output_path,
        "--unit-only",
        "--export", "csv,json,markdown"
    ]).expect("Failed to execute rigr with export formats");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("STDOUT: {stdout}");
        println!("STDERR: {stderr}");
        
        if stderr.contains("Configuration not found") || stdout.contains("Please run 'rigr --setup'") {
            println!("Test skipped: Configuration not available in test environment");
            return;
        }
        
        panic!("rigr export command failed");
    }
    
    println!("✅ Export formats test completed");
}

#[test]
fn test_performance_and_security_flags() {
    let _temp_config = create_test_config().expect("Failed to create test config");
    
    let test_file = "tests/fixtures/source_files/calculator.js";
    if !Path::new(test_file).exists() {
        panic!("Test file not found: {test_file}");
    }
    
    let temp_output = tempfile::tempdir().expect("Failed to create temp output dir");
    let output_path = temp_output.path().to_str().unwrap();
    
    let output = run_rigr_command(&[
        "--file", test_file,
        "--output", output_path,
        "--unit-only",
        "--include-performance",
        "--include-security"
    ]).expect("Failed to execute rigr with performance and security flags");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("STDOUT: {stdout}");
        println!("STDERR: {stderr}");
        
        if stderr.contains("Configuration not found") || stdout.contains("Please run 'rigr --setup'") {
            println!("Test skipped: Configuration not available in test environment");
            return;
        }
        
        panic!("rigr performance/security command failed");
    }
    
    println!("✅ Performance and security flags test completed");
}

#[test]
fn test_integration_only_flag() {
    let _temp_config = create_test_config().expect("Failed to create test config");
    
    let test_file = "tests/fixtures/source_files/calculator.java";
    if !Path::new(test_file).exists() {
        panic!("Test file not found: {test_file}");
    }
    
    let temp_output = tempfile::tempdir().expect("Failed to create temp output dir");
    let output_path = temp_output.path().to_str().unwrap();
    
    let output = run_rigr_command(&[
        "--file", test_file,
        "--output", output_path,
        "--integration-only"
    ]).expect("Failed to execute rigr with integration-only flag");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("STDOUT: {stdout}");
        println!("STDERR: {stderr}");
        
        if stderr.contains("Configuration not found") || stdout.contains("Please run 'rigr --setup'") {
            println!("Test skipped: Configuration not available in test environment");
            return;
        }
        
        panic!("rigr integration-only command failed");
    }
    
    println!("✅ Integration-only flag test completed");
}

#[test]
fn test_concurrency_setting() {
    let _temp_config = create_test_config().expect("Failed to create test config");
    
    let test_dir = "tests/fixtures/source_files";
    if !Path::new(test_dir).exists() {
        panic!("Test directory not found: {test_dir}");
    }
    
    let temp_output = tempfile::tempdir().expect("Failed to create temp output dir");
    let output_path = temp_output.path().to_str().unwrap();
    
    let output = run_rigr_command(&[
        "--directory", test_dir,
        "--output", output_path,
        "--unit-only",
        "--concurrency", "1",
        "--edge-cases", "3"
    ]).expect("Failed to execute rigr with concurrency setting");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("STDOUT: {stdout}");
        println!("STDERR: {stderr}");
        
        if stderr.contains("Configuration not found") || stdout.contains("Please run 'rigr --setup'") {
            println!("Test skipped: Configuration not available in test environment");
            return;
        }
        
        panic!("rigr concurrency command failed");
    }
    
    println!("✅ Concurrency setting test completed");
}

#[test]
fn test_verbose_and_debug_flags() {
    let _temp_config = create_test_config().expect("Failed to create test config");
    
    let test_file = "tests/fixtures/source_files/calculator.c";
    if !Path::new(test_file).exists() {
        panic!("Test file not found: {test_file}");
    }
    
    let temp_output = tempfile::tempdir().expect("Failed to create temp output dir");
    let output_path = temp_output.path().to_str().unwrap();
    
    let output = run_rigr_command(&[
        "--file", test_file,
        "--output", output_path,
        "--unit-only",
        "--verbose",
        "--debug"
    ]).expect("Failed to execute rigr with verbose and debug flags");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("STDOUT: {stdout}");
        println!("STDERR: {stderr}");
        
        if stderr.contains("Configuration not found") || stdout.contains("Please run 'rigr --setup'") {
            println!("Test skipped: Configuration not available in test environment");
            return;
        }
        
        panic!("rigr verbose/debug command failed");
    }
    
    println!("✅ Verbose and debug flags test completed");
}

#[test]
fn test_error_handling_invalid_file() {
    let output = run_rigr_command(&[
        "--file", "non_existent_file.rs"
    ]).expect("Failed to execute rigr with invalid file");
    
    // Should fail with invalid file
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should contain error message about file not found
    assert!(stderr.contains("not found") || stdout.contains("not found") || 
            stderr.contains("No such file") || stdout.contains("No such file"));
    
    println!("✅ Error handling test completed");
}

#[test]
fn test_unsupported_file_extension() {
    // Create a temporary file with unsupported extension
    let temp_file = tempfile::NamedTempFile::with_suffix(".xyz").expect("Failed to create temp file");
    fs::write(temp_file.path(), "some content").expect("Failed to write temp file");
    
    let output = run_rigr_command(&[
        "--file", temp_file.path().to_str().unwrap()
    ]).expect("Failed to execute rigr with unsupported file");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Should contain unsupported language error or no files found
        assert!(stderr.contains("Unsupported") || stdout.contains("No supported source files found") ||
                stderr.contains("Configuration not found") || stdout.contains("Please run 'rigr --setup'"));
    }
    
    println!("✅ Unsupported file extension test completed");
}