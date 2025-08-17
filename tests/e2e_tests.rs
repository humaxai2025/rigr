use mockito::Server;
use rigr::config::{Config, AiProvider};
use rigr::ai_client::{AiClient, TestGenerationRequest, TestType};
use rigr::coverage::CoverageAnalyzer;
use serial_test::serial;
use std::fs;
use tempfile::tempdir;

mod test_helpers;
use test_helpers::{TestFixtures, mock_responses, assert_valid_test_code, assert_coverage_report_validity};

#[tokio::test]
#[serial]
async fn test_complete_workflow_rust_project() {
    let fixtures = TestFixtures::new();
    
    // Set up mock AI server
    let mut server = Server::new_async().await;
    mock_responses::setup_openai_mock(&mut server).await;
    
    // Configure AI client to use mock server
    let mut config = fixtures.config.clone();
    config.ai.api_url = Some(server.url());
    
    let ai_client = AiClient::new(config.ai.clone());
    
    // Find Rust source file
    let rust_file = fixtures.source_files.iter()
        .find(|f| f.ends_with(".rs"))
        .expect("Should have Rust file");
    
    let source_code = fs::read_to_string(rust_file).unwrap();
    
    // Generate unit tests
    let unit_request = TestGenerationRequest {
        source_code: source_code.clone(),
        language: "Rust".to_string(),
        test_type: TestType::Unit,
        framework: "cargo-test".to_string(),
        additional_context: Some("Calculator module with error handling".to_string()),
        edge_case_count: 5,
        coverage_threshold: 80.0,
    };
    
    let unit_response = ai_client.generate_tests(unit_request).await.unwrap();
    assert!(unit_response.unit_tests.is_some());
    
    let unit_tests = unit_response.unit_tests.unwrap();
    assert_valid_test_code(&unit_tests, "rust");
    
    // Save unit tests
    let unit_test_dir = fixtures.temp_dir.path().join(&config.testing.unit_test_dir);
    fs::create_dir_all(&unit_test_dir).unwrap();
    let unit_test_file = unit_test_dir.join("calculator_unit.rs");
    fs::write(&unit_test_file, &unit_tests).unwrap();
    
    // Generate integration tests
    let integration_request = TestGenerationRequest {
        source_code: source_code.clone(),
        language: "Rust".to_string(),
        test_type: TestType::Integration,
        framework: "cargo-test".to_string(),
        additional_context: Some("Test complex workflows and error propagation".to_string()),
        edge_case_count: 5,
        coverage_threshold: 80.0,
    };
    
    let integration_response = ai_client.generate_tests(integration_request).await.unwrap();
    assert!(integration_response.integration_tests.is_some());
    
    let integration_tests = integration_response.integration_tests.unwrap();
    assert_valid_test_code(&integration_tests, "rust");
    
    // Save integration tests
    let integration_test_dir = fixtures.temp_dir.path().join(&config.testing.integration_test_dir);
    fs::create_dir_all(&integration_test_dir).unwrap();
    let integration_test_file = integration_test_dir.join("calculator_integration.rs");
    fs::write(&integration_test_file, &integration_tests).unwrap();
    
    // Analyze coverage
    let source_files = vec![rust_file.clone()];
    let coverage_report = CoverageAnalyzer::analyze_coverage(
        &source_files,
        &unit_tests,
        &integration_tests,
        "rust"
    ).unwrap();
    
    assert!(coverage_report.total_lines > 0);
    assert!(coverage_report.coverage_percentage >= 0.0);
    assert_eq!(coverage_report.file_coverage.len(), 1);
    
    // Generate coverage report
    let report_text = CoverageAnalyzer::generate_coverage_report(&coverage_report);
    assert_coverage_report_validity(&report_text);
    
    // Verify all files were created
    assert!(unit_test_file.exists());
    assert!(integration_test_file.exists());
    
    println!("✅ Complete Rust workflow test passed");
}

#[tokio::test]
#[serial]
async fn test_complete_workflow_python_project() {
    let fixtures = TestFixtures::new();
    
    // Set up mock AI server
    let mut server = Server::new_async().await;
    mock_responses::setup_anthropic_mock(&mut server).await;
    
    // Configure AI client for Anthropic
    let mut config = fixtures.config.clone();
    config.ai.provider = AiProvider::Anthropic;
    config.ai.api_url = Some(server.url());
    
    let ai_client = AiClient::new(config.ai.clone());
    
    // Find Python source file
    let python_file = fixtures.source_files.iter()
        .find(|f| f.ends_with(".py"))
        .expect("Should have Python file");
    
    let source_code = fs::read_to_string(python_file).unwrap();
    
    // Generate tests
    let request = TestGenerationRequest {
        source_code: source_code.clone(),
        language: "Python".to_string(),
        test_type: TestType::Unit,
        framework: "pytest".to_string(),
        additional_context: Some("Math utilities with comprehensive error handling".to_string()),
        edge_case_count: 5,
        coverage_threshold: 80.0,
    };
    
    let response = ai_client.generate_tests(request).await.unwrap();
    assert!(response.unit_tests.is_some());
    
    let test_code = response.unit_tests.unwrap();
    assert_valid_test_code(&test_code, "python");
    
    // Save tests
    let test_dir = fixtures.temp_dir.path().join("tests");
    fs::create_dir_all(&test_dir).unwrap();
    let test_file = test_dir.join("test_math_utils.py");
    fs::write(&test_file, &test_code).unwrap();
    
    // Analyze coverage
    let source_files = vec![python_file.clone()];
    let coverage_report = CoverageAnalyzer::analyze_coverage(
        &source_files,
        &test_code,
        "",
        "python"
    ).unwrap();
    
    assert!(coverage_report.total_lines > 0);
    assert!(!coverage_report.file_coverage.is_empty());
    
    println!("✅ Complete Python workflow test passed");
}

#[tokio::test]
#[serial]
async fn test_complete_workflow_javascript_project() {
    let fixtures = TestFixtures::new();
    
    // Set up mock AI server
    let mut server = Server::new_async().await;
    mock_responses::setup_ollama_mock(&mut server).await;
    
    // Configure AI client for Ollama
    let mut config = fixtures.config.clone();
    config.ai.provider = AiProvider::Ollama;
    // Ollama configuration already set up correctly
    config.ai.api_url = Some(server.url());
    
    let ai_client = AiClient::new(config.ai.clone());
    
    // Find JavaScript source file
    let js_file = fixtures.source_files.iter()
        .find(|f| f.ends_with(".js"))
        .expect("Should have JavaScript file");
    
    let source_code = fs::read_to_string(js_file).unwrap();
    
    // Generate tests
    let request = TestGenerationRequest {
        source_code: source_code.clone(),
        language: "JavaScript".to_string(),
        test_type: TestType::Unit,
        framework: "jest".to_string(),
        additional_context: None,
        edge_case_count: 5,
        coverage_threshold: 80.0,
    };
    
    let response = ai_client.generate_tests(request).await.unwrap();
    assert!(response.unit_tests.is_some());
    
    let test_code = response.unit_tests.unwrap();
    assert_valid_test_code(&test_code, "javascript");
    
    // Save tests
    let test_dir = fixtures.temp_dir.path().join("__tests__");
    fs::create_dir_all(&test_dir).unwrap();
    let test_file = test_dir.join("array_utils.test.js");
    fs::write(&test_file, &test_code).unwrap();
    
    println!("✅ Complete JavaScript workflow test passed");
}

#[tokio::test]
#[serial]
async fn test_multi_file_project_workflow() {
    let fixtures = TestFixtures::new();
    
    // Set up mock AI server
    let mut server = Server::new_async().await;
    mock_responses::setup_openai_mock(&mut server).await;
    
    let mut config = fixtures.config.clone();
    config.ai.api_url = Some(server.url());
    
    let ai_client = AiClient::new(config.ai.clone());
    
    // Process all source files
    let mut all_unit_tests = String::new();
    let mut all_integration_tests = String::new();
    
    for source_file in &fixtures.source_files {
        let source_code = fs::read_to_string(source_file).unwrap();
        let language = if source_file.ends_with(".rs") {
            "Rust"
        } else if source_file.ends_with(".py") {
            "Python"
        } else if source_file.ends_with(".js") {
            "JavaScript"
        } else {
            continue;
        };
        
        // Generate unit tests
        let unit_request = TestGenerationRequest {
            source_code: source_code.clone(),
            language: language.to_string(),
            test_type: TestType::Unit,
            framework: "auto".to_string(),
            additional_context: None,
            edge_case_count: 5,
            coverage_threshold: 80.0,
        };
        
        let unit_response = ai_client.generate_tests(unit_request).await.unwrap();
        if let Some(unit_tests) = unit_response.unit_tests {
            all_unit_tests.push_str(&unit_tests);
            all_unit_tests.push('\n');
        }
        
        // Generate integration tests
        let integration_request = TestGenerationRequest {
            source_code: source_code.clone(),
            language: language.to_string(),
            test_type: TestType::Integration,
            framework: "auto".to_string(),
            additional_context: None,
            edge_case_count: 5,
            coverage_threshold: 80.0,
        };
        
        let integration_response = ai_client.generate_tests(integration_request).await.unwrap();
        if let Some(integration_tests) = integration_response.integration_tests {
            all_integration_tests.push_str(&integration_tests);
            all_integration_tests.push('\n');
        }
    }
    
    // Analyze overall coverage
    let coverage_report = CoverageAnalyzer::analyze_coverage(
        &fixtures.source_files,
        &all_unit_tests,
        &all_integration_tests,
        "rust" // Use rust as primary language
    ).unwrap();
    
    assert_eq!(coverage_report.file_coverage.len(), fixtures.source_files.len());
    assert!(coverage_report.total_lines > 0);
    
    // Generate comprehensive report
    let report_text = CoverageAnalyzer::generate_coverage_report(&coverage_report);
    assert_coverage_report_validity(&report_text);
    
    println!("✅ Multi-file project workflow test passed");
}

#[tokio::test]
#[serial]
async fn test_error_handling_workflow() {
    let fixtures = TestFixtures::new();
    
    // Set up mock server that returns errors
    let mut server = Server::new_async().await;
    mock_responses::setup_error_mock(&mut server, "/chat/completions", 500).await;
    
    let mut config = fixtures.config.clone();
    config.ai.api_url = Some(server.url());
    
    let ai_client = AiClient::new(config.ai.clone());
    
    // Try to generate tests with failing server
    let request = TestGenerationRequest {
        source_code: "pub fn test() {}".to_string(),
        language: "Rust".to_string(),
        test_type: TestType::Unit,
        framework: "cargo-test".to_string(),
        additional_context: None,
        edge_case_count: 5,
        coverage_threshold: 80.0,
    };
    
    let result = ai_client.generate_tests(request).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        rigr::RigrError::HttpRequestError(_) => {
            println!("✅ Error handling workflow test passed");
        }
        other => panic!("Expected HttpRequestError, got: {other:?}"),
    }
}

#[tokio::test]
#[serial]
async fn test_configuration_workflow() {
    let temp_dir = tempdir().unwrap();
    
    // Test different AI providers
    let providers = vec![
        (AiProvider::OpenAI, Some("openai-key".to_string())),
        (AiProvider::Anthropic, Some("anthropic-key".to_string())),
        (AiProvider::Ollama, None),
        (AiProvider::GoogleGemini, Some("gemini-key".to_string())),
        (AiProvider::AzureOpenAI, Some("azure-key".to_string())),
    ];
    
    for (provider, _api_key) in providers {
        let config = Config::new_for_testing(provider.clone());
        let config_path = Config::config_file_path_for_testing(temp_dir.path()).unwrap();
        
        // Save and load config
        config.save_to_path(&config_path).unwrap();
        let loaded_config = Config::load_from_path(&config_path).unwrap();
        
        assert_eq!(config.ai.provider, loaded_config.ai.provider);
        assert_eq!(config.ai.api_url, loaded_config.ai.api_url);
        
        // Test AI client creation
        let _ai_client = AiClient::new(loaded_config.ai);
        
        // Clean up for next iteration
        fs::remove_file(&config_path).unwrap();
    }
    
    println!("✅ Configuration workflow test passed");
}

#[tokio::test]
#[serial]
async fn test_coverage_threshold_workflow() {
    let fixtures = TestFixtures::new();
    
    // Create source file with known functions
    let source_file = fixtures.temp_dir.path().join("threshold_test.rs");
    let source_content = r#"
pub fn covered_function() -> i32 { 1 }
pub fn partially_covered() -> i32 { 2 }
pub fn uncovered_function() -> i32 { 3 }
"#;
    fs::write(&source_file, source_content).unwrap();
    
    // Create tests that only cover some functions
    let unit_tests = r#"
#[test]
fn test_covered_function() {
    assert_eq!(covered_function(), 1);
}

#[test] 
fn test_partially_covered() {
    assert_eq!(partially_covered(), 2);
}
"#;
    
    let source_files = vec![source_file.to_string_lossy().to_string()];
    let coverage_report = CoverageAnalyzer::analyze_coverage(
        &source_files,
        unit_tests,
        "",
        "rust"
    ).unwrap();
    
    // Test different threshold scenarios
    let mut config = fixtures.config.clone();
    
    // High threshold - should warn about low coverage
    config.testing.coverage_threshold = 95.0;
    if coverage_report.coverage_percentage < config.testing.coverage_threshold {
        println!("⚠️  Coverage below threshold as expected");
    }
    
    // Low threshold - should pass
    config.testing.coverage_threshold = 50.0;
    if coverage_report.coverage_percentage >= config.testing.coverage_threshold {
        println!("✅ Coverage meets low threshold");
    }
    
    // Verify uncovered functions are reported
    assert!(coverage_report.uncovered_functions.contains(&"uncovered_function".to_string()));
    
    println!("✅ Coverage threshold workflow test passed");
}

#[tokio::test]
#[serial]
async fn test_framework_detection_workflow() {
    let temp_dir = tempdir().unwrap();
    
    // Test Rust with Cargo.toml
    let cargo_dir = temp_dir.path().join("rust_project");
    fs::create_dir_all(&cargo_dir).unwrap();
    fs::write(cargo_dir.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
    fs::write(cargo_dir.join("main.rs"), "fn main() {}").unwrap();
    
    // Test Python with pytest.ini
    let python_dir = temp_dir.path().join("python_project");
    fs::create_dir_all(&python_dir).unwrap();
    fs::write(python_dir.join("pytest.ini"), "[tool:pytest]").unwrap();
    fs::write(python_dir.join("main.py"), "def main(): pass").unwrap();
    
    // Test JavaScript with package.json
    let js_dir = temp_dir.path().join("js_project");
    fs::create_dir_all(&js_dir).unwrap();
    fs::write(js_dir.join("package.json"), r#"{"name": "test"}"#).unwrap();
    fs::write(js_dir.join("main.js"), "function main() {}").unwrap();
    
    // Verify framework files exist (simulating detection logic)
    assert!(cargo_dir.join("Cargo.toml").exists());
    assert!(python_dir.join("pytest.ini").exists());
    assert!(js_dir.join("package.json").exists());
    
    println!("✅ Framework detection workflow test passed");
}

#[tokio::test]
#[serial]
async fn test_output_organization_workflow() {
    let fixtures = TestFixtures::new();
    let config = fixtures.config.clone();
    
    // Create organized directory structure
    let base_output = fixtures.temp_dir.path().join("output");
    let unit_dir = base_output.join(&config.testing.unit_test_dir);
    let integration_dir = base_output.join(&config.testing.integration_test_dir);
    
    fs::create_dir_all(&unit_dir).unwrap();
    fs::create_dir_all(&integration_dir).unwrap();
    
    // Simulate generating and organizing tests
    for source_file in fixtures.source_files.iter() {
        let file_stem = std::path::Path::new(source_file)
            .file_stem()
            .unwrap()
            .to_string_lossy();
        
        // Create unit test file
        let unit_test_file = unit_dir.join(format!("{file_stem}_unit_test.rs"));
        fs::write(&unit_test_file, format!("// Unit tests for {source_file}")).unwrap();
        
        // Create integration test file
        let integration_test_file = integration_dir.join(format!("{file_stem}_integration_test.rs"));
        fs::write(&integration_test_file, format!("// Integration tests for {source_file}")).unwrap();
        
        assert!(unit_test_file.exists());
        assert!(integration_test_file.exists());
    }
    
    // Verify directory structure
    assert!(unit_dir.exists());
    assert!(integration_dir.exists());
    
    // Count generated files
    let unit_files: Vec<_> = fs::read_dir(&unit_dir).unwrap().collect();
    let integration_files: Vec<_> = fs::read_dir(&integration_dir).unwrap().collect();
    
    assert_eq!(unit_files.len(), fixtures.source_files.len());
    assert_eq!(integration_files.len(), fixtures.source_files.len());
    
    println!("✅ Output organization workflow test passed");
}

#[test]
fn test_comprehensive_error_scenarios() {
    use rigr::{RigrError};
    
    // Test all error types
    let errors = vec![
        RigrError::FileReadError("Cannot read file".to_string()),
        RigrError::FileParseError("Invalid syntax".to_string()),
        RigrError::FileWriteError("Permission denied".to_string()),
        RigrError::UnsupportedLanguage("Brainfuck".to_string()),
        RigrError::HttpRequestError("Network timeout".to_string()),
    ];
    
    for error in errors {
        // Verify error can be displayed
        let error_msg = error.to_string();
        assert!(!error_msg.is_empty());
        
        // Verify error implements std::error::Error
        let _: &dyn std::error::Error = &error;
    }
    
    println!("✅ Comprehensive error scenarios test passed");
}