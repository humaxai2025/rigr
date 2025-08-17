use thiserror::Error;

#[derive(Error, Debug)]
pub enum RigrError {
    #[error("Security error: {0}")]
    SecurityError(String),
    #[error("Failed to read file: {0}")]
    FileReadError(String),
    #[error("Failed to parse file: {0}")]
    FileParseError(String),
    #[error("Failed to write file: {0}")]
    FileWriteError(String),
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("HTTP request failed: {0}")]
    HttpRequestError(String),
}

// Removed old TestGenerator trait - focusing only on test case generation
pub mod config;
pub mod ai_client;
pub mod coverage;
pub mod export;