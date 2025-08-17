use rigr::config::{Config, AiProvider, AiConfig, TestingConfig};
use rigr::ai_client::{TestGenerationRequest, TestGenerationResponse, TestType};
use serde_json::json;
use std::fs;
use tempfile::TempDir;

pub struct TestFixtures {
    pub temp_dir: TempDir,
    pub config: Config,
    pub source_files: Vec<String>,
}

impl Default for TestFixtures {
    fn default() -> Self {
        Self::new()
    }
}

impl TestFixtures {
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = create_test_config();
        let source_files = create_test_source_files(&temp_dir);
        
        Self {
            temp_dir,
            config,
            source_files,
        }
    }
    
    pub fn get_config_path(&self) -> String {
        let config_path = Config::config_file_path_for_testing(self.temp_dir.path()).unwrap();
        self.config.save_to_path(&config_path).unwrap();
        config_path.to_string_lossy().to_string()
    }
}

pub fn create_test_config() -> Config {
    Config {
        ai: AiConfig {
            provider: AiProvider::OpenAI,
            api_url: Some("https://api.openai.com/v1".to_string()),
            model: Some("gpt-4".to_string()),
        },
        testing: TestingConfig {
            default_framework: "cargo-test".to_string(),
            unit_test_dir: "tests/unit".to_string(),
            integration_test_dir: "tests/integration".to_string(),
            coverage_threshold: 80.0,
        },
    }
}

pub fn create_test_source_files(temp_dir: &TempDir) -> Vec<String> {
    let mut source_files = Vec::new();
    
    // Rust source file
    let rust_file = temp_dir.path().join("calculator.rs");
    let rust_content = r#"
//! A simple calculator module for demonstration

use std::fmt;

#[derive(Debug, PartialEq)]
pub enum CalculationError {
    DivisionByZero,
    Overflow,
}

impl fmt::Display for CalculationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CalculationError::DivisionByZero => write!(f, "Division by zero"),
            CalculationError::Overflow => write!(f, "Calculation overflow"),
        }
    }
}

impl std::error::Error for CalculationError {}

/// Adds two numbers together
pub fn add(a: i32, b: i32) -> Result<i32, CalculationError> {
    match a.checked_add(b) {
        Some(result) => Ok(result),
        None => Err(CalculationError::Overflow),
    }
}

/// Subtracts the second number from the first
pub fn subtract(a: i32, b: i32) -> Result<i32, CalculationError> {
    match a.checked_sub(b) {
        Some(result) => Ok(result),
        None => Err(CalculationError::Overflow),
    }
}

/// Multiplies two numbers
pub fn multiply(a: i32, b: i32) -> Result<i32, CalculationError> {
    match a.checked_mul(b) {
        Some(result) => Ok(result),
        None => Err(CalculationError::Overflow),
    }
}

/// Divides the first number by the second
pub fn divide(a: i32, b: i32) -> Result<i32, CalculationError> {
    if b == 0 {
        return Err(CalculationError::DivisionByZero);
    }
    match a.checked_div(b) {
        Some(result) => Ok(result),
        None => Err(CalculationError::Overflow),
    }
}

/// Calculates the power of a number
pub fn power(base: i32, exponent: u32) -> Result<i32, CalculationError> {
    match base.checked_pow(exponent) {
        Some(result) => Ok(result),
        None => Err(CalculationError::Overflow),
    }
}

/// A helper function that's not easily testable
fn internal_helper(x: i32, y: i32) -> i32 {
    x * y + 42
}

/// Complex business logic function
pub fn complex_calculation(inputs: &[i32]) -> Result<i32, CalculationError> {
    if inputs.is_empty() {
        return Ok(0);
    }
    
    let mut result = inputs[0];
    for &input in &inputs[1..] {
        result = add(result, input)?;
    }
    
    Ok(result)
}
"#;
    fs::write(&rust_file, rust_content).unwrap();
    source_files.push(rust_file.to_string_lossy().to_string());
    
    // Python source file
    let python_file = temp_dir.path().join("math_utils.py");
    let python_content = r#"""
Math utilities module for testing purposes.
"""

from typing import List, Optional
import math

class MathError(Exception):
    """Custom exception for math operations."""
    pass

def factorial(n: int) -> int:
    """Calculate factorial of a number."""
    if n < 0:
        raise MathError("Factorial is not defined for negative numbers")
    if n == 0 or n == 1:
        return 1
    
    result = 1
    for i in range(2, n + 1):
        result *= i
    return result

def fibonacci(n: int) -> int:
    """Calculate the nth Fibonacci number."""
    if n < 0:
        raise MathError("Fibonacci is not defined for negative numbers")
    if n == 0:
        return 0
    if n == 1:
        return 1
    
    a, b = 0, 1
    for _ in range(2, n + 1):
        a, b = b, a + b
    return b

def is_prime(n: int) -> bool:
    """Check if a number is prime."""
    if n < 2:
        return False
    if n == 2:
        return True
    if n % 2 == 0:
        return False
    
    for i in range(3, int(math.sqrt(n)) + 1, 2):
        if n % i == 0:
            return False
    return True

def greatest_common_divisor(a: int, b: int) -> int:
    """Calculate the greatest common divisor of two numbers."""
    while b:
        a, b = b, a % b
    return abs(a)

def least_common_multiple(a: int, b: int) -> int:
    """Calculate the least common multiple of two numbers."""
    if a == 0 or b == 0:
        return 0
    return abs(a * b) // greatest_common_divisor(a, b)

def sum_of_squares(numbers: List[int]) -> int:
    """Calculate the sum of squares of a list of numbers."""
    return sum(x * x for x in numbers)

def average(numbers: List[float]) -> Optional[float]:
    """Calculate the average of a list of numbers."""
    if not numbers:
        return None
    return sum(numbers) / len(numbers)

def _internal_helper(x: int) -> int:
    """Internal helper function that shouldn't be tested directly."""
    return x * 2 + 1
"#;
    fs::write(&python_file, python_content).unwrap();
    source_files.push(python_file.to_string_lossy().to_string());
    
    // JavaScript source file
    let js_file = temp_dir.path().join("array_utils.js");
    let js_content = r#"
/**
 * Array utility functions for testing
 */

class ArrayUtilsError extends Error {
    constructor(message) {
        super(message);
        this.name = 'ArrayUtilsError';
    }
}

/**
 * Find the maximum value in an array
 * @param {number[]} arr - Array of numbers
 * @returns {number} Maximum value
 */
function findMax(arr) {
    if (!arr || arr.length === 0) {
        throw new ArrayUtilsError('Array cannot be empty');
    }
    return Math.max(...arr);
}

/**
 * Find the minimum value in an array
 * @param {number[]} arr - Array of numbers
 * @returns {number} Minimum value
 */
function findMin(arr) {
    if (!arr || arr.length === 0) {
        throw new ArrayUtilsError('Array cannot be empty');
    }
    return Math.min(...arr);
}

/**
 * Calculate the sum of all elements in an array
 * @param {number[]} arr - Array of numbers
 * @returns {number} Sum of all elements
 */
function sum(arr) {
    if (!arr) {
        return 0;
    }
    return arr.reduce((total, num) => total + num, 0);
}

/**
 * Remove duplicates from an array
 * @param {any[]} arr - Array with potential duplicates
 * @returns {any[]} Array without duplicates
 */
function removeDuplicates(arr) {
    if (!arr) {
        return [];
    }
    return [...new Set(arr)];
}

/**
 * Sort array in ascending order
 * @param {number[]} arr - Array to sort
 * @returns {number[]} Sorted array
 */
function sortAscending(arr) {
    if (!arr) {
        return [];
    }
    return [...arr].sort((a, b) => a - b);
}

/**
 * Chunk array into smaller arrays of specified size
 * @param {any[]} arr - Array to chunk
 * @param {number} size - Size of each chunk
 * @returns {any[][]} Array of chunks
 */
function chunk(arr, size) {
    if (!arr || size <= 0) {
        return [];
    }
    
    const chunks = [];
    for (let i = 0; i < arr.length; i += size) {
        chunks.push(arr.slice(i, i + size));
    }
    return chunks;
}

// Internal helper function
function _validateArray(arr) {
    return Array.isArray(arr);
}

module.exports = {
    ArrayUtilsError,
    findMax,
    findMin,
    sum,
    removeDuplicates,
    sortAscending,
    chunk
};
"#;
    fs::write(&js_file, js_content).unwrap();
    source_files.push(js_file.to_string_lossy().to_string());
    
    source_files
}

pub fn create_mock_ai_response() -> TestGenerationResponse {
    TestGenerationResponse {
        unit_tests: Some(r#"
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_positive_numbers() {
        assert_eq!(add(2, 3).unwrap(), 5);
    }

    #[test]
    fn test_add_negative_numbers() {
        assert_eq!(add(-2, -3).unwrap(), -5);
    }

    #[test]
    fn test_add_overflow() {
        assert!(matches!(add(i32::MAX, 1), Err(CalculationError::Overflow)));
    }

    #[test]
    fn test_divide_by_zero() {
        assert!(matches!(divide(10, 0), Err(CalculationError::DivisionByZero)));
    }

    #[test]
    fn test_multiply_normal() {
        assert_eq!(multiply(4, 5).unwrap(), 20);
    }
}
"#.to_string()),
        integration_tests: Some(r#"
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complex_calculation_flow() {
        let inputs = vec![1, 2, 3, 4, 5];
        let result = complex_calculation(&inputs).unwrap();
        assert_eq!(result, 15);
    }

    #[test]
    fn test_calculator_error_propagation() {
        // Test that errors propagate correctly through complex operations
        let large_numbers = vec![i32::MAX, 1];
        assert!(complex_calculation(&large_numbers).is_err());
    }
}
"#.to_string()),
        coverage_estimate: 87.5,
        framework_used: "cargo-test".to_string(),
        edge_cases_covered: vec![
            "Addition overflow".to_string(),
            "Division by zero".to_string(),
            "Empty input handling".to_string(),
            "Negative number edge cases".to_string(),
            "Maximum value boundaries".to_string(),
        ],
    }
}

pub fn create_test_request() -> TestGenerationRequest {
    TestGenerationRequest {
        source_code: "pub fn add(a: i32, b: i32) -> i32 { a + b }".to_string(),
        language: "Rust".to_string(),
        test_type: TestType::Unit,
        framework: "cargo-test".to_string(),
        additional_context: Some("This is a simple addition function for testing".to_string()),
        edge_case_count: 5,
        coverage_threshold: 80.0,
    }
}

pub fn create_openai_mock_response() -> String {
    json!({
        "choices": [{
            "message": {
                "content": json!({
                    "unit_tests": "#[test] fn test_add() { assert_eq!(add(2, 3), 5); }",
                    "integration_tests": "#[test] fn test_integration() { assert!(true); }",
                    "coverage_estimate": 85.0,
                    "framework_used": "cargo-test",
                    "edge_cases_covered": ["positive numbers", "negative numbers", "zero values"]
                }).to_string()
            }
        }]
    }).to_string()
}

pub fn create_anthropic_mock_response() -> String {
    json!({
        "content": [{
            "text": json!({
                "unit_tests": "def test_add(): assert add(2, 3) == 5",
                "integration_tests": "def test_integration(): assert complex_function() is not None",
                "coverage_estimate": 90.0,
                "framework_used": "pytest",
                "edge_cases_covered": ["boundary values", "error conditions", "empty inputs"]
            }).to_string()
        }]
    }).to_string()
}

pub fn create_ollama_mock_response() -> String {
    json!({
        "response": json!({
            "unit_tests": "describe('Calculator', () => { it('should add numbers', () => { assert.equal(add(2, 3), 5); }); });",
            "integration_tests": "describe('Integration', () => { it('should work', () => { assert(true); }); });",
            "coverage_estimate": 75.0,
            "framework_used": "mocha",
            "edge_cases_covered": ["basic functionality"]
        }).to_string()
    }).to_string()
}

pub fn assert_valid_test_code(code: &str, language: &str) {
    match language.to_lowercase().as_str() {
        "rust" => {
            assert!(code.contains("#[test]") || code.contains("#[cfg(test)]"));
            assert!(code.contains("assert") || code.contains("panic"));
        }
        "python" => {
            assert!(code.contains("def test_") || code.contains("assert"));
        }
        "javascript" | "typescript" => {
            assert!(code.contains("test(") || code.contains("it(") || code.contains("describe("));
        }
        _ => {
            // For other languages, just check that it's not empty
            assert!(!code.trim().is_empty());
        }
    }
}

pub fn assert_coverage_report_validity(report_text: &str) {
    assert!(report_text.contains("Coverage Report"));
    assert!(report_text.contains("%"));
    assert!(report_text.contains("lines"));
    
    // Check for presence of coverage assessment
    let has_assessment = report_text.contains("Excellent") ||
                        report_text.contains("Good") ||
                        report_text.contains("Fair") ||
                        report_text.contains("Low");
    assert!(has_assessment);
}

// Mock HTTP server responses for testing
pub mod mock_responses {
    use mockito::Server;
    
    
    pub async fn setup_openai_mock(server: &mut Server) {
        server.mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(super::create_openai_mock_response())
            .create_async()
            .await;
    }
    
    pub async fn setup_anthropic_mock(server: &mut Server) {
        server.mock("POST", "/v1/messages")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(super::create_anthropic_mock_response())
            .create_async()
            .await;
    }
    
    pub async fn setup_ollama_mock(server: &mut Server) {
        server.mock("POST", "/api/generate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(super::create_ollama_mock_response())
            .create_async()
            .await;
    }
    
    pub async fn setup_error_mock(server: &mut Server, path: &str, status: usize) {
        server.mock("POST", path)
            .with_status(status)
            .with_body("Error occurred")
            .create_async()
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixtures_creation() {
        let fixtures = TestFixtures::new();
        assert!(!fixtures.source_files.is_empty());
        assert_eq!(fixtures.config.ai.provider, AiProvider::OpenAI);
        assert!(fixtures.temp_dir.path().exists());
    }

    #[test]
    fn test_mock_response_creation() {
        let response = create_mock_ai_response();
        assert!(response.unit_tests.is_some());
        assert!(response.integration_tests.is_some());
        assert!(response.coverage_estimate > 0.0);
        assert!(!response.edge_cases_covered.is_empty());
    }

    #[test]
    fn test_valid_test_code_assertion() {
        assert_valid_test_code("#[test] fn test() { assert!(true); }", "rust");
        assert_valid_test_code("def test_something(): assert True", "python");
        assert_valid_test_code("test('should work', () => {})", "javascript");
    }

    #[test]
    fn test_coverage_report_assertion() {
        let report = "Coverage Report\n85% (100/120 lines)\nGood coverage!";
        assert_coverage_report_validity(report);
    }
}