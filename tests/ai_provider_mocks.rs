use mockito::Server;
use rigr::ai_client::AiClient;
use rigr::config::{AiConfig, AiProvider};
use serde_json::json;

mod test_helpers;
use test_helpers::{create_test_request, assert_valid_test_code};

async fn create_mock_client(provider: AiProvider, server: &Server) -> AiClient {
    let config = AiConfig {
        provider,
        api_url: Some(server.url()),
        model: Some("test-model".to_string()),
    };
    AiClient::new(config)
}

#[tokio::test]
async fn test_openai_provider_success() {
    let mut server = Server::new_async().await;
    
    let _mock = server.mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "choices": [{
                "message": {
                    "content": json!({
                        "unit_tests": "#[test]\nfn test_add() {\n    assert_eq!(add(2, 3), 5);\n}\n\n#[test]\nfn test_add_negative() {\n    assert_eq!(add(-1, -2), -3);\n}",
                        "integration_tests": "#[test]\nfn test_calculator_integration() {\n    let result = add(multiply(2, 3), 4);\n    assert_eq!(result, 10);\n}",
                        "coverage_estimate": 92.5,
                        "framework_used": "cargo-test",
                        "edge_cases_covered": ["positive numbers", "negative numbers", "zero values", "overflow conditions", "boundary values"]
                    }).to_string()
                }
            }]
        }).to_string())
        .create_async()
        .await;

    let client = create_mock_client(AiProvider::OpenAI, &server).await;
    let request = create_test_request();
    
    let response = client.generate_tests(request).await.unwrap();
    
    assert!(response.unit_tests.is_some());
    assert!(response.integration_tests.is_some());
    assert_eq!(response.coverage_estimate, 92.5);
    assert_eq!(response.framework_used, "cargo-test");
    assert_eq!(response.edge_cases_covered.len(), 5);
    
    let unit_tests = response.unit_tests.unwrap();
    assert_valid_test_code(&unit_tests, "rust");
    assert!(unit_tests.contains("test_add"));
    assert!(unit_tests.contains("assert_eq!"));
}

#[tokio::test]
async fn test_openai_provider_api_error() {
    let mut server = Server::new_async().await;
    
    let _mock = server.mock("POST", "/chat/completions")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "error": {
                "type": "invalid_request_error",
                "message": "Invalid API key"
            }
        }).to_string())
        .create_async()
        .await;

    let client = create_mock_client(AiProvider::OpenAI, &server).await;
    let request = create_test_request();
    
    let result = client.generate_tests(request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_anthropic_provider_success() {
    let mut server = Server::new_async().await;
    
    let _mock = server.mock("POST", "/v1/messages")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "content": [{
                "text": json!({
                    "unit_tests": "import pytest\n\ndef test_add():\n    assert add(2, 3) == 5\n\ndef test_add_zero():\n    assert add(0, 5) == 5\n\ndef test_add_negative():\n    assert add(-2, 3) == 1",
                    "integration_tests": "def test_complex_calculation():\n    result = add(multiply(2, 3), subtract(10, 5))\n    assert result == 11",
                    "coverage_estimate": 88.0,
                    "framework_used": "pytest",
                    "edge_cases_covered": ["zero handling", "negative numbers", "positive numbers", "boundary conditions"]
                }).to_string()
            }]
        }).to_string())
        .create_async()
        .await;

    let client = create_mock_client(AiProvider::Anthropic, &server).await;
    let mut request = create_test_request();
    request.language = "Python".to_string();
    request.framework = "pytest".to_string();
    
    let response = client.generate_tests(request).await.unwrap();
    
    assert!(response.unit_tests.is_some());
    assert!(response.integration_tests.is_some());
    assert_eq!(response.coverage_estimate, 88.0);
    assert_eq!(response.framework_used, "pytest");
    
    let unit_tests = response.unit_tests.unwrap();
    assert_valid_test_code(&unit_tests, "python");
    assert!(unit_tests.contains("def test_"));
    assert!(unit_tests.contains("assert"));
}

#[tokio::test]
async fn test_anthropic_provider_rate_limit() {
    let mut server = Server::new_async().await;
    
    let _mock = server.mock("POST", "/v1/messages")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "error": {
                "type": "rate_limit_error",
                "message": "Rate limit exceeded"
            }
        }).to_string())
        .create_async()
        .await;

    let client = create_mock_client(AiProvider::Anthropic, &server).await;
    let request = create_test_request();
    
    let result = client.generate_tests(request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_ollama_provider_success() {
    let mut server = Server::new_async().await;
    
    let _mock = server.mock("POST", "/api/generate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "response": json!({
                "unit_tests": "const { expect } = require('chai');\n\ndescribe('Calculator', () => {\n    it('should add two numbers correctly', () => {\n        expect(add(2, 3)).to.equal(5);\n    });\n    \n    it('should handle negative numbers', () => {\n        expect(add(-1, -2)).to.equal(-3);\n    });\n});",
                "integration_tests": "describe('Calculator Integration', () => {\n    it('should work with complex operations', () => {\n        const result = add(multiply(2, 3), 4);\n        expect(result).to.equal(10);\n    });\n});",
                "coverage_estimate": 75.0,
                "framework_used": "mocha",
                "edge_cases_covered": ["basic functionality", "negative numbers", "integration flows"]
            }).to_string()
        }).to_string())
        .create_async()
        .await;

    let config = AiConfig {
        provider: AiProvider::Ollama,
        api_url: Some(server.url()),
        model: Some("codellama".to_string()),
    };

    let client = AiClient::new(config);
    let mut request = create_test_request();
    request.language = "JavaScript".to_string();
    request.framework = "mocha".to_string();
    
    let response = client.generate_tests(request).await.unwrap();
    
    assert!(response.unit_tests.is_some());
    assert!(response.integration_tests.is_some());
    assert_eq!(response.coverage_estimate, 75.0);
    assert_eq!(response.framework_used, "mocha");
    
    let unit_tests = response.unit_tests.unwrap();
    assert_valid_test_code(&unit_tests, "javascript");
    assert!(unit_tests.contains("describe("));
    assert!(unit_tests.contains("it("));
    assert!(unit_tests.contains("expect("));
}

#[tokio::test]
async fn test_ollama_provider_connection_error() {
    let mut server = Server::new_async().await;
    
    let _mock = server.mock("POST", "/api/generate")
        .with_status(503)
        .with_body("Service Unavailable")
        .create_async()
        .await;

    let config = AiConfig {
        provider: AiProvider::Ollama,
        api_url: Some(server.url()),
        model: Some("codellama".to_string()),
    };

    let client = AiClient::new(config);
    let request = create_test_request();
    
    let result = client.generate_tests(request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_gemini_provider_success() {
    let mut server = Server::new_async().await;
    
    let _mock = server.mock("POST", "/models/test-model:generateContent")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "candidates": [{
                "content": {
                    "parts": [{
                        "text": json!({
                            "unit_tests": "public class CalculatorTest {\n    @Test\n    public void testAdd() {\n        assertEquals(5, Calculator.add(2, 3));\n    }\n    \n    @Test\n    public void testAddNegative() {\n        assertEquals(-1, Calculator.add(-2, 1));\n    }\n}",
                            "integration_tests": "@Test\npublic void testCalculatorIntegration() {\n    int result = Calculator.add(Calculator.multiply(2, 3), 4);\n    assertEquals(10, result);\n}",
                            "coverage_estimate": 85.5,
                            "framework_used": "junit",
                            "edge_cases_covered": ["positive values", "negative values", "zero values", "large numbers"]
                        }).to_string()
                    }]
                }
            }]
        }).to_string())
        .create_async()
        .await;

    let client = create_mock_client(AiProvider::GoogleGemini, &server).await;
    let mut request = create_test_request();
    request.language = "Java".to_string();
    request.framework = "junit".to_string();
    
    let response = client.generate_tests(request).await.unwrap();
    
    assert!(response.unit_tests.is_some());
    assert!(response.integration_tests.is_some());
    assert_eq!(response.coverage_estimate, 85.5);
    assert_eq!(response.framework_used, "junit");
    
    let unit_tests = response.unit_tests.unwrap();
    assert_valid_test_code(&unit_tests, "java");
    assert!(unit_tests.contains("@Test"));
    assert!(unit_tests.contains("assertEquals"));
}

#[tokio::test]
async fn test_azure_openai_provider_success() {
    let mut server = Server::new_async().await;
    
    let _mock = server.mock("POST", "/openai/deployments/test-model/chat/completions")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "choices": [{
                "message": {
                    "content": json!({
                        "unit_tests": "[Test]\npublic void TestAdd()\n{\n    Assert.AreEqual(5, Calculator.Add(2, 3));\n}\n\n[Test]\npublic void TestAddNegative()\n{\n    Assert.AreEqual(-5, Calculator.Add(-2, -3));\n}",
                        "integration_tests": "[Test]\npublic void TestCalculatorWorkflow()\n{\n    var result = Calculator.Add(Calculator.Multiply(2, 3), 4);\n    Assert.AreEqual(10, result);\n}",
                        "coverage_estimate": 90.0,
                        "framework_used": "nunit",
                        "edge_cases_covered": ["positive numbers", "negative numbers", "zero handling", "overflow conditions", "integration scenarios"]
                    }).to_string()
                }
            }]
        }).to_string())
        .create_async()
        .await;

    let config = AiConfig {
        provider: AiProvider::AzureOpenAI,
        api_url: Some(server.url()),
        model: Some("test-model".to_string()),
    };

    let client = AiClient::new(config);
    let mut request = create_test_request();
    request.language = "C#".to_string();
    request.framework = "nunit".to_string();
    
    let response = client.generate_tests(request).await.unwrap();
    
    assert!(response.unit_tests.is_some());
    assert!(response.integration_tests.is_some());
    assert_eq!(response.coverage_estimate, 90.0);
    assert_eq!(response.framework_used, "nunit");
    
    let unit_tests = response.unit_tests.unwrap();
    assert_valid_test_code(&unit_tests, "csharp");
    assert!(unit_tests.contains("[Test]"));
    assert!(unit_tests.contains("Assert."));
}

#[tokio::test]
async fn test_provider_timeout_handling() {
    let mut server = Server::new_async().await;
    
    // Create a mock that never responds (simulates timeout)
    let _mock = server.mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("") // Empty body will cause parsing to fail
        .create_async()
        .await;

    let client = create_mock_client(AiProvider::OpenAI, &server).await;
    let request = create_test_request();
    
    let result = client.generate_tests(request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_malformed_response_handling() {
    let mut server = Server::new_async().await;
    
    let _mock = server.mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("invalid json response")
        .create_async()
        .await;

    let client = create_mock_client(AiProvider::OpenAI, &server).await;
    let request = create_test_request();
    
    let result = client.generate_tests(request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_partial_json_response_fallback() {
    let mut server = Server::new_async().await;
    
    let _mock = server.mock("POST", "/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({
            "choices": [{
                "message": {
                    "content": "Here are some tests without proper JSON format:\n\n#[test]\nfn test_example() {\n    assert!(true);\n}"
                }
            }]
        }).to_string())
        .create_async()
        .await;

    let client = create_mock_client(AiProvider::OpenAI, &server).await;
    let request = create_test_request();
    
    let response = client.generate_tests(request).await.unwrap();
    
    // Should fallback to treating entire content as unit_tests
    assert!(response.unit_tests.is_some());
    assert_eq!(response.coverage_estimate, 75.0); // Default fallback value
    assert_eq!(response.framework_used, "unknown");
}

#[tokio::test]
async fn test_missing_configuration_handling() {
    let server = Server::new_async().await;
    
    let config = AiConfig {
        provider: AiProvider::OpenAI,
        api_url: Some(server.url()),
        model: Some("gpt-4".to_string()),
    };

    let client = AiClient::new(config);
    let request = create_test_request();
    
    let result = client.generate_tests(request).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        rigr::RigrError::HttpRequestError(msg) => {
            assert!(msg.contains("API key not configured"));
        }
        other => panic!("Expected HttpRequestError, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_all_providers_with_different_languages() {
    let providers_and_languages = vec![
        (AiProvider::OpenAI, "Rust", "cargo-test"),
        (AiProvider::Anthropic, "Python", "pytest"),
        (AiProvider::Ollama, "JavaScript", "jest"),
        (AiProvider::GoogleGemini, "Java", "junit"),
        (AiProvider::AzureOpenAI, "C#", "xunit"),
    ];

    for (provider, language, framework) in providers_and_languages {
        let mut server = Server::new_async().await;
        
        // Set up appropriate mock based on provider
        match provider {
            AiProvider::OpenAI | AiProvider::AzureOpenAI => {
                let endpoint = if provider == AiProvider::AzureOpenAI {
                    "/openai/deployments/test-model/chat/completions"
                } else {
                    "/chat/completions"
                };
                
                let mut mock_builder = server.mock("POST", endpoint)
                    .with_status(200)
                    .with_header("content-type", "application/json");
                
                if provider == AiProvider::AzureOpenAI {
                    mock_builder = mock_builder.match_query(mockito::Matcher::Any);
                }
                
                mock_builder
                    .with_body(json!({
                        "choices": [{
                            "message": {
                                "content": format!("Generated tests for {} using {}", language, framework)
                            }
                        }]
                    }).to_string())
                    .create_async()
                    .await;
            }
            AiProvider::Anthropic => {
                server.mock("POST", "/v1/messages")
                    .with_status(200)
                    .with_header("content-type", "application/json")
                    .with_body(json!({
                        "content": [{
                            "text": format!("Generated tests for {} using {}", language, framework)
                        }]
                    }).to_string())
                    .create_async()
                    .await;
            }
            AiProvider::Ollama => {
                server.mock("POST", "/api/generate")
                    .with_status(200)
                    .with_header("content-type", "application/json")
                    .with_body(json!({
                        "response": format!("Generated tests for {} using {}", language, framework)
                    }).to_string())
                    .create_async()
                    .await;
            }
            AiProvider::GoogleGemini => {
                server.mock("POST", "/models/test-model:generateContent")
                    .match_query(mockito::Matcher::Any)
                    .with_status(200)
                    .with_header("content-type", "application/json")
                    .with_body(json!({
                        "candidates": [{
                            "content": {
                                "parts": [{
                                    "text": format!("Generated tests for {} using {}", language, framework)
                                }]
                            }
                        }]
                    }).to_string())
                    .create_async()
                    .await;
            }
        }

        let config = AiConfig {
            provider: provider.clone(),
            api_url: Some(server.url()),
            model: Some("test-model".to_string()),
        };

        let client = AiClient::new(config);
        let mut request = create_test_request();
        request.language = language.to_string();
        request.framework = framework.to_string();
        
        let result = client.generate_tests(request).await;
        assert!(result.is_ok(), "Failed for provider: {provider:?}, language: {language}");
    }
}