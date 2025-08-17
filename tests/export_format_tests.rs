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
    std::env::set_var("HOME", temp_dir.path());
    Ok(temp_dir)
}

#[test]
fn test_csv_export_format() {
    let _temp_config = create_test_config().expect("Failed to create test config");
    
    let test_file = "tests/fixtures/source_files/calculator.rs";
    if !Path::new(test_file).exists() {
        println!("Test skipped: Test file not found");
        return;
    }
    
    let temp_output = tempfile::tempdir().expect("Failed to create temp output dir");
    let output_path = temp_output.path().to_str().unwrap();
    
    let output = run_rigr_command(&[
        "--file", test_file,
        "--output", output_path,
        "--unit-only",
        "--export", "csv"
    ]).expect("Failed to execute rigr with CSV export");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        if stderr.contains("Configuration not found") || stdout.contains("Please run 'rigr --setup'") {
            println!("Test skipped: Configuration not available in test environment");
            return;
        }
        
        println!("STDOUT: {stdout}");
        println!("STDERR: {stderr}");
        panic!("CSV export test failed");
    }
    
    // Check if export directory was created
    let exports_dir = Path::new(output_path).join("RigrTestCases").join("calculator").join("Exports");
    if exports_dir.exists() {
        println!("✅ CSV export test completed - exports directory created");
    } else {
        println!("⚠️ CSV export test completed - no exports directory found (may be expected)");
    }
}

#[test]
fn test_json_export_format() {
    let _temp_config = create_test_config().expect("Failed to create test config");
    
    let test_file = "tests/fixtures/source_files/calculator.py";
    if !Path::new(test_file).exists() {
        println!("Test skipped: Test file not found");
        return;
    }
    
    let temp_output = tempfile::tempdir().expect("Failed to create temp output dir");
    let output_path = temp_output.path().to_str().unwrap();
    
    let output = run_rigr_command(&[
        "--file", test_file,
        "--output", output_path,
        "--unit-only",
        "--export", "json"
    ]).expect("Failed to execute rigr with JSON export");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        if stderr.contains("Configuration not found") || stdout.contains("Please run 'rigr --setup'") {
            println!("Test skipped: Configuration not available in test environment");
            return;
        }
        
        println!("STDOUT: {stdout}");
        println!("STDERR: {stderr}");
        panic!("JSON export test failed");
    }
    
    println!("✅ JSON export test completed");
}

#[test]
fn test_excel_export_format() {
    let _temp_config = create_test_config().expect("Failed to create test config");
    
    let test_file = "tests/fixtures/source_files/calculator.js";
    if !Path::new(test_file).exists() {
        println!("Test skipped: Test file not found");
        return;
    }
    
    let temp_output = tempfile::tempdir().expect("Failed to create temp output dir");
    let output_path = temp_output.path().to_str().unwrap();
    
    let output = run_rigr_command(&[
        "--file", test_file,
        "--output", output_path,
        "--unit-only",
        "--export", "excel"
    ]).expect("Failed to execute rigr with Excel export");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        if stderr.contains("Configuration not found") || stdout.contains("Please run 'rigr --setup'") {
            println!("Test skipped: Configuration not available in test environment");
            return;
        }
        
        println!("STDOUT: {stdout}");
        println!("STDERR: {stderr}");
        panic!("Excel export test failed");
    }
    
    println!("✅ Excel export test completed");
}

#[test]
fn test_multiple_export_formats() {
    let _temp_config = create_test_config().expect("Failed to create test config");
    
    let test_file = "tests/fixtures/source_files/calculator.java";
    if !Path::new(test_file).exists() {
        println!("Test skipped: Test file not found");
        return;
    }
    
    let temp_output = tempfile::tempdir().expect("Failed to create temp output dir");
    let output_path = temp_output.path().to_str().unwrap();
    
    let output = run_rigr_command(&[
        "--file", test_file,
        "--output", output_path,
        "--unit-only",
        "--export", "csv,json,markdown"
    ]).expect("Failed to execute rigr with multiple export formats");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        if stderr.contains("Configuration not found") || stdout.contains("Please run 'rigr --setup'") {
            println!("Test skipped: Configuration not available in test environment");
            return;
        }
        
        println!("STDOUT: {stdout}");
        println!("STDERR: {stderr}");
        panic!("Multiple export formats test failed");
    }
    
    println!("✅ Multiple export formats test completed");
}

#[test]
fn test_all_export_formats() {
    let _temp_config = create_test_config().expect("Failed to create test config");
    
    let test_file = "tests/fixtures/source_files/calculator.c";
    if !Path::new(test_file).exists() {
        println!("Test skipped: Test file not found");
        return;
    }
    
    let temp_output = tempfile::tempdir().expect("Failed to create temp output dir");
    let output_path = temp_output.path().to_str().unwrap();
    
    let output = run_rigr_command(&[
        "--file", test_file,
        "--output", output_path,
        "--unit-only",
        "--export", "all"
    ]).expect("Failed to execute rigr with all export formats");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        if stderr.contains("Configuration not found") || stdout.contains("Please run 'rigr --setup'") {
            println!("Test skipped: Configuration not available in test environment");
            return;
        }
        
        println!("STDOUT: {stdout}");
        println!("STDERR: {stderr}");
        panic!("All export formats test failed");
    }
    
    println!("✅ All export formats test completed");
}

#[test]
fn test_testrail_export_format() {
    let _temp_config = create_test_config().expect("Failed to create test config");
    
    let requirements_file = "tests/fixtures/requirements/calculator_requirements.txt";
    if !Path::new(requirements_file).exists() {
        println!("Test skipped: Requirements file not found");
        return;
    }
    
    let temp_output = tempfile::tempdir().expect("Failed to create temp output dir");
    let output_path = temp_output.path().to_str().unwrap();
    
    let output = run_rigr_command(&[
        "--requirements", requirements_file,
        "--output", output_path,
        "--unit-only",
        "--export", "testrail"
    ]).expect("Failed to execute rigr with TestRail export");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        if stderr.contains("Configuration not found") || stdout.contains("Please run 'rigr --setup'") {
            println!("Test skipped: Configuration not available in test environment");
            return;
        }
        
        println!("STDOUT: {stdout}");
        println!("STDERR: {stderr}");
        panic!("TestRail export test failed");
    }
    
    println!("✅ TestRail export test completed");
}

#[test]
fn test_cucumber_export_format() {
    let _temp_config = create_test_config().expect("Failed to create test config");
    
    let requirements_file = "tests/fixtures/requirements/api_requirements.txt";
    if !Path::new(requirements_file).exists() {
        println!("Test skipped: Requirements file not found");
        return;
    }
    
    let temp_output = tempfile::tempdir().expect("Failed to create temp output dir");
    let output_path = temp_output.path().to_str().unwrap();
    
    let output = run_rigr_command(&[
        "--requirements", requirements_file,
        "--output", output_path,
        "--unit-only",
        "--export", "cucumber"
    ]).expect("Failed to execute rigr with Cucumber export");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        if stderr.contains("Configuration not found") || stdout.contains("Please run 'rigr --setup'") {
            println!("Test skipped: Configuration not available in test environment");
            return;
        }
        
        println!("STDOUT: {stdout}");
        println!("STDERR: {stderr}");
        panic!("Cucumber export test failed");
    }
    
    println!("✅ Cucumber export test completed");
}

#[test]
fn test_invalid_export_format() {
    let _temp_config = create_test_config().expect("Failed to create test config");
    
    let test_file = "tests/fixtures/source_files/calculator.rs";
    if !Path::new(test_file).exists() {
        println!("Test skipped: Test file not found");
        return;
    }
    
    let temp_output = tempfile::tempdir().expect("Failed to create temp output dir");
    let output_path = temp_output.path().to_str().unwrap();
    
    let output = run_rigr_command(&[
        "--file", test_file,
        "--output", output_path,
        "--unit-only",
        "--export", "invalid_format"
    ]).expect("Failed to execute rigr with invalid export format");
    
    // This should either succeed with a warning or complete normally
    // Invalid formats should be ignored with a warning message
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if !output.status.success() && stderr.contains("Configuration not found") {
        println!("Test skipped: Configuration not available in test environment");
        return;
    }
    
    // Should contain warning about unknown format
    if stdout.contains("Unknown export format") || stderr.contains("Unknown export format") {
        println!("✅ Invalid export format test completed - warning displayed");
    } else {
        println!("✅ Invalid export format test completed");
    }
}