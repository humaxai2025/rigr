use rigr::config::{Config, AiConfig, TestingConfig, AiProvider};
use std::fs;

#[test]
fn test_config_creation() {
    let config = Config::new_for_testing(AiProvider::OpenAI);
    
    assert_eq!(config.ai.provider, AiProvider::OpenAI);
    assert!(config.ai.api_url.is_some());
    assert!(config.ai.model.is_some());
    assert_eq!(config.testing.coverage_threshold, 99.0);
}

#[test]
fn test_ai_provider_display() {
    assert_eq!(format!("{}", AiProvider::OpenAI), "OpenAI");
    assert_eq!(format!("{}", AiProvider::Anthropic), "Anthropic Claude");
    assert_eq!(format!("{}", AiProvider::Ollama), "Ollama (Local)");
    assert_eq!(format!("{}", AiProvider::GoogleGemini), "Google Gemini");
    assert_eq!(format!("{}", AiProvider::AzureOpenAI), "Azure OpenAI");
}

#[test]
fn test_testing_config_default() {
    let config = TestingConfig::default();
    
    assert_eq!(config.default_framework, "cargo-test");
    assert_eq!(config.unit_test_dir, "tests/unit");
    assert_eq!(config.integration_test_dir, "tests/integration");
    assert_eq!(config.coverage_threshold, 99.0);
}

#[test]
fn test_config_serialization() {
    let config = Config::new_for_testing(AiProvider::Anthropic);
    
    let serialized = toml::to_string_pretty(&config).expect("Failed to serialize config");
    let deserialized: Config = toml::from_str(&serialized).expect("Failed to deserialize config");
    
    assert_eq!(config.ai.provider, deserialized.ai.provider);
    assert_eq!(config.testing.coverage_threshold, deserialized.testing.coverage_threshold);
}

#[test]
fn test_config_save_and_load() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("config.toml");
    
    let original_config = Config::new_for_testing(AiProvider::GoogleGemini);
    
    // Save config
    original_config.save_to_path(&config_path).expect("Failed to save config");
    
    // Verify file exists
    assert!(config_path.exists(), "Config file should exist after save");
    
    // Load config
    let loaded_config = Config::load_from_path(&config_path).expect("Failed to load config");
    
    assert_eq!(original_config.ai.provider, loaded_config.ai.provider);
    assert_eq!(original_config.testing.coverage_threshold, loaded_config.testing.coverage_threshold);
}

#[test]
fn test_config_load_nonexistent() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let nonexistent_path = temp_dir.path().join("nonexistent_config.toml");
    
    let result = Config::load_from_path(&nonexistent_path);
    assert!(result.is_err(), "Loading nonexistent config should fail");
}

#[test]
fn test_config_load_invalid_toml() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("invalid_config.toml");
    
    // Write invalid TOML
    fs::write(&config_path, "invalid toml content [[[").expect("Failed to write invalid config");
    
    let result = Config::load_from_path(&config_path);
    assert!(result.is_err(), "Loading invalid TOML should fail");
}

#[test]
fn test_config_file_path_for_testing() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    
    let config_path = Config::config_file_path_for_testing(temp_dir.path())
        .expect("Failed to get config path for testing");
    
    assert!(config_path.to_str().unwrap().contains("rigr"));
    assert!(config_path.to_str().unwrap().ends_with("config.toml"));
}

#[test]
fn test_ai_config_equality() {
    let config1 = AiConfig {
        provider: AiProvider::OpenAI,
        api_url: Some("https://api.openai.com/v1".to_string()),
        model: Some("gpt-4".to_string()),
    };
    
    let config2 = AiConfig {
        provider: AiProvider::OpenAI,
        api_url: Some("https://api.openai.com/v1".to_string()),
        model: Some("gpt-4".to_string()),
    };
    
    let config3 = AiConfig {
        provider: AiProvider::Anthropic,
        api_url: Some("https://api.anthropic.com".to_string()),
        model: Some("claude-3-sonnet".to_string()),
    };
    
    assert_eq!(config1, config2);
    assert_ne!(config1, config3);
}

#[test]
fn test_testing_config_custom() {
    let mut config = Config::new_for_testing(AiProvider::Ollama);
    config.testing = TestingConfig {
        default_framework: "jest".to_string(),
        unit_test_dir: "custom/unit".to_string(),
        integration_test_dir: "custom/integration".to_string(),
        coverage_threshold: 95.0,
    };
    
    assert_eq!(config.testing.default_framework, "jest");
    assert_eq!(config.testing.unit_test_dir, "custom/unit");
    assert_eq!(config.testing.integration_test_dir, "custom/integration");
    assert_eq!(config.testing.coverage_threshold, 95.0);
}

#[test]
fn test_all_ai_providers() {
    let providers = [
        AiProvider::OpenAI,
        AiProvider::Anthropic,
        AiProvider::Ollama,
        AiProvider::GoogleGemini,
        AiProvider::AzureOpenAI,
    ];
    
    for provider in providers {
        let config = Config::new_for_testing(provider.clone());
        assert_eq!(config.ai.provider, provider);
        assert!(config.ai.api_url.is_some());
        assert!(config.ai.model.is_some());
        
        // Test that each provider can be displayed
        let _display_string = format!("{provider}");
    }
}