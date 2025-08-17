use crate::RigrError;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct CoverageReport {
    pub total_lines: usize,
    pub covered_lines: usize,
    pub coverage_percentage: f64,
    pub file_coverage: HashMap<String, FileCoverage>,
    pub uncovered_functions: Vec<String>,
}

#[derive(Debug)]
pub struct FileCoverage {
    pub total_lines: usize,
    pub covered_lines: usize,
    pub coverage_percentage: f64,
    pub functions: Vec<FunctionCoverage>,
}

#[derive(Debug, Clone)]
pub struct FunctionCoverage {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub is_covered: bool,
    pub test_cases: Vec<String>,
}

pub struct CoverageAnalyzer;

impl CoverageAnalyzer {
    pub fn analyze_coverage(
        source_files: &[String],
        unit_tests: &str,
        integration_tests: &str,
        language: &str,
    ) -> Result<CoverageReport, RigrError> {
        let mut total_lines = 0;
        let mut file_coverage = HashMap::new();
        let mut all_functions = Vec::new();

        for file_path in source_files {
            let content = std::fs::read_to_string(file_path)
                .map_err(|e| RigrError::FileReadError(format!("Failed to read {file_path}: {e}")))?;
            
            let functions = Self::extract_functions(&content, language)?;
            let lines = content.lines().count();
            total_lines += lines;
            all_functions.extend(functions.iter().cloned());

            let covered_functions = Self::analyze_function_coverage(&functions, unit_tests, integration_tests);
            let covered_lines = Self::estimate_covered_lines(&functions, &covered_functions);
            
            let coverage_percentage = if lines > 0 {
                (covered_lines as f64 / lines as f64) * 100.0
            } else {
                0.0
            };

            file_coverage.insert(file_path.clone(), FileCoverage {
                total_lines: lines,
                covered_lines,
                coverage_percentage,
                functions: covered_functions,
            });
        }

        let total_covered_lines: usize = file_coverage.values().map(|fc| fc.covered_lines).sum();
        let overall_coverage = if total_lines > 0 {
            (total_covered_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        let uncovered_functions = all_functions
            .iter()
            .filter(|f| !file_coverage.values()
                .flat_map(|fc| &fc.functions)
                .any(|covered_f| covered_f.name == f.name && covered_f.is_covered))
            .map(|f| f.name.clone())
            .collect();

        Ok(CoverageReport {
            total_lines,
            covered_lines: total_covered_lines,
            coverage_percentage: overall_coverage,
            file_coverage,
            uncovered_functions,
        })
    }

    pub fn extract_functions(content: &str, language: &str) -> Result<Vec<FunctionCoverage>, RigrError> {
        let mut functions = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        match language.to_lowercase().as_str() {
            "rust" => {
                for (i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim_start();
                    if trimmed.starts_with("fn ") || 
                       trimmed.starts_with("pub fn ") ||
                       trimmed.starts_with("pub async fn ") ||
                       trimmed.starts_with("async fn ") ||
                       trimmed.starts_with("pub unsafe fn ") ||
                       trimmed.starts_with("unsafe fn ") ||
                       trimmed.starts_with("pub const fn ") ||
                       trimmed.starts_with("const fn ") {
                        if let Some(fn_name) = Self::extract_rust_function_name(line) {
                            let end_line = Self::find_function_end(&lines, i, "{", "}");
                            functions.push(FunctionCoverage {
                                name: fn_name,
                                line_start: i + 1,
                                line_end: end_line + 1,
                                is_covered: false,
                                test_cases: Vec::new(),
                            });
                        }
                    }
                }
            },
            "python" => {
                for (i, line) in lines.iter().enumerate() {
                    if line.trim_start().starts_with("def ") {
                        if let Some(fn_name) = Self::extract_python_function_name(line) {
                            let end_line = Self::find_python_function_end(&lines, i);
                            functions.push(FunctionCoverage {
                                name: fn_name,
                                line_start: i + 1,
                                line_end: end_line + 1,
                                is_covered: false,
                                test_cases: Vec::new(),
                            });
                        }
                    }
                }
            },
            "javascript" | "typescript" => {
                for (i, line) in lines.iter().enumerate() {
                    if line.contains("function ") || line.contains("const ") && line.contains("=>") {
                        if let Some(fn_name) = Self::extract_js_function_name(line) {
                            let end_line = Self::find_function_end(&lines, i, "{", "}");
                            functions.push(FunctionCoverage {
                                name: fn_name,
                                line_start: i + 1,
                                line_end: end_line + 1,
                                is_covered: false,
                                test_cases: Vec::new(),
                            });
                        }
                    }
                }
            },
            "java" | "csharp" => {
                for (i, line) in lines.iter().enumerate() {
                    if (line.contains("public ") || line.contains("private ") || line.contains("protected "))
                        && (line.contains("void ") || line.contains("int ") || line.contains("string ") || line.contains("bool ")) {
                        if let Some(fn_name) = Self::extract_java_csharp_function_name(line) {
                            let end_line = Self::find_function_end(&lines, i, "{", "}");
                            functions.push(FunctionCoverage {
                                name: fn_name,
                                line_start: i + 1,
                                line_end: end_line + 1,
                                is_covered: false,
                                test_cases: Vec::new(),
                            });
                        }
                    }
                }
            },
            "cobol" => {
                for (i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim_start();
                    // COBOL procedure/function definitions
                    if (trimmed.starts_with("PROCEDURE ") && trimmed.contains(" DIVISION"))
                        || (trimmed.starts_with(" PROCEDURE ") && trimmed.contains(" DIVISION"))
                        || Self::is_cobol_paragraph(trimmed)
                        || Self::is_cobol_section(trimmed) {
                        if let Some(fn_name) = Self::extract_cobol_procedure_name(line) {
                            let end_line = Self::find_cobol_procedure_end(&lines, i);
                            functions.push(FunctionCoverage {
                                name: fn_name,
                                line_start: i + 1,
                                line_end: end_line + 1,
                                is_covered: false,
                                test_cases: Vec::new(),
                            });
                        }
                    }
                }
            },
            "fortran" => {
                for (i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim_start().to_uppercase();
                    // Fortran subroutine, function, and program definitions
                    if trimmed.starts_with("SUBROUTINE ") || trimmed.starts_with("FUNCTION ") 
                        || trimmed.starts_with("PROGRAM ") || trimmed.starts_with("INTEGER FUNCTION ")
                        || trimmed.starts_with("REAL FUNCTION ") || trimmed.starts_with("LOGICAL FUNCTION ")
                        || trimmed.starts_with("CHARACTER FUNCTION ") {
                        if let Some(fn_name) = Self::extract_fortran_procedure_name(line) {
                            let end_line = Self::find_fortran_procedure_end(&lines, i);
                            functions.push(FunctionCoverage {
                                name: fn_name,
                                line_start: i + 1,
                                line_end: end_line + 1,
                                is_covered: false,
                                test_cases: Vec::new(),
                            });
                        }
                    }
                }
            },
            "pascal" | "delphi" => {
                for (i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim_start().to_lowercase();
                    // Pascal/Delphi procedure, function, and constructor definitions
                    if trimmed.starts_with("procedure ") || trimmed.starts_with("function ")
                        || trimmed.starts_with("constructor ") || trimmed.starts_with("destructor ")
                        || trimmed.starts_with("class procedure ") || trimmed.starts_with("class function ") {
                        if let Some(fn_name) = Self::extract_pascal_procedure_name(line) {
                            let end_line = Self::find_pascal_procedure_end(&lines, i);
                            functions.push(FunctionCoverage {
                                name: fn_name,
                                line_start: i + 1,
                                line_end: end_line + 1,
                                is_covered: false,
                                test_cases: Vec::new(),
                            });
                        }
                    }
                }
            },
            "plsql" | "pl/sql" => {
                for (i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim_start().to_uppercase();
                    // PL/SQL procedure, function, package definitions
                    if trimmed.starts_with("PROCEDURE ") || trimmed.starts_with("FUNCTION ")
                        || trimmed.starts_with("CREATE OR REPLACE PROCEDURE ")
                        || trimmed.starts_with("CREATE OR REPLACE FUNCTION ")
                        || trimmed.starts_with("CREATE PROCEDURE ") || trimmed.starts_with("CREATE FUNCTION ")
                        || (trimmed.starts_with("CREATE ") && trimmed.contains(" PACKAGE")) {
                        if let Some(fn_name) = Self::extract_plsql_procedure_name(line) {
                            let end_line = Self::find_plsql_procedure_end(&lines, i);
                            functions.push(FunctionCoverage {
                                name: fn_name,
                                line_start: i + 1,
                                line_end: end_line + 1,
                                is_covered: false,
                                test_cases: Vec::new(),
                            });
                        }
                    }
                }
            },
            "rpg" | "rpgle" => {
                for (i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim_start().to_uppercase();
                    // RPG procedure definitions (modern RPG ILE)
                    if trimmed.starts_with("DCL-PROC ") || trimmed.starts_with("P ") 
                        || (line.len() > 6 && line.chars().nth(5) == Some('P') && line.chars().nth(6) == Some(' '))
                        || trimmed.contains("BEGSR") {
                        if let Some(fn_name) = Self::extract_rpg_procedure_name(line) {
                            let end_line = Self::find_rpg_procedure_end(&lines, i);
                            functions.push(FunctionCoverage {
                                name: fn_name,
                                line_start: i + 1,
                                line_end: end_line + 1,
                                is_covered: false,
                                test_cases: Vec::new(),
                            });
                        }
                    }
                }
            },
            "ada" => {
                for (i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim_start().to_lowercase();
                    // Ada procedure, function, task, and package definitions
                    if trimmed.starts_with("procedure ") || trimmed.starts_with("function ")
                        || trimmed.starts_with("task ") || trimmed.starts_with("package ")
                        || trimmed.starts_with("generic ") || trimmed.starts_with("protected ") {
                        if let Some(fn_name) = Self::extract_ada_procedure_name(line) {
                            let end_line = Self::find_ada_procedure_end(&lines, i);
                            functions.push(FunctionCoverage {
                                name: fn_name,
                                line_start: i + 1,
                                line_end: end_line + 1,
                                is_covered: false,
                                test_cases: Vec::new(),
                            });
                        }
                    }
                }
            },
            "go" | "golang" => {
                for (i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim_start();
                    if trimmed.starts_with("func ") {
                        if let Some(fn_name) = Self::extract_go_function_name(line) {
                            let end_line = Self::find_function_end(&lines, i, "{", "}");
                            functions.push(FunctionCoverage {
                                name: fn_name,
                                line_start: i + 1,
                                line_end: end_line + 1,
                                is_covered: false,
                                test_cases: Vec::new(),
                            });
                        }
                    }
                }
            },
            _ => {
                return Err(RigrError::UnsupportedLanguage(format!("Coverage analysis not supported for {language}")));
            }
        }

        Ok(functions)
    }

    fn extract_rust_function_name(line: &str) -> Option<String> {
        let trimmed = line.trim();
        if let Some(fn_pos) = trimmed.find("fn ") {
            let after_fn = &trimmed[fn_pos + 3..];
            // Find function name, handling generic parameters
            if let Some(paren_pos) = after_fn.find('(') {
                let name_part = after_fn[..paren_pos].trim();
                // If there are generics, extract just the function name before '<'
                if let Some(generic_pos) = name_part.find('<') {
                    return Some(name_part[..generic_pos].trim().to_string());
                } else {
                    return Some(name_part.to_string());
                }
            }
        }
        None
    }

    fn extract_python_function_name(line: &str) -> Option<String> {
        let trimmed = line.trim();
        if let Some(def_pos) = trimmed.find("def ") {
            let after_def = &trimmed[def_pos + 4..];
            if let Some(paren_pos) = after_def.find('(') {
                return Some(after_def[..paren_pos].trim().to_string());
            }
        }
        None
    }

    fn extract_js_function_name(line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        if let Some(function_pos) = trimmed.find("function ") {
            let after_function = &trimmed[function_pos + 9..];
            if let Some(paren_pos) = after_function.find('(') {
                return Some(after_function[..paren_pos].trim().to_string());
            }
        }
        
        if trimmed.contains("const ") && trimmed.contains("=>") {
            if let Some(const_pos) = trimmed.find("const ") {
                let after_const = &trimmed[const_pos + 6..];
                if let Some(equals_pos) = after_const.find('=') {
                    return Some(after_const[..equals_pos].trim().to_string());
                }
            }
        }
        
        None
    }

    fn extract_java_csharp_function_name(line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        for part in parts.iter() {
            if part.contains('(') {
                return Some(part.split('(').next()?.to_string());
            }
        }
        None
    }

    // COBOL helper functions
    fn is_cobol_paragraph(line: &str) -> bool {
        let trimmed = line.trim();
        // COBOL paragraphs end with a period and don't contain certain keywords
        trimmed.ends_with('.') && !trimmed.contains(' ') && 
        !trimmed.is_empty() && trimmed.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '.')
    }

    fn is_cobol_section(line: &str) -> bool {
        let trimmed = line.trim().to_uppercase();
        trimmed.ends_with(" SECTION.") || trimmed.ends_with(" SECTION")
    }

    fn extract_cobol_procedure_name(line: &str) -> Option<String> {
        let trimmed = line.trim().to_uppercase();
        
        if trimmed.contains(" DIVISION") {
            // Extract division name
            if let Some(pos) = trimmed.find(" DIVISION") {
                return Some(trimmed[..pos].trim().to_string());
            }
        } else if trimmed.ends_with('.') {
            // Extract paragraph/section name
            let name = trimmed.trim_end_matches('.');
            if name.ends_with(" SECTION") {
                return Some(name.trim_end_matches(" SECTION").trim().to_string());
            } else {
                return Some(name.to_string());
            }
        }
        None
    }

    fn find_cobol_procedure_end(lines: &[&str], start: usize) -> usize {
        // COBOL procedures end at the next paragraph/section or end of program
        for (i, line) in lines.iter().enumerate().skip(start + 1) {
            let trimmed = line.trim().to_uppercase();
            if Self::is_cobol_paragraph(line) || Self::is_cobol_section(line) 
                || trimmed.starts_with("STOP RUN") || trimmed.starts_with("EXIT") {
                return i - 1;
            }
        }
        lines.len() - 1
    }

    // Fortran helper functions
    fn extract_fortran_procedure_name(line: &str) -> Option<String> {
        let trimmed = line.trim().to_uppercase();
        let keywords = ["SUBROUTINE ", "FUNCTION ", "PROGRAM ", "INTEGER FUNCTION ", 
                       "REAL FUNCTION ", "LOGICAL FUNCTION ", "CHARACTER FUNCTION "];
        
        for keyword in &keywords {
            if let Some(after_keyword) = trimmed.strip_prefix(keyword) {
                if let Some(paren_pos) = after_keyword.find('(') {
                    return Some(after_keyword[..paren_pos].trim().to_string());
                } else if let Some(space_pos) = after_keyword.find(' ') {
                    return Some(after_keyword[..space_pos].trim().to_string());
                } else {
                    return Some(after_keyword.trim().to_string());
                }
            }
        }
        None
    }

    fn find_fortran_procedure_end(lines: &[&str], start: usize) -> usize {
        let start_line = lines[start].trim().to_uppercase();
        let end_keyword = if start_line.starts_with("SUBROUTINE ") {
            "END SUBROUTINE"
        } else if start_line.starts_with("FUNCTION ") || start_line.contains(" FUNCTION ") {
            "END FUNCTION"
        } else if start_line.starts_with("PROGRAM ") {
            "END PROGRAM"
        } else {
            "END"
        };
        
        for (i, line) in lines.iter().enumerate().skip(start + 1) {
            let trimmed = line.trim().to_uppercase();
            if trimmed.starts_with(end_keyword) || trimmed == "END" {
                return i;
            }
        }
        lines.len() - 1
    }

    // Pascal/Delphi helper functions
    fn extract_pascal_procedure_name(line: &str) -> Option<String> {
        let trimmed = line.trim().to_lowercase();
        let keywords = ["procedure ", "function ", "constructor ", "destructor ", 
                       "class procedure ", "class function "];
        
        for keyword in &keywords {
            if let Some(after_keyword) = trimmed.strip_prefix(keyword) {
                if let Some(paren_pos) = after_keyword.find('(') {
                    return Some(after_keyword[..paren_pos].trim().to_string());
                } else if let Some(semicolon_pos) = after_keyword.find(';') {
                    return Some(after_keyword[..semicolon_pos].trim().to_string());
                } else if let Some(colon_pos) = after_keyword.find(':') {
                    return Some(after_keyword[..colon_pos].trim().to_string());
                }
            }
        }
        None
    }

    fn find_pascal_procedure_end(lines: &[&str], start: usize) -> usize {
        let mut begin_count = 0;
        let mut found_begin = false;
        
        for (i, line) in lines.iter().enumerate().skip(start) {
            let trimmed = line.trim().to_lowercase();
            if trimmed.contains("begin") {
                begin_count += 1;
                found_begin = true;
            } else if trimmed.contains("end") {
                begin_count -= 1;
                if found_begin && begin_count == 0 {
                    return i;
                }
            }
        }
        lines.len() - 1
    }

    // PL/SQL helper functions
    fn extract_plsql_procedure_name(line: &str) -> Option<String> {
        let trimmed = line.trim().to_uppercase();
        let keywords = ["CREATE OR REPLACE PROCEDURE ", "CREATE OR REPLACE FUNCTION ", 
                       "CREATE PROCEDURE ", "CREATE FUNCTION ", "PROCEDURE ", "FUNCTION "];
        
        for keyword in &keywords {
            if let Some(after_keyword) = trimmed.strip_prefix(keyword) {
                if let Some(paren_pos) = after_keyword.find('(') {
                    return Some(after_keyword[..paren_pos].trim().to_string());
                } else if let Some(space_pos) = after_keyword.find(' ') {
                    return Some(after_keyword[..space_pos].trim().to_string());
                }
            }
        }
        
        // Handle package definitions
        if trimmed.contains(" PACKAGE") {
            if let Some(package_pos) = trimmed.find(" PACKAGE") {
                let before_package = &trimmed[..package_pos];
                if let Some(space_pos) = before_package.rfind(' ') {
                    return Some(before_package[space_pos + 1..].trim().to_string());
                }
            }
        }
        None
    }

    fn find_plsql_procedure_end(lines: &[&str], start: usize) -> usize {
        for (i, line) in lines.iter().enumerate().skip(start + 1) {
            let trimmed = line.trim().to_uppercase();
            if trimmed.starts_with("END") && (trimmed.ends_with(";") || trimmed.contains("END ")) {
                return i;
            }
        }
        lines.len() - 1
    }

    // RPG helper functions
    fn extract_rpg_procedure_name(line: &str) -> Option<String> {
        let trimmed = line.trim().to_uppercase();
        
        if let Some(after_dcl) = trimmed.strip_prefix("DCL-PROC ") {
            // Modern RPG ILE
            if let Some(space_pos) = after_dcl.find(' ') {
                return Some(after_dcl[..space_pos].trim().to_string());
            } else {
                return Some(after_dcl.trim().to_string());
            }
        } else if line.len() > 6 && line.chars().nth(5) == Some('P') {
            // Fixed format RPG procedure
            let proc_name = line[6..].trim();
            if let Some(space_pos) = proc_name.find(' ') {
                return Some(proc_name[..space_pos].to_string());
            } else {
                return Some(proc_name.to_string());
            }
        } else if trimmed.contains("BEGSR") {
            // RPG subroutine
            if let Some(begsr_pos) = trimmed.find("BEGSR") {
                let after_begsr = &trimmed[begsr_pos + 5..].trim();
                return Some(after_begsr.to_string());
            }
        }
        None
    }

    fn find_rpg_procedure_end(lines: &[&str], start: usize) -> usize {
        for (i, line) in lines.iter().enumerate().skip(start + 1) {
            let trimmed = line.trim().to_uppercase();
            if trimmed.starts_with("END-PROC") || trimmed.contains("ENDSR") 
                || (line.len() > 6 && line.chars().nth(5) == Some('P') && line.contains("END")) {
                return i;
            }
        }
        lines.len() - 1
    }

    // Ada helper functions
    fn extract_ada_procedure_name(line: &str) -> Option<String> {
        let trimmed = line.trim().to_lowercase();
        let keywords = ["procedure ", "function ", "task ", "package ", "generic ", "protected "];
        
        for keyword in &keywords {
            if let Some(after_keyword) = trimmed.strip_prefix(keyword) {
                if let Some(paren_pos) = after_keyword.find('(') {
                    return Some(after_keyword[..paren_pos].trim().to_string());
                } else if let Some(space_pos) = after_keyword.find(' ') {
                    return Some(after_keyword[..space_pos].trim().to_string());
                } else if let Some(semicolon_pos) = after_keyword.find(';') {
                    return Some(after_keyword[..semicolon_pos].trim().to_string());
                }
            }
        }
        None
    }

    fn find_ada_procedure_end(lines: &[&str], start: usize) -> usize {
        let _start_line = lines[start].trim().to_lowercase();
        let proc_name = Self::extract_ada_procedure_name(lines[start]);
        
        for (i, line) in lines.iter().enumerate().skip(start + 1) {
            let trimmed = line.trim().to_lowercase();
            if trimmed.starts_with("end ") {
                if let Some(name) = &proc_name {
                    if trimmed.contains(name) {
                        return i;
                    }
                } else if trimmed == "end;" || trimmed.starts_with("end ") {
                    return i;
                }
            }
        }
        lines.len() - 1
    }

    // Go helper function
    fn extract_go_function_name(line: &str) -> Option<String> {
        let trimmed = line.trim();
        if let Some(after_func) = trimmed.strip_prefix("func ") {
            // Handle receiver functions: func (r *Receiver) FunctionName
            if after_func.starts_with('(') {
                if let Some(closing_paren) = after_func.find(')') {
                    let after_receiver = &after_func[closing_paren + 1..].trim();
                    if let Some(paren_pos) = after_receiver.find('(') {
                        return Some(after_receiver[..paren_pos].trim().to_string());
                    }
                }
            } else {
                // Regular function: func FunctionName
                if let Some(paren_pos) = after_func.find('(') {
                    return Some(after_func[..paren_pos].trim().to_string());
                }
            }
        }
        None
    }

    fn find_function_end(lines: &[&str], start: usize, open_char: &str, close_char: &str) -> usize {
        let mut brace_count = 0;
        let mut found_open = false;
        
        for (i, line) in lines.iter().enumerate().skip(start) {
            for char in line.chars() {
                if char.to_string() == open_char {
                    brace_count += 1;
                    found_open = true;
                } else if char.to_string() == close_char {
                    brace_count -= 1;
                    if found_open && brace_count == 0 {
                        return i;
                    }
                }
            }
        }
        
        lines.len() - 1
    }

    fn find_python_function_end(lines: &[&str], start: usize) -> usize {
        let def_line = lines[start];
        let base_indent = def_line.len() - def_line.trim_start().len();
        
        for (i, line) in lines.iter().enumerate().skip(start + 1) {
            if line.trim().is_empty() {
                continue;
            }
            
            let current_indent = line.len() - line.trim_start().len();
            if current_indent <= base_indent {
                return i - 1;
            }
        }
        
        lines.len() - 1
    }

    fn analyze_function_coverage(
        functions: &[FunctionCoverage],
        unit_tests: &str,
        integration_tests: &str,
    ) -> Vec<FunctionCoverage> {
        let all_tests = format!("{unit_tests}\n{integration_tests}");
        
        functions.iter().map(|func| {
            let is_covered = all_tests.contains(&func.name) 
                || all_tests.to_lowercase().contains(&func.name.to_lowercase());
            
            let mut test_cases = Vec::new();
            
            for line in all_tests.lines() {
                if line.contains(&func.name) && (line.contains("test") || line.contains("it(") || line.contains("describe(")) {
                    test_cases.push(line.trim().to_string());
                }
            }

            FunctionCoverage {
                name: func.name.clone(),
                line_start: func.line_start,
                line_end: func.line_end,
                is_covered,
                test_cases,
            }
        }).collect()
    }

    fn estimate_covered_lines(
        _functions: &[FunctionCoverage],
        covered_functions: &[FunctionCoverage],
    ) -> usize {
        let mut covered_lines = 0;
        
        for covered_func in covered_functions {
            if covered_func.is_covered {
                covered_lines += covered_func.line_end - covered_func.line_start + 1;
            }
        }
        
        covered_lines
    }

    pub fn generate_coverage_report(report: &CoverageReport) -> String {
        let mut output = String::new();
        
        output.push_str("📊 Test Coverage Report\n");
        output.push_str("========================\n\n");
        
        output.push_str(&format!("Overall Coverage: {:.1}% ({}/{} lines)\n\n", report.coverage_percentage, report.covered_lines, report.total_lines));
        
        output.push_str("File Coverage:\n");
        output.push_str("--------------\n");
        
        for (file_path, file_cov) in &report.file_coverage {
            let file_name = Path::new(file_path).file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file_path);
            
            output.push_str(&format!("{}: {:.1}% ({file_name}/{} lines)\n", file_cov.coverage_percentage, file_cov.covered_lines, file_cov.total_lines));
        }
        
        if !report.uncovered_functions.is_empty() {
            output.push_str("\nUncovered Functions:\n");
            output.push_str("-------------------\n");
            for func in &report.uncovered_functions {
                output.push_str(&format!("• {func}\n"));
            }
        }
        
        output.push_str("\n🎯 Coverage Target: Aim for 80%+ coverage\n");
        
        if report.coverage_percentage >= 90.0 {
            output.push_str("✅ Excellent coverage!\n");
        } else if report.coverage_percentage >= 80.0 {
            output.push_str("👍 Good coverage!\n");
        } else if report.coverage_percentage >= 60.0 {
            output.push_str("⚠️ Fair coverage - consider adding more tests\n");
        } else {
            output.push_str("❌ Low coverage - more tests needed\n");
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    fn create_test_rust_code() -> &'static str {
        r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

pub fn multiply(x: f64, y: f64) -> f64 {
    x * y
}

fn complex_function(input: &str) -> Result<String, std::io::Error> {
    if input.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Empty input"));
    }
    Ok(input.to_uppercase())
}
"#
    }

    fn create_test_python_code() -> &'static str {
        r#"
def add(a, b):
    return a + b

def subtract(a, b):
    return a - b

def multiply(x, y):
    return x * y

def complex_function(input_str):
    if not input_str:
        raise ValueError("Empty input")
    return input_str.upper()
"#
    }

    fn create_test_unit_tests() -> &'static str {
        r#"
#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);
    assert_eq!(add(-1, 1), 0);
}

#[test] 
fn test_multiply() {
    assert_eq!(multiply(2.0, 3.0), 6.0);
}

def test_add():
    assert add(2, 3) == 5
    assert add(-1, 1) == 0

def test_subtract():
    assert subtract(5, 2) == 3
"#
    }

    fn create_test_integration_tests() -> &'static str {
        r#"
#[test]
fn test_complex_function_integration() {
    assert!(complex_function("test").is_ok());
}

def test_complex_function_integration():
    result = complex_function("test")
    assert result == "TEST"
"#
    }

    #[test]
    fn test_extract_rust_functions() {
        let code = create_test_rust_code();
        let functions = CoverageAnalyzer::extract_functions(code, "rust").unwrap();
        
        assert_eq!(functions.len(), 4);
        
        let function_names: Vec<&str> = functions.iter().map(|f| f.name.as_str()).collect();
        assert!(function_names.contains(&"add"));
        assert!(function_names.contains(&"subtract"));
        assert!(function_names.contains(&"multiply"));
        assert!(function_names.contains(&"complex_function"));
        
        let add_fn = functions.iter().find(|f| f.name == "add").unwrap();
        assert_eq!(add_fn.line_start, 2);
        assert!(add_fn.line_end > add_fn.line_start);
    }

    #[test]
    fn test_extract_python_functions() {
        let code = create_test_python_code();
        let functions = CoverageAnalyzer::extract_functions(code, "python").unwrap();
        
        assert_eq!(functions.len(), 4);
        
        let function_names: Vec<&str> = functions.iter().map(|f| f.name.as_str()).collect();
        assert!(function_names.contains(&"add"));
        assert!(function_names.contains(&"subtract"));
        assert!(function_names.contains(&"multiply"));
        assert!(function_names.contains(&"complex_function"));
    }

    #[test]
    fn test_extract_rust_function_name() {
        assert_eq!(
            CoverageAnalyzer::extract_rust_function_name("fn add(a: i32, b: i32) -> i32 {"),
            Some("add".to_string())
        );
        
        assert_eq!(
            CoverageAnalyzer::extract_rust_function_name("pub fn multiply(x: f64, y: f64) -> f64 {"),
            Some("multiply".to_string())
        );
        
        assert_eq!(
            CoverageAnalyzer::extract_rust_function_name("    fn helper_function() {"),
            Some("helper_function".to_string())
        );
        
        assert_eq!(
            CoverageAnalyzer::extract_rust_function_name("let x = 5;"),
            None
        );
    }

    #[test]
    fn test_extract_python_function_name() {
        assert_eq!(
            CoverageAnalyzer::extract_python_function_name("def add(a, b):"),
            Some("add".to_string())
        );
        
        assert_eq!(
            CoverageAnalyzer::extract_python_function_name("    def helper_function():"),
            Some("helper_function".to_string())
        );
        
        assert_eq!(
            CoverageAnalyzer::extract_python_function_name("def complex_function(input_str, default=None):"),
            Some("complex_function".to_string())
        );
        
        assert_eq!(
            CoverageAnalyzer::extract_python_function_name("x = 5"),
            None
        );
    }

    #[test]
    fn test_extract_js_function_name() {
        assert_eq!(
            CoverageAnalyzer::extract_js_function_name("function add(a, b) {"),
            Some("add".to_string())
        );
        
        assert_eq!(
            CoverageAnalyzer::extract_js_function_name("const multiply = (x, y) => {"),
            Some("multiply".to_string())
        );
        
        assert_eq!(
            CoverageAnalyzer::extract_js_function_name("const helper = () => {"),
            Some("helper".to_string())
        );
        
        assert_eq!(
            CoverageAnalyzer::extract_js_function_name("let x = 5;"),
            None
        );
    }

    #[test]
    fn test_analyze_function_coverage() {
        let functions = vec![
            FunctionCoverage {
                name: "add".to_string(),
                line_start: 1,
                line_end: 3,
                is_covered: false,
                test_cases: Vec::new(),
            },
            FunctionCoverage {
                name: "multiply".to_string(),
                line_start: 5,
                line_end: 7,
                is_covered: false,
                test_cases: Vec::new(),
            },
            FunctionCoverage {
                name: "unused_function".to_string(),
                line_start: 9,
                line_end: 11,
                is_covered: false,
                test_cases: Vec::new(),
            },
        ];

        let unit_tests = create_test_unit_tests();
        let integration_tests = create_test_integration_tests();

        let covered_functions = CoverageAnalyzer::analyze_function_coverage(
            &functions, unit_tests, integration_tests
        );

        assert_eq!(covered_functions.len(), 3);
        
        let add_coverage = covered_functions.iter().find(|f| f.name == "add").unwrap();
        assert!(add_coverage.is_covered);
        assert!(!add_coverage.test_cases.is_empty());
        
        let multiply_coverage = covered_functions.iter().find(|f| f.name == "multiply").unwrap();
        assert!(multiply_coverage.is_covered);
        
        let unused_coverage = covered_functions.iter().find(|f| f.name == "unused_function").unwrap();
        assert!(!unused_coverage.is_covered);
        assert!(unused_coverage.test_cases.is_empty());
    }

    #[test]
    fn test_analyze_coverage_comprehensive() {
        let temp_dir = tempdir().unwrap();
        let source_file1 = temp_dir.path().join("lib1.rs");
        let source_file2 = temp_dir.path().join("lib2.rs");
        
        fs::write(&source_file1, create_test_rust_code()).unwrap();
        fs::write(&source_file2, "pub fn extra_function(x: i32) -> i32 { x * 2 }").unwrap();
        
        let source_files = vec![
            source_file1.to_string_lossy().to_string(),
            source_file2.to_string_lossy().to_string(),
        ];
        
        let unit_tests = create_test_unit_tests();
        let integration_tests = create_test_integration_tests();

        let report = CoverageAnalyzer::analyze_coverage(
            &source_files, unit_tests, integration_tests, "rust"
        ).unwrap();

        assert!(report.total_lines > 0);
        assert!(report.coverage_percentage >= 0.0);
        assert!(report.coverage_percentage <= 100.0);
        assert_eq!(report.file_coverage.len(), 2);
        
        // Check that some functions are marked as uncovered
        assert!(!report.uncovered_functions.is_empty());
        assert!(report.uncovered_functions.contains(&"extra_function".to_string()));
    }

    #[test]
    fn test_generate_coverage_report() {
        let mut file_coverage = HashMap::new();
        file_coverage.insert("test.rs".to_string(), FileCoverage {
            total_lines: 100,
            covered_lines: 85,
            coverage_percentage: 85.0,
            functions: vec![
                FunctionCoverage {
                    name: "covered_function".to_string(),
                    line_start: 1,
                    line_end: 10,
                    is_covered: true,
                    test_cases: vec!["test_covered".to_string()],
                },
                FunctionCoverage {
                    name: "uncovered_function".to_string(),
                    line_start: 11,
                    line_end: 20,
                    is_covered: false,
                    test_cases: Vec::new(),
                },
            ],
        });

        let report = CoverageReport {
            total_lines: 100,
            covered_lines: 85,
            coverage_percentage: 85.0,
            file_coverage,
            uncovered_functions: vec!["uncovered_function".to_string()],
        };

        let report_text = CoverageAnalyzer::generate_coverage_report(&report);
        
        assert!(report_text.contains("Test Coverage Report"));
        assert!(report_text.contains("85.0%"));
        assert!(report_text.contains("test.rs"));
        assert!(report_text.contains("uncovered_function"));
        assert!(report_text.contains("Good coverage!"));
    }

    #[test]
    fn test_coverage_report_different_thresholds() {
        let create_report = |coverage: f64| -> CoverageReport {
            CoverageReport {
                total_lines: 100,
                covered_lines: coverage as usize,
                coverage_percentage: coverage,
                file_coverage: HashMap::new(),
                uncovered_functions: Vec::new(),
            }
        };

        // Test excellent coverage
        let excellent_report = create_report(95.0);
        let excellent_text = CoverageAnalyzer::generate_coverage_report(&excellent_report);
        assert!(excellent_text.contains("Excellent coverage!"));

        // Test good coverage
        let good_report = create_report(85.0);
        let good_text = CoverageAnalyzer::generate_coverage_report(&good_report);
        assert!(good_text.contains("Good coverage!"));

        // Test fair coverage
        let fair_report = create_report(70.0);
        let fair_text = CoverageAnalyzer::generate_coverage_report(&fair_report);
        assert!(fair_text.contains("Fair coverage"));

        // Test low coverage
        let low_report = create_report(50.0);
        let low_text = CoverageAnalyzer::generate_coverage_report(&low_report);
        assert!(low_text.contains("Low coverage"));
    }

    #[test]
    fn test_find_function_end_rust() {
        let lines = vec![
            "fn add(a: i32, b: i32) -> i32 {",
            "    let result = a + b;",
            "    result",
            "}",
            "",
            "fn multiply(x: f64, y: f64) -> f64 {",
            "    x * y",
            "}",
        ];

        let end_pos = CoverageAnalyzer::find_function_end(&lines, 0, "{", "}");
        assert_eq!(end_pos, 3);

        let end_pos2 = CoverageAnalyzer::find_function_end(&lines, 5, "{", "}");
        assert_eq!(end_pos2, 7);
    }

    #[test]
    fn test_find_python_function_end() {
        let lines = vec![
            "def add(a, b):",
            "    return a + b",
            "",
            "def subtract(a, b):",
            "    result = a - b",
            "    return result",
            "",
            "x = 5",
        ];

        let end_pos = CoverageAnalyzer::find_python_function_end(&lines, 0);
        assert_eq!(end_pos, 2);

        let end_pos2 = CoverageAnalyzer::find_python_function_end(&lines, 3);
        assert_eq!(end_pos2, 6);
    }

    #[test]
    fn test_unsupported_language_error() {
        let result = CoverageAnalyzer::extract_functions("some code", "unsupported_lang");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            RigrError::UnsupportedLanguage(msg) => {
                assert!(msg.contains("Coverage analysis not supported"));
            }
            _ => panic!("Expected UnsupportedLanguage error"),
        }
    }

    #[test]
    fn test_empty_file_coverage() {
        let empty_functions: Vec<FunctionCoverage> = Vec::new();
        let covered = CoverageAnalyzer::analyze_function_coverage(&empty_functions, "", "");
        assert!(covered.is_empty());
    }

    #[test]
    fn test_file_coverage_calculation() {
        let file_coverage = FileCoverage {
            total_lines: 50,
            covered_lines: 40,
            coverage_percentage: 80.0,
            functions: Vec::new(),
        };

        assert_eq!(file_coverage.total_lines, 50);
        assert_eq!(file_coverage.covered_lines, 40);
        assert_eq!(file_coverage.coverage_percentage, 80.0);
    }

    #[test]
    fn test_function_coverage_clone() {
        let original = FunctionCoverage {
            name: "test_function".to_string(),
            line_start: 1,
            line_end: 10,
            is_covered: true,
            test_cases: vec!["test1".to_string(), "test2".to_string()],
        };

        let cloned = original.clone();
        assert_eq!(original.name, cloned.name);
        assert_eq!(original.line_start, cloned.line_start);
        assert_eq!(original.line_end, cloned.line_end);
        assert_eq!(original.is_covered, cloned.is_covered);
        assert_eq!(original.test_cases, cloned.test_cases);
    }

    #[test]
    fn test_coverage_with_case_insensitive_matching() {
        let functions = vec![
            FunctionCoverage {
                name: "TestFunction".to_string(),
                line_start: 1,
                line_end: 3,
                is_covered: false,
                test_cases: Vec::new(),
            },
        ];

        let test_code = "test_testfunction() { /* test for TestFunction */ }";
        
        let covered = CoverageAnalyzer::analyze_function_coverage(&functions, test_code, "");
        let test_func_coverage = &covered[0];
        
        assert!(test_func_coverage.is_covered);
    }
}