use crate::RigrError;
use dialoguer::{Select, Input};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AiProvider {
    OpenAI,
    Anthropic,
    Ollama,
    GoogleGemini,
    AzureOpenAI,
}

impl std::fmt::Display for AiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProvider::OpenAI => write!(f, "OpenAI"),
            AiProvider::Anthropic => write!(f, "Anthropic Claude"),
            AiProvider::Ollama => write!(f, "Ollama (Local)"),
            AiProvider::GoogleGemini => write!(f, "Google Gemini"),
            AiProvider::AzureOpenAI => write!(f, "Azure OpenAI"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiConfig {
    pub provider: AiProvider,
    pub api_url: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestingConfig {
    pub default_framework: String,
    pub unit_test_dir: String,
    pub integration_test_dir: String,
    pub coverage_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub ai: AiConfig,
    pub testing: TestingConfig,
}

impl Default for TestingConfig {
    fn default() -> Self {
        Self {
            default_framework: "cargo-test".to_string(),
            unit_test_dir: "tests/unit".to_string(),
            integration_test_dir: "tests/integration".to_string(),
            coverage_threshold: 99.0,
        }
    }
}

impl Config {
    pub fn config_file_path() -> Result<PathBuf, RigrError> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| RigrError::FileReadError("Unable to find home directory".to_string()))?;
        
        let rigr_dir = home_dir.join(".rigr");
        if !rigr_dir.exists() {
            fs::create_dir_all(&rigr_dir)
                .map_err(|e| RigrError::FileWriteError(format!("Failed to create config directory: {e}")))?;
        }
        
        Ok(rigr_dir.join("config.toml"))
    }

    pub fn config_file_path_for_testing(test_dir: &std::path::Path) -> Result<PathBuf, RigrError> {
        let rigr_dir = test_dir.join("rigr");
        if !rigr_dir.exists() {
            fs::create_dir_all(&rigr_dir)
                .map_err(|e| RigrError::FileWriteError(format!("Failed to create config directory: {e}")))?;
        }
        
        Ok(rigr_dir.join("config.toml"))
    }

    pub fn load() -> Result<Self, RigrError> {
        let config_path = Self::config_file_path()?;
        Self::load_from_path(&config_path)
    }

    pub fn load_from_path(config_path: &PathBuf) -> Result<Self, RigrError> {
        if !config_path.exists() {
            return Err(RigrError::FileReadError(
                format!("Configuration file not found at: {}", config_path.display())
            ));
        }

        let content = fs::read_to_string(config_path)
            .map_err(|e| RigrError::FileReadError(format!("Failed to read config file at {}: {e}", config_path.display())))?;
        
        toml::from_str(&content)
            .map_err(|e| RigrError::FileParseError(format!("Failed to parse config file at {}: {e}", config_path.display())))
    }

    pub fn save(&self) -> Result<(), RigrError> {
        let config_path = Self::config_file_path()?;
        self.save_to_path(&config_path)
    }

    pub fn save_to_path(&self, config_path: &PathBuf) -> Result<(), RigrError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to serialize config: {e}")))?;
        
        fs::write(config_path, content)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to write config file: {e}")))?;
        
        println!("Configuration saved to: {}", config_path.display());
        Ok(())
    }

    pub fn setup_interactive() -> Result<Self, RigrError> {
        println!("🚀 Welcome to Rigr AI Test Generator Setup!");
        println!("Let's configure your AI provider and testing preferences.\n");

        let providers = [
            AiProvider::OpenAI,
            AiProvider::Anthropic,
            AiProvider::Ollama,
            AiProvider::GoogleGemini,
            AiProvider::AzureOpenAI,
        ];

        let provider_names: Vec<String> = providers.iter().map(|p| p.to_string()).collect();
        
        let provider_index = Select::new()
            .with_prompt("Select your AI provider")
            .items(&provider_names)
            .default(2) // Default to Ollama (index 2)
            .interact()
            .map_err(|e| RigrError::FileParseError(format!("Selection failed: {e}")))?;

        let provider = providers[provider_index].clone();

        // Display environment variable instructions for API keys
        match provider {
            AiProvider::OpenAI => {
                println!("\n📋 For OpenAI, set the environment variable:");
                println!("   OPENAI_API_KEY=your_api_key_here");
                println!("   Get your key from: https://platform.openai.com/api-keys");
            },
            AiProvider::Anthropic => {
                println!("\n📋 For Anthropic, set the environment variable:");
                println!("   ANTHROPIC_API_KEY=your_api_key_here");
                println!("   Get your key from: https://console.anthropic.com/");
            },
            AiProvider::GoogleGemini => {
                println!("\n📋 For Google Gemini, set the environment variable:");
                println!("   GOOGLE_AI_API_KEY=your_api_key_here");
                println!("   Get your key from: https://aistudio.google.com/");
            },
            AiProvider::AzureOpenAI => {
                println!("\n📋 For Azure OpenAI, set the environment variable:");
                println!("   AZURE_OPENAI_API_KEY=your_api_key_here");
                println!("   Get your key from Azure Portal");
            },
            AiProvider::Ollama => {
                println!("\n📋 Ollama runs locally - no API key required!");
            },
        }
        
        let (api_url, model) = match provider {
            AiProvider::OpenAI => {
                let model = Input::new()
                    .with_prompt("Enter model name (default: gpt-4)")
                    .default("gpt-4".to_string())
                    .interact()
                    .map_err(|e| RigrError::FileParseError(format!("Input failed: {e}")))?;

                (Some("https://api.openai.com/v1".to_string()), Some(model))
            },
            AiProvider::Anthropic => {
                let model = Input::new()
                    .with_prompt("Enter model name (default: claude-3-sonnet-20240229)")
                    .default("claude-3-sonnet-20240229".to_string())
                    .interact()
                    .map_err(|e| RigrError::FileParseError(format!("Input failed: {e}")))?;

                (Some("https://api.anthropic.com".to_string()), Some(model))
            },
            AiProvider::Ollama => {
                let url = Input::new()
                    .with_prompt("Enter Ollama URL (default: http://localhost:11434)")
                    .default("http://localhost:11434".to_string())
                    .interact()
                    .map_err(|e| RigrError::FileParseError(format!("Input failed: {e}")))?;
                
                let model = Input::new()
                    .with_prompt("Enter model name (default: codellama)")
                    .default("codellama".to_string())
                    .interact()
                    .map_err(|e| RigrError::FileParseError(format!("Input failed: {e}")))?;

                (Some(url), Some(model))
            },
            AiProvider::GoogleGemini => {
                let model = Input::new()
                    .with_prompt("Enter model name (default: gemini-pro)")
                    .default("gemini-pro".to_string())
                    .interact()
                    .map_err(|e| RigrError::FileParseError(format!("Input failed: {e}")))?;

                (Some("https://generativelanguage.googleapis.com/v1".to_string()), Some(model))
            },
            AiProvider::AzureOpenAI => {
                let endpoint = Input::new()
                    .with_prompt("Enter your Azure OpenAI endpoint")
                    .interact()
                    .map_err(|e| RigrError::FileParseError(format!("Input failed: {e}")))?;
                
                let model = Input::new()
                    .with_prompt("Enter deployment name")
                    .interact()
                    .map_err(|e| RigrError::FileParseError(format!("Input failed: {e}")))?;

                (Some(endpoint), Some(model))
            },
        };

        let unit_test_dir = Input::new()
            .with_prompt("Directory for unit tests (default: tests/unit)")
            .default("tests/unit".to_string())
            .interact()
            .map_err(|e| RigrError::FileParseError(format!("Input failed: {e}")))?;

        let integration_test_dir = Input::new()
            .with_prompt("Directory for integration tests (default: tests/integration)")
            .default("tests/integration".to_string())
            .interact()
            .map_err(|e| RigrError::FileParseError(format!("Input failed: {e}")))?;

        let coverage_threshold: f64 = Input::new()
            .with_prompt("Target coverage threshold % (default: 80)")
            .default(80.0)
            .interact()
            .map_err(|e| RigrError::FileParseError(format!("Input failed: {e}")))?;

        let config = Config {
            ai: AiConfig {
                provider,
                api_url,
                model,
            },
            testing: TestingConfig {
                default_framework: "auto".to_string(),
                unit_test_dir,
                integration_test_dir,
                coverage_threshold,
            },
        };

        config.save()?;
        println!("\n✅ Configuration completed successfully!");
        
        Ok(config)
    }

    // For testing purposes
    pub fn new_for_testing(provider: AiProvider) -> Self {
        Config {
            ai: AiConfig {
                provider,
                api_url: Some("http://test.example.com".to_string()),
                model: Some("test-model".to_string()),
            },
            testing: TestingConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

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
        let config = Config::new_for_testing(AiProvider::OpenAI);

        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let config_path = Config::config_file_path_for_testing(&temp_dir.path().to_path_buf()).unwrap();
        
        let original_config = Config::new_for_testing(AiProvider::Anthropic);

        // Save config
        original_config.save_to_path(&config_path).unwrap();
        
        // Load config
        let loaded_config = Config::load_from_path(&config_path).unwrap();
        
        assert_eq!(original_config, loaded_config);
    }

    #[test]
    fn test_config_load_nonexistent_file() {
        let temp_dir = tempdir().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent").join("config.toml");
        
        let result = Config::load_from_path(&nonexistent_path);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            RigrError::FileReadError(msg) => {
                assert!(msg.contains("Configuration file not found"));
            },
            _ => panic!("Expected FileReadError"),
        }
    }

    #[test]
    fn test_config_load_invalid_toml() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("invalid_config.toml");
        
        fs::write(&config_path, "invalid toml content [[[").unwrap();
        
        let result = Config::load_from_path(&config_path);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            RigrError::FileParseError(msg) => {
                assert!(msg.contains("Failed to parse config file"));
            },
            _ => panic!("Expected FileParseError"),
        }
    }

    #[test]
    fn test_config_different_providers() {
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
            assert_eq!(config.ai.api_url, Some("http://test.example.com".to_string()));
        }
    }

    #[test]
    fn test_config_ollama_provider() {
        let config = Config::new_for_testing(AiProvider::Ollama);
        assert_eq!(config.ai.provider, AiProvider::Ollama);
        assert_eq!(config.ai.api_url, Some("http://test.example.com".to_string()));
    }

    #[test]
    fn test_config_custom_testing_config() {
        let mut config = Config::new_for_testing(AiProvider::OpenAI);
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
    fn test_ai_config_equality() {
        let config1 = AiConfig {
            provider: AiProvider::OpenAI,
            api_url: Some("url1".to_string()),
            model: Some("model1".to_string()),
        };

        let config2 = AiConfig {
            provider: AiProvider::OpenAI,
            api_url: Some("url1".to_string()),
            model: Some("model1".to_string()),
        };

        let config3 = AiConfig {
            provider: AiProvider::Anthropic,
            api_url: Some("url1".to_string()),
            model: Some("model1".to_string()),
        };

        assert_eq!(config1, config2);
        assert_ne!(config1, config3);
    }
}