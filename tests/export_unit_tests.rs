use rigr::export::{TestCase, TestCaseExporter, ExportFormat};
use tempfile::TempDir;
use std::fs;

fn create_sample_test_case() -> TestCase {
    TestCase {
        test_id: "TC001".to_string(),
        test_name: "Test Addition Function".to_string(),
        test_type: "Unit".to_string(),
        priority: "High".to_string(),
        description: "Verify that the addition function correctly adds two positive numbers".to_string(),
        preconditions: "Calculator is initialized".to_string(),
        test_steps: vec![
            "Initialize calculator with value 0".to_string(),
            "Call add(5)".to_string(),
            "Call add(3)".to_string(),
        ],
        expected_results: "Result should be 8".to_string(),
        postconditions: "Calculator state is updated".to_string(),
        test_data: "Input: 5, 3".to_string(),
        pass_fail_criteria: "Result equals 8".to_string(),
        category: "Arithmetic".to_string(),
        tags: vec!["basic".to_string(), "addition".to_string()],
        estimated_duration: "2 minutes".to_string(),
        automation_candidate: true,
    }
}

fn create_sample_markdown_file(temp_dir: &TempDir) -> String {
    let markdown_content = r#"# Test Case Document

## Test Case 1: Addition Test
- **ID**: TC001
- **Name**: Test Addition Function
- **Type**: Unit
- **Priority**: High
- **Description**: Verify that the addition function correctly adds two positive numbers
- **Preconditions**: Calculator is initialized
- **Steps**: 
  1. Initialize calculator with value 0
  2. Call add(5)
  3. Call add(3)
- **Expected**: Result should be 8
- **Test Data**: Input: 5, 3

## Test Case 2: Division by Zero Test
- **ID**: TC002
- **Name**: Test Division by Zero
- **Type**: Unit
- **Priority**: High
- **Description**: Verify that division by zero is handled properly
- **Preconditions**: Calculator is initialized
- **Steps**: 
  1. Initialize calculator with value 10
  2. Call divide(0)
- **Expected**: Error should be thrown
- **Test Data**: Input: 10, 0
"#;

    let file_path = temp_dir.path().join("test_cases.md");
    fs::write(&file_path, markdown_content).expect("Failed to write test markdown file");
    file_path.to_str().unwrap().to_string()
}

#[test]
fn test_export_csv_format() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let markdown_file = create_sample_markdown_file(&temp_dir);
    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    let result = TestCaseExporter::export_test_cases(
        &[markdown_file],
        output_dir.to_str().unwrap(),
        &[ExportFormat::CSV]
    );

    assert!(result.is_ok(), "CSV export should succeed");

    let csv_file = output_dir.join("rigr_test_cases.csv");
    assert!(csv_file.exists(), "CSV file should be created");

    let csv_content = fs::read_to_string(&csv_file).expect("Failed to read CSV file");
    assert!(csv_content.contains("Test ID,Test Name"), "CSV should have headers");
    assert!(csv_content.contains("TC001"), "CSV should contain test case ID");
}

#[test]
fn test_export_json_format() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let markdown_file = create_sample_markdown_file(&temp_dir);
    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    let result = TestCaseExporter::export_test_cases(
        &[markdown_file],
        output_dir.to_str().unwrap(),
        &[ExportFormat::JSON]
    );

    assert!(result.is_ok(), "JSON export should succeed");

    let json_file = output_dir.join("rigr_test_cases.json");
    assert!(json_file.exists(), "JSON file should be created");

    let json_content = fs::read_to_string(&json_file).expect("Failed to read JSON file");
    assert!(json_content.contains("TC001"), "JSON should contain test case ID");
    assert!(json_content.contains("test_cases"), "JSON should have test_cases array");
}

#[test]
fn test_export_markdown_format() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let markdown_file = create_sample_markdown_file(&temp_dir);
    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    let result = TestCaseExporter::export_test_cases(
        &[markdown_file],
        output_dir.to_str().unwrap(),
        &[ExportFormat::Markdown]
    );

    assert!(result.is_ok(), "Markdown export should succeed");

    let md_file = output_dir.join("rigr_test_cases_consolidated.md");
    assert!(md_file.exists(), "Markdown file should be created");

    let md_content = fs::read_to_string(&md_file).expect("Failed to read Markdown file");
    assert!(md_content.contains("## Test Case"), "Markdown should contain test case headers");
    assert!(md_content.contains("TC001"), "Markdown should contain test case ID");
}

#[test]
fn test_export_multiple_formats() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let markdown_file = create_sample_markdown_file(&temp_dir);
    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    let result = TestCaseExporter::export_test_cases(
        &[markdown_file],
        output_dir.to_str().unwrap(),
        &[ExportFormat::CSV, ExportFormat::JSON, ExportFormat::Markdown]
    );

    assert!(result.is_ok(), "Multiple format export should succeed");

    // Check all files were created
    assert!(output_dir.join("rigr_test_cases.csv").exists(), "CSV file should be created");
    assert!(output_dir.join("rigr_test_cases.json").exists(), "JSON file should be created");
    assert!(output_dir.join("rigr_test_cases_consolidated.md").exists(), "Markdown file should be created");
}

#[test]
fn test_export_empty_input() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    let result = TestCaseExporter::export_test_cases(
        &[], // Empty input
        output_dir.to_str().unwrap(),
        &[ExportFormat::CSV]
    );

    assert!(result.is_err(), "Empty input should result in error");
}

#[test]
fn test_export_nonexistent_file() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    let result = TestCaseExporter::export_test_cases(
        &["nonexistent_file.md".to_string()],
        output_dir.to_str().unwrap(),
        &[ExportFormat::CSV]
    );

    assert!(result.is_err(), "Nonexistent file should result in error");
}

#[test]
fn test_test_case_creation() {
    let test_case = create_sample_test_case();
    
    assert_eq!(test_case.test_id, "TC001");
    assert_eq!(test_case.test_name, "Test Addition Function");
    assert_eq!(test_case.test_type, "Unit");
    assert_eq!(test_case.priority, "High");
    assert!(test_case.automation_candidate);
    assert_eq!(test_case.tags.len(), 2);
    assert!(test_case.tags.contains(&"basic".to_string()));
    assert!(test_case.tags.contains(&"addition".to_string()));
}