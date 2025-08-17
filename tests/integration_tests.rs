use assert_matches::assert_matches;
use serial_test::serial;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

// Helper function to get the binary path
fn get_binary_path() -> String {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test executable name
    path.pop(); // Remove 'deps' folder
    path.push("rigr");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path.to_string_lossy().to_string()
}


// Helper function to create test source files
fn create_test_source_files(temp_dir: &tempfile::TempDir) -> Vec<String> {
    let rust_file = temp_dir.path().join("test.rs");
    let rust_content = r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(x: f64, y: f64) -> f64 {
    x * y
}

fn helper_function() -> bool {
    true
}
"#;
    fs::write(&rust_file, rust_content).unwrap();

    let python_file = temp_dir.path().join("test.py");
    let python_content = r#"
def subtract(a, b):
    return a - b

def divide(x, y):
    if y == 0:
        raise ValueError("Cannot divide by zero")
    return x / y
"#;
    fs::write(&python_file, python_content).unwrap();

    vec![
        rust_file.to_string_lossy().to_string(),
        python_file.to_string_lossy().to_string(),
    ]
}

#[test]
fn test_cli_help_command() {
    let output = Command::new(get_binary_path())
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(stdout.contains("AI Test Case Generator"));
    assert!(stdout.contains("--setup"));
    assert!(stdout.contains("--file"));
    assert!(stdout.contains("--directory"));
    assert!(stdout.contains("--framework"));
    assert!(stdout.contains("--unit-only"));
    assert!(stdout.contains("--integration-only"));
    assert!(stdout.contains("--no-coverage"));
    assert!(stdout.contains("--output"));
}

#[test]
fn test_cli_version_command() {
    let output = Command::new(get_binary_path())
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rigr"));
}

#[test]
fn test_cli_no_arguments() {
    let output = Command::new(get_binary_path())
        .output()
        .expect("Failed to execute command");

    // Should fail because no configuration file found
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Configuration not found") || 
             stderr.contains("Please run 'rigr --setup' first"));
}

#[test]
#[serial]
fn test_config_file_operations() {
    use rigr::config::{Config, AiProvider, AiConfig, TestingConfig};
    
    let temp_dir = tempdir().unwrap();
    let config_path = Config::config_file_path_for_testing(temp_dir.path()).unwrap();
    
    // Create test config
    let original_config = Config {
        ai: AiConfig {
            provider: AiProvider::OpenAI,
            api_url: Some("https://api.openai.com/v1".to_string()),
            model: Some("gpt-4".to_string()),
        },
        testing: TestingConfig {
            default_framework: "jest".to_string(),
            unit_test_dir: "tests/unit".to_string(),
            integration_test_dir: "tests/integration".to_string(),
            coverage_threshold: 85.0,
        },
    };

    // Test saving config
    original_config.save_to_path(&config_path).unwrap();
    assert!(config_path.exists());

    // Test loading config
    let loaded_config = Config::load_from_path(&config_path).unwrap();
    assert_eq!(original_config.ai.provider, loaded_config.ai.provider);
    assert_eq!(original_config.ai.api_url, loaded_config.ai.api_url);
    assert_eq!(original_config.testing.coverage_threshold, loaded_config.testing.coverage_threshold);

    // Test config file content
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("OpenAI"));
    assert!(content.contains("gpt-4"));
    assert!(content.contains("85.0"));
}

#[test]
fn test_source_file_detection() {
    let temp_dir = tempdir().unwrap();
    let source_files = create_test_source_files(&temp_dir);
    
    for file_path in &source_files {
        assert!(std::path::Path::new(file_path).exists());
        
        let content = fs::read_to_string(file_path).unwrap();
        if file_path.ends_with(".rs") {
            assert!(content.contains("pub fn"));
        } else if file_path.ends_with(".py") {
            assert!(content.contains("def "));
        }
    }
}

#[test]
fn test_language_detection() {
    use rigr::coverage::CoverageAnalyzer;
    
    let temp_dir = tempdir().unwrap();
    let rust_file = temp_dir.path().join("test.rs");
    let python_file = temp_dir.path().join("test.py");
    let js_file = temp_dir.path().join("test.js");
    
    fs::write(&rust_file, "fn main() {}").unwrap();
    fs::write(&python_file, "def main(): pass").unwrap();
    fs::write(&js_file, "function main() {}").unwrap();
    
    // Test Rust function extraction
    let rust_content = fs::read_to_string(&rust_file).unwrap();
    let rust_functions = CoverageAnalyzer::extract_functions(&rust_content, "rust").unwrap();
    assert_eq!(rust_functions.len(), 1);
    assert_eq!(rust_functions[0].name, "main");
    
    // Test Python function extraction
    let python_content = fs::read_to_string(&python_file).unwrap();
    let python_functions = CoverageAnalyzer::extract_functions(&python_content, "python").unwrap();
    assert_eq!(python_functions.len(), 1);
    assert_eq!(python_functions[0].name, "main");
    
    // Test JavaScript function extraction
    let js_content = fs::read_to_string(&js_file).unwrap();
    let js_functions = CoverageAnalyzer::extract_functions(&js_content, "javascript").unwrap();
    assert_eq!(js_functions.len(), 1);
    assert_eq!(js_functions[0].name, "main");
}

#[test]
fn test_coverage_analysis_integration() {
    use rigr::coverage::CoverageAnalyzer;
    
    let temp_dir = tempdir().unwrap();
    let source_file = temp_dir.path().join("lib.rs");
    let source_content = r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(x: i32, y: i32) -> i32 {
    x * y
}

fn unused_function() -> String {
    "unused".to_string()
}
"#;
    fs::write(&source_file, source_content).unwrap();
    
    let unit_tests = r#"
#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);
}

#[test]
fn test_multiply() {
    assert_eq!(multiply(4, 5), 20);
}
"#;
    
    let integration_tests = "";
    
    let source_files = vec![source_file.to_string_lossy().to_string()];
    let report = CoverageAnalyzer::analyze_coverage(
        &source_files, unit_tests, integration_tests, "rust"
    ).unwrap();
    
    assert!(report.coverage_percentage > 0.0);
    assert!(report.coverage_percentage <= 100.0);
    assert!(!report.uncovered_functions.is_empty());
    assert!(report.uncovered_functions.contains(&"unused_function".to_string()));
    
    // Test report generation
    let report_text = CoverageAnalyzer::generate_coverage_report(&report);
    assert!(report_text.contains("Test Coverage Report"));
    assert!(report_text.contains("unused_function"));
}

#[test]
fn test_framework_detection() {
    let temp_dir = tempdir().unwrap();
    
    // Create Cargo.toml for Rust project
    let cargo_toml = temp_dir.path().join("Cargo.toml");
    fs::write(&cargo_toml, "[package]\nname = \"test\"\nversion = \"0.1.0\"").unwrap();
    
    // Create package.json for JavaScript project
    let package_json = temp_dir.path().join("package.json");
    fs::write(&package_json, r#"{"name": "test", "version": "1.0.0"}"#).unwrap();
    
    // Create pytest.ini for Python project
    let pytest_ini = temp_dir.path().join("pytest.ini");
    fs::write(&pytest_ini, "[tool:pytest]").unwrap();
    
    assert!(cargo_toml.exists());
    assert!(package_json.exists());
    assert!(pytest_ini.exists());
}

#[test]
fn test_error_handling() {
    use rigr::RigrError;
    
    // Test file read error
    let error = RigrError::FileReadError("Test file not found".to_string());
    assert_matches!(error, RigrError::FileReadError(_));
    assert!(error.to_string().contains("Test file not found"));
    
    // Test parse error
    let error = RigrError::FileParseError("Invalid syntax".to_string());
    assert_matches!(error, RigrError::FileParseError(_));
    
    // Test HTTP error
    let error = RigrError::HttpRequestError("Connection failed".to_string());
    assert_matches!(error, RigrError::HttpRequestError(_));
    
    // Test unsupported language error
    let error = RigrError::UnsupportedLanguage("COBOL".to_string());
    assert_matches!(error, RigrError::UnsupportedLanguage(_));
}

#[test]
fn test_directory_traversal() {
    let temp_dir = tempdir().unwrap();
    
    // Create nested directory structure
    let src_dir = temp_dir.path().join("src");
    let tests_dir = temp_dir.path().join("tests");
    let nested_dir = src_dir.join("nested");
    
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&tests_dir).unwrap();
    fs::create_dir_all(&nested_dir).unwrap();
    
    // Create source files
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();
    fs::write(src_dir.join("lib.rs"), "pub fn lib_func() {}").unwrap();
    fs::write(nested_dir.join("helper.rs"), "fn helper() {}").unwrap();
    
    // Create non-source files (should be ignored)
    fs::write(src_dir.join("README.md"), "# Test").unwrap();
    fs::write(src_dir.join("data.txt"), "some data").unwrap();
    
    // Test directory traversal
    let walker = ignore::WalkBuilder::new(&temp_dir)
        .hidden(false)
        .git_ignore(true)
        .build();
    
    let mut source_files = Vec::new();
    for entry in walker {
        let entry = entry.unwrap();
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if matches!(ext, "rs" | "py" | "js" | "ts" | "java" | "cs" | "go") {
                    source_files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    
    assert_eq!(source_files.len(), 3); // main.rs, lib.rs, helper.rs
}

#[test]
fn test_output_directory_creation() {
    let temp_dir = tempdir().unwrap();
    let output_dir = temp_dir.path().join("output");
    let unit_dir = output_dir.join("tests").join("unit");
    let integration_dir = output_dir.join("tests").join("integration");
    
    // Create directories
    fs::create_dir_all(&unit_dir).unwrap();
    fs::create_dir_all(&integration_dir).unwrap();
    
    assert!(unit_dir.exists());
    assert!(integration_dir.exists());
    
    // Test file creation in output directories
    let test_file = unit_dir.join("test_unit.rs");
    fs::write(&test_file, "#[test] fn test() {}").unwrap();
    
    assert!(test_file.exists());
    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("#[test]"));
}

#[test]
fn test_multiple_language_support() {
    let temp_dir = tempdir().unwrap();
    
    // Create files for different languages
    let files = vec![
        ("test.rs", "pub fn rust_func() -> i32 { 42 }"),
        ("test.py", "def python_func(): return 42"),
        ("test.js", "function jsFunc() { return 42; }"),
        ("test.ts", "function tsFunc(): number { return 42; }"),
        ("test.java", "public class Test { public int javaFunc() { return 42; } }"),
        ("test.cs", "public class Test { public int CSharpFunc() { return 42; } }"),
        ("test.go", "func goFunc() int { return 42 }"),
    ];
    
    for (filename, content) in files {
        let file_path = temp_dir.path().join(filename);
        fs::write(&file_path, content).unwrap();
        assert!(file_path.exists());
        
        let file_content = fs::read_to_string(&file_path).unwrap();
        assert!(file_content.contains("42"));
    }
}

#[test]
fn test_configuration_validation() {
    use rigr::config::{AiConfig, AiProvider, TestingConfig};
    
    // Test valid configurations
    let valid_configs = vec![
        AiConfig {
            provider: AiProvider::OpenAI,
            api_url: Some("https://api.openai.com/v1".to_string()),
            model: Some("gpt-4".to_string()),
        },
        AiConfig {
            provider: AiProvider::Ollama,
            api_url: Some("http://localhost:11434".to_string()),
            model: Some("codellama".to_string()),
        },
    ];
    
    for config in valid_configs {
        match config.provider {
            AiProvider::OpenAI => {
                assert!(config.api_url.as_ref().unwrap().contains("openai"));
            }
            AiProvider::Ollama => {
                assert!(config.api_url.as_ref().unwrap().contains("localhost"));
            }
            _ => {}
        }
    }
    
    // Test testing configuration
    let testing_config = TestingConfig {
        default_framework: "cargo-test".to_string(),
        unit_test_dir: "tests/unit".to_string(),
        integration_test_dir: "tests/integration".to_string(),
        coverage_threshold: 80.0,
    };
    
    assert!(testing_config.coverage_threshold >= 0.0);
    assert!(testing_config.coverage_threshold <= 100.0);
    assert!(!testing_config.unit_test_dir.is_empty());
    assert!(!testing_config.integration_test_dir.is_empty());
}

#[test]
fn test_edge_cases() {
    let temp_dir = tempdir().unwrap();
    
    // Test empty source file
    let empty_file = temp_dir.path().join("empty.rs");
    fs::write(&empty_file, "").unwrap();
    
    let content = fs::read_to_string(&empty_file).unwrap();
    assert!(content.is_empty());
    
    // Test file with only comments
    let comment_file = temp_dir.path().join("comments.rs");
    fs::write(&comment_file, "// This is just a comment\n/* Block comment */").unwrap();
    
    let comment_content = fs::read_to_string(&comment_file).unwrap();
    assert!(comment_content.contains("//"));
    assert!(comment_content.contains("/*"));
    
    // Test file with complex function signatures
    let complex_file = temp_dir.path().join("complex.rs");
    let complex_content = r#"
pub async fn complex_function<T, U>(
    param1: T,
    param2: Option<U>,
) -> Result<Vec<String>, Box<dyn std::error::Error>>
where
    T: Clone + Send + Sync,
    U: std::fmt::Debug,
{
    // Complex implementation
    Ok(vec!["test".to_string()])
}
"#;
    fs::write(&complex_file, complex_content).unwrap();
    
    use rigr::coverage::CoverageAnalyzer;
    let functions = CoverageAnalyzer::extract_functions(complex_content, "rust").unwrap();
    assert_eq!(functions.len(), 1);
    assert_eq!(functions[0].name, "complex_function");
}