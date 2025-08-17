use crate::config::{AiConfig, AiProvider};
use crate::RigrError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use regex::Regex;
use tracing::{warn, debug};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TestGenerationRequest {
    pub source_code: String,
    pub language: String,
    pub test_type: TestType,
    pub framework: String,
    pub additional_context: Option<String>,
    pub edge_case_count: usize,
    pub coverage_threshold: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TestType {
    Unit,
    Integration,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TestGenerationResponse {
    pub unit_tests: Option<String>,
    pub integration_tests: Option<String>,
    pub coverage_estimate: f64,
    pub framework_used: String,
    pub edge_cases_covered: Vec<String>,
}

pub struct AiClient {
    config: AiConfig,
    client: Client,
}

impl AiClient {
    pub fn new(config: AiConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
    
    /// Sanitize source code before sending to AI services
    pub fn sanitize_source_for_ai(&self, source_code: &str) -> Result<String, RigrError> {
        let mut sanitized = source_code.to_string();
        
        // Remove or redact sensitive patterns
        let sensitive_patterns = [
            (r#"(?i)api_key\s*[:=]\s*['"][^'"]+['"]"#, "api_key = \"[REDACTED]\""),
            (r#"(?i)password\s*[:=]\s*['"][^'"]+['"]"#, "password = \"[REDACTED]\""),
            (r#"(?i)secret\s*[:=]\s*['"][^'"]+['"]"#, "secret = \"[REDACTED]\""),
            (r#"(?i)token\s*[:=]\s*['"][^'"]+['"]"#, "token = \"[REDACTED]\""),
            (r"\b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b", "[CREDIT-CARD-REDACTED]"),
            (r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b", "[EMAIL-REDACTED]"),
            (r"\b(?:\d{1,3}\.){3}\d{1,3}\b", "[IP-ADDRESS-REDACTED]"),
        ];
        
        for (pattern, replacement) in &sensitive_patterns {
            let regex = Regex::new(pattern).map_err(|_| RigrError::SecurityError("Regex compilation failed".to_string()))?;
            sanitized = regex.replace_all(&sanitized, *replacement).to_string();
        }
        
        // Log redaction if any changes were made
        if sanitized != source_code {
            warn!("Sensitive data patterns detected and redacted before sending to AI service");
            debug!("Redacted {} characters of potentially sensitive content", source_code.len() - sanitized.len());
        }
        
        // Truncate if too long to prevent prompt injection
        const MAX_SOURCE_LENGTH: usize = 100_000; // 100KB limit for AI processing
        if sanitized.len() > MAX_SOURCE_LENGTH {
            sanitized.truncate(MAX_SOURCE_LENGTH);
            sanitized.push_str("\n\n[... truncated for security ...]");
            warn!("Source code truncated to {} characters for security", MAX_SOURCE_LENGTH);
        }
        
        Ok(sanitized)
    }
    
    /// Get language-specific testing guidance for comprehensive test case generation
    fn get_language_specific_testing_guidance(&self, language: &str) -> &'static str {
        get_language_specific_testing_guidance(language)
    }

    pub async fn generate_tests(&self, request: TestGenerationRequest) -> Result<TestGenerationResponse, RigrError> {
        let prompt = self.build_comprehensive_prompt(&request);
        
        match self.config.provider {
            AiProvider::OpenAI => self.call_openai(&prompt, &request).await,
            AiProvider::Anthropic => self.call_anthropic(&prompt, &request).await,
            AiProvider::Ollama => self.call_ollama(&prompt, &request).await,
            AiProvider::GoogleGemini => self.call_gemini(&prompt, &request).await,
            AiProvider::AzureOpenAI => self.call_azure_openai(&prompt, &request).await,
        }
    }

    pub async fn generate_test_cases(&self, request: TestGenerationRequest) -> Result<String, RigrError> {
        let prompt = self.build_test_cases_prompt(&request)?;
        
        match self.config.provider {
            AiProvider::OpenAI => self.call_openai_text(&prompt).await,
            AiProvider::Anthropic => self.call_anthropic_text(&prompt).await,
            AiProvider::Ollama => self.call_ollama_text(&prompt).await,
            AiProvider::GoogleGemini => self.call_gemini_text(&prompt).await,
            AiProvider::AzureOpenAI => self.call_azure_openai_text(&prompt).await,
        }
    }

    pub async fn generate_comprehensive_test_cases(&self, request: TestGenerationRequest) -> Result<String, RigrError> {
        let prompt = self.build_comprehensive_test_cases_prompt(&request)?;
        
        match self.config.provider {
            AiProvider::OpenAI => self.call_openai_text(&prompt).await,
            AiProvider::Anthropic => self.call_anthropic_text(&prompt).await,
            AiProvider::Ollama => self.call_ollama_text(&prompt).await,
            AiProvider::GoogleGemini => self.call_gemini_text(&prompt).await,
            AiProvider::AzureOpenAI => self.call_azure_openai_text(&prompt).await,
        }
    }

    fn build_comprehensive_prompt(&self, request: &TestGenerationRequest) -> String {
        let test_type_instruction = match request.test_type {
            TestType::Unit => "Generate unit tests that test individual functions and methods",
            TestType::Integration => "Generate integration tests that test component interactions",
        };

        let framework_instructions = match request.framework.as_str() {
            "cargo-test" | "rust-test" => {
                "Write Rust tests using #[test] attributes and assert macros. Use proper Rust syntax."
            },
            "pytest" => "Write Python tests using pytest framework.",
            "jest" => "Write JavaScript tests using Jest framework.",
            "junit" => "Write Java tests using JUnit framework.",
            // Legacy language testing frameworks
            "cobol-test" => "Write COBOL test programs using standard COBOL testing practices with test data and expected results.",
            "fortran-test" => "Write Fortran test programs using subroutines and test drivers with appropriate assertions.",
            "pascal-test" => "Write Pascal/Delphi test procedures using standard unit testing practices.",
            "plsql-test" => "Write PL/SQL test blocks using DBMS_OUTPUT for assertions and test result reporting.",
            "rpg-test" => "Write RPG/RPGLE test programs using standard testing procedures and indicators.",
            "ada-test" => "Write Ada test procedures using Ada.Text_IO for output and standard assertion practices.",
            _ => "Write tests using appropriate conventions for the language.",
        };
        
        // Add legacy language specific instructions
        let legacy_instructions = match request.language.to_lowercase().as_str() {
            "cobol" => "\n\nCOBOL TESTING NOTES:\n- Use WORKING-STORAGE for test data\n- Test with various PIC clause combinations\n- Include file handling test scenarios\n- Test paragraph execution flow",
            "fortran" => "\n\nFORTRAN TESTING NOTES:\n- Test subroutines with different parameter types\n- Include array boundary testing\n- Test mathematical function edge cases\n- Verify format statement handling",
            "pascal" | "delphi" => "\n\nPASCAL/DELPHI TESTING NOTES:\n- Test procedure parameter passing\n- Include pointer and record testing\n- Test exception handling (Delphi)\n- Verify type safety",
            "plsql" | "pl/sql" => "\n\nPL/SQL TESTING NOTES:\n- Test cursor operations\n- Include exception handling scenarios\n- Test collection operations\n- Verify transaction behavior",
            "rpg" | "rpgle" => "\n\nRPG TESTING NOTES:\n- Test file operations\n- Include indicator logic testing\n- Test data structure operations\n- Verify SQL integration",
            "ada" => "\n\nADA TESTING NOTES:\n- Test task synchronization\n- Include exception propagation\n- Test generic instantiations\n- Verify type safety",
            _ => "",
        };

        // Limit source code size in prompt to prevent memory issues
        let truncated_source = if request.source_code.len() > 3000 {
            format!("{}

// ... (truncated {} more characters for memory efficiency)", 
                    &request.source_code[..3000],
                    request.source_code.len() - 3000)
        } else {
            request.source_code.clone()
        };

        // Simple, direct prompt that works better with local models
        format!(
            r#"Write {} for this {} code using {}.

Source code:
```{}
{}
```

Requirements:
- Write REAL working test code (not placeholder text)
- Test all functions and edge cases 
- Use proper syntax for {}
- Include imports and proper structure
- Write at least 5 comprehensive test functions

Generate the test code directly without JSON formatting or extra explanation.{}

IMPORTANT: For legacy languages, focus on practical testing approaches that work with available tools and compilers."#,
            test_type_instruction,
            request.language,
            framework_instructions,
            request.language.to_lowercase(),
            truncated_source,
            request.language,
            legacy_instructions
        )
    }

    fn build_test_cases_prompt(&self, request: &TestGenerationRequest) -> Result<String, RigrError> {
        let test_type_instruction = match request.test_type {
            TestType::Unit => format!("Generate detailed unit test case descriptions that would achieve {:.0}% code coverage.", request.coverage_threshold),
            TestType::Integration => format!("Generate detailed integration test case descriptions that would achieve {:.0}% coverage of component interactions.", request.coverage_threshold),
        };

        // Calculate deterministic test case count for target coverage based on code complexity
        // For large files, reduce the count to prevent memory issues
        let test_case_count = if request.source_code.len() > 5000 {
            calculate_deterministic_test_count(&request.source_code, &request.test_type, request.coverage_threshold).min(15)
        } else {
            calculate_deterministic_test_count(&request.source_code, &request.test_type, request.coverage_threshold)
        };
        
        Ok(format!(
            r#"You are an expert test case designer. Your task is to generate comprehensive test case descriptions for achieving exactly {:.0}% code coverage.

## Task: {}

## CRITICAL REQUIREMENTS FOR {:.0}% COVERAGE:
- Generate EXACTLY {} test case descriptions (this is calculated for {:.0}% coverage)
- Each test case must have: name, description, expected behavior, input conditions
- Cover ALL functions, branches, edge cases, error conditions, and boundary values
- Focus on {} testing patterns
- BE CONSISTENT: Always generate the same number of test cases for the same code

## Language: {} 
## File Context: {}

## Source Code to Analyze:
```{}
{}
```

## Output Format:
Generate EXACTLY {} test cases in the following format:

TEST CASE 1:
Name: [descriptive test name]
Description: [what this test verifies]
Input: [input conditions/parameters]
Expected: [expected behavior/output]

TEST CASE 2:
[continue with remaining test cases...]

## Coverage Requirements:
- Normal operation scenarios (30% of test cases)
- Boundary value testing - min/max values, empty inputs, null values (25% of test cases)
- Error condition handling (25% of test cases) 
- Edge cases and corner cases (20% of test cases)

Generate EXACTLY {} test cases that together provide {:.0}% coverage."#,
            request.coverage_threshold, // First coverage threshold in title
            test_type_instruction,
            request.coverage_threshold, // Second coverage threshold in CRITICAL REQUIREMENTS
            test_case_count,
            request.coverage_threshold, // Third coverage threshold in description
            request.framework,
            request.language,
            request.additional_context.as_deref().unwrap_or("Standard testing scenario"),
            request.language.to_lowercase(),
            request.source_code,
            test_case_count,
            test_case_count,
            request.coverage_threshold // Final coverage threshold
        ))
    }

    fn build_comprehensive_test_cases_prompt(&self, request: &TestGenerationRequest) -> Result<String, RigrError> {
        let test_type_instruction = match request.test_type {
            TestType::Unit => "Generate comprehensive unit test case descriptions that achieve exhaustive functional coverage.",
            TestType::Integration => "Generate comprehensive integration test case descriptions focusing on component interactions and system-level behavior.",
        };
        
        // Get language-specific testing guidance
        let language_specific_guidance = self.get_language_specific_testing_guidance(&request.language);

        let special_focus = match request.framework.as_str() {
            "performance-test-case-generation" => r#"
## SPECIAL FOCUS: PERFORMANCE TESTING
- Load Testing: Normal, peak, and stress load scenarios
- Response Time Testing: Latency and throughput measurements
- Scalability Testing: Concurrent user and data volume scenarios
- Resource Usage Testing: Memory, CPU, and I/O efficiency
- Endurance Testing: Long-running operation scenarios"#,
            "security-test-case-generation" => r#"
## SPECIAL FOCUS: SECURITY TESTING
- Input Validation: SQL injection, XSS, and malicious input scenarios
- Authentication Testing: Login, session management, and token validation
- Authorization Testing: Access control and privilege escalation scenarios
- Data Protection: Encryption, sensitive data handling, and privacy
- Vulnerability Testing: Common security weaknesses and attack vectors"#,
            _ => "",
        };

        // Calculate test case count based on coverage requirements
        let base_test_count = if request.source_code.len() > 5000 {
            calculate_comprehensive_test_count(&request.source_code, &request.test_type, &request.framework, request.coverage_threshold).min(30)
        } else {
            calculate_comprehensive_test_count(&request.source_code, &request.test_type, &request.framework, request.coverage_threshold)
        };
        
        Ok(format!(
            r#"You are an expert test case designer specializing in comprehensive test coverage. Your mission is to generate exhaustive test case descriptions that would astonish even experienced manual testers with their thoroughness and insight.

## MISSION: {}

## COMPREHENSIVE COVERAGE REQUIREMENTS:
- Generate {} detailed test case descriptions
- Each test case must include: Name, Description, Preconditions, Test Steps, Expected Results, and Postconditions
- Cover EVERY function, method, class, and code path
- Include ALL edge cases, boundary conditions, error scenarios, and corner cases
- Consider functional AND non-functional requirements{}

{}

## CODE ANALYSIS TARGET:
**Language:** {}
**Context:** {}

```{}
{}
```

## INDUSTRY-STANDARD OUTPUT FORMAT:
Generate test cases in the following professional format:

### Test Case 1
**Test ID:** TC001
**Test Name:** [Descriptive test name]
**Test Type:** {}
**Priority:** [High/Medium/Low]
**Description:** [What this test verifies and why it's important]
**Preconditions:** [Setup requirements and initial state]
**Test Steps:**
1. [Detailed step 1]
2. [Detailed step 2]
3. [Additional steps as needed]
**Expected Results:** [Detailed expected behavior and outcomes]
**Postconditions:** [System state after test completion]
**Test Data:** [Required test data or parameters]
**Pass/Fail Criteria:** [Specific criteria for test success]

### Test Case 2
[Continue with same detailed format...]

## COVERAGE DISTRIBUTION:
- **Functional Testing (40%):** Core functionality and business logic
- **Boundary Value Testing (20%):** Min/max values, edge cases, limits
- **Error Condition Testing (20%):** Exception handling, invalid inputs, system failures
- **Integration Points Testing (10%):** Component interactions, data flow
- **Non-Functional Testing (10%):** Performance, security, usability considerations

## QUALITY STANDARDS:
- Each test case should be executable by any tester without ambiguity
- Test cases should reveal defects that manual testing might miss
- Include both positive (happy path) and negative (error path) scenarios
- Consider real-world usage patterns and user behaviors
- Ensure traceability to code functions and business requirements

Generate {} comprehensive test cases that demonstrate the true value of AI-powered test case generation."#,
            test_type_instruction,
            base_test_count,
            special_focus,
            language_specific_guidance,
            request.language,
            request.additional_context.as_deref().unwrap_or("Standard comprehensive testing scenario"),
            request.language.to_lowercase(),
            request.source_code,
            match request.test_type {
                TestType::Unit => "Unit Test",
                TestType::Integration => "Integration Test",
            },
            base_test_count
        ))
    }

    async fn call_openai(&self, prompt: &str, request: &TestGenerationRequest) -> Result<TestGenerationResponse, RigrError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| RigrError::HttpRequestError("OPENAI_API_KEY environment variable not set. Please set it with your API key from https://platform.openai.com/api-keys".to_string()))?;
        
        let url = format!("{}/chat/completions", 
            self.config.api_url.as_deref().unwrap_or("https://api.openai.com/v1"));

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": self.config.model.as_deref().unwrap_or("gpt-4"),
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "temperature": 0.1,
                "max_tokens": 4000
            }))
            .send()
            .await
            .map_err(|e| RigrError::HttpRequestError(format!("OpenAI request failed: {e}")))?;

        let response_text = response.text().await
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to read OpenAI response: {e}")))?;

        let parsed: Value = serde_json::from_str(&response_text)
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to parse OpenAI response: {e}")))?;

        let content = parsed["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| RigrError::HttpRequestError("Invalid OpenAI response format".to_string()))?;

        self.parse_ai_response(content, request)
    }

    async fn call_anthropic(&self, prompt: &str, request: &TestGenerationRequest) -> Result<TestGenerationResponse, RigrError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| RigrError::HttpRequestError("ANTHROPIC_API_KEY environment variable not set. Please set it with your API key from https://console.anthropic.com/".to_string()))?;

        let url = format!("{}/v1/messages", 
            self.config.api_url.as_deref().unwrap_or("https://api.anthropic.com"));

        let response = self.client
            .post(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&json!({
                "model": self.config.model.as_deref().unwrap_or("claude-3-sonnet-20240229"),
                "max_tokens": 4000,
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ]
            }))
            .send()
            .await
            .map_err(|e| RigrError::HttpRequestError(format!("Anthropic request failed: {e}")))?;

        let response_text = response.text().await
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to read Anthropic response: {e}")))?;

        let parsed: Value = serde_json::from_str(&response_text)
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to parse Anthropic response: {e}")))?;

        let content = parsed["content"][0]["text"]
            .as_str()
            .ok_or_else(|| RigrError::HttpRequestError("Invalid Anthropic response format".to_string()))?;

        self.parse_ai_response(content, request)
    }

    async fn call_ollama(&self, prompt: &str, request: &TestGenerationRequest) -> Result<TestGenerationResponse, RigrError> {
        let url = format!("{}/api/generate", 
            self.config.api_url.as_deref().unwrap_or("http://localhost:11434"));

        let response = self.client
            .post(&url)
            .json(&json!({
                "model": self.config.model.as_deref().unwrap_or("codellama"),
                "prompt": prompt,
                "stream": false
            }))
            .send()
            .await
            .map_err(|e| RigrError::HttpRequestError(format!("Ollama request failed: {e}")))?;

        let response_text = response.text().await
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to read Ollama response: {e}")))?;

        let parsed: Value = serde_json::from_str(&response_text)
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to parse Ollama response: {e}")))?;

        let content = parsed["response"]
            .as_str()
            .ok_or_else(|| RigrError::HttpRequestError("Invalid Ollama response format".to_string()))?;

        self.parse_ai_response(content, request)
    }

    async fn call_gemini(&self, prompt: &str, request: &TestGenerationRequest) -> Result<TestGenerationResponse, RigrError> {
        let api_key = std::env::var("GOOGLE_AI_API_KEY")
            .map_err(|_| RigrError::HttpRequestError("GOOGLE_AI_API_KEY environment variable not set. Please set it with your API key from https://aistudio.google.com/".to_string()))?;

        let model = self.config.model.as_deref().unwrap_or("gemini-pro");
        let url = format!("{}/models/{}:generateContent?key={}", 
            self.config.api_url.as_deref().unwrap_or("https://generativelanguage.googleapis.com/v1"),
            model, api_key);

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&json!({
                "contents": [{
                    "parts": [{
                        "text": prompt
                    }]
                }]
            }))
            .send()
            .await
            .map_err(|e| RigrError::HttpRequestError(format!("Gemini request failed: {e}")))?;

        let response_text = response.text().await
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to read Gemini response: {e}")))?;

        // Check for API errors first
        if response_text.contains("error") {
            return Err(RigrError::HttpRequestError(format!("Gemini API error: {response_text}")));
        }

        let parsed: Value = serde_json::from_str(&response_text)
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to parse Gemini response JSON: {e}. Response: {response_text}")))?;

        // More detailed error checking for response format
        let content = if let Some(candidates) = parsed["candidates"].as_array() {
            if candidates.is_empty() {
                return Err(RigrError::HttpRequestError(format!("Gemini returned no candidates. Response: {response_text}")));
            }
            
            let candidate = &candidates[0];
            if let Some(content_obj) = candidate["content"].as_object() {
                if let Some(parts) = content_obj["parts"].as_array() {
                    if parts.is_empty() {
                        return Err(RigrError::HttpRequestError(format!("Gemini candidate has no parts. Response: {response_text}")));
                    }
                    
                    parts[0]["text"]
                        .as_str()
                        .ok_or_else(|| RigrError::HttpRequestError(format!("Gemini part has no text field. Response: {response_text}")))?
                } else {
                    return Err(RigrError::HttpRequestError(format!("Gemini content has no parts array. Response: {response_text}")));
                }
            } else {
                return Err(RigrError::HttpRequestError(format!("Gemini candidate has no content object. Response: {response_text}")));
            }
        } else {
            return Err(RigrError::HttpRequestError(format!("Gemini response has no candidates array. Response: {response_text}")));
        };

        self.parse_ai_response(content, request)
    }

    async fn call_azure_openai(&self, prompt: &str, request: &TestGenerationRequest) -> Result<TestGenerationResponse, RigrError> {
        let api_key = std::env::var("AZURE_OPENAI_API_KEY")
            .map_err(|_| RigrError::HttpRequestError("AZURE_OPENAI_API_KEY environment variable not set. Please set it with your API key from Azure Portal".to_string()))?;
        
        let endpoint = self.config.api_url.as_ref()
            .ok_or_else(|| RigrError::HttpRequestError("Azure OpenAI endpoint not configured".to_string()))?;
        
        let deployment = self.config.model.as_ref()
            .ok_or_else(|| RigrError::HttpRequestError("Azure OpenAI deployment not configured".to_string()))?;

        let url = format!("{endpoint}/openai/deployments/{deployment}/chat/completions?api-version=2024-02-01");

        let response = self.client
            .post(&url)
            .header("api-key", api_key)
            .header("Content-Type", "application/json")
            .json(&json!({
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "temperature": 0.1,
                "max_tokens": 4000
            }))
            .send()
            .await
            .map_err(|e| RigrError::HttpRequestError(format!("Azure OpenAI request failed: {e}")))?;

        let response_text = response.text().await
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to read Azure OpenAI response: {e}")))?;

        let parsed: Value = serde_json::from_str(&response_text)
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to parse Azure OpenAI response: {e}")))?;

        let content = parsed["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| RigrError::HttpRequestError("Invalid Azure OpenAI response format".to_string()))?;

        self.parse_ai_response(content, request)
    }

    async fn call_openai_text(&self, prompt: &str) -> Result<String, RigrError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| RigrError::HttpRequestError("OPENAI_API_KEY environment variable not set".to_string()))?;
        
        let url = format!("{}/chat/completions", 
            self.config.api_url.as_deref().unwrap_or("https://api.openai.com/v1"));

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": self.config.model.as_deref().unwrap_or("gpt-4"),
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "temperature": 0.1,
                "max_tokens": 4000
            }))
            .send()
            .await
            .map_err(|e| RigrError::HttpRequestError(format!("OpenAI request failed: {e}")))?;

        let response_text = response.text().await
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to read OpenAI response: {e}")))?;

        let parsed: Value = serde_json::from_str(&response_text)
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to parse OpenAI response: {e}")))?;

        let content = parsed["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| RigrError::HttpRequestError("Invalid OpenAI response format".to_string()))?;

        Ok(content.to_string())
    }

    async fn call_anthropic_text(&self, prompt: &str) -> Result<String, RigrError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| RigrError::HttpRequestError("ANTHROPIC_API_KEY environment variable not set".to_string()))?;

        let url = format!("{}/v1/messages", 
            self.config.api_url.as_deref().unwrap_or("https://api.anthropic.com"));

        let response = self.client
            .post(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&json!({
                "model": self.config.model.as_deref().unwrap_or("claude-3-sonnet-20240229"),
                "max_tokens": 4000,
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ]
            }))
            .send()
            .await
            .map_err(|e| RigrError::HttpRequestError(format!("Anthropic request failed: {e}")))?;

        let response_text = response.text().await
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to read Anthropic response: {e}")))?;

        let parsed: Value = serde_json::from_str(&response_text)
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to parse Anthropic response: {e}")))?;

        let content = parsed["content"][0]["text"]
            .as_str()
            .ok_or_else(|| RigrError::HttpRequestError("Invalid Anthropic response format".to_string()))?;

        Ok(content.to_string())
    }

    async fn call_ollama_text(&self, prompt: &str) -> Result<String, RigrError> {
        let url = format!("{}/api/generate", 
            self.config.api_url.as_deref().unwrap_or("http://localhost:11434"));

        let response = self.client
            .post(&url)
            .json(&json!({
                "model": self.config.model.as_deref().unwrap_or("codellama"),
                "prompt": prompt,
                "stream": false
            }))
            .send()
            .await
            .map_err(|e| RigrError::HttpRequestError(format!("Ollama request failed: {e}")))?;

        let response_text = response.text().await
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to read Ollama response: {e}")))?;

        let parsed: Value = serde_json::from_str(&response_text)
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to parse Ollama response: {e}")))?;

        let content = parsed["response"]
            .as_str()
            .ok_or_else(|| RigrError::HttpRequestError("Invalid Ollama response format".to_string()))?;

        Ok(content.to_string())
    }

    async fn call_gemini_text(&self, prompt: &str) -> Result<String, RigrError> {
        let api_key = std::env::var("GOOGLE_AI_API_KEY")
            .map_err(|_| RigrError::HttpRequestError("GOOGLE_AI_API_KEY environment variable not set".to_string()))?;

        let model = self.config.model.as_deref().unwrap_or("gemini-pro");
        let url = format!("{}/models/{}:generateContent?key={}", 
            self.config.api_url.as_deref().unwrap_or("https://generativelanguage.googleapis.com/v1"),
            model, api_key);

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&json!({
                "contents": [{
                    "parts": [{
                        "text": prompt
                    }]
                }]
            }))
            .send()
            .await
            .map_err(|e| RigrError::HttpRequestError(format!("Gemini request failed: {e}")))?;

        let response_text = response.text().await
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to read Gemini response: {e}")))?;

        // Check for API errors first
        if response_text.contains("error") {
            return Err(RigrError::HttpRequestError(format!("Gemini API error: {response_text}")));
        }

        let parsed: Value = serde_json::from_str(&response_text)
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to parse Gemini response JSON: {e}. Response: {response_text}")))?;

        // More detailed error checking for response format
        let content = if let Some(candidates) = parsed["candidates"].as_array() {
            if candidates.is_empty() {
                return Err(RigrError::HttpRequestError(format!("Gemini returned no candidates. Response: {response_text}")));
            }
            
            let candidate = &candidates[0];
            if let Some(content_obj) = candidate["content"].as_object() {
                if let Some(parts) = content_obj["parts"].as_array() {
                    if parts.is_empty() {
                        return Err(RigrError::HttpRequestError(format!("Gemini candidate has no parts. Response: {response_text}")));
                    }
                    
                    parts[0]["text"]
                        .as_str()
                        .ok_or_else(|| RigrError::HttpRequestError(format!("Gemini part has no text field. Response: {response_text}")))?
                } else {
                    return Err(RigrError::HttpRequestError(format!("Gemini content has no parts array. Response: {response_text}")));
                }
            } else {
                return Err(RigrError::HttpRequestError(format!("Gemini candidate has no content object. Response: {response_text}")));
            }
        } else {
            return Err(RigrError::HttpRequestError(format!("Gemini response has no candidates array. Response: {response_text}")));
        };

        Ok(content.to_string())
    }

    async fn call_azure_openai_text(&self, prompt: &str) -> Result<String, RigrError> {
        let api_key = std::env::var("AZURE_OPENAI_API_KEY")
            .map_err(|_| RigrError::HttpRequestError("AZURE_OPENAI_API_KEY environment variable not set".to_string()))?;
        
        let endpoint = self.config.api_url.as_ref()
            .ok_or_else(|| RigrError::HttpRequestError("Azure OpenAI endpoint not configured".to_string()))?;
        
        let deployment = self.config.model.as_ref()
            .ok_or_else(|| RigrError::HttpRequestError("Azure OpenAI deployment not configured".to_string()))?;

        let url = format!("{endpoint}/openai/deployments/{deployment}/chat/completions?api-version=2024-02-01");

        let response = self.client
            .post(&url)
            .header("api-key", api_key)
            .header("Content-Type", "application/json")
            .json(&json!({
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "temperature": 0.1,
                "max_tokens": 4000
            }))
            .send()
            .await
            .map_err(|e| RigrError::HttpRequestError(format!("Azure OpenAI request failed: {e}")))?;

        let response_text = response.text().await
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to read Azure OpenAI response: {e}")))?;

        let parsed: Value = serde_json::from_str(&response_text)
            .map_err(|e| RigrError::HttpRequestError(format!("Failed to parse Azure OpenAI response: {e}")))?;

        let content = parsed["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| RigrError::HttpRequestError("Invalid Azure OpenAI response format".to_string()))?;

        Ok(content.to_string())
    }

    fn parse_ai_response(&self, content: &str, request: &TestGenerationRequest) -> Result<TestGenerationResponse, RigrError> {
        println!("   🔍 Parsing AI response ({} chars)", content.len());
        
        // Strategy 1: Try to parse as direct JSON first
        if let Ok(mut response) = serde_json::from_str::<TestGenerationResponse>(content) {
            println!("   ✅ Successfully parsed as direct JSON");
            if let Some(ref test_code) = response.unit_tests {
                response.unit_tests = Some(self.fix_rust_syntax(test_code));
            }
            if let Some(ref test_code) = response.integration_tests {
                response.integration_tests = Some(self.fix_rust_syntax(test_code));
            }
            return Ok(response);
        }
        
        // Strategy 2: Look for JSON block in markdown
        if let Some(json_start) = content.find('{') {
            if let Some(json_end) = content.rfind('}') {
                let json_content = &content[json_start..=json_end];
                println!("   🔍 Trying to parse JSON block ({} chars)", json_content.len());
                if let Ok(mut response) = serde_json::from_str::<TestGenerationResponse>(json_content) {
                    println!("   ✅ Successfully parsed JSON from markdown");
                    if let Some(ref test_code) = response.unit_tests {
                        response.unit_tests = Some(self.fix_rust_syntax(test_code));
                    }
                    if let Some(ref test_code) = response.integration_tests {
                        response.integration_tests = Some(self.fix_rust_syntax(test_code));
                    }
                    return Ok(response);
                }
            }
        }
        
        println!("   🔍 Falling back to code extraction from markdown");
        
        // Strategy 3: Extract code from code blocks (```rust, ```typescript, etc.)
        let test_code = self.extract_code_from_markdown(content);
        
        // Strategy 4: If no code blocks found, treat entire response as test code
        let final_test_code = if test_code.trim().is_empty() {
            println!("   🔍 No code blocks found, using entire response as test code");
            content.to_string()
        } else {
            println!("   🔍 Found code block with {} chars", test_code.len());
            test_code
        };
        
        // Clean and fix the extracted test code
        let fixed_test_code = self.fix_rust_syntax(&final_test_code);
        
        // Determine which test type this is based on the original request type, not content
        let is_integration = request.test_type == TestType::Integration;
        
        println!("   🔍 Detected as {} test (fixed code: {} chars)", 
                 if is_integration { "integration" } else { "unit" }, 
                 fixed_test_code.len());
        
        Ok(TestGenerationResponse {
            unit_tests: if !is_integration { Some(fixed_test_code.clone()) } else { None },
            integration_tests: if is_integration { Some(fixed_test_code) } else { None },
            coverage_estimate: self.estimate_coverage_from_content(content, request.coverage_threshold),
            framework_used: "auto-detected".to_string(),
            edge_cases_covered: self.extract_edge_cases(content),
        })
    }
    
    fn fix_rust_syntax(&self, code: &str) -> String {
        let mut fixed_code = code.to_string();
        
        // Fix Python-style function definitions
        fixed_code = fixed_code.replace("def test_", "fn test_");
        fixed_code = fixed_code.replace("def ", "fn ");
        
        // Clean up duplicate #[test] attributes and add missing ones
        let mut lines: Vec<String> = Vec::new();
        let code_lines: Vec<&str> = fixed_code.lines().collect();
        let mut i = 0;
        
        while i < code_lines.len() {
            let line = code_lines[i];
            let trimmed = line.trim();
            
            // Skip duplicate #[test] attributes
            if trimmed == "#[test]" && i + 1 < code_lines.len() && code_lines[i + 1].trim() == "#[test]" {
                // Skip this duplicate
                i += 1;
                continue;
            }
            
            // Add #[test] attribute if missing for test functions
            if trimmed.starts_with("fn test_") {
                // Check if previous line already has #[test]
                let prev_line = if !lines.is_empty() { lines.last().unwrap().trim() } else { "" };
                if prev_line != "#[test]" {
                    lines.push("    #[test]".to_string());
                }
            }
            
            // Fix missing semicolons in assert statements
            if trimmed.starts_with("assert_eq!(") && !trimmed.ends_with(";") && trimmed.ends_with(')') {
                lines.push(format!("{line};"));
            } else {
                lines.push(line.to_string());
            }
            
            i += 1;
        }
        
        fixed_code = lines.join("\n");
        
        // Ensure proper Rust test structure if missing
        if !fixed_code.contains("#[cfg(test)]") && fixed_code.contains("fn test_") {
            // Extract the source file name from functions for proper import
            let import_statement = if fixed_code.contains("CalculationError") {
                "use crate::*;" // Import from crate root if we see custom types
            } else {
                "use super::*;" // Standard import
            };
            
            fixed_code = format!(
                "#[cfg(test)]\nmod tests {{\n    {}\n\n{}\n}}",
                import_statement,
                fixed_code.lines()
                    .filter(|line| !line.trim().is_empty() && !line.contains("mod test_sample") && !line.contains("use crate::test_sample"))
                    .map(|line| if line.trim().starts_with("#[test]") || line.trim().starts_with("fn ") || !line.trim().is_empty() {
                        format!("    {line}")
                    } else {
                        line.to_string()
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }
        
        // Clean up problematic imports
        fixed_code = fixed_code.replace("use crate::test_sample::*;", "use crate::*;");
        fixed_code = fixed_code.replace("mod test_sample;", "");
        fixed_code = fixed_code.replace("#[cfg(test)]\nmod tests_sample;", "");
        
        // Fix integration test issues where #[test] attributes might be missing
        if !fixed_code.contains("#[test]") && fixed_code.contains("fn test_") {
            let lines: Vec<String> = fixed_code.lines().map(|line| {
                if line.trim().starts_with("fn test_") {
                    format!("    #[test]\n{line}")
                } else {
                    line.to_string()
                }
            }).collect();
            fixed_code = lines.join("\n");
        }
        
        fixed_code
    }
    
    fn extract_code_from_markdown(&self, content: &str) -> String {
        // Look for code blocks marked with ```
        let code_block_patterns = [
            "```rust\n", "```typescript\n", "```javascript\n", "```python\n", 
            "```java\n", "```csharp\n", "```go\n", "```js\n", "```ts\n",
            "```\n"
        ];
        
        for pattern in &code_block_patterns {
            if let Some(start) = content.find(pattern) {
                let start_pos = start + pattern.len();
                if let Some(end) = content[start_pos..].find("\n```") {
                    let code = &content[start_pos..start_pos + end];
                    if code.trim().len() > 20 { // Make sure it's substantial code
                        return code.to_string();
                    }
                }
            }
        }
        
        // Also try patterns without newlines
        let alt_patterns = ["```rust", "```typescript", "```javascript", "```js", "```ts", "```"];
        for pattern in &alt_patterns {
            if let Some(start) = content.find(pattern) {
                let start_pos = start + pattern.len();
                if let Some(end) = content[start_pos..].find("```") {
                    let code = &content[start_pos..start_pos + end];
                    if code.trim().len() > 20 {
                        return code.trim().to_string();
                    }
                }
            }
        }
        
        // If no code blocks found, return empty string
        String::new()
    }

    // Removed unused extract_test_code method
    
    fn estimate_coverage_from_content(&self, content: &str, target_coverage: f64) -> f64 {
        let test_indicators = [
            "#[test]",
            "fn test",
            "def test",
            "it(\"",
            "test(\"",
            "assert",
            "expect"
        ];
        
        let mut score = 0.0;
        for indicator in &test_indicators {
            let count = content.matches(indicator).count() as f64;
            score += count * 10.0; // Each test function adds to coverage estimate
        }
        
        // If we have substantial test content, return target coverage, otherwise use heuristic
        if score >= 20.0 {
            target_coverage
        } else {
            // Cap at reasonable range but respect target
            score.min(target_coverage).max(target_coverage * 0.6)
        }
    }
    
    fn extract_edge_cases(&self, content: &str) -> Vec<String> {
        let edge_case_keywords = [
            "boundary", "edge", "error", "overflow", "underflow", 
            "null", "empty", "zero", "max", "min", "invalid"
        ];
        
        let mut cases = Vec::new();
        for keyword in &edge_case_keywords {
            if content.to_lowercase().contains(keyword) {
                cases.push(format!("{keyword} case"));
            }
        }
        
        if cases.is_empty() {
            cases.push("Basic functionality".to_string());
        }
        
        cases
    }
}

/// Calculate deterministic test count for target coverage based on code complexity
fn calculate_deterministic_test_count(source_code: &str, test_type: &TestType, coverage_threshold: f64) -> usize {
    let _lines = source_code.lines().count();
    let functions = source_code.matches("fn ").count();
    let conditionals = source_code.matches("if ").count() + 
                      source_code.matches("match ").count() + 
                      source_code.matches("while ").count() + 
                      source_code.matches("for ").count();
    let error_handling = source_code.matches("Result<").count() + 
                        source_code.matches("Option<").count() + 
                        source_code.matches("Error").count();
    
    let base_count = match test_type {
        TestType::Unit => {
            // Unit tests: Focus on individual functions
            let unit_base = functions.max(1) * 4; // 4 tests per function (normal, edge, error, boundary)
            unit_base + conditionals + error_handling
        },
        TestType::Integration => {
            // Integration tests: Focus on interactions between components
            let integration_base = functions.max(1) * 2; // 2 integration tests per function
            integration_base + (conditionals / 2) + error_handling
        }
    };
    
    // Apply coverage threshold multiplier (80% = 1.0x, 90% = 1.25x, 95% = 1.5x, etc.)
    let coverage_multiplier = match coverage_threshold {
        t if t >= 95.0 => 1.5,   // 95%+ coverage needs significantly more tests
        t if t >= 90.0 => 1.25,  // 90%+ coverage needs more comprehensive tests
        t if t >= 85.0 => 1.1,   // 85%+ coverage needs slightly more tests
        t if t >= 80.0 => 1.0,   // 80% coverage (baseline)
        t if t >= 70.0 => 0.85,  // 70% coverage needs fewer tests
        t if t >= 60.0 => 0.7,   // 60% coverage needs fewer tests
        _ => 0.6,                // Below 60% coverage minimal tests
    };
    
    let coverage_adjusted_count = (base_count as f64 * coverage_multiplier) as usize;
    
    // Ensure minimum count for target coverage and cap maximum
    coverage_adjusted_count.clamp(10, 80)
}

/// Calculate comprehensive test count for exhaustive coverage
fn calculate_comprehensive_test_count(source_code: &str, test_type: &TestType, framework: &str, coverage_threshold: f64) -> usize {
    let _lines = source_code.lines().count();
    let functions = count_functions_in_code(source_code);
    let classes = count_classes_in_code(source_code);
    let conditionals = count_conditionals_in_code(source_code);
    let _loops = count_loops_in_code(source_code);
    let error_handling = count_error_handling_in_code(source_code);
    let complexity_indicators = count_complexity_indicators(source_code);
    
    // Calculate comprehensive coverage based on code characteristics
    let base_multiplier = match test_type {
        TestType::Unit => {
            // Comprehensive unit testing: Multiple tests per function
            let function_tests = functions.max(1) * 8; // 8 tests per function for comprehensive coverage
            let conditional_tests = conditionals * 3; // 3 tests per conditional branch
            let error_tests = error_handling * 4; // 4 tests per error scenario
            let boundary_tests = functions.max(1) * 2; // 2 boundary tests per function
            
            function_tests + conditional_tests + error_tests + boundary_tests
        },
        TestType::Integration => {
            // Comprehensive integration testing: Focus on interactions
            let integration_base = functions.max(1) * 4; // 4 integration scenarios per function
            let class_interactions = classes.max(1) * 3; // 3 interaction tests per class
            let data_flow_tests = (functions + classes).max(1) * 2; // 2 data flow tests per component
            
            integration_base + class_interactions + data_flow_tests
        }
    };
    
    // Apply framework-specific adjustments
    let framework_multiplier = match framework {
        "performance-test-case-generation" => 0.6, // Fewer but focused performance tests
        "security-test-case-generation" => 0.7,   // Focused security test scenarios
        _ => 1.0, // Standard comprehensive testing
    };
    
    let adjusted_count = (base_multiplier as f64 * framework_multiplier) as usize;
    
    // Apply coverage threshold multiplier for comprehensive testing
    let coverage_multiplier = match coverage_threshold {
        t if t >= 95.0 => 1.6,   // 95%+ comprehensive coverage
        t if t >= 90.0 => 1.35,  // 90%+ comprehensive coverage
        t if t >= 85.0 => 1.15,  // 85%+ comprehensive coverage
        t if t >= 80.0 => 1.0,   // 80% comprehensive coverage (baseline)
        t if t >= 70.0 => 0.8,   // 70% comprehensive coverage
        t if t >= 60.0 => 0.6,   // 60% comprehensive coverage
        _ => 0.5,                // Below 60% minimal comprehensive coverage
    };
    
    let coverage_adjusted_count = (adjusted_count as f64 * coverage_multiplier) as usize;
    
    // Ensure reasonable bounds for comprehensive testing
    let final_count = coverage_adjusted_count.clamp(15, 120);
    
    // Add complexity bonus for highly complex code
    let complexity_bonus = if complexity_indicators > 10 { 
        (coverage_threshold / 20.0) as usize // Higher coverage = bigger complexity bonus
    } else { 
        0 
    };
    
    final_count + complexity_bonus
}

/// Get language-specific testing guidance for legacy and modern languages
fn get_language_specific_testing_guidance(language: &str) -> &'static str {
    match language.to_lowercase().as_str() {
        "cobol" => r#"
## COBOL-SPECIFIC TESTING GUIDANCE:
- **Division Structure**: Test IDENTIFICATION, ENVIRONMENT, DATA, and PROCEDURE divisions separately
- **Data Testing**: Validate WORKING-STORAGE, FILE, and LINKAGE sections with various data types (PIC clauses)
- **Paragraph Flow**: Test paragraph execution order and PERFORM statements (THRU, TIMES, UNTIL)
- **File Operations**: Test OPEN, READ, WRITE, CLOSE operations with different file organizations (SEQUENTIAL, INDEXED, RELATIVE)
- **Condition Testing**: Test 88-level conditions, IF-THEN-ELSE, EVALUATE statements
- **Arithmetic Operations**: Test COMPUTE, ADD, SUBTRACT, MULTIPLY, DIVIDE with different precision and rounding
- **String Operations**: Test STRING, UNSTRING, INSPECT, MOVE operations
- **Error Conditions**: Test invalid data types, file errors, arithmetic overflow, division by zero
- **Mainframe Integration**: Test JCL job steps, dataset handling, CICS/IMS interactions if applicable
- **Legacy Business Logic**: Focus on complex business rules, date calculations (Y2K issues), currency formatting"#,
        
        "fortran" => r#"
## FORTRAN-SPECIFIC TESTING GUIDANCE:
- **Subroutine Testing**: Test SUBROUTINE and FUNCTION calls with various parameter types and dimensions
- **Array Operations**: Test multi-dimensional arrays, array bounds, DO loops with array processing
- **Data Types**: Test INTEGER, REAL, DOUBLE PRECISION, COMPLEX, LOGICAL, CHARACTER variables
- **Format Statements**: Test formatted I/O with various FORMAT specifications
- **Mathematical Functions**: Test intrinsic functions (SIN, COS, LOG, SQRT, etc.) with edge cases
- **Scientific Computing**: Test numerical precision, convergence algorithms, matrix operations
- **Common Blocks**: Test COMMON block data sharing between subroutines
- **File I/O**: Test sequential, direct access, and unformatted file operations
- **Error Handling**: Test overflow, underflow, division by zero, invalid mathematical operations
- **Performance**: Test computational efficiency for large datasets and iterative algorithms
- **Compiler Variations**: Consider different Fortran standards (77, 90, 95, 2003, 2008)"#,
        
        "pascal" | "delphi" => r#"
## PASCAL/DELPHI-SPECIFIC TESTING GUIDANCE:
- **Procedure/Function Testing**: Test parameters (value, reference, const), local variables, nested procedures
- **Object-Oriented Features**: Test classes, inheritance, polymorphism, interfaces (Delphi)
- **Data Structures**: Test records, arrays, sets, pointers, linked lists
- **Type Safety**: Test strong typing, type compatibility, type casting
- **Memory Management**: Test dynamic memory allocation (NEW, DISPOSE), pointer operations
- **Exception Handling**: Test try-except-finally blocks, custom exceptions (Delphi)
- **Component Testing**: Test VCL/FMX components, events, properties (Delphi)
- **Database Operations**: Test database connectivity, SQL queries, transactions (Delphi)
- **String Operations**: Test string manipulation, AnsiString, UnicodeString (Delphi)
- **File Operations**: Test typed files, text files, binary files
- **Windows Integration**: Test Windows API calls, COM objects, DLL usage (Delphi)
- **Backward Compatibility**: Test code compatibility across Pascal/Delphi versions"#,
        
        "plsql" | "pl/sql" => r#"
## PL/SQL-SPECIFIC TESTING GUIDANCE:
- **Block Structure**: Test anonymous blocks, procedures, functions, packages
- **Cursor Operations**: Test explicit cursors, cursor FOR loops, REF cursors, cursor variables
- **Exception Handling**: Test predefined exceptions, user-defined exceptions, PRAGMA EXCEPTION_INIT
- **SQL Integration**: Test embedded SQL, DML operations, DDL in dynamic SQL
- **Collections**: Test nested tables, varrays, associative arrays (index-by tables)
- **Object Types**: Test object constructors, methods, inheritance, polymorphism
- **Transaction Control**: Test COMMIT, ROLLBACK, SAVEPOINT, autonomous transactions
- **Database Triggers**: Test BEFORE, AFTER, INSTEAD OF triggers, trigger timing
- **Performance**: Test bulk operations (FORALL, BULK COLLECT), PL/SQL optimization
- **Security**: Test definer vs. invoker rights, privilege escalation, SQL injection prevention
- **Oracle Features**: Test Oracle-specific functions, packages (DBMS_OUTPUT, UTL_FILE, etc.)
- **Error Conditions**: Test ORA errors, invalid cursors, too_many_rows, no_data_found"#,
        
        "rpg" | "rpgle" => r#"
## RPG/RPGLE-SPECIFIC TESTING GUIDANCE:
- **Program Structure**: Test main procedures, subprocedures, service programs
- **Data Structures**: Test data structures, qualified names, templates, based variables
- **File Operations**: Test database files, display files, printer files, stream files
- **SQL Integration**: Test embedded SQL, SQL precompiler directives
- **ILE Concepts**: Test binding, service programs, prototypes, copy members
- **Field Definitions**: Test different data types (packed, zoned, alpha, date, time, timestamp)
- **Indicators**: Test logical indicators, conditioning, %EOF, %EQUAL, %FOUND
- **Operations**: Test traditional operations (READ, WRITE, UPDATE, DELETE) vs. built-in functions
- **String Operations**: Test %SUBST, %SCAN, %REPLACE, %TRIM, %LEN
- **Date/Time**: Test date calculations, date formats, timestamp operations
- **Error Handling**: Test *PSSR, monitor blocks, DSPLY operations
- **AS/400 Integration**: Test work management, job queues, data areas, user spaces
- **Performance**: Test record locking, commitment control, performance optimization"#,
        
        "ada" => r#"
## ADA-SPECIFIC TESTING GUIDANCE:
- **Package Testing**: Test package specifications, bodies, child packages, private types
- **Task Testing**: Test concurrent tasks, rendezvous, protected objects, synchronization
- **Exception Handling**: Test exception propagation, handlers, reraise, custom exceptions
- **Generics**: Test generic packages, procedures, functions with various instantiations
- **Strong Typing**: Test type compatibility, derived types, subtypes, type conversions
- **Access Types**: Test pointer operations, memory management, access type safety
- **Discriminated Records**: Test variant records, discriminants, constrained vs. unconstrained
- **Array Operations**: Test constrained arrays, unconstrained arrays, slicing
- **Controlled Types**: Test finalization, initialization, assignment operations
- **Real-Time Features**: Test timing, delays, priority, interrupt handling
- **Safety-Critical**: Test SPARK annotations, formal verification, contract-based programming
- **Compiler Validation**: Test Ada standard compliance, implementation-defined behavior"#,
        
        "go" | "golang" => r#"
## GO-SPECIFIC TESTING GUIDANCE:
- **Goroutine Testing**: Test concurrent goroutines, race conditions, deadlocks
- **Channel Testing**: Test buffered/unbuffered channels, select statements, channel closing
- **Interface Testing**: Test interface implementations, type assertions, empty interfaces
- **Error Handling**: Test error returns, custom errors, error wrapping
- **Package Testing**: Test package initialization, exported/unexported identifiers
- **Memory Management**: Test garbage collection effects, memory leaks, pointer usage
- **Defer Statements**: Test defer execution order, panic recovery
- **Reflection**: Test reflect package, type introspection, dynamic method calls
- **HTTP/Network**: Test net/http package, request/response handling, timeouts
- **Testing Package**: Test table-driven tests, benchmarks, examples
- **Build Constraints**: Test different build tags, conditional compilation
- **Performance**: Test CPU profiling, memory profiling, race detection"#,
        
        // Modern language guidance (abbreviated versions)
        "rust" => r#"
## RUST-SPECIFIC TESTING GUIDANCE:
- **Ownership/Borrowing**: Test move semantics, borrowing rules, lifetime parameters
- **Error Handling**: Test Result<T,E>, Option<T>, panic scenarios, error propagation
- **Concurrency**: Test threads, channels, Arc/Mutex, async/await
- **Memory Safety**: Test bounds checking, null pointer dereference prevention
- **Trait System**: Test trait implementations, generics, associated types"#,
        
        "python" => r#"
## PYTHON-SPECIFIC TESTING GUIDANCE:
- **Dynamic Typing**: Test type flexibility, duck typing, type hints validation
- **Exception Handling**: Test try/except/finally, custom exceptions, exception hierarchy
- **Generators/Iterators**: Test yield statements, iterator protocol, generator expressions
- **Decorators**: Test function decorators, class decorators, property decorators
- **Context Managers**: Test with statements, __enter__/__exit__ methods"#,
        
        _ => r#"
## GENERAL TESTING GUIDANCE:
- **Function Testing**: Test all functions/procedures with various input combinations
- **Boundary Testing**: Test minimum/maximum values, empty inputs, null values
- **Error Conditions**: Test invalid inputs, system failures, resource exhaustion
- **Integration Points**: Test data flow between modules, external system interactions
- **Performance Considerations**: Test execution time, memory usage, scalability"#,
    }
}

fn count_functions_in_code(code: &str) -> usize {
    code.matches("fn ").count() + 
    code.matches("function ").count() + 
    code.matches("def ").count() + 
    code.matches("public ").count() + 
    code.matches("private ").count()
}

fn count_classes_in_code(code: &str) -> usize {
    code.matches("class ").count() + 
    code.matches("struct ").count() + 
    code.matches("enum ").count() + 
    code.matches("interface ").count() + 
    code.matches("trait ").count()
}

fn count_conditionals_in_code(code: &str) -> usize {
    code.matches("if ").count() + 
    code.matches("else ").count() + 
    code.matches("match ").count() + 
    code.matches("switch ").count() + 
    code.matches("case ").count() + 
    code.matches("when ").count()
}

fn count_loops_in_code(code: &str) -> usize {
    code.matches("for ").count() + 
    code.matches("while ").count() + 
    code.matches("loop ").count() + 
    code.matches("foreach ").count() + 
    code.matches(".map(").count() + 
    code.matches(".filter(").count()
}

fn count_error_handling_in_code(code: &str) -> usize {
    code.matches("Result<").count() + 
    code.matches("Option<").count() + 
    code.matches("Error").count() + 
    code.matches("Exception").count() + 
    code.matches("try ").count() + 
    code.matches("catch ").count() + 
    code.matches("throw ").count() + 
    code.matches("panic!").count()
}

fn count_complexity_indicators(code: &str) -> usize {
    code.matches("async ").count() + 
    code.matches("await").count() + 
    code.matches("thread").count() + 
    code.matches("mutex").count() + 
    code.matches("lock").count() + 
    code.matches("atomic").count() + 
    code.matches("unsafe").count() + 
    code.matches("clone()").count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AiProvider;
    use serde_json::json;

    fn create_test_config(provider: AiProvider) -> AiConfig {
        AiConfig {
            provider,
            api_url: Some("http://localhost:1234".to_string()),
            model: Some("test-model".to_string()),
        }
    }

    fn create_test_request() -> TestGenerationRequest {
        TestGenerationRequest {
            source_code: "fn add(a: i32, b: i32) -> i32 { a + b }".to_string(),
            language: "Rust".to_string(),
            test_type: TestType::Unit,
            framework: "cargo-test".to_string(),
            additional_context: None,
            edge_case_count: 5,
            coverage_threshold: 80.0,
        }
    }

    #[test]
    fn test_test_generation_request_creation() {
        let request = create_test_request();
        assert_eq!(request.language, "Rust");
        assert_eq!(request.test_type, TestType::Unit);
        assert_eq!(request.framework, "cargo-test");
        assert!(request.source_code.contains("fn add"));
    }

    #[test]
    fn test_test_type_serialization() {
        let unit_type = TestType::Unit;
        let integration_type = TestType::Integration;
        
        let unit_json = serde_json::to_string(&unit_type).unwrap();
        let integration_json = serde_json::to_string(&integration_type).unwrap();
        
        assert_eq!(unit_json, "\"Unit\"");
        assert_eq!(integration_json, "\"Integration\"");
    }

    #[test]
    fn test_ai_client_creation() {
        let config = create_test_config(AiProvider::OpenAI);
        let client = AiClient::new(config.clone());
        
        assert_eq!(client.config.provider, AiProvider::OpenAI);
        assert_eq!(client.config.api_url, Some("http://localhost:1234".to_string()));
    }

    #[test]
    fn test_build_comprehensive_prompt() {
        let config = create_test_config(AiProvider::OpenAI);
        let client = AiClient::new(config);
        let request = create_test_request();
        
        let prompt = client.build_comprehensive_prompt(&request);
        
        assert!(prompt.contains("unit tests that test individual functions"));
        assert!(prompt.contains("Rust tests using #[test] attributes"));
        assert!(prompt.contains("Rust"));
        assert!(prompt.contains("fn add"));
        assert!(prompt.contains("Write REAL working test code"));
    }

    #[test]
    fn test_build_prompt_integration_tests() {
        let config = create_test_config(AiProvider::OpenAI);
        let client = AiClient::new(config);
        let mut request = create_test_request();
        request.test_type = TestType::Integration;
        
        let prompt = client.build_comprehensive_prompt(&request);
        
        assert!(prompt.contains("integration tests that test component interactions"));
        assert!(prompt.contains("component interactions"));
    }

    #[test]
    fn test_parse_ai_response_valid_json() {
        let config = create_test_config(AiProvider::OpenAI);
        let client = AiClient::new(config);
        
        let json_response = r##"
        {
            "unit_tests": "#[test] fn test_add() { assert_eq!(add(2, 3), 5); }",
            "integration_tests": null,
            "coverage_estimate": 95.5,
            "framework_used": "cargo-test",
            "edge_cases_covered": ["positive numbers", "zero values", "negative numbers"]
        }
        "##;
        
        let request = create_test_request();
        let result = client.parse_ai_response(json_response, &request).unwrap();
        
        assert!(result.unit_tests.is_some());
        assert!(result.unit_tests.unwrap().contains("test_add"));
        assert_eq!(result.coverage_estimate, 95.5);
        assert_eq!(result.framework_used, "cargo-test");
        assert_eq!(result.edge_cases_covered.len(), 3);
    }

    #[test]
    fn test_parse_ai_response_partial_json() {
        let config = create_test_config(AiProvider::OpenAI);
        let client = AiClient::new(config);
        
        let response_with_text = r##"
        Here are the generated tests:
        
        {
            "unit_tests": "#[test] fn test_example() { assert!(true); }",
            "integration_tests": null,
            "coverage_estimate": 80.0,
            "framework_used": "cargo-test",
            "edge_cases_covered": ["basic case"]
        }
        
        These tests cover the main functionality.
        "##;
        
        let request = create_test_request();
        let result = client.parse_ai_response(response_with_text, &request).unwrap();
        
        assert!(result.unit_tests.is_some());
        assert_eq!(result.coverage_estimate, 80.0);
    }

    #[test]
    fn test_parse_ai_response_no_json() {
        let config = create_test_config(AiProvider::OpenAI);
        let client = AiClient::new(config);
        
        let text_response = "Here are some tests without JSON format";
        
        let request = create_test_request();
        let result = client.parse_ai_response(text_response, &request).unwrap();
        
        assert_eq!(result.unit_tests, Some(text_response.to_string()));
        assert_eq!(result.coverage_estimate, 48.0); // 80.0 * 0.6 for content with no test indicators
        assert_eq!(result.framework_used, "auto-detected");
    }

    #[tokio::test]
    async fn test_call_openai_success() {
        // Set environment variable for the test
        std::env::set_var("OPENAI_API_KEY", "test-api-key");
        
        let mut server = mockito::Server::new_async().await;
        let config = AiConfig {
            provider: AiProvider::OpenAI,
            api_url: Some(server.url()),
            model: Some("gpt-4".to_string()),
        };

        let mock = server.mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({
                "choices": [{
                    "message": {
                        "content": json!({
                            "unit_tests": "#[test] fn test_add() { assert_eq!(add(2, 3), 5); }",
                            "integration_tests": null,
                            "coverage_estimate": 85.0,
                            "framework_used": "cargo-test",
                            "edge_cases_covered": ["positive", "negative", "zero"]
                        }).to_string()
                    }
                }]
            }).to_string())
            .create_async()
            .await;

        let client = AiClient::new(config);
        let request = create_test_request();
        let result = client.call_openai("test prompt", &request).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(response.unit_tests.is_some());
        assert_eq!(response.coverage_estimate, 85.0);
    }

    // Note: Test for missing API key is environment-dependent and can interfere
    // with other tests. The functionality is tested at integration level instead.

    #[tokio::test]
    async fn test_call_anthropic_success() {
        // Set environment variable for the test
        std::env::set_var("ANTHROPIC_API_KEY", "test-api-key");
        
        let mut server = mockito::Server::new_async().await;
        let config = AiConfig {
            provider: AiProvider::Anthropic,
            api_url: Some(server.url()),
            model: Some("claude-3-sonnet".to_string()),
        };

        let mock = server.mock("POST", "/v1/messages")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({
                "content": [{
                    "text": json!({
                        "unit_tests": "test code here",
                        "integration_tests": null,
                        "coverage_estimate": 90.0,
                        "framework_used": "cargo-test",
                        "edge_cases_covered": ["edge case 1"]
                    }).to_string()
                }]
            }).to_string())
            .create_async()
            .await;

        let client = AiClient::new(config);
        let request = create_test_request();
        let result = client.call_anthropic("test prompt", &request).await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_call_ollama_success() {
        let mut server = mockito::Server::new_async().await;
        let config = AiConfig {
            provider: AiProvider::Ollama,
            api_url: Some(server.url()),
            model: Some("codellama".to_string()),
        };

        let mock = server.mock("POST", "/api/generate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({
                "response": json!({
                    "unit_tests": "ollama generated tests",
                    "integration_tests": null,
                    "coverage_estimate": 75.0,
                    "framework_used": "cargo-test",
                    "edge_cases_covered": ["basic"]
                }).to_string()
            }).to_string())
            .create_async()
            .await;

        let client = AiClient::new(config);
        let request = create_test_request();
        let result = client.call_ollama("test prompt", &request).await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_tests_integration() {
        // Set environment variable for the test
        std::env::set_var("OPENAI_API_KEY", "test-api-key");
        
        let mut server = mockito::Server::new_async().await;
        let config = AiConfig {
            provider: AiProvider::OpenAI,
            api_url: Some(server.url()),
            model: Some("gpt-4".to_string()),
        };

        let mock = server.mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({
                "choices": [{
                    "message": {
                        "content": json!({
                            "unit_tests": "#[test] fn test_integration() { /* integration test */ }",
                            "integration_tests": "#[test] fn test_full_flow() { /* full flow test */ }",
                            "coverage_estimate": 92.0,
                            "framework_used": "cargo-test",
                            "edge_cases_covered": ["integration case 1", "integration case 2"]
                        }).to_string()
                    }
                }]
            }).to_string())
            .create_async()
            .await;

        let client = AiClient::new(config);
        let request = TestGenerationRequest {
            source_code: "fn complex_function() -> Result<(), Error> { Ok(()) }".to_string(),
            language: "Rust".to_string(),
            test_type: TestType::Integration,
            framework: "cargo-test".to_string(),
            additional_context: Some("This function handles complex business logic".to_string()),
            edge_case_count: 5,
            coverage_threshold: 90.0,
        };

        let result = client.generate_tests(request).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(response.unit_tests.is_some());
        assert!(response.integration_tests.is_some());
        assert_eq!(response.coverage_estimate, 92.0);
        assert_eq!(response.edge_cases_covered.len(), 2);
    }

    #[test]
    fn test_test_generation_response_equality() {
        let response1 = TestGenerationResponse {
            unit_tests: Some("test1".to_string()),
            integration_tests: None,
            coverage_estimate: 80.0,
            framework_used: "jest".to_string(),
            edge_cases_covered: vec!["case1".to_string()],
        };

        let response2 = TestGenerationResponse {
            unit_tests: Some("test1".to_string()),
            integration_tests: None,
            coverage_estimate: 80.0,
            framework_used: "jest".to_string(),
            edge_cases_covered: vec!["case1".to_string()],
        };

        assert_eq!(response1, response2);
    }

    #[test]
    fn test_additional_context_in_prompt() {
        let config = create_test_config(AiProvider::OpenAI);
        let client = AiClient::new(config);
        
        let mut request = create_test_request();
        request.additional_context = Some("This function is critical for user authentication".to_string());
        
        let prompt = client.build_test_cases_prompt(&request).unwrap();
        
        assert!(prompt.contains("This function is critical for user authentication"));
    }

    #[test]
    fn test_prompt_contains_framework_specific_instructions() {
        let config = create_test_config(AiProvider::OpenAI);
        let client = AiClient::new(config);
        
        let mut request = create_test_request();
        request.framework = "pytest".to_string();
        request.language = "Python".to_string();
        
        let prompt = client.build_comprehensive_prompt(&request);
        
        assert!(prompt.contains("Python tests using pytest framework"));
        assert!(prompt.contains("Python"));
    }
}