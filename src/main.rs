use clap::Parser;
use rigr::config::Config;
use rigr::ai_client::{AiClient, TestGenerationRequest, TestType};
use rigr::export::{TestCaseExporter, ExportFormat};
use rigr::RigrError;
use std::path::Path;
use std::fs;
use ignore::WalkBuilder;
use futures::future::join_all;
use std::sync::Arc;
use calamine::{Reader, Xlsx, open_workbook};
use docx_rs::*;
use lopdf::Document as PdfDocument;
use url::Url;
use git2::Repository;
use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use regex::Regex;

/// Initialize logging system based on user preferences
fn init_logging(verbose: bool, debug: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Create logs directory if it doesn't exist
    std::fs::create_dir_all("logs")?;
    
    // Create file appender for structured logging
    let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "rigr.log");
    
    // Determine log level
    let log_level = if debug {
        "rigr=debug,warn"
    } else if verbose {
        "rigr=info,warn"
    } else {
        "rigr=warn,error"
    };
    
    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
                .compact()
                .with_filter(
                    if verbose || debug {
                        EnvFilter::try_from_default_env()
                            .unwrap_or_else(|_| EnvFilter::new(log_level))
                    } else {
                        EnvFilter::new("warn,error")
                    }
                )
        )
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(file_appender)
                .with_filter(EnvFilter::new("rigr=debug"))
        )
        .init();
    
    debug!("Logging system initialized");
    Ok(())
}

/// Display user-friendly progress message
fn show_progress(message: &str) {
    println!("🔄 {message}");
}

/// Display success message
fn show_success(message: &str) {
    println!("✅ {message}");
}

/// Display warning message
fn show_warning(message: &str) {
    println!("⚠️  {message}");
}

/// Display error message
fn show_error(message: &str) {
    println!("❌ {message}");
}

/// Display information message
fn show_info(message: &str) {
    println!("ℹ️  {message}");
}

/// Validate file path to prevent path traversal attacks
fn validate_file_path(path_str: &str) -> Result<std::path::PathBuf, RigrError> {
    let path = std::path::Path::new(path_str);
    
    // Reject paths with traversal attempts
    if path_str.contains("..") {
        return Err(RigrError::SecurityError("Path traversal detected".to_string()));
    }
    
    // Canonicalize path to resolve any remaining traversal attempts
    let canonical = path.canonicalize()
        .map_err(|_| RigrError::SecurityError("Invalid or inaccessible file path".to_string()))?;
    
    // Ensure the file exists and is readable
    if !canonical.exists() {
        return Err(RigrError::FileReadError("File does not exist".to_string()));
    }
    
    if !canonical.is_file() {
        return Err(RigrError::FileReadError("Path is not a file".to_string()));
    }
    
    Ok(canonical)
}

/// Validate source code content for potentially dangerous patterns
fn validate_source_code(content: &str) -> Result<(), RigrError> {
    // Check file size limits (prevent DoS)
    const MAX_FILE_SIZE: usize = 50 * 1024 * 1024; // 50MB limit
    if content.len() > MAX_FILE_SIZE {
        return Err(RigrError::SecurityError("File size exceeds maximum allowed limit".to_string()));
    }
    
    // Check for suspicious patterns that could indicate malicious code
    let suspicious_patterns = [
        r"system\s*\(",
        r"exec\s*\(",
        r"eval\s*\(",
        r"subprocess\.",
        r"__import__\s*\(",
        r"os\.system",
        r"Runtime\.exec",
        r"ProcessBuilder",
        r"cmd\.exe",
        r"powershell",
        r"/bin/sh",
        r"/bin/bash",
        r"rm\s+-rf\s+/",
        r"del\s+/[sf]\s+",
    ];
    
    for pattern in &suspicious_patterns {
        let regex = Regex::new(pattern).map_err(|_| RigrError::SecurityError("Regex compilation failed".to_string()))?;
        if regex.is_match(content) {
            warn!("Potentially unsafe code pattern detected: {}", pattern);
            // Log but don't block - many legitimate programs might contain these patterns
            // In a stricter environment, you might want to return an error here
        }
    }
    
    Ok(())
}


/// AI-Powered Comprehensive Test Case Generator
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run the setup wizard to configure AI provider
    #[arg(long)]
    setup: bool,

    /// Path to the source file or directory to analyze
    #[arg(short, long)]
    file: Option<String>,
    /// Path to requirements file(s) (txt, docx, pdf, xlsx) to generate tests from
    #[arg(short, long)]
    requirements: Option<String>,

    /// GitHub repository URL to clone and analyze
    #[arg(short, long)]
    github: Option<String>,

    /// Directory to analyze (recursive)
    #[arg(short, long)]
    directory: Option<String>,

    /// Generate only unit test cases
    #[arg(long)]
    unit_only: bool,

    /// Generate only integration test cases
    #[arg(long)]
    integration_only: bool,

    /// Output directory for generated test cases
    #[arg(short, long)]
    output: Option<String>,

    /// Number of edge cases to generate (default: 10)
    #[arg(long, default_value = "10")]
    edge_cases: usize,

    /// Maximum concurrent file processing (default: 2)
    #[arg(long, default_value = "2")]
    concurrency: usize,

    /// Include performance test cases
    #[arg(long)]
    include_performance: bool,

    /// Include security test cases
    #[arg(long)]
    include_security: bool,

    /// Export test cases to standard formats (csv,json,testrail,xray,azure,allure,junit,github,cucumber,excel,zephyr,testlink,confluence,postman,robot)
    #[arg(long, value_delimiter = ',')]
    export: Option<Vec<String>>,

    /// Enable verbose output with detailed logging
    #[arg(short, long)]
    verbose: bool,

    /// Enable debug logging (most detailed)
    #[arg(long)]
    debug: bool,
}

/// Represents the content of a requirements document
#[derive(Debug, Clone)]
struct RequirementsDocument {
    content: String,
    file_path: String,
    file_type: String,
}

/// Parse requirements document based on file extension
fn parse_requirements_document(file_path: &str) -> Result<RequirementsDocument, RigrError> {
    let path = Path::new(file_path);
    let extension = path.extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| RigrError::FileReadError("Invalid file extension".to_string()))?
        .to_lowercase();

    let content = match extension.as_str() {
        "txt" => parse_text_file(file_path)?,
        "docx" => parse_docx_file(file_path)?,
        "pdf" => parse_pdf_file(file_path)?,
        "xlsx" | "xls" => parse_excel_file(file_path)?,
        _ => return Err(RigrError::FileReadError(
            format!("Unsupported file format: {extension}")
        )),
    };

    Ok(RequirementsDocument {
        content,
        file_path: file_path.to_string(),
        file_type: extension,
    })
}

/// Parse plain text file with security validation
fn parse_text_file(file_path: &str) -> Result<String, RigrError> {
    let validated_path = validate_file_path(file_path)?;
    let content = fs::read_to_string(&validated_path)
        .map_err(|e| RigrError::FileReadError(format!("Failed to read text file: {e}")))?;
    
    validate_source_code(&content)?;
    Ok(content)
}

/// Parse DOCX file
fn parse_docx_file(file_path: &str) -> Result<String, RigrError> {
    let file_data = fs::read(file_path)
        .map_err(|e| RigrError::FileReadError(format!("Failed to read DOCX file: {e}")))?;
    
    let docx = read_docx(&file_data)
        .map_err(|e| RigrError::FileReadError(format!("Failed to parse DOCX: {e}")))?;
    
    let mut content = String::new();
    for child in docx.document.children {
        if let DocumentChild::Paragraph(p) = child {
            for run in p.children {
                if let ParagraphChild::Run(r) = run {
                    for child in r.children {
                        if let RunChild::Text(text) = child {
                            content.push_str(&text.text);
                        }
                    }
                }
            }
            content.push('\n');
        }
    }
    
    Ok(content)
}

/// Parse PDF file
fn parse_pdf_file(file_path: &str) -> Result<String, RigrError> {
    let doc = PdfDocument::load(file_path)
        .map_err(|e| RigrError::FileReadError(format!("Failed to load PDF: {e}")))?;
    
    let mut content = String::new();
    
    for page_num in 1..=doc.get_pages().len() {
        match doc.extract_text(&[page_num as u32]) {
            Ok(text) => {
                content.push_str(&text);
                content.push('\n');
            }
            Err(e) => {
                println!("Warning: Failed to extract text from page {page_num}: {e}");
            }
        }
    }
    
    if content.trim().is_empty() {
        return Err(RigrError::FileReadError("No text content found in PDF".to_string()));
    }
    
    Ok(content)
}

/// Validate and parse GitHub URL with enhanced security
fn validate_github_url(url_str: &str) -> Result<String, RigrError> {
    // Basic input validation
    if url_str.len() > 2048 {
        return Err(RigrError::SecurityError("URL too long".to_string()));
    }
    
    let url = Url::parse(url_str)
        .map_err(|_| RigrError::SecurityError("Invalid URL format".to_string()))?;
    
    // Strict validation - only HTTPS allowed
    if url.scheme() != "https" {
        return Err(RigrError::SecurityError("Only HTTPS URLs are allowed".to_string()));
    }
    
    // Exact host matching to prevent subdomain attacks
    match url.host_str() {
        Some("github.com") => {},
        Some("www.github.com") => {},
        _ => return Err(RigrError::SecurityError("Only github.com URLs are allowed".to_string())),
    }
    
    // Validate path format and prevent malicious paths
    let path = url.path();
    if path.contains("..") || path.contains("//") || path.is_empty() || path == "/" {
        return Err(RigrError::SecurityError("Invalid repository path".to_string()));
    }
    
    // Ensure it's a proper repository path format (/owner/repo)
    let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if path_segments.len() < 2 || path_segments[0].is_empty() || path_segments[1].is_empty() {
        return Err(RigrError::SecurityError("Invalid repository path format".to_string()));
    }
    
    // Validate owner and repo names (GitHub naming rules)
    let owner = path_segments[0];
    let repo = path_segments[1].trim_end_matches(".git");
    
    if !is_valid_github_name(owner) || !is_valid_github_name(repo) {
        return Err(RigrError::SecurityError("Invalid repository or owner name".to_string()));
    }
    
    // Convert web URL to git clone URL if necessary
    let clone_url = if url_str.ends_with(".git") {
        url_str.to_string()
    } else {
        format!("{}.git", url_str.trim_end_matches('/'))
    };
    
    info!("Validated GitHub URL: {}", clone_url);
    Ok(clone_url)
}

/// Validate GitHub repository/owner names according to GitHub rules
fn is_valid_github_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 39 {
        return false;
    }
    
    // GitHub usernames and repo names can contain alphanumeric chars, hyphens, and dots
    // But cannot start or end with hyphens or dots
    if name.starts_with('-') || name.ends_with('-') || 
       name.starts_with('.') || name.ends_with('.') {
        return false;
    }
    
    // Only allow alphanumeric, hyphens, underscores, and dots
    name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// Clone GitHub repository to a temporary directory
async fn clone_github_repository(github_url: &str) -> Result<String, RigrError> {
    let clone_url = validate_github_url(github_url)?;
    
    // Extract repository name from URL for directory naming
    let url = Url::parse(&clone_url)
        .map_err(|_| RigrError::FileReadError("Invalid clone URL".to_string()))?;
    let repo_name = url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .unwrap_or("repo")
        .trim_end_matches(".git");
    
    // Create temporary directory for cloning
    let temp_dir = std::env::temp_dir();
    let clone_dir = temp_dir.join(format!("rigr_clone_{repo_name}"));
    
    // Remove existing directory if it exists
    if clone_dir.exists() {
        fs::remove_dir_all(&clone_dir)
            .map_err(|e| RigrError::FileReadError(format!("Failed to remove existing clone directory: {e}")))?;
    }
    
    show_progress(&format!("Cloning repository: {github_url}"));
    debug!("Clone directory: {}", clone_dir.display());
    info!("Cloning repository to: {}", clone_dir.display());
    
    // Clone the repository
    Repository::clone(&clone_url, &clone_dir)
        .map_err(|e| RigrError::FileReadError(format!("Failed to clone repository: {e}")))?;
    
    show_success("Repository cloned successfully");
    info!("Repository cloned successfully");
    
    Ok(clone_dir.display().to_string())
}

/// Process GitHub repository - clone and analyze
async fn process_github_repository(github_url: &str, config: &Config, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    // Clone the repository
    let clone_dir = clone_github_repository(github_url).await?;
    
    show_progress("Analyzing cloned repository...");
    info!("Starting repository analysis");
    
    // Create a new args struct with the cloned directory
    let mut repo_args = args.clone();
    repo_args.directory = Some(clone_dir.clone());
    repo_args.file = None; // Clear file option to avoid conflicts
    
    // Collect source files from the cloned repository
    let source_files = match collect_source_files(&repo_args) {
        Ok(files) => files,
        Err(e) => {
            show_error("Failed to collect source files from repository");
            error!("Error collecting source files from repository: {}", e);
            // Clean up cloned directory
            if let Err(cleanup_err) = fs::remove_dir_all(&clone_dir) {
                warn!("Failed to cleanup clone directory: {}", cleanup_err);
            }
            return Err(e.into());
        }
    };
    
    if source_files.is_empty() {
        show_warning("No supported source files found in the repository");
        warn!("No supported source files found in repository");
        // Clean up cloned directory
        if let Err(cleanup_err) = fs::remove_dir_all(&clone_dir) {
            warn!("Failed to cleanup clone directory: {}", cleanup_err);
        }
        return Ok(());
    }
    
    show_info(&format!("Found {} source file(s) to analyze", source_files.len()));
    debug!("Processing with concurrency limit: {}", args.concurrency);
    info!("Found {} files in repository", source_files.len());
    
    // Process files concurrently in batches (same as regular file processing)
    let config = Arc::new(config.clone());
    let args = Arc::new(repo_args);
    
    // Create a semaphore to limit concurrency
    let semaphore = Arc::new(tokio::sync::Semaphore::new(args.concurrency));
    
    // Process files in smaller batches to improve memory efficiency
    let batch_size = args.concurrency * 2;
    let mut processed_files = Vec::new();
    
    for batch in source_files.chunks(batch_size) {
        let batch_tasks: Vec<_> = batch.iter().map(|file_path| {
            let config = Arc::clone(&config);
            let args = Arc::clone(&args);
            let semaphore = Arc::clone(&semaphore);
            let file_path = file_path.clone();
            
            tokio::spawn(async move {
                // Acquire semaphore permit to limit concurrency
                let _permit = semaphore.acquire().await.unwrap();
                
                match process_file(&file_path, &config, &args).await {
                    Ok(_) => Some(file_path),
                    Err(e) => {
                        eprintln!("❌ Error processing {file_path}: {e}");
                        // Exit immediately for unsupported language errors
                        if matches!(e, RigrError::UnsupportedLanguage(_)) {
                            std::process::exit(1);
                        }
                        None
                    }
                }
            })
        }).collect();
        
        // Wait for current batch to complete
        let batch_results = join_all(batch_tasks).await;
        
        // Collect successful results
        for file_path in batch_results.into_iter().flatten().flatten() {
            processed_files.push(file_path);
        }
    }
    
    // Generate comprehensive test case summary report
    if !processed_files.is_empty() {
        println!("\n📊 Generating comprehensive test case summary...");
        match generate_test_case_summary(&processed_files, &config, &args).await {
            Ok(_) => println!("✅ Test case summary generation completed"),
            Err(e) => eprintln!("❌ Test case summary generation failed: {e}"),
        }
        
        // Export test cases to standard formats if requested
        if let Some(ref export_formats) = args.export {
            println!("\n📤 Exporting test cases to standard formats...");
            match export_test_cases_to_standard_formats(&processed_files, &config, &args, export_formats).await {
                Ok(_) => println!("✅ Test case export completed"),
                Err(e) => eprintln!("❌ Test case export failed: {e}"),
            }
        }
    }
    
    println!("\n🎉 GitHub repository analysis completed successfully!");
    println!("📁 Repository: {github_url}");
    println!("📁 Check the 'RigrTestCases' directory for all generated files:");
    println!("   📝 UnitTestCases/ - Comprehensive unit test case descriptions");
    println!("   🔗 IntegrationTestCases/ - Integration test case descriptions");
    println!("   📊 test_case_summary.html - Complete test case analysis dashboard");
    
    if args.include_performance {
        println!("   ⚡ PerformanceTestCases/ - Performance test scenarios");
    }
    
    if args.include_security {
        println!("   🔒 SecurityTestCases/ - Security test scenarios");
    }
    
    // Clean up cloned directory
    println!("\n🧹 Cleaning up temporary clone directory...");
    if let Err(cleanup_err) = fs::remove_dir_all(&clone_dir) {
        eprintln!("⚠️ Failed to cleanup clone directory: {cleanup_err}");
    } else {
        println!("✅ Cleanup completed");
    }
    
    Ok(())
}

/// Parse Excel file
fn parse_excel_file(file_path: &str) -> Result<String, RigrError> {
    let mut workbook: Xlsx<_> = open_workbook(file_path)
        .map_err(|e| RigrError::FileReadError(format!("Failed to open Excel file: {e}")))?;
    
    let mut content = String::new();
    
    // Get all worksheet names
    let sheet_names = workbook.sheet_names().to_owned();
    
    for sheet_name in sheet_names {
        content.push_str(&format!("=== Worksheet: {sheet_name} ===\n"));
        
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            for row in range.rows() {
                let row_text: Vec<String> = row.iter()
                    .map(|cell| format!("{cell}"))
                    .collect();
                content.push_str(&row_text.join("\t"));
                content.push('\n');
            }
        }
        content.push('\n');
    }
    
    Ok(content)
}

/// Detect language from requirements content
fn detect_language_from_requirements(content: &str) -> String {
    // Analyze content to determine the most appropriate test format
    let content_lower = content.to_lowercase();
    
    if content_lower.contains("api") || content_lower.contains("endpoint") || content_lower.contains("rest") {
        "API Requirements".to_string()
    } else if content_lower.contains("database") || content_lower.contains("sql") || content_lower.contains("table") {
        "Database Requirements".to_string()
    } else if content_lower.contains("ui") || content_lower.contains("user interface") || content_lower.contains("screen") {
        "UI Requirements".to_string()
    } else if content_lower.contains("function") || content_lower.contains("method") || content_lower.contains("algorithm") {
        "Functional Requirements".to_string()
    } else {
        "General Requirements".to_string()
    }
}

/// Process requirements file and generate test cases
async fn process_requirements_file(requirements_path: &str, config: &Config, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    // Parse the requirements document
    let requirements_doc = parse_requirements_document(requirements_path)?;
    
    debug!("File type: {}", requirements_doc.file_type);
    debug!("Content length: {} characters", requirements_doc.content.len());
    info!("Processing {} file with {} characters", requirements_doc.file_type, requirements_doc.content.len());
    
    // Detect the type of requirements
    let language = detect_language_from_requirements(&requirements_doc.content);
    show_info(&format!("Detected requirements type: {language}"));
    info!("Detected requirements type: {}", language);
    
    // Extract filename without extension for workspace naming
    let file_stem = Path::new(requirements_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("requirements");
    
    // Generate test cases based on requirements
    let mut processed_files = Vec::new();
    
    // Create AI client
    let ai_client = AiClient::new(config.ai.clone());
    
    if !args.integration_only {
        show_progress("Generating unit test cases from requirements...");
        info!("Starting unit test case generation");
        let unit_request = TestGenerationRequest {
            source_code: requirements_doc.content.clone(),
            language: language.clone(),
            test_type: TestType::Unit,
            framework: "Requirements-based Testing".to_string(),
            additional_context: Some(format!("Requirements from: {}", requirements_doc.file_path)),
            edge_case_count: args.edge_cases,
            coverage_threshold: 80.0,
        };
        
        match ai_client.generate_test_cases(unit_request).await {
            Ok(test_cases) => {
                let output_path = save_test_cases_to_file(&test_cases, file_stem, &TestType::Unit, args)?;
                show_success(&format!("Unit test cases saved to: {output_path}"));
                info!("Unit test cases generated successfully");
            }
            Err(e) => {
                show_error("Failed to generate unit test cases");
                error!("Unit test case generation failed: {}", e);
            }
        }
    }
    
    if !args.unit_only {
        show_progress("Generating integration test cases from requirements...");
        info!("Starting integration test case generation");
        let integration_request = TestGenerationRequest {
            source_code: requirements_doc.content.clone(),
            language: language.clone(),
            test_type: TestType::Integration,
            framework: "Requirements-based Testing".to_string(),
            additional_context: Some(format!("Requirements from: {}", requirements_doc.file_path)),
            edge_case_count: args.edge_cases,
            coverage_threshold: 80.0,
        };
        
        match ai_client.generate_test_cases(integration_request).await {
            Ok(test_cases) => {
                let output_path = save_test_cases_to_file(&test_cases, file_stem, &TestType::Integration, args)?;
                show_success(&format!("Integration test cases saved to: {output_path}"));
                info!("Integration test cases generated successfully");
            }
            Err(e) => {
                show_error("Failed to generate integration test cases");
                error!("Integration test case generation failed: {}", e);
            }
        }
    }
    
    if args.include_performance {
        show_progress("Generating performance test cases from requirements...");
        info!("Starting performance test case generation");
        let performance_request = TestGenerationRequest {
            source_code: requirements_doc.content.clone(),
            language: language.clone(),
            test_type: TestType::Unit, // Use Unit type but specify performance context
            framework: "Requirements-based Performance Testing".to_string(),
            additional_context: Some(format!("Performance test requirements from: {}", requirements_doc.file_path)),
            edge_case_count: args.edge_cases / 2, // Fewer performance test cases
            coverage_threshold: 80.0,
        };
        
        match ai_client.generate_test_cases(performance_request).await {
            Ok(test_cases) => {
                let output_path = save_test_cases_to_file_with_type(&test_cases, file_stem, "performance", args)?;
                show_success(&format!("Performance test cases saved to: {output_path}"));
                info!("Performance test cases generated successfully");
            }
            Err(e) => {
                show_error("Failed to generate performance test cases");
                error!("Performance test case generation failed: {}", e);
            }
        }
    }
    
    if args.include_security {
        show_progress("Generating security test cases from requirements...");
        info!("Starting security test case generation");
        let security_request = TestGenerationRequest {
            source_code: requirements_doc.content.clone(),
            language: language.clone(),
            test_type: TestType::Integration, // Use Integration type for security
            framework: "Requirements-based Security Testing".to_string(),
            additional_context: Some(format!("Security test requirements from: {}", requirements_doc.file_path)),
            edge_case_count: args.edge_cases / 2, // Fewer security test cases
            coverage_threshold: 80.0,
        };
        
        match ai_client.generate_test_cases(security_request).await {
            Ok(test_cases) => {
                let output_path = save_test_cases_to_file_with_type(&test_cases, file_stem, "security", args)?;
                show_success(&format!("Security test cases saved to: {output_path}"));
                info!("Security test cases generated successfully");
            }
            Err(e) => {
                show_error("Failed to generate security test cases");
                error!("Security test case generation failed: {}", e);
            }
        }
    }
    
    processed_files.push(requirements_doc.file_path.clone());
    
    // Generate comprehensive test case summary
    if !processed_files.is_empty() {
        println!("\n📊 Generating comprehensive test case summary...");
        match generate_test_case_summary(&processed_files, config, args).await {
            Ok(_) => println!("✅ Test case summary generation completed"),
            Err(e) => eprintln!("❌ Test case summary generation failed: {e}"),
        }
    }
    
    // Export test cases if requested
    if let Some(export_formats) = &args.export {
        println!("\n📤 Exporting test cases...");
        match export_test_cases_to_standard_formats(&processed_files, config, args, export_formats).await {
            Ok(_) => println!("✅ Test case export completed"),
            Err(e) => eprintln!("❌ Test case export failed: {e}"),
        }
    }
    
    println!("\n🎉 Requirements-based test case generation completed successfully!");
    println!("📁 Check the 'RigrTestCases' directory for all generated files:");
    println!("   📝 UnitTestCases/ - Comprehensive unit test case descriptions");
    if !args.unit_only {
        println!("   🔗 IntegrationTestCases/ - Integration test case descriptions");
    }
    if args.include_performance {
        println!("   ⚡ PerformanceTestCases/ - Performance test scenarios");
    }
    if args.include_security {
        println!("   🔒 SecurityTestCases/ - Security test scenarios");
    }
    println!("   📊 test_case_summary.html - Complete test case analysis dashboard");
    
    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize logging system
    if let Err(e) = init_logging(args.verbose, args.debug) {
        eprintln!("Warning: Failed to initialize logging: {e}");
    }

    info!("Rigr started with args: {:?}", args);

    if args.setup {
        show_progress("Configuring AI provider settings...");
        match Config::setup_interactive() {
            Ok(_) => {
                show_success("Setup completed! You can now generate comprehensive test cases.");
                info!("Setup completed successfully");
                return;
            }
            Err(e) => {
                show_error(&format!("Setup failed: {e}"));
                error!("Setup failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    show_progress("Loading configuration...");
    let config = match Config::load() {
        Ok(config) => {
            debug!("Configuration loaded successfully");
            config
        },
        Err(e) => {
            show_error("Configuration not found or invalid");
            show_info("Please run 'rigr --setup' to configure your AI provider");
            error!("Configuration load failed: {}", e);
            std::process::exit(1);
        }
    };

    // Handle requirements files if specified (priority over source files)
    if let Some(requirements_path) = &args.requirements {
        show_progress(&format!("Processing requirements from: {requirements_path}"));
        info!("Processing requirements file: {}", requirements_path);
        match process_requirements_file(requirements_path, &config, &args).await {
            Ok(_) => {
                show_success("Requirements processing completed successfully");
                return;
            },
            Err(e) => {
                show_error("Requirements processing failed");
                error!("Requirements processing failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Handle GitHub repository if specified (priority over local files)
    if let Some(github_url) = &args.github {
        show_progress(&format!("Analyzing GitHub repository: {github_url}"));
        info!("Processing GitHub repository: {}", github_url);
        match process_github_repository(github_url, &config, &args).await {
            Ok(_) => {
                show_success("GitHub repository analysis completed successfully");
                return;
            },
            Err(e) => {
                show_error("GitHub repository processing failed");
                error!("GitHub repository processing failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    show_progress("Discovering source files...");
    let source_files = match collect_source_files(&args) {
        Ok(files) => files,
        Err(e) => {
            show_error("Failed to collect source files");
            error!("Error collecting source files: {}", e);
            std::process::exit(1);
        }
    };

    if source_files.is_empty() {
        show_warning("No supported source files found to analyze");
        warn!("No source files found");
        std::process::exit(1);
    }

    show_info(&format!("Found {} source file(s) to analyze", source_files.len()));
    debug!("Processing with concurrency limit: {}", args.concurrency);
    info!("Starting analysis of {} files", source_files.len());
    
    // Process files concurrently in batches
    let config = Arc::new(config);
    let args = Arc::new(args);
    
    // Create a semaphore to limit concurrency
    let semaphore = Arc::new(tokio::sync::Semaphore::new(args.concurrency));
    
    // Process files in smaller batches to improve memory efficiency
    let batch_size = args.concurrency * 2;
    let mut processed_files = Vec::new();
    
    for batch in source_files.chunks(batch_size) {
        let batch_tasks: Vec<_> = batch.iter().map(|file_path| {
            let config = Arc::clone(&config);
            let args = Arc::clone(&args);
            let semaphore = Arc::clone(&semaphore);
            let file_path = file_path.clone();
            
            tokio::spawn(async move {
                // Acquire semaphore permit to limit concurrency
                let _permit = semaphore.acquire().await.unwrap();
                
                match process_file(&file_path, &config, &args).await {
                    Ok(_) => Some(file_path),
                    Err(e) => {
                        eprintln!("❌ Error processing {file_path}: {e}");
                        // Exit immediately for unsupported language errors
                        if matches!(e, RigrError::UnsupportedLanguage(_)) {
                            std::process::exit(1);
                        }
                        None
                    }
                }
            })
        }).collect();
        
        // Wait for current batch to complete
        let batch_results = join_all(batch_tasks).await;
        
        // Collect successful results
        for file_path in batch_results.into_iter().flatten().flatten() {
            processed_files.push(file_path);
        }
    }

    // Generate comprehensive test case summary report
    if !processed_files.is_empty() {
        println!("\n📊 Generating comprehensive test case summary...");
        match generate_test_case_summary(&processed_files, &config, &args).await {
            Ok(_) => println!("✅ Test case summary generation completed"),
            Err(e) => eprintln!("❌ Test case summary generation failed: {e}"),
        }
        
        // Export test cases to standard formats if requested
        if let Some(ref export_formats) = args.export {
            println!("\n📤 Exporting test cases to standard formats...");
            match export_test_cases_to_standard_formats(&processed_files, &config, &args, export_formats).await {
                Ok(_) => println!("✅ Test case export completed"),
                Err(e) => eprintln!("❌ Test case export failed: {e}"),
            }
        }
    }

    println!("\n🎉 Rigr comprehensive test case generation completed successfully!");
    println!("📁 Check the 'RigrTestCases' directory for all generated files:");
    println!("   📝 UnitTestCases/ - Comprehensive unit test case descriptions");
    println!("   🔗 IntegrationTestCases/ - Integration test case descriptions");
    println!("   📊 test_case_summary.html - Complete test case analysis dashboard");
    
    if args.include_performance {
        println!("   ⚡ PerformanceTestCases/ - Performance test scenarios");
    }
    
    if args.include_security {
        println!("   🔒 SecurityTestCases/ - Security test scenarios");
    }
}

fn collect_source_files(args: &Args) -> Result<Vec<String>, RigrError> {
    let mut files = Vec::new();

    if let Some(file_path) = &args.file {
        if Path::new(file_path).exists() {
            files.push(file_path.clone());
        } else {
            return Err(RigrError::FileReadError(format!("File not found: {file_path}")));
        }
    }

    if let Some(dir_path) = &args.directory {
        let dir = Path::new(dir_path);
        if !dir.is_dir() {
            return Err(RigrError::FileReadError(format!("Directory not found: {dir_path}")));
        }

        let walker = WalkBuilder::new(dir)
            .hidden(false)
            .git_ignore(true)
            .build();

        for entry in walker {
            let entry = entry.map_err(|e| RigrError::FileReadError(e.to_string()))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if matches!(ext, 
                        // Modern languages
                        "rs" | "py" | "js" | "ts" | "java" | "kt" | "cs" | "go" | "swift" | "php" | "rb" | "scala" | "groovy" |
                        // C/C++
                        "c" | "h" | "cpp" | "cxx" | "cc" | "hpp" | "hxx" |
                        // Assembly
                        "asm" | "s" |
                        // Perl
                        "pl" | "pm" |
                        // Visual Basic
                        "vb" | "vbs" | "bas" |
                        // PowerShell
                        "ps1" | "psm1" |
                        // Batch/Shell
                        "bat" | "cmd" | "sh" | "bash" | "zsh" |
                        // MUMPS/M
                        "m" |
                        // Natural (Adabas)
                        "nat" |
                        // JCL
                        "jcl" |
                        // Mainframe Assembler
                        "hlasm" | "bal" |
                        // Legacy languages already supported
                        "cbl" | "cob" | "cpy" | "f" | "f77" | "f90" | "f95" | "f03" | "f08" | "for" |
                        "pas" | "pp" | "inc" | "dpr" | "dpk" | "dfm" | "sql" | "pls" | "plsql" |
                        "pks" | "pkb" | "rpg" | "rpgle" | "sqlrpgle" | "rpgleinc" | "ads" | "adb" | "ada"
                    ) {
                        if let Some(path_str) = path.to_str() {
                            files.push(path_str.to_string());
                        }
                    }
                }
            }
        }
    }

    if args.file.is_none() && args.directory.is_none() && args.requirements.is_none() && args.github.is_none() {
        return Err(RigrError::FileReadError(
            "Please specify either --file, --directory, --requirements, or --github option".to_string()
        ));
    }

    Ok(files)
}

async fn process_file(file_path: &str, config: &Config, args: &Args) -> Result<(), RigrError> {
    if args.verbose {
        show_progress(&format!("Processing: {file_path}"));
    }
    
    let path = Path::new(file_path);
    let language = detect_language(path)?;
    
    debug!("Processing file: {} (Language: {})", file_path, language);
    info!("Processing {} file: {}", language, file_path);

    // Check file size before reading
    let file_metadata = fs::metadata(file_path)
        .map_err(|e| RigrError::FileReadError(format!("Failed to get metadata for {file_path}: {e}")))?;
    
    const MAX_FILE_SIZE: u64 = 2 * 1024 * 1024; // 2MB limit
    if file_metadata.len() > MAX_FILE_SIZE {
        return process_large_file_in_chunks(file_path, config, args).await;
    }

    let file_content = fs::read_to_string(file_path)
        .map_err(|e| RigrError::FileReadError(format!("Failed to read {file_path}: {e}")))?;

    // Skip empty files
    if file_content.trim().is_empty() {
        if args.verbose {
            show_warning("Skipping empty file");
        }
        debug!("Skipping empty file: {}", file_path);
        return Ok(());
    }

    let ai_client = AiClient::new(config.ai.clone());
    
    let mut generation_successful = false;
    
    if !args.integration_only {
        show_progress("Generating comprehensive unit test cases...");
        debug!("Starting unit test generation for: {}", file_path);
        
        let unit_cases_request = TestGenerationRequest {
            source_code: file_content.clone(),
            language: language.clone(),
            test_type: TestType::Unit,
            framework: "comprehensive-test-case-generation".to_string(),
            additional_context: Some(format!(
                "Generate exhaustive unit test case descriptions for {}% coverage. Include functional, boundary, error, and edge cases. Source file: {}", 
                config.testing.coverage_threshold, file_path
            )),
            edge_case_count: args.edge_cases,
            coverage_threshold: config.testing.coverage_threshold,
        };
        
        match ai_client.generate_comprehensive_test_cases(unit_cases_request).await {
            Ok(test_cases) => {
                match get_output_path(file_path, "unit", config, args) {
                    Ok(cases_path) => {
                        if let Err(e) = save_test_cases(&cases_path, &test_cases) {
                            if args.verbose {
                                show_error("Failed to save unit test cases");
                            }
                            error!("Failed to save unit test cases: {}", e);
                        } else {
                            if args.verbose {
                                show_success(&format!("Unit test cases saved to: {cases_path}"));
                            }
                            info!("Unit test cases saved: {}", cases_path);
                            generation_successful = true;
                        }
                    }
                    Err(e) => {
                        if args.verbose {
                            show_error("Failed to determine unit test cases path");
                        }
                        error!("Failed to determine unit test cases path: {}", e);
                    }
                }
            }
            Err(e) => {
                if args.verbose {
                    show_error("Failed to generate unit test cases");
                }
                error!("Failed to generate unit test cases: {}", e);
            }
        }
    }
    
    if !args.unit_only {
        println!("   🔗 Generating comprehensive integration test cases...");
        
        let integration_cases_request = TestGenerationRequest {
            source_code: file_content.clone(),
            language: language.clone(),
            test_type: TestType::Integration,
            framework: "comprehensive-test-case-generation".to_string(),
            additional_context: Some(format!(
                "Generate exhaustive integration test case descriptions for {}% coverage. Focus on component interactions, data flow, and system integration. Source file: {}", 
                config.testing.coverage_threshold, file_path
            )),
            edge_case_count: args.edge_cases,
            coverage_threshold: config.testing.coverage_threshold,
        };
        
        match ai_client.generate_comprehensive_test_cases(integration_cases_request).await {
            Ok(test_cases) => {
                match get_output_path(file_path, "integration", config, args) {
                    Ok(cases_path) => {
                        if let Err(e) = save_test_cases(&cases_path, &test_cases) {
                            eprintln!("   ❌ Failed to save integration test cases: {e}");
                        } else {
                            println!("   ✅ Integration test cases saved to: {cases_path}");
                            generation_successful = true;
                        }
                    }
                    Err(e) => eprintln!("   ❌ Failed to determine integration test cases path: {e}"),
                }
            }
            Err(e) => eprintln!("   ❌ Failed to generate integration test cases: {e}"),
        }
    }

    // Generate performance test cases if requested
    if args.include_performance {
        println!("   ⚡ Generating performance test cases...");
        
        let performance_cases_request = TestGenerationRequest {
            source_code: file_content.clone(),
            language: language.clone(),
            test_type: TestType::Unit, // Use Unit type but specify performance context
            framework: "performance-test-case-generation".to_string(),
            additional_context: Some(format!(
                "Generate performance test case descriptions focusing on load, stress, scalability, and response time testing. Source file: {file_path}"
            )),
            edge_case_count: args.edge_cases / 2, // Fewer performance test cases
            coverage_threshold: config.testing.coverage_threshold,
        };
        
        match ai_client.generate_comprehensive_test_cases(performance_cases_request).await {
            Ok(test_cases) => {
                match get_output_path(file_path, "performance", config, args) {
                    Ok(cases_path) => {
                        if let Err(e) = save_test_cases(&cases_path, &test_cases) {
                            eprintln!("   ❌ Failed to save performance test cases: {e}");
                        } else {
                            println!("   ✅ Performance test cases saved to: {cases_path}");
                        }
                    }
                    Err(e) => eprintln!("   ❌ Failed to determine performance test cases path: {e}"),
                }
            }
            Err(e) => eprintln!("   ❌ Failed to generate performance test cases: {e}"),
        }
    }

    // Generate security test cases if requested
    if args.include_security {
        println!("   🔒 Generating security test cases...");
        
        let security_cases_request = TestGenerationRequest {
            source_code: file_content.clone(),
            language: language.clone(),
            test_type: TestType::Integration, // Use Integration type for security
            framework: "security-test-case-generation".to_string(),
            additional_context: Some(format!(
                "Generate security test case descriptions focusing on input validation, authentication, authorization, injection attacks, and data protection. Source file: {file_path}"
            )),
            edge_case_count: args.edge_cases / 2, // Fewer security test cases
            coverage_threshold: config.testing.coverage_threshold,
        };
        
        match ai_client.generate_comprehensive_test_cases(security_cases_request).await {
            Ok(test_cases) => {
                match get_output_path(file_path, "security", config, args) {
                    Ok(cases_path) => {
                        if let Err(e) = save_test_cases(&cases_path, &test_cases) {
                            eprintln!("   ❌ Failed to save security test cases: {e}");
                        } else {
                            println!("   ✅ Security test cases saved to: {cases_path}");
                        }
                    }
                    Err(e) => eprintln!("   ❌ Failed to determine security test cases path: {e}"),
                }
            }
            Err(e) => eprintln!("   ❌ Failed to generate security test cases: {e}"),
        }
    }
    
    if !generation_successful {
        return Err(RigrError::FileWriteError(format!("Failed to generate test cases for {file_path}")));
    }
    
    Ok(())
}

async fn process_large_file_in_chunks(
    file_path: &str, 
    config: &Config, 
    args: &Args
) -> Result<(), RigrError> {
    println!("   📦 File too large, processing in chunks...");
    
    let file_content = fs::read_to_string(file_path)
        .map_err(|e| RigrError::FileReadError(format!("Failed to read large file {file_path}: {e}")))?;
    
    // Split file into smaller logical chunks (functions, classes, etc.)
    let chunks = split_code_into_logical_chunks(&file_content);
    println!("   📦 Split into {} logical chunks", chunks.len());
    
    let path = Path::new(file_path);
    let language = detect_language(path)?;
    
    let ai_client = AiClient::new(config.ai.clone());
    let mut all_unit_test_cases = Vec::new();
    let mut all_integration_test_cases = Vec::new();
    
    // Process chunks sequentially to avoid memory overload
    for (i, chunk) in chunks.iter().enumerate() {
        if chunk.trim().is_empty() { continue; }
        
        println!("   🔧 Processing chunk {}/{} ({} chars)", i + 1, chunks.len(), chunk.len());
        
        // Generate test cases for this chunk
        if !args.integration_only {
            let unit_request = TestGenerationRequest {
                source_code: chunk.clone(),
                language: language.clone(),
                test_type: TestType::Unit,
                framework: "comprehensive-test-case-generation".to_string(),
                additional_context: Some(format!("Chunk {}/{} of large file: {}", i + 1, chunks.len(), file_path)),
                edge_case_count: 5, // Reduced for chunks
                coverage_threshold: config.testing.coverage_threshold,
            };
            
            match ai_client.generate_comprehensive_test_cases(unit_request).await {
                Ok(test_cases) => {
                    all_unit_test_cases.push(test_cases);
                }
                Err(e) => eprintln!("   ❌ Failed to generate unit test cases for chunk {}: {}", i + 1, e),
            }
        }
        
        if !args.unit_only {
            let integration_request = TestGenerationRequest {
                source_code: chunk.clone(),
                language: language.clone(),
                test_type: TestType::Integration,
                framework: "comprehensive-test-case-generation".to_string(),
                additional_context: Some(format!("Integration chunk {}/{} of large file: {}", i + 1, chunks.len(), file_path)),
                edge_case_count: 3, // Reduced for chunks
                coverage_threshold: config.testing.coverage_threshold,
            };
            
            match ai_client.generate_comprehensive_test_cases(integration_request).await {
                Ok(test_cases) => {
                    all_integration_test_cases.push(test_cases);
                }
                Err(e) => eprintln!("   ❌ Failed to generate integration test cases for chunk {}: {}", i + 1, e),
            }
        }
        
        // Add small delay to prevent overwhelming the AI service
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    // Combine all chunk test cases and save
    if !all_unit_test_cases.is_empty() {
        let combined_unit_test_cases = combine_test_cases(&all_unit_test_cases, "Unit");
        match get_output_path(file_path, "unit", config, args) {
            Ok(unit_output_path) => {
                if let Err(e) = save_test_cases(&unit_output_path, &combined_unit_test_cases) {
                    eprintln!("   ❌ Failed to save combined unit test cases: {e}");
                } else {
                    println!("   ✅ Combined unit test cases saved to: {unit_output_path}");
                }
            }
            Err(e) => eprintln!("   ❌ Failed to determine unit test cases output path: {e}"),
        }
    }
    
    if !all_integration_test_cases.is_empty() {
        let combined_integration_test_cases = combine_test_cases(&all_integration_test_cases, "Integration");
        match get_output_path(file_path, "integration", config, args) {
            Ok(integration_output_path) => {
                if let Err(e) = save_test_cases(&integration_output_path, &combined_integration_test_cases) {
                    eprintln!("   ❌ Failed to save combined integration test cases: {e}");
                } else {
                    println!("   ✅ Combined integration test cases saved to: {integration_output_path}");
                }
            }
            Err(e) => eprintln!("   ❌ Failed to determine integration test cases output path: {e}"),
        }
    }
    
    println!("   📈 Large file processing completed");
    Ok(())
}

fn split_code_into_logical_chunks(content: &str) -> Vec<String> {
    const CHUNK_SIZE: usize = 2000; // 2KB chunks
    let mut chunks = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    
    let mut current_chunk = String::new();
    let mut brace_depth = 0;
    let mut in_function = false;
    
    for line in lines {
        current_chunk.push_str(line);
        current_chunk.push('\n');
        
        // Track function boundaries for better chunking
        if line.trim().starts_with("function ") || line.trim().starts_with("fn ") || 
           line.trim().starts_with("def ") || line.trim().starts_with("class ") {
            in_function = true;
        }
        
        // Count braces to find logical boundaries
        for ch in line.chars() {
            match ch {
                '{' => brace_depth += 1,
                '}' => {
                    brace_depth -= 1;
                    if brace_depth == 0 && in_function {
                        in_function = false;
                        // End of function/class - good place to split
                        if current_chunk.len() > CHUNK_SIZE {
                            chunks.push(current_chunk.clone());
                            current_chunk.clear();
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Force split if chunk gets too large
        if current_chunk.len() > CHUNK_SIZE * 2 {
            chunks.push(current_chunk.clone());
            current_chunk.clear();
        }
    }
    
    // Add remaining content
    if !current_chunk.trim().is_empty() {
        chunks.push(current_chunk);
    }
    
    // If we only got one chunk, split it by lines
    if chunks.len() == 1 && chunks[0].len() > CHUNK_SIZE {
        let first_chunk = chunks[0].clone();
        let lines: Vec<&str> = first_chunk.lines().collect();
        chunks.clear();
        
        let mut current = String::new();
        for line in lines {
            current.push_str(line);
            current.push('\n');
            
            if current.len() > CHUNK_SIZE {
                chunks.push(current.clone());
                current.clear();
            }
        }
        
        if !current.trim().is_empty() {
            chunks.push(current);
        }
    }
    
    chunks
}

fn combine_test_cases(test_case_chunks: &[String], test_type: &str) -> String {
    let mut combined = String::new();
    
    combined.push_str(&format!("# Combined {test_type} Test Cases\n\n"));
    combined.push_str(&format!("This document contains comprehensive {} test cases generated by Rigr.\n\n", test_type.to_lowercase()));
    
    for (i, test_cases) in test_case_chunks.iter().enumerate() {
        combined.push_str(&format!("## Chunk {} Test Cases\n\n", i + 1));
        combined.push_str(test_cases);
        combined.push_str("\n\n---\n\n");
    }
    
    combined
}

fn detect_language(path: &Path) -> Result<String, RigrError> {
    match path.extension().and_then(|s| s.to_str()) {
        // Modern languages
        Some("rs") => Ok("Rust".to_string()),
        Some("py") => Ok("Python".to_string()),
        Some("js") => Ok("JavaScript".to_string()),
        Some("jsx") => Ok("JavaScript".to_string()),
        Some("ts") => Ok("TypeScript".to_string()),
        Some("tsx") => Ok("TypeScript".to_string()),
        Some("java") => Ok("Java".to_string()),
        Some("kt") => Ok("Kotlin".to_string()),
        Some("cs") => Ok("C#".to_string()),
        Some("go") => Ok("Go".to_string()),
        Some("swift") => Ok("Swift".to_string()),
        Some("php") => Ok("PHP".to_string()),
        Some("rb") => Ok("Ruby".to_string()),
        Some("scala") => Ok("Scala".to_string()),
        Some("groovy") => Ok("Groovy".to_string()),
        
        // C/C++
        Some("c") => Ok("C".to_string()),
        Some("h") => Ok("C".to_string()),
        Some("cpp") | Some("cxx") | Some("cc") => Ok("C++".to_string()),
        Some("hpp") | Some("hxx") => Ok("C++".to_string()),
        
        // Assembly
        Some("asm") | Some("s") => Ok("Assembly".to_string()),
        
        // Perl
        Some("pl") | Some("pm") => Ok("Perl".to_string()),
        
        // Visual Basic
        Some("vb") => Ok("Visual Basic .NET".to_string()),
        Some("vbs") => Ok("VBScript".to_string()),
        Some("bas") => Ok("Visual Basic".to_string()),
        
        // PowerShell
        Some("ps1") | Some("psm1") => Ok("PowerShell".to_string()),
        
        // Batch/Shell scripts
        Some("bat") | Some("cmd") => Ok("Batch".to_string()),
        Some("sh") | Some("bash") => Ok("Shell Script".to_string()),
        Some("zsh") => Ok("Zsh Script".to_string()),
        
        // MUMPS/M
        Some("m") => Ok("MUMPS".to_string()),
        
        // Natural (Adabas)
        Some("nat") => Ok("Natural".to_string()),
        
        // JCL
        Some("jcl") => Ok("JCL".to_string()),
        
        // Mainframe Assembler
        Some("hlasm") | Some("bal") => Ok("Mainframe Assembler".to_string()),
        
        // Legacy languages - Tier 1 Priority (already supported)
        Some("cbl") | Some("cob") | Some("cpy") => Ok("COBOL".to_string()),
        Some("f") | Some("f77") | Some("f90") | Some("f95") | Some("f03") | Some("f08") | Some("for") => Ok("Fortran".to_string()),
        Some("pas") | Some("pp") | Some("inc") => Ok("Pascal".to_string()),
        Some("dpr") | Some("dpk") | Some("dfm") => Ok("Delphi".to_string()),
        Some("sql") | Some("pls") | Some("plsql") | Some("pks") | Some("pkb") => Ok("PL/SQL".to_string()),
        Some("rpg") | Some("rpgle") | Some("sqlrpgle") | Some("rpgleinc") => Ok("RPG".to_string()),
        Some("ads") | Some("adb") | Some("ada") => Ok("Ada".to_string()),
        
        _ => Err(RigrError::UnsupportedLanguage(
            format!("Unsupported file extension: {:?}", path.extension())
        )),
    }
}

fn detect_workspace_name(source_path: &str) -> Result<String, RigrError> {
    let path = Path::new(source_path);
    
    // 1. Try to extract program name from source file name
    if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
        // Remove common suffixes like _test, test_, etc.
        let clean_name = file_stem
            .trim_end_matches("_test")
            .trim_end_matches("_tests") 
            .trim_start_matches("test_")
            .trim_end_matches("Test");
        
        if !clean_name.is_empty() && clean_name != "main" && clean_name != "lib" {
            return Ok(clean_name.to_string());
        }
    }
    
    // 2. Check for project config files in target directory (not rigr directory)
    let source_dir = path.parent().unwrap_or(Path::new("."));
    let mut current_dir = source_dir;
    
    loop {
        // Check for Cargo.toml - but skip if it's the rigr project itself
        let cargo_toml = current_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = fs::read_to_string(&cargo_toml) {
                for line in content.lines() {
                    if line.trim().starts_with("name") && line.contains("=") {
                        if let Some(name) = line.split("=").nth(1) {
                            let clean_name = name.trim().trim_matches('"').trim_matches('\'');
                            // Don't use "rigr" as the workspace name - that's our tool name
                            if !clean_name.is_empty() && clean_name != "rigr" {
                                return Ok(clean_name.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        // Check for package.json (Node.js projects)
        let package_json = current_dir.join("package.json");
        if package_json.exists() {
            if let Ok(content) = fs::read_to_string(&package_json) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                        if name != "rigr" {
                            return Ok(name.to_string());
                        }
                    }
                }
            }
        }
        
        // Move to parent directory
        if let Some(parent) = current_dir.parent() {
            current_dir = parent;
        } else {
            break;
        }
    }
    
    // 3. Use source directory name as fallback
    if let Some(dir_name) = source_dir.file_name().and_then(|n| n.to_str()) {
        if dir_name != "rigr" && dir_name != "src" {
            return Ok(dir_name.to_string());
        }
    }
    
    // 4. Use current directory name as final fallback
    if let Ok(current_dir) = std::env::current_dir() {
        if let Some(dir_name) = current_dir.file_name().and_then(|n| n.to_str()) {
            if dir_name != "rigr" {
                return Ok(dir_name.to_string());
            }
        }
    }
    
    // 5. Final fallback - use the file stem
    if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
        return Ok(file_stem.to_string());
    }
    
    Ok("project".to_string())
}

fn get_output_path(source_path: &str, test_type: &str, _config: &Config, args: &Args) -> Result<String, RigrError> {
    let source_file = Path::new(source_path);
    let file_stem = source_file.file_stem()
        .ok_or_else(|| RigrError::FileWriteError("Invalid source file path".to_string()))?
        .to_str()
        .ok_or_else(|| RigrError::FileWriteError("Invalid UTF-8 in file path".to_string()))?;
    
    let base_dir = if let Some(output_dir) = &args.output {
        Path::new(output_dir).canonicalize()
            .unwrap_or_else(|_| Path::new(output_dir).to_path_buf())
    } else {
        std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf())
    };
    
    // Create RigrTestCases directory structure with workspace subfolder
    let workspace_name = detect_workspace_name(source_path)?;
    let rigr_test_cases_dir = base_dir.join("RigrTestCases").join(workspace_name);
    fs::create_dir_all(&rigr_test_cases_dir)
        .map_err(|e| RigrError::FileWriteError(format!("Failed to create RigrTestCases directory: {e}")))?;
    
    let (test_dir, filename) = match test_type {
        "unit" => {
            let cases_dir = rigr_test_cases_dir.join("UnitTestCases");
            fs::create_dir_all(&cases_dir)
                .map_err(|e| RigrError::FileWriteError(format!("Failed to create UnitTestCases directory: {e}")))?;
            (cases_dir, format!("{file_stem}_unit_test_cases.md"))
        },
        "integration" => {
            let cases_dir = rigr_test_cases_dir.join("IntegrationTestCases");
            fs::create_dir_all(&cases_dir)
                .map_err(|e| RigrError::FileWriteError(format!("Failed to create IntegrationTestCases directory: {e}")))?;
            (cases_dir, format!("{file_stem}_integration_test_cases.md"))
        },
        "performance" => {
            let cases_dir = rigr_test_cases_dir.join("PerformanceTestCases");
            fs::create_dir_all(&cases_dir)
                .map_err(|e| RigrError::FileWriteError(format!("Failed to create PerformanceTestCases directory: {e}")))?;
            (cases_dir, format!("{file_stem}_performance_test_cases.md"))
        },
        "security" => {
            let cases_dir = rigr_test_cases_dir.join("SecurityTestCases");
            fs::create_dir_all(&cases_dir)
                .map_err(|e| RigrError::FileWriteError(format!("Failed to create SecurityTestCases directory: {e}")))?;
            (cases_dir, format!("{file_stem}_security_test_cases.md"))
        },
        _ => {
            let test_dir = rigr_test_cases_dir.join("TestCases");
            fs::create_dir_all(&test_dir)
                .map_err(|e| RigrError::FileWriteError(format!("Failed to create test directory: {e}")))?;
            (test_dir, format!("{file_stem}_{test_type}_test_cases.md"))
        }
    };
    
    let full_path = test_dir.join(filename);
    Ok(full_path.display().to_string())
}

fn save_test_cases(output_path: &str, test_content: &str) -> Result<(), RigrError> {
    fs::write(output_path, test_content)
        .map_err(|e| RigrError::FileWriteError(format!("Failed to write test cases to {output_path}: {e}")))
}

/// Save test cases to file and return the output path
fn save_test_cases_to_file(test_cases: &str, file_stem: &str, test_type: &TestType, args: &Args) -> Result<String, RigrError> {
    let base_dir = if let Some(output_dir) = &args.output {
        Path::new(output_dir)
    } else {
        Path::new(".")
    };
    
    let rigr_test_cases_dir = base_dir.join("RigrTestCases").join(file_stem);
    
    let (subdir, suffix) = match test_type {
        TestType::Unit => ("UnitTestCases", "unit_test_cases"),
        TestType::Integration => ("IntegrationTestCases", "integration_test_cases"),
    };
    
    let test_cases_dir = rigr_test_cases_dir.join(subdir);
    fs::create_dir_all(&test_cases_dir)
        .map_err(|e| RigrError::FileWriteError(format!("Failed to create directory: {e}")))?;
    
    let output_path = test_cases_dir.join(format!("{file_stem}_{suffix}.md"));
    save_test_cases(output_path.to_str().unwrap(), test_cases)?;
    
    Ok(output_path.display().to_string())
}

/// Save test cases to file with custom type and return the output path
fn save_test_cases_to_file_with_type(test_cases: &str, file_stem: &str, test_type: &str, args: &Args) -> Result<String, RigrError> {
    let base_dir = if let Some(output_dir) = &args.output {
        Path::new(output_dir)
    } else {
        Path::new(".")
    };
    
    let rigr_test_cases_dir = base_dir.join("RigrTestCases").join(file_stem);
    
    let (subdir, suffix) = match test_type {
        "performance" => ("PerformanceTestCases", "performance_test_cases"),
        "security" => ("SecurityTestCases", "security_test_cases"),
        "unit" => ("UnitTestCases", "unit_test_cases"),
        "integration" => ("IntegrationTestCases", "integration_test_cases"),
        _ => ("TestCases", "test_cases"),
    };
    
    let suffix = if !matches!(test_type, "performance" | "security" | "unit" | "integration") {
        format!("{test_type}_test_cases")
    } else {
        suffix.to_string()
    };
    
    let test_cases_dir = rigr_test_cases_dir.join(subdir);
    fs::create_dir_all(&test_cases_dir)
        .map_err(|e| RigrError::FileWriteError(format!("Failed to create directory: {e}")))?;
    
    let output_path = test_cases_dir.join(format!("{file_stem}_{suffix}.md"));
    save_test_cases(output_path.to_str().unwrap(), test_cases)?;
    
    Ok(output_path.display().to_string())
}

async fn generate_test_case_summary(
    source_files: &[String],
    config: &Config,
    args: &Args
) -> Result<(), RigrError> {
    println!("📊 Generating comprehensive test case summary...");
    
    let base_dir = if let Some(output_dir) = &args.output {
        Path::new(output_dir)
    } else {
        Path::new(".")
    };
    
    // Detect workspace name for the first source file
    let workspace_name = if !source_files.is_empty() {
        detect_workspace_name(&source_files[0]).unwrap_or_else(|_| "project".to_string())
    } else {
        "project".to_string()
    };
    
    let rigr_test_cases_dir = base_dir.join("RigrTestCases").join(&workspace_name);
    let summary_path = rigr_test_cases_dir.join("test_case_summary.html");
    
    // Collect test case metrics
    let mut unit_test_case_count = 0;
    let mut integration_test_case_count = 0;
    let mut performance_test_case_count = 0;
    let mut security_test_case_count = 0;
    let mut test_case_files = Vec::new();
    
    // Count test case files and their content
    if let Ok(entries) = fs::read_dir(rigr_test_cases_dir.join("UnitTestCases")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                test_case_files.push(path.display().to_string());
                unit_test_case_count += count_test_cases_in_file(&path)?;
            }
        }
    }
    
    if let Ok(entries) = fs::read_dir(rigr_test_cases_dir.join("IntegrationTestCases")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                test_case_files.push(path.display().to_string());
                integration_test_case_count += count_test_cases_in_file(&path)?;
            }
        }
    }

    if args.include_performance {
        if let Ok(entries) = fs::read_dir(rigr_test_cases_dir.join("PerformanceTestCases")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    test_case_files.push(path.display().to_string());
                    performance_test_case_count += count_test_cases_in_file(&path)?;
                }
            }
        }
    }

    if args.include_security {
        if let Ok(entries) = fs::read_dir(rigr_test_cases_dir.join("SecurityTestCases")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    test_case_files.push(path.display().to_string());
                    security_test_case_count += count_test_cases_in_file(&path)?;
                }
            }
        }
    }

    let total_test_cases = unit_test_case_count + integration_test_case_count + performance_test_case_count + security_test_case_count;
    
    // Generate HTML content
    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rigr Comprehensive Test Case Summary</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 10px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.3);
            overflow: hidden;
        }}
        .header {{
            background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
            color: white;
            padding: 40px;
            text-align: center;
        }}
        .header h1 {{
            margin: 0;
            font-size: 2.5em;
            font-weight: 300;
        }}
        .header p {{
            margin: 10px 0 0 0;
            opacity: 0.9;
        }}
        .ai-info {{
            background: rgba(255,255,255,0.1);
            padding: 10px 20px;
            border-radius: 5px;
            margin-top: 15px;
            font-size: 0.9em;
        }}
        .content {{
            padding: 40px;
        }}
        .metrics-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin-bottom: 40px;
        }}
        .metric-card {{
            background: #f8f9fa;
            padding: 30px;
            border-radius: 8px;
            text-align: center;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            transition: transform 0.3s ease;
        }}
        .metric-card:hover {{
            transform: translateY(-5px);
        }}
        .metric-number {{
            font-size: 3em;
            font-weight: bold;
            color: #4facfe;
            margin: 0;
        }}
        .metric-label {{
            font-size: 1.1em;
            color: #6c757d;
            margin-top: 10px;
        }}
        .file-list {{
            background: #f8f9fa;
            border-radius: 8px;
            padding: 20px;
            margin: 20px 0;
        }}
        .file-list h3 {{
            margin-top: 0;
            color: #495057;
        }}
        .file-item {{
            background: white;
            padding: 10px 15px;
            margin: 5px 0;
            border-radius: 5px;
            border-left: 4px solid #4facfe;
            font-family: monospace;
            font-size: 0.9em;
        }}
        .timestamp {{
            text-align: center;
            color: #6c757d;
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #dee2e6;
        }}
        .feature-badge {{
            display: inline-block;
            background: #28a745;
            color: white;
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 0.8em;
            margin: 2px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🧪 Rigr Comprehensive Test Case Summary</h1>
            <p>AI-Generated Test Case Analysis & Coverage Report</p>
            <div class="ai-info">
                🤖 Generated using {} AI model - {:.0}% Coverage Target
                <br>
                <span class="feature-badge">Unit Tests</span>
                <span class="feature-badge">Integration Tests</span>
                {}{}
            </div>
        </div>
        <div class="content">
            <div class="metrics-grid">
                <div class="metric-card">
                    <div class="metric-number">{}</div>
                    <div class="metric-label">Total Test Cases</div>
                </div>
                <div class="metric-card">
                    <div class="metric-number">{}</div>
                    <div class="metric-label">Unit Test Cases</div>
                </div>
                <div class="metric-card">
                    <div class="metric-number">{}</div>
                    <div class="metric-label">Integration Test Cases</div>
                </div>
                <div class="metric-card">
                    <div class="metric-number">{}</div>
                    <div class="metric-label">Source Files Analyzed</div>
                </div>
                {}{}
            </div>
            
            <div class="file-list">
                <h3>📁 Generated Test Case Files</h3>
                {}
            </div>
            
            <div class="file-list">
                <h3>🎯 Test Coverage Analysis</h3>
                <div class="file-item">✅ Functional Test Coverage: Unit and integration tests for core functionality</div>
                <div class="file-item">🔍 Boundary Value Testing: Edge cases and limit testing scenarios</div>
                <div class="file-item">❌ Error Condition Testing: Exception handling and error scenarios</div>
                <div class="file-item">🔄 Data Flow Testing: Input/output validation and transformation</div>
                <div class="file-item">🏗️ Structural Testing: Code path and branch coverage scenarios</div>
                {}{}
            </div>
            
            <div class="timestamp">
                Generated on {} with Rigr v1.0.0 - Comprehensive Test Case Generator
            </div>
        </div>
    </div>
</body>
</html>"#,
        config.ai.provider,
        config.testing.coverage_threshold,
        if args.include_performance { r#"<span class="feature-badge">Performance Tests</span>"# } else { "" },
        if args.include_security { r#"<span class="feature-badge">Security Tests</span>"# } else { "" },
        total_test_cases,
        unit_test_case_count,
        integration_test_case_count,
        source_files.len(),
        if args.include_performance { 
            format!(r#"<div class="metric-card">
                    <div class="metric-number">{performance_test_case_count}</div>
                    <div class="metric-label">Performance Test Cases</div>
                </div>"#)
        } else { String::new() },
        if args.include_security { 
            format!(r#"<div class="metric-card">
                    <div class="metric-number">{security_test_case_count}</div>
                    <div class="metric-label">Security Test Cases</div>
                </div>"#)
        } else { String::new() },
        test_case_files.iter()
            .map(|f| format!("<div class=\"file-item\">{}</div>", f.split('\\').next_back().unwrap_or(f)))
            .collect::<Vec<_>>()
            .join("\n                "),
        if args.include_performance { 
            r#"<div class="file-item">⚡ Performance Testing: Load, stress, and scalability test scenarios</div>"#
        } else { "" },
        if args.include_security { 
            r#"<div class="file-item">🔒 Security Testing: Authentication, authorization, and vulnerability test scenarios</div>"#
        } else { "" },
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    
    fs::write(&summary_path, html_content)
        .map_err(|e| RigrError::FileWriteError(format!("Failed to write test case summary: {e}")))?;
    
    println!("✅ Test case summary generated: {}", summary_path.display());
    Ok(())
}

fn count_test_cases_in_file(file_path: &Path) -> Result<usize, RigrError> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| RigrError::FileReadError(format!("Failed to read file: {e}")))?;
    
    let mut total_count = 0;
    
    // Look for test case patterns line by line to handle ranges properly
    for line in content.lines() {
        if line.contains("## Test Case") || line.contains("### Test Case") || 
           line.contains("**Test Case") || line.contains("TEST CASE") {
            
            // Check if this is a range like "Test Cases 7-26" or "Test Case 9 - 30"
            let is_range = line.contains("Cases ") || line.contains("Case ");
            if is_range {
                // Look for patterns like "7-26", "9 - 30", etc.
                if let Some(dash_pos) = line.rfind(" - ") {
                    // Handle format "Test Case 9 - 30"
                    let before_dash = &line[..dash_pos];
                    let after_dash = &line[dash_pos + 3..];
                    
                    // Extract numbers around the dash
                    if let Some(start_num) = before_dash.split_whitespace().last() {
                        if let Some(end_num) = after_dash.split_whitespace().next() {
                            let end_clean = end_num.trim_end_matches('*');
                            if let (Ok(start), Ok(end)) = (start_num.parse::<usize>(), end_clean.parse::<usize>()) {
                                total_count += end - start + 1;
                                continue;
                            }
                        }
                    }
                } else if let Some(dash_pos) = line.rfind('-') {
                    // Handle format "Test Cases 7-26"
                    let before_dash = &line[..dash_pos];
                    let after_dash = &line[dash_pos + 1..];
                    
                    if let Some(start_num) = before_dash.split_whitespace().last() {
                        if let Some(end_num) = after_dash.split_whitespace().next() {
                            let end_clean = end_num.trim_end_matches('*');
                            if let (Ok(start), Ok(end)) = (start_num.parse::<usize>(), end_clean.parse::<usize>()) {
                                total_count += end - start + 1;
                                continue;
                            }
                        }
                    }
                }
            }
            
            // Single test case
            total_count += 1;
        }
    }
    
    Ok(total_count.max(1)) // At least 1 if the file exists
}

async fn export_test_cases_to_standard_formats(
    source_files: &[String],
    _config: &Config,
    args: &Args,
    export_formats: &[String]
) -> Result<(), RigrError> {
    let base_dir = if let Some(output_dir) = &args.output {
        Path::new(output_dir)
    } else {
        Path::new(".")
    };
    
    // Detect workspace name for the first source file
    let workspace_name = if !source_files.is_empty() {
        detect_workspace_name(&source_files[0]).unwrap_or_else(|_| "project".to_string())
    } else {
        "project".to_string()
    };
    
    let rigr_test_cases_dir = base_dir.join("RigrTestCases").join(&workspace_name);
    let export_dir = rigr_test_cases_dir.join("Exports");
    
    // Collect all generated markdown files
    let mut markdown_files = Vec::new();
    
    // Collect unit test case files
    if let Ok(entries) = fs::read_dir(rigr_test_cases_dir.join("UnitTestCases")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                markdown_files.push(path.display().to_string());
            }
        }
    }
    
    // Collect integration test case files
    if let Ok(entries) = fs::read_dir(rigr_test_cases_dir.join("IntegrationTestCases")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                markdown_files.push(path.display().to_string());
            }
        }
    }
    
    // Collect performance test case files if included
    if args.include_performance {
        if let Ok(entries) = fs::read_dir(rigr_test_cases_dir.join("PerformanceTestCases")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    markdown_files.push(path.display().to_string());
                }
            }
        }
    }
    
    // Collect security test case files if included
    if args.include_security {
        if let Ok(entries) = fs::read_dir(rigr_test_cases_dir.join("SecurityTestCases")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    markdown_files.push(path.display().to_string());
                }
            }
        }
    }
    
    if markdown_files.is_empty() {
        return Err(RigrError::FileReadError("No test case files found to export".to_string()));
    }
    
    // Parse export formats
    let mut formats = Vec::new();
    for format_str in export_formats {
        match format_str.to_lowercase().as_str() {
            "csv" => formats.push(ExportFormat::CSV),
            "json" => formats.push(ExportFormat::JSON),
            "xml" => formats.push(ExportFormat::XML),
            "testrail" => formats.push(ExportFormat::TestRail),
            "xray" | "jira" => formats.push(ExportFormat::JiraXray),
            "azure" | "azuredevops" => formats.push(ExportFormat::AzureDevOps),
            "markdown" | "md" => formats.push(ExportFormat::Markdown),
            // High Priority Formats
            "allure" => formats.push(ExportFormat::Allure),
            "junit" => formats.push(ExportFormat::JUnitXML),
            "github" => formats.push(ExportFormat::GitHubIssues),
            "cucumber" | "gherkin" => formats.push(ExportFormat::Cucumber),
            "excel" | "xlsx" => formats.push(ExportFormat::ExcelXLSX),
            // Medium Priority Formats
            "zephyr" => formats.push(ExportFormat::Zephyr),
            "testlink" => formats.push(ExportFormat::TestLink),
            "confluence" => formats.push(ExportFormat::Confluence),
            "postman" => formats.push(ExportFormat::PostmanCollections),
            "robot" | "robotframework" => formats.push(ExportFormat::RobotFramework),
            "all" => {
                formats.extend(vec![
                    ExportFormat::CSV,
                    ExportFormat::JSON,
                    ExportFormat::TestRail,
                    ExportFormat::JiraXray,
                    ExportFormat::AzureDevOps,
                    ExportFormat::Markdown,
                    // High Priority Formats
                    ExportFormat::Allure,
                    ExportFormat::JUnitXML,
                    ExportFormat::GitHubIssues,
                    ExportFormat::Cucumber,
                    ExportFormat::ExcelXLSX,
                    // Medium Priority Formats
                    ExportFormat::Zephyr,
                    ExportFormat::TestLink,
                    ExportFormat::Confluence,
                    ExportFormat::PostmanCollections,
                    ExportFormat::RobotFramework,
                ]);
            },
            _ => {
                eprintln!("⚠️ Unknown export format: {format_str}. Supported: csv, json, testrail, xray, azure, markdown, allure, junit, github, cucumber, excel, zephyr, testlink, confluence, postman, robot, all");
            }
        }
    }
    
    if formats.is_empty() {
        return Err(RigrError::FileWriteError("No valid export formats specified".to_string()));
    }
    
    // Perform export
    TestCaseExporter::export_test_cases(
        &markdown_files,
        &export_dir.display().to_string(),
        &formats
    )?;
    
    println!("📊 Export Summary:");
    println!("   📁 Workspace: {workspace_name}");
    println!("   📄 Source Files: {}", markdown_files.len());
    println!("   📊 Export Formats: {}", formats.len());
    println!("   💾 Export Directory: {}", export_dir.display());
    
    Ok(())
}