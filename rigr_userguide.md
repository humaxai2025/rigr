# 📖 Rigr User Guide

**Complete reference for using Rigr - AI-Powered Test Case Generator**

Version 1.0.0 | Last Updated: 2025

---

## 📋 Table of Contents

1. [Getting Started](#getting-started)
2. [Command Line Options](#-command-line-options)
3. [Input Sources](#-input-sources)
4. [Test Case Types](#-test-case-types)
5. [Export Formats](#-export-formats)
6. [Configuration](#-configuration)
7. [Advanced Usage](#-advanced-usage)
8. [Examples](#-examples)
9. [Troubleshooting](#-troubleshooting)
10. [Best Practices](#-best-practices)

---

## 🚀 Getting Started

### Installation

1. Download the appropriate binary for your platform from the releases page
2. Place the `rigr` executable in your PATH
3. Run `rigr --help` to verify installation

### Initial Setup

Before using Rigr for the first time, configure your AI provider:

```bash
rigr --setup
```

This interactive wizard will guide you through:
- AI provider selection (OpenAI, Azure OpenAI, etc.)
- API key configuration
- Default preferences setup

---

## 🛠️ Command Line Options

### Basic Syntax

```bash
rigr [OPTIONS]
```

### Core Options

#### Input Sources
- `--file <FILE>` (`-f`): Analyze a single source file
- `--directory <DIRECTORY>` (`-d`): Analyze all supported files in a directory (recursive)
- `--requirements <REQUIREMENTS>` (`-r`): Generate tests from requirements document
- `--github <GITHUB>` (`-g`): Clone and analyze a GitHub repository

#### Test Configuration
- `--unit-only`: Generate only unit test cases
- `--integration-only`: Generate only integration test cases
- `--include-performance`: Include performance test cases
- `--include-security`: Include security test cases
- `--edge-cases <NUM>`: Number of edge cases to generate (default: 10)

#### Output & Export
- `--output <OUTPUT>` (`-o`): Specify output directory
- `--export <FORMATS>`: Export to standard formats (comma-separated)

#### Performance & Debugging
- `--concurrency <NUM>`: Maximum concurrent file processing (default: 2)
- `--verbose` (`-v`): Enable verbose output with detailed logging
- `--debug`: Enable debug logging (most detailed)

#### Utility
- `--setup`: Run the configuration wizard
- `--help` (`-h`): Display help information
- `--version` (`-V`): Display version information

---

## 📥 Input Sources

### 1. Single File Analysis

Analyze a specific source code file:

```bash
# Basic file analysis
rigr --file src/calculator.py

# With output directory
rigr --file src/calculator.py --output tests/

# Generate only unit tests
rigr --file src/main.rs --unit-only

# Include security tests
rigr --file api/auth.js --include-security
```

**Supported File Types**: `.py`, `.js`, `.ts`, `.java`, `.kt`, `.cs`, `.go`, `.rs`, `.swift`, `.php`, `.rb`, `.scala`, `.groovy`, `.c`, `.cpp`, `.h`, `.hpp`, `.cbl`, `.f90`, `.pas`, `.ada`, `.pl`, `.ps1`, `.sh`, `.bat`, and more.

### 2. Directory Analysis

Recursively analyze all supported files in a directory:

```bash
# Analyze entire directory
rigr --directory src/

# With custom concurrency
rigr --directory large_project/ --concurrency 8

# Filter to specific test types
rigr --directory backend/ --unit-only --include-performance

# Export results
rigr --directory api/ --export junit,csv --output test_results/
```

**Features**:
- Automatic file discovery
- Respects `.gitignore` patterns
- Concurrent processing for performance
- Progress tracking for large directories

### 3. Requirements Documents

Generate test cases directly from business requirements:

```bash
# Basic requirements processing
rigr --requirements project_spec.docx

# Multiple requirement files
rigr --requirements spec.txt,requirements.pdf,user_stories.xlsx

# Focus on unit tests only
rigr --requirements business_rules.docx --unit-only

# Export to test management tools
rigr --requirements spec.pdf --export testrail,xray
```

**Supported Formats**:
- `.txt`: Plain text requirements
- `.docx`: Microsoft Word documents
- `.pdf`: PDF documents
- `.xlsx`: Excel spreadsheets

### 4. GitHub Repository Analysis

Clone and analyze repositories directly:

```bash
# Public repository
rigr --github https://github.com/user/project

# With authentication (for private repos)
rigr --github https://github.com/company/private-repo --output analysis/

# Focus on specific test types
rigr --github https://github.com/org/legacy-app --include-security --include-performance

# High concurrency for large repos
rigr --github https://github.com/large-org/monorepo --concurrency 16
```

---

## 🧪 Test Case Types

### Unit Tests (Default)

**Purpose**: Test individual functions and methods in isolation

```bash
rigr --file calculator.py --unit-only
```

**Generated Tests Include**:
- Function parameter validation
- Return value verification
- Exception handling
- Edge cases and boundary conditions
- Mock dependencies
- State verification

**Example Output**:
```
Test Case: test_divide_by_zero
- Input: divide(10, 0)
- Expected: ZeroDivisionError
- Category: Error Handling
- Priority: High
```

### Integration Tests

**Purpose**: Test component interactions and system integration

```bash
rigr --file api_service.js --integration-only
```

**Generated Tests Include**:
- API endpoint testing
- Database integration
- Service communication
- End-to-end workflows
- External dependency integration

### Security Tests

**Purpose**: Identify potential security vulnerabilities

```bash
rigr --file user_auth.py --include-security
```

**Generated Tests Include**:
- Input validation attacks (SQL injection, XSS)
- Authentication bypass attempts
- Authorization boundary testing
- Data exposure vulnerabilities
- Rate limiting validation
- Encryption/decryption testing

### Performance Tests

**Purpose**: Validate system performance characteristics

```bash
rigr --file data_processor.java --include-performance
```

**Generated Tests Include**:
- Load testing scenarios
- Stress testing conditions
- Memory usage validation
- Response time verification
- Throughput measurement
- Scalability testing

### Edge Cases

**Purpose**: Test boundary conditions and unusual inputs

```bash
rigr --file validator.cs --edge-cases 20
```

**Generated Tests Include**:
- Minimum/maximum value boundaries
- Empty/null input handling
- Large data set processing
- Concurrent access scenarios
- Network failure simulation
- Resource exhaustion conditions

---

## 📤 Export Formats

Rigr supports 16+ export formats for seamless integration with your testing ecosystem.

### Development Formats

#### CSV
```bash
rigr --file app.py --export csv
```
- **File**: `rigr_test_cases.csv`
- **Use Case**: Spreadsheet analysis, data import
- **Contains**: Test ID, Name, Steps, Expected Results, Priority

#### JSON
```bash
rigr --file app.py --export json
```
- **File**: `rigr_test_cases.json`
- **Use Case**: API integration, custom tooling
- **Structure**: Structured test case objects with metadata

#### Excel (XLSX)
```bash
rigr --file app.py --export excel
```
- **File**: `rigr_test_cases.xlsx`
- **Use Case**: Professional reporting, stakeholder reviews
- **Features**: Multiple sheets, formatting, charts

#### Markdown
```bash
rigr --file app.py --export markdown
```
- **File**: `rigr_test_cases_consolidated.md`
- **Use Case**: Documentation, version control
- **Features**: Hierarchical organization, readable format

### Test Management Tools

#### TestRail
```bash
rigr --file app.py --export testrail
```
- **File**: `rigr_test_cases_testrail.csv`
- **Integration**: Direct import into TestRail
- **Fields**: Section, Title, Type, Priority, Estimate, References

#### Jira Xray
```bash
rigr --file app.py --export xray
```
- **File**: `rigr_test_cases_xray.json`
- **Integration**: Xray Test Management for Jira
- **Features**: Test execution tracking, requirement traceability

#### Azure DevOps
```bash
rigr --file app.py --export azure
```
- **File**: `rigr_test_cases_azure.json`
- **Integration**: Azure Test Plans
- **Features**: Work item integration, test suite organization

#### Zephyr Scale
```bash
rigr --file app.py --export zephyr
```
- **File**: `rigr_test_cases_zephyr.json`
- **Integration**: Zephyr Scale for Jira
- **Features**: Test cycle management, reporting

### Testing Frameworks

#### JUnit XML
```bash
rigr --file app.py --export junit
```
- **File**: `rigr_test_cases_junit.xml`
- **Integration**: JUnit, Maven, Gradle
- **Features**: Test suite structure, CI/CD integration

#### Allure Framework
```bash
rigr --file app.py --export allure
```
- **Directory**: `allure-results/`
- **Integration**: Allure reporting
- **Features**: Rich test reports, historical trends

#### Cucumber/Gherkin
```bash
rigr --file app.py --export cucumber
```
- **Directory**: `features/`
- **Integration**: Cucumber, SpecFlow, Behave
- **Features**: BDD format, natural language scenarios

#### Robot Framework
```bash
rigr --file app.py --export robot
```
- **File**: `rigr_test_cases.robot`
- **Integration**: Robot Framework
- **Features**: Keyword-driven testing, test libraries

### Collaboration Tools

#### GitHub Issues
```bash
rigr --file app.py --export github
```
- **File**: `rigr_test_cases_github_issues.json`
- **Integration**: GitHub Issues API
- **Features**: Automated issue creation, labels, assignments

#### Confluence
```bash
rigr --file app.py --export confluence
```
- **File**: `rigr_test_cases_confluence.txt`
- **Integration**: Atlassian Confluence
- **Features**: Wiki markup, page structure

#### TestLink
```bash
rigr --file app.py --export testlink
```
- **File**: `rigr_test_cases_testlink.xml`
- **Integration**: TestLink test management
- **Features**: Test project structure, requirements linking

### API Testing

#### Postman Collections
```bash
rigr --file api.js --export postman
```
- **File**: `rigr_test_cases_postman.json`
- **Integration**: Postman, Newman
- **Features**: API test automation, environment variables

### Multi-Format Export

Export to multiple formats simultaneously:

```bash
# Common combination
rigr --file app.py --export csv,json,junit

# Complete export
rigr --file app.py --export all

# Test management focus
rigr --file app.py --export testrail,xray,azure,zephyr

# Framework integration
rigr --file app.py --export junit,allure,cucumber,robot
```

---

## ⚙️ Configuration

### AI Provider Setup

Rigr supports multiple AI providers for test generation:

#### OpenAI (Default)
```bash
rigr --setup
# Follow prompts to configure:
# - API Key
# - Model selection (GPT-4, GPT-3.5-turbo)
# - Temperature settings
```

#### Azure OpenAI
```bash
rigr --setup
# Configure:
# - Azure endpoint
# - API key
# - Deployment name
# - API version
```

### Configuration File

Rigr creates a configuration file at `~/.rigr/config.toml`:

```toml
[ai_provider]
provider = "openai"
api_key = "your-api-key"
model = "gpt-4"
temperature = 0.7
max_tokens = 2000

[defaults]
concurrency = 2
edge_cases = 10
include_performance = false
include_security = false
output_directory = "RigrTestCases"

[export]
default_formats = ["json", "csv"]
include_timestamps = true
generate_summary = true
```

### Environment Variables

Override configuration with environment variables:

```bash
export RIGR_API_KEY="your-api-key"
export RIGR_MODEL="gpt-4"
export RIGR_CONCURRENCY=4
export RIGR_OUTPUT_DIR="custom_output"

rigr --file app.py
```

---

## 🔧 Advanced Usage

### Batch Processing

Process multiple files with custom settings:

```bash
# High-performance batch processing
rigr --directory large_codebase/ \
     --concurrency 16 \
     --unit-only \
     --edge-cases 15 \
     --export junit,allure,csv \
     --output batch_results/
```

### Selective Analysis

Focus on specific aspects of your codebase:

```bash
# Security-focused analysis
rigr --directory api/ \
     --include-security \
     --export confluence,github \
     --output security_audit/

# Performance testing focus
rigr --directory services/ \
     --include-performance \
     --edge-cases 20 \
     --export allure,junit
```

### Pipeline Integration

Integrate Rigr into CI/CD pipelines:

```yaml
# GitHub Actions example
- name: Generate Test Cases
  run: |
    rigr --directory src/ \
         --export junit,allure \
         --output test-results/ \
         --concurrency 4
         
- name: Upload Test Cases
  uses: actions/upload-artifact@v2
  with:
    name: generated-tests
    path: test-results/
```

### Custom Workflows

Create reusable commands with shell scripts:

```bash
#!/bin/bash
# rigr-security-audit.sh

rigr --directory "$1" \
     --include-security \
     --include-performance \
     --export confluence,testrail,csv \
     --output "security-audit-$(date +%Y%m%d)" \
     --verbose

echo "Security audit complete. Results in security-audit-$(date +%Y%m%d)/"
```

---

## 💡 Examples

### Example 1: Single File Analysis

**Scenario**: Analyze a Python calculator module

```bash
rigr --file calculator.py --export junit,csv --output calculator_tests/
```

**Generated Output**:
```
calculator_tests/
├── RigrTestCases/
│   └── calculator/
│       ├── UnitTestCases/
│       │   └── calculator_unit_test_cases.md
│       ├── Exports/
│       │   ├── rigr_test_cases.csv
│       │   ├── rigr_test_cases_junit.xml
│       │   └── IMPORT_INSTRUCTIONS.md
│       └── test_case_summary.html
```

### Example 2: Legacy System Analysis

**Scenario**: Modernize COBOL banking system with comprehensive tests

```bash
rigr --directory cobol_banking/ \
     --include-security \
     --include-performance \
     --export testrail,excel,confluence \
     --output legacy_modernization/ \
     --concurrency 4 \
     --verbose
```

**Use Case**: Generate modern test coverage for legacy systems without existing tests.

### Example 3: Requirements-Based Testing

**Scenario**: Generate tests from product requirements document

```bash
rigr --requirements "Product Requirements v2.1.docx" \
     --export xray,cucumber,postman \
     --output requirements_tests/ \
     --unit-only
```

**Generated Tests**: Business rule validation, user story scenarios, API contract testing.

### Example 4: API Security Audit

**Scenario**: Security testing for REST API

```bash
rigr --directory api_gateway/ \
     --include-security \
     --export github,confluence \
     --output security_analysis/ \
     --edge-cases 25
```

**Generated Tests**: 
- Authentication bypass attempts
- SQL injection scenarios
- Rate limiting validation
- Data exposure tests

### Example 5: Open Source Project Analysis

**Scenario**: Analyze a GitHub repository

```bash
rigr --github https://github.com/fastapi/fastapi \
     --include-performance \
     --export allure,junit,csv \
     --output fastapi_analysis/ \
     --concurrency 8
```

**Use Case**: Contribute comprehensive test cases to open source projects.

### Example 6: Multi-Language Enterprise Application

**Scenario**: Full-stack application with multiple languages

```bash
# Backend services (Java/Kotlin)
rigr --directory backend/ \
     --include-security \
     --include-performance \
     --export junit,allure \
     --output backend_tests/

# Frontend application (TypeScript)
rigr --directory frontend/src/ \
     --unit-only \
     --export cucumber,postman \
     --output frontend_tests/

# Database scripts (SQL)
rigr --directory database/ \
     --include-performance \
     --export testrail,csv \
     --output database_tests/
```

---

## 🔍 Troubleshooting

### Common Issues

#### 1. "No supported source files found"

**Cause**: Rigr couldn't find any files with supported extensions.

**Solutions**:
```bash
# Check if files exist in directory
ls -la your_directory/

# Use specific file instead of directory
rigr --file specific_file.py

# Check supported extensions
rigr --help | grep -A 20 "Supported languages"
```

#### 2. "Unsupported language" error

**Cause**: File extension not recognized.

**Solutions**:
```bash
# Check file extension
file your_file.ext

# Rename file with correct extension
mv script.txt script.py

# Use requirements instead for text files
rigr --requirements script.txt
```

#### 3. "AI provider configuration missing"

**Cause**: AI provider not configured.

**Solution**:
```bash
rigr --setup
```

#### 4. "Failed to generate test cases"

**Possible Causes**:
- API key invalid or expired
- Network connectivity issues
- File parsing errors
- AI service unavailable

**Solutions**:
```bash
# Check API key
rigr --setup

# Test with verbose output
rigr --file simple.py --verbose

# Check network connectivity
curl -I https://api.openai.com

# Try with different file
rigr --file basic_example.py
```

#### 5. Export format issues

**Cause**: Invalid export format specified.

**Solution**:
```bash
# Check available formats
rigr --help | grep -A 5 "export"

# Use valid format names
rigr --file app.py --export csv,json,junit
```

### Performance Issues

#### Slow processing with large directories

**Solutions**:
```bash
# Increase concurrency
rigr --directory large_dir/ --concurrency 8

# Process in smaller chunks
rigr --directory large_dir/module1/ --concurrency 4
rigr --directory large_dir/module2/ --concurrency 4

# Use unit-only for faster processing
rigr --directory large_dir/ --unit-only --concurrency 8
```

#### Memory usage with large files

**Solutions**:
```bash
# Process files individually
for file in large_files/*.py; do
  rigr --file "$file" --output individual_results/
done

# Reduce edge cases
rigr --directory src/ --edge-cases 5 --concurrency 2
```

### Debug Mode

Enable detailed logging for troubleshooting:

```bash
rigr --file problematic.py --debug --verbose
```

This provides:
- Detailed parsing information
- AI API request/response logs
- File processing steps
- Error stack traces

### Getting Help

1. **Documentation**: Check this user guide and README
2. **GitHub Issues**: Search existing issues or create new ones
3. **Debug Logs**: Always include `--debug --verbose` output
4. **Version Info**: Include `rigr --version` output
5. **System Info**: Include OS, Rust version, system architecture

---

## 🎯 Best Practices

### File Organization

**Structure your output directory**:
```bash
# Use descriptive output directories
rigr --directory src/ --output "tests-$(date +%Y%m%d)"

# Separate by test type
rigr --directory api/ --include-security --output security-tests/
rigr --directory api/ --include-performance --output performance-tests/
```

### Batch Processing Strategy

**For large codebases**:
1. Start with core modules using `--unit-only`
2. Add security tests for critical components
3. Generate integration tests for service boundaries
4. Create performance tests for bottlenecks

```bash
# Phase 1: Core functionality
rigr --directory core/ --unit-only --export junit

# Phase 2: Security critical
rigr --directory auth/ --include-security --export testrail

# Phase 3: Performance critical
rigr --directory services/ --include-performance --export allure
```

### Export Strategy

**Choose formats based on your workflow**:

```bash
# Development workflow
rigr --file app.py --export junit,csv

# QA workflow  
rigr --file app.py --export testrail,excel

# DevOps workflow
rigr --file app.py --export allure,junit,github

# Complete documentation
rigr --file app.py --export confluence,markdown
```

### Configuration Management

**Team configuration**:
```bash
# Create team config file
cat > team-rigr-config.toml << EOF
[defaults]
concurrency = 4
edge_cases = 15
include_security = true
output_directory = "generated-tests"

[export]
default_formats = ["junit", "allure", "csv"]
EOF

# Use team config
export RIGR_CONFIG=team-rigr-config.toml
rigr --directory src/
```

### Quality Assurance

**Review generated tests**:
1. Check test case relevance and accuracy
2. Verify edge cases cover actual business scenarios
3. Validate export formats import correctly
4. Ensure security tests align with threat model

### Continuous Integration

**Pipeline best practices**:
```yaml
# Separate test generation from execution
- name: Generate Tests
  run: rigr --directory src/ --export junit --output generated-tests/
  
- name: Review Generated Tests
  run: echo "Review test coverage and quality"
  
- name: Execute Generated Tests
  run: pytest generated-tests/
```

### Documentation

**Maintain test documentation**:
```bash
# Generate comprehensive documentation
rigr --directory src/ \
     --export confluence,markdown \
     --output documentation/ \
     --include-performance \
     --include-security

# Version control test cases
git add documentation/
git commit -m "Update generated test cases for release v2.1"
```

---

## 📚 Additional Resources

### Command Reference Quick Card

```bash
# Basic usage
rigr --file <path> [--output <dir>] [--export <formats>]

# Directory analysis
rigr --directory <path> [--concurrency <num>] [--unit-only]

# Requirements processing
rigr --requirements <file> [--unit-only] [--export <formats>]

# GitHub analysis
rigr --github <url> [--concurrency <num>] [--output <dir>]

# Test types
--unit-only                 # Only unit tests
--integration-only          # Only integration tests
--include-security         # Add security tests
--include-performance      # Add performance tests
--edge-cases <num>         # Number of edge cases

# Export formats
csv,json,excel,markdown    # Documentation formats
junit,allure,cucumber      # Testing frameworks
testrail,xray,azure        # Test management
github,confluence          # Collaboration
postman,robot             # Specialized tools
```

### Supported Languages Reference

| Category | Extensions | Languages |
|----------|------------|-----------|
| Modern Web | `.js, .jsx, .ts, .tsx` | JavaScript, TypeScript |
| Backend | `.py, .java, .kt, .cs, .go, .rs` | Python, Java, Kotlin, C#, Go, Rust |
| Mobile/Modern | `.swift, .kt, .scala, .groovy` | Swift, Kotlin, Scala, Groovy |
| Web/Scripting | `.php, .rb` | PHP, Ruby |
| Systems | `.c, .h, .cpp, .hpp, .asm` | C, C++, Assembly |
| Legacy Enterprise | `.cbl, .f90, .pas, .ada` | COBOL, Fortran, Pascal, Ada |
| Database | `.sql, .pls, .plsql` | SQL, PL/SQL |
| IBM | `.rpg, .rpgle, .cpy` | RPG, COBOL copybooks |
| Scripting | `.pl, .ps1, .sh, .bat` | Perl, PowerShell, Shell, Batch |

---

**Need more help?** 
- 📖 [Full Documentation](https://docs.rigr.dev)
- 💬 [Community Discord](https://discord.gg/rigr)
- 🐛 [GitHub Issues](https://github.com/your-org/rigr/issues)
- 📧 [Support Email](mailto:support@rigr.dev)

---

*This user guide is maintained alongside Rigr development. For the latest version, visit our [documentation site](https://docs.rigr.dev).*