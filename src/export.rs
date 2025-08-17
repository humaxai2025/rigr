use crate::RigrError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use uuid::Uuid;
use rust_xlsxwriter::{Workbook, Format};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub test_id: String,
    pub test_name: String,
    pub test_type: String,
    pub priority: String,
    pub description: String,
    pub preconditions: String,
    pub test_steps: Vec<String>,
    pub expected_results: String,
    pub postconditions: String,
    pub test_data: String,
    pub pass_fail_criteria: String,
    pub category: String,
    pub tags: Vec<String>,
    pub estimated_duration: String,
    pub automation_candidate: bool,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    CSV,
    JSON,
    XML,
    TestRail,
    JiraXray,
    AzureDevOps,
    Markdown,
    // High Priority Formats
    Allure,
    JUnitXML,
    GitHubIssues,
    Cucumber,
    ExcelXLSX,
    // Medium Priority Formats
    Zephyr,
    TestLink,
    Confluence,
    PostmanCollections,
    RobotFramework,
}

pub struct TestCaseExporter;

impl TestCaseExporter {
    pub fn parse_test_cases_from_markdown(markdown_content: &str) -> Result<Vec<TestCase>, RigrError> {
        // Try parsing with the simple requirements-based format first
        if let Ok(test_cases) = Self::parse_simple_test_cases(markdown_content) {
            if !test_cases.is_empty() {
                return Ok(test_cases);
            }
        }
        
        // Fall back to the structured markdown format
        Self::parse_structured_markdown(markdown_content)
    }

    /// Parse simple test case format used by requirements-based generation
    /// Format: TEST CASE X:\nName: ...\nDescription: ...\nInput: ...\nExpected: ...
    pub fn parse_simple_test_cases(content: &str) -> Result<Vec<TestCase>, RigrError> {
        let mut test_cases = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i].trim();
            
            // Look for "TEST CASE X:" pattern
            if line.starts_with("TEST CASE") && line.ends_with(":") {
                let mut test_case = TestCase {
                    test_id: format!("TC{:03}", test_cases.len() + 1),
                    test_name: "".to_string(),
                    test_type: "Unit Test".to_string(),
                    priority: "Medium".to_string(),
                    description: "".to_string(),
                    preconditions: "".to_string(),
                    test_steps: Vec::new(),
                    expected_results: "".to_string(),
                    postconditions: "".to_string(),
                    test_data: "".to_string(),
                    pass_fail_criteria: "".to_string(),
                    category: "Functional".to_string(),
                    tags: Vec::new(),
                    estimated_duration: "5 minutes".to_string(),
                    automation_candidate: true,
                };
                
                i += 1; // Move to next line
                
                // Parse the fields for this test case
                while i < lines.len() {
                    let field_line = lines[i].trim();
                    
                    if field_line.is_empty() {
                        i += 1;
                        continue;
                    }
                    
                    // Check if we've reached the next test case
                    if field_line.starts_with("TEST CASE") && field_line.ends_with(":") {
                        break; // Start processing next test case
                    }
                    
                    if field_line.starts_with("Name:") {
                        test_case.test_name = field_line.replace("Name:", "").trim().to_string();
                    } else if field_line.starts_with("Description:") {
                        test_case.description = field_line.replace("Description:", "").trim().to_string();
                    } else if field_line.starts_with("Input:") {
                        test_case.test_data = field_line.replace("Input:", "").trim().to_string();
                        test_case.test_steps.push(format!("Execute test with input: {}", test_case.test_data));
                    } else if field_line.starts_with("Expected:") {
                        test_case.expected_results = field_line.replace("Expected:", "").trim().to_string();
                        test_case.pass_fail_criteria = format!("Test passes if: {}", test_case.expected_results);
                    }
                    
                    i += 1;
                }
                
                // Set default values if fields are empty
                if test_case.test_name.is_empty() {
                    test_case.test_name = format!("Test Case {}", test_cases.len() + 1);
                }
                if test_case.description.is_empty() {
                    test_case.description = "Generated test case description".to_string();
                }
                if test_case.test_steps.is_empty() {
                    test_case.test_steps.push("Execute the test scenario".to_string());
                }
                if test_case.expected_results.is_empty() {
                    test_case.expected_results = "Test should complete successfully".to_string();
                }
                
                // Set category and tags based on content analysis
                test_case.category = if test_case.description.to_lowercase().contains("security") || 
                                        test_case.description.to_lowercase().contains("injection") {
                    "Security".to_string()
                } else if test_case.description.to_lowercase().contains("performance") {
                    "Performance".to_string()
                } else {
                    "Functional".to_string()
                };
                
                test_case.tags = Self::extract_tags(&test_case.description, &test_case.test_name);
                
                test_cases.push(test_case);
            } else {
                i += 1;
            }
        }
        
        Ok(test_cases)
    }

    /// Parse structured markdown format with ### headers and ** fields
    pub fn parse_structured_markdown(markdown_content: &str) -> Result<Vec<TestCase>, RigrError> {
        let mut test_cases = Vec::new();
        let lines: Vec<&str> = markdown_content.lines().collect();
        let mut current_test_case: Option<TestCase> = None;
        let mut current_section = String::new();
        let mut test_counter = 1;

        for line in lines {
            let trimmed = line.trim();
            
            // Detect start of new test case
            if trimmed.starts_with("### Test Case") || trimmed.starts_with("## Test Case") {
                // Save previous test case if exists
                if let Some(test_case) = current_test_case.take() {
                    test_cases.push(test_case);
                }
                
                // Start new test case
                current_test_case = Some(TestCase {
                    test_id: format!("TC{test_counter:03}"),
                    test_name: "".to_string(),
                    test_type: "Unit Test".to_string(),
                    priority: "Medium".to_string(),
                    description: "".to_string(),
                    preconditions: "".to_string(),
                    test_steps: Vec::new(),
                    expected_results: "".to_string(),
                    postconditions: "".to_string(),
                    test_data: "".to_string(),
                    pass_fail_criteria: "".to_string(),
                    category: "Functional".to_string(),
                    tags: Vec::new(),
                    estimated_duration: "5 minutes".to_string(),
                    automation_candidate: true,
                });
                test_counter += 1;
                continue;
            }

            if let Some(ref mut test_case) = current_test_case {
                // Parse test case fields
                if trimmed.starts_with("**Test ID:**") {
                    test_case.test_id = trimmed.replace("**Test ID:**", "").trim().to_string();
                } else if trimmed.starts_with("**Test Name:**") {
                    test_case.test_name = trimmed.replace("**Test Name:**", "").trim().to_string();
                } else if trimmed.starts_with("**Test Type:**") {
                    test_case.test_type = trimmed.replace("**Test Type:**", "").trim().to_string();
                } else if trimmed.starts_with("**Priority:**") {
                    test_case.priority = trimmed.replace("**Priority:**", "").trim().to_string();
                } else if trimmed.starts_with("**Description:**") {
                    test_case.description = trimmed.replace("**Description:**", "").trim().to_string();
                    current_section = "description".to_string();
                } else if trimmed.starts_with("**Preconditions:**") {
                    test_case.preconditions = trimmed.replace("**Preconditions:**", "").trim().to_string();
                    current_section = "preconditions".to_string();
                } else if trimmed.starts_with("**Test Steps:**") {
                    current_section = "test_steps".to_string();
                } else if trimmed.starts_with("**Expected Results:**") {
                    test_case.expected_results = trimmed.replace("**Expected Results:**", "").trim().to_string();
                    current_section = "expected_results".to_string();
                } else if trimmed.starts_with("**Postconditions:**") {
                    test_case.postconditions = trimmed.replace("**Postconditions:**", "").trim().to_string();
                    current_section = "postconditions".to_string();
                } else if trimmed.starts_with("**Test Data:**") {
                    test_case.test_data = trimmed.replace("**Test Data:**", "").trim().to_string();
                    current_section = "test_data".to_string();
                } else if trimmed.starts_with("**Pass/Fail Criteria:**") {
                    test_case.pass_fail_criteria = trimmed.replace("**Pass/Fail Criteria:**", "").trim().to_string();
                    current_section = "pass_fail_criteria".to_string();
                } else if trimmed.starts_with(char::is_numeric) && current_section == "test_steps" {
                    // Extract numbered step
                    if let Some(step_content) = trimmed.split_once('.') {
                        test_case.test_steps.push(step_content.1.trim().to_string());
                    }
                } else if !trimmed.is_empty() && !trimmed.starts_with("**") && !trimmed.starts_with("#") {
                    // Continue content for current section
                    match current_section.as_str() {
                        "description" => {
                            if !test_case.description.is_empty() {
                                test_case.description.push(' ');
                            }
                            test_case.description.push_str(trimmed);
                        },
                        "preconditions" => {
                            if !test_case.preconditions.is_empty() {
                                test_case.preconditions.push(' ');
                            }
                            test_case.preconditions.push_str(trimmed);
                        },
                        "expected_results" => {
                            if !test_case.expected_results.is_empty() {
                                test_case.expected_results.push(' ');
                            }
                            test_case.expected_results.push_str(trimmed);
                        },
                        "postconditions" => {
                            if !test_case.postconditions.is_empty() {
                                test_case.postconditions.push(' ');
                            }
                            test_case.postconditions.push_str(trimmed);
                        },
                        "test_data" => {
                            if !test_case.test_data.is_empty() {
                                test_case.test_data.push(' ');
                            }
                            test_case.test_data.push_str(trimmed);
                        },
                        "pass_fail_criteria" => {
                            if !test_case.pass_fail_criteria.is_empty() {
                                test_case.pass_fail_criteria.push(' ');
                            }
                            test_case.pass_fail_criteria.push_str(trimmed);
                        },
                        _ => {}
                    }
                }
            }
        }

        // Add the last test case
        if let Some(test_case) = current_test_case {
            test_cases.push(test_case);
        }

        // Clean up and validate test cases
        for test_case in &mut test_cases {
            if test_case.test_name.is_empty() {
                test_case.test_name = format!("Test Case {}", test_case.test_id);
            }
            if test_case.description.is_empty() {
                test_case.description = "Generated test case description".to_string();
            }
            
            // Set category based on test type
            test_case.category = match test_case.test_type.to_lowercase().as_str() {
                t if t.contains("integration") => "Integration".to_string(),
                t if t.contains("performance") => "Performance".to_string(),
                t if t.contains("security") => "Security".to_string(),
                _ => "Functional".to_string(),
            };

            // Set tags based on content analysis
            test_case.tags = Self::extract_tags(&test_case.description, &test_case.test_name);
        }

        Ok(test_cases)
    }

    fn extract_tags(description: &str, test_name: &str) -> Vec<String> {
        let mut tags = Vec::new();
        let content = format!("{} {}", test_name.to_lowercase(), description.to_lowercase());

        // Common test patterns
        if content.contains("boundary") || content.contains("edge") || content.contains("limit") {
            tags.push("boundary-testing".to_string());
        }
        if content.contains("error") || content.contains("exception") || content.contains("fail") {
            tags.push("error-handling".to_string());
        }
        if content.contains("null") || content.contains("empty") || content.contains("invalid") {
            tags.push("input-validation".to_string());
        }
        if content.contains("performance") || content.contains("load") || content.contains("stress") {
            tags.push("performance".to_string());
        }
        if content.contains("security") || content.contains("auth") || content.contains("permission") {
            tags.push("security".to_string());
        }
        if content.contains("integration") || content.contains("component") || content.contains("interaction") {
            tags.push("integration".to_string());
        }

        if tags.is_empty() {
            tags.push("functional".to_string());
        }

        tags
    }

    pub fn export_to_csv(test_cases: &[TestCase], output_path: &str) -> Result<(), RigrError> {
        let mut writer = csv::Writer::from_path(output_path)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to create CSV file: {e}")))?;

        // Write header
        writer.write_record([
            "Test ID",
            "Test Name", 
            "Test Type",
            "Priority",
            "Category",
            "Description",
            "Preconditions",
            "Test Steps",
            "Expected Results",
            "Postconditions",
            "Test Data",
            "Pass/Fail Criteria",
            "Tags",
            "Estimated Duration",
            "Automation Candidate"
        ]).map_err(|e| RigrError::FileWriteError(format!("Failed to write CSV header: {e}")))?;

        // Write test cases
        for test_case in test_cases {
            let test_steps_joined = test_case.test_steps.join(" | ");
            let tags_joined = test_case.tags.join(", ");
            
            writer.write_record([
                &test_case.test_id,
                &test_case.test_name,
                &test_case.test_type,
                &test_case.priority,
                &test_case.category,
                &test_case.description,
                &test_case.preconditions,
                &test_steps_joined,
                &test_case.expected_results,
                &test_case.postconditions,
                &test_case.test_data,
                &test_case.pass_fail_criteria,
                &tags_joined,
                &test_case.estimated_duration,
                &test_case.automation_candidate.to_string(),
            ]).map_err(|e| RigrError::FileWriteError(format!("Failed to write test case to CSV: {e}")))?;
        }

        writer.flush()
            .map_err(|e| RigrError::FileWriteError(format!("Failed to flush CSV file: {e}")))?;

        Ok(())
    }

    pub fn export_to_json(test_cases: &[TestCase], output_path: &str) -> Result<(), RigrError> {
        let json_content = serde_json::to_string_pretty(test_cases)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to serialize to JSON: {e}")))?;

        fs::write(output_path, json_content)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to write JSON file: {e}")))?;

        Ok(())
    }

    pub fn export_to_testrail_csv(test_cases: &[TestCase], output_path: &str) -> Result<(), RigrError> {
        let mut writer = csv::Writer::from_path(output_path)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to create TestRail CSV file: {e}")))?;

        // TestRail specific header format
        writer.write_record([
            "ID",
            "Title",
            "Section",
            "Priority",
            "Type",
            "Preconditions",
            "Steps",
            "Expected Result",
            "References",
            "Automation Type",
            "Custom Tags"
        ]).map_err(|e| RigrError::FileWriteError(format!("Failed to write TestRail header: {e}")))?;

        for test_case in test_cases {
            let steps_formatted = test_case.test_steps.iter()
                .enumerate()
                .map(|(i, step)| format!("{}. {}", i + 1, step))
                .collect::<Vec<_>>()
                .join("\n");

            let priority_mapped = match test_case.priority.to_lowercase().as_str() {
                "high" => "High",
                "low" => "Low", 
                "critical" => "Critical",
                _ => "Medium",
            };

            writer.write_record([
                &test_case.test_id,
                &test_case.test_name,
                &test_case.category,
                priority_mapped,
                &test_case.test_type,
                &test_case.preconditions,
                &steps_formatted,
                &test_case.expected_results,
                "", // References - empty for auto-generated tests
                &(if test_case.automation_candidate { "Automated".to_string() } else { "Manual".to_string() }),
                &test_case.tags.join(", ")
            ]).map_err(|e| RigrError::FileWriteError(format!("Failed to write TestRail test case: {e}")))?;
        }

        writer.flush()
            .map_err(|e| RigrError::FileWriteError(format!("Failed to flush TestRail CSV: {e}")))?;

        Ok(())
    }

    pub fn export_to_jira_xray_json(test_cases: &[TestCase], output_path: &str) -> Result<(), RigrError> {
        let xray_tests: Vec<serde_json::Value> = test_cases.iter().map(|tc| {
            serde_json::json!({
                "fields": {
                    "summary": tc.test_name,
                    "description": tc.description,
                    "issuetype": {
                        "name": "Test"
                    },
                    "labels": tc.tags,
                    "priority": {
                        "name": tc.priority
                    },
                    "customfield_testtype": tc.test_type,
                    "customfield_preconditions": tc.preconditions,
                    "customfield_teststeps": tc.test_steps.iter().enumerate().map(|(i, step)| {
                        serde_json::json!({
                            "step": i + 1,
                            "action": step,
                            "data": "",
                            "result": ""
                        })
                    }).collect::<Vec<_>>(),
                    "customfield_expectedresult": tc.expected_results
                }
            })
        }).collect();

        let xray_payload = serde_json::json!({
            "issueUpdates": xray_tests
        });

        let json_content = serde_json::to_string_pretty(&xray_payload)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to serialize Xray JSON: {e}")))?;

        fs::write(output_path, json_content)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to write Xray JSON file: {e}")))?;

        Ok(())
    }

    pub fn export_to_azure_devops_json(test_cases: &[TestCase], output_path: &str) -> Result<(), RigrError> {
        let azure_tests: Vec<serde_json::Value> = test_cases.iter().map(|tc| {
            serde_json::json!({
                "op": "add",
                "path": "/fields/System.Title",
                "value": tc.test_name,
                "workItemType": "Test Case",
                "fields": {
                    "System.Description": tc.description,
                    "System.AreaPath": tc.category,
                    "Microsoft.VSTS.Common.Priority": match tc.priority.as_str() {
                        "Critical" => 1,
                        "High" => 2,
                        "Medium" => 3,
                        "Low" => 4,
                        _ => 3
                    },
                    "Microsoft.VSTS.TCM.Steps": tc.test_steps.iter().enumerate().map(|(i, step)| {
                        format!("<step id=\"{}\" type=\"ActionStep\"><parameterizedString isformatted=\"true\">{}</parameterizedString><parameterizedString isformatted=\"true\">{}</parameterizedString><description/></step>", 
                               i + 1, step, tc.expected_results)
                    }).collect::<Vec<_>>().join(""),
                    "Microsoft.VSTS.TCM.LocalDataSource": tc.test_data,
                    "System.Tags": tc.tags.join("; ")
                }
            })
        }).collect();

        let json_content = serde_json::to_string_pretty(&azure_tests)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to serialize Azure DevOps JSON: {e}")))?;

        fs::write(output_path, json_content)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to write Azure DevOps JSON file: {e}")))?;

        Ok(())
    }

    pub fn export_test_cases(
        markdown_files: &[String],
        output_dir: &str,
        formats: &[ExportFormat]
    ) -> Result<(), RigrError> {
        // Create export directory
        fs::create_dir_all(output_dir)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to create export directory: {e}")))?;

        let mut all_test_cases = Vec::new();

        // Parse all markdown files
        for markdown_file in markdown_files {
            if Path::new(markdown_file).exists() {
                let content = fs::read_to_string(markdown_file)
                    .map_err(|e| RigrError::FileReadError(format!("Failed to read {markdown_file}: {e}")))?;
                
                let mut test_cases = Self::parse_test_cases_from_markdown(&content)?;
                
                // Add source file context to test cases
                if let Some(file_name) = Path::new(markdown_file).file_stem().and_then(|s| s.to_str()) {
                    for test_case in &mut test_cases {
                        test_case.tags.push(format!("source:{file_name}"));
                    }
                }
                
                all_test_cases.extend(test_cases);
            }
        }

        if all_test_cases.is_empty() {
            return Err(RigrError::FileReadError("No test cases found to export".to_string()));
        }

        // Export in requested formats
        for format in formats {
            match format {
                ExportFormat::CSV => {
                    let csv_path = format!("{output_dir}/rigr_test_cases.csv");
                    Self::export_to_csv(&all_test_cases, &csv_path)?;
                    println!("✅ CSV export: {csv_path}");
                },
                ExportFormat::JSON => {
                    let json_path = format!("{output_dir}/rigr_test_cases.json");
                    Self::export_to_json(&all_test_cases, &json_path)?;
                    println!("✅ JSON export: {json_path}");
                },
                ExportFormat::TestRail => {
                    let testrail_path = format!("{output_dir}/rigr_test_cases_testrail.csv");
                    Self::export_to_testrail_csv(&all_test_cases, &testrail_path)?;
                    println!("✅ TestRail CSV export: {testrail_path}");
                },
                ExportFormat::JiraXray => {
                    let xray_path = format!("{output_dir}/rigr_test_cases_xray.json");
                    Self::export_to_jira_xray_json(&all_test_cases, &xray_path)?;
                    println!("✅ Jira Xray JSON export: {xray_path}");
                },
                ExportFormat::AzureDevOps => {
                    let azure_path = format!("{output_dir}/rigr_test_cases_azure.json");
                    Self::export_to_azure_devops_json(&all_test_cases, &azure_path)?;
                    println!("✅ Azure DevOps JSON export: {azure_path}");
                },
                ExportFormat::XML => {
                    // TODO: Implement XML export for enterprise tools
                    println!("⚠️ XML export not yet implemented");
                },
                ExportFormat::Markdown => {
                    // Already in markdown format - just copy consolidated version
                    let md_path = format!("{output_dir}/rigr_test_cases_consolidated.md");
                    Self::export_consolidated_markdown(&all_test_cases, &md_path)?;
                    println!("✅ Consolidated Markdown export: {md_path}");
                },
                // High Priority Formats
                ExportFormat::Allure => {
                    let allure_path = format!("{output_dir}/allure_test_results.json");
                    Self::export_to_allure_json(&all_test_cases, &allure_path)?;
                    println!("✅ Allure Framework export: {output_dir}/allure-results/");
                },
                ExportFormat::JUnitXML => {
                    let junit_path = format!("{output_dir}/rigr_test_cases_junit.xml");
                    Self::export_to_junit_xml(&all_test_cases, &junit_path)?;
                    println!("✅ JUnit XML export: {junit_path}");
                },
                ExportFormat::GitHubIssues => {
                    let github_path = format!("{output_dir}/rigr_test_cases_github_issues.json");
                    Self::export_to_github_issues_json(&all_test_cases, &github_path)?;
                    println!("✅ GitHub Issues export: {github_path}");
                },
                ExportFormat::Cucumber => {
                    let cucumber_path = format!("{output_dir}/cucumber_features");
                    Self::export_to_cucumber_feature(&all_test_cases, &cucumber_path)?;
                    println!("✅ Cucumber/Gherkin export: {output_dir}/features/");
                },
                ExportFormat::ExcelXLSX => {
                    let excel_path = format!("{output_dir}/rigr_test_cases.xlsx");
                    Self::export_to_excel_xlsx(&all_test_cases, &excel_path)?;
                    println!("✅ Excel XLSX export: {excel_path}");
                },
                // Medium Priority Formats
                ExportFormat::Zephyr => {
                    let zephyr_path = format!("{output_dir}/rigr_test_cases_zephyr.json");
                    Self::export_to_zephyr_json(&all_test_cases, &zephyr_path)?;
                    println!("✅ Zephyr Scale export: {zephyr_path}");
                },
                ExportFormat::TestLink => {
                    let testlink_path = format!("{output_dir}/rigr_test_cases_testlink.xml");
                    Self::export_to_testlink_xml(&all_test_cases, &testlink_path)?;
                    println!("✅ TestLink XML export: {testlink_path}");
                },
                ExportFormat::Confluence => {
                    let confluence_path = format!("{output_dir}/rigr_test_cases_confluence.txt");
                    Self::export_to_confluence_markup(&all_test_cases, &confluence_path)?;
                    println!("✅ Confluence Markup export: {confluence_path}");
                },
                ExportFormat::PostmanCollections => {
                    let postman_path = format!("{output_dir}/rigr_test_cases_postman.json");
                    Self::export_to_postman_collection(&all_test_cases, &postman_path)?;
                    println!("✅ Postman Collection export: {postman_path}");
                },
                ExportFormat::RobotFramework => {
                    let robot_path = format!("{output_dir}/rigr_test_cases.robot");
                    Self::export_to_robot_framework(&all_test_cases, &robot_path)?;
                    println!("✅ Robot Framework export: {robot_path}");
                },
            }
        }

        // Generate import instructions
        let instructions_path = format!("{output_dir}/IMPORT_INSTRUCTIONS.md");
        Self::generate_import_instructions(&instructions_path, formats)?;
        println!("📋 Import instructions: {instructions_path}");

        println!("\n🎉 Exported {} test cases in {} format(s)", all_test_cases.len(), formats.len());
        
        Ok(())
    }

    fn export_consolidated_markdown(test_cases: &[TestCase], output_path: &str) -> Result<(), RigrError> {
        let mut content = String::new();
        content.push_str("# Rigr Generated Test Cases - Consolidated\n\n");
        content.push_str(&format!("**Total Test Cases:** {}\n", test_cases.len()));
        content.push_str(&format!("**Generated:** {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        for (i, test_case) in test_cases.iter().enumerate() {
            content.push_str(&format!("## Test Case {}\n", i + 1));
            content.push_str(&format!("**Test ID:** {}\n", test_case.test_id));
            content.push_str(&format!("**Test Name:** {}\n", test_case.test_name));
            content.push_str(&format!("**Test Type:** {}\n", test_case.test_type));
            content.push_str(&format!("**Priority:** {}\n", test_case.priority));
            content.push_str(&format!("**Category:** {}\n", test_case.category));
            content.push_str(&format!("**Description:** {}\n", test_case.description));
            content.push_str(&format!("**Preconditions:** {}\n", test_case.preconditions));
            content.push_str("**Test Steps:**\n");
            for (step_num, step) in test_case.test_steps.iter().enumerate() {
                content.push_str(&format!("{}. {}\n", step_num + 1, step));
            }
            content.push_str(&format!("**Expected Results:** {}\n", test_case.expected_results));
            content.push_str(&format!("**Postconditions:** {}\n", test_case.postconditions));
            content.push_str(&format!("**Test Data:** {}\n", test_case.test_data));
            content.push_str(&format!("**Pass/Fail Criteria:** {}\n", test_case.pass_fail_criteria));
            content.push_str(&format!("**Tags:** {}\n", test_case.tags.join(", ")));
            content.push_str(&format!("**Estimated Duration:** {}\n", test_case.estimated_duration));
            content.push_str(&format!("**Automation Candidate:** {}\n\n", test_case.automation_candidate));
            content.push_str("---\n\n");
        }

        fs::write(output_path, content)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to write consolidated markdown: {e}")))?;

        Ok(())
    }

    // High Priority Exports
    pub fn export_to_allure_json(test_cases: &[TestCase], output_path: &str) -> Result<(), RigrError> {
        let allure_results: Vec<serde_json::Value> = test_cases.iter().map(|tc| {
            let test_uuid = Uuid::new_v4().to_string();
            
            serde_json::json!({
                "uuid": test_uuid,
                "name": tc.test_name,
                "fullName": format!("{}.{}", tc.category, tc.test_name.replace(" ", "_")),
                "description": tc.description,
                "descriptionHtml": format!("<p>{}</p>", tc.description),
                "status": "unknown",
                "statusDetails": {
                    "known": false,
                    "muted": false,
                    "flaky": false
                },
                "stage": "pending",
                "steps": tc.test_steps.iter().map(|step| {
                    serde_json::json!({
                        "name": step,
                        "status": "unknown",
                        "statusDetails": {},
                        "stage": "pending",
                        "steps": [],
                        "attachments": [],
                        "parameters": [],
                        "start": 0,
                        "stop": 0
                    })
                }).collect::<Vec<_>>(),
                "attachments": [],
                "parameters": [
                    {
                        "name": "Test Type",
                        "value": tc.test_type
                    },
                    {
                        "name": "Priority", 
                        "value": tc.priority
                    },
                    {
                        "name": "Category",
                        "value": tc.category
                    }
                ],
                "labels": tc.tags.iter().map(|tag| {
                    serde_json::json!({
                        "name": "tag",
                        "value": tag
                    })
                }).chain(vec![
                    serde_json::json!({
                        "name": "suite",
                        "value": tc.category
                    }),
                    serde_json::json!({
                        "name": "severity",
                        "value": match tc.priority.to_lowercase().as_str() {
                            "critical" => "blocker",
                            "high" => "critical", 
                            "medium" => "normal",
                            "low" => "minor",
                            _ => "normal"
                        }
                    }),
                    serde_json::json!({
                        "name": "feature",
                        "value": tc.category
                    })
                ]).collect::<Vec<_>>(),
                "links": [],
                "start": 0,
                "stop": 0,
                "testCaseId": tc.test_id,
                "historyId": format!("{}.{}", tc.category, tc.test_name.replace(" ", "_")),
                "testClass": tc.category,
                "testMethod": tc.test_name.replace(" ", "_"),
                "beforeStages": [],
                "afterStages": [],
                "extra": {
                    "categories": [],
                    "tags": tc.tags,
                    "severity": tc.priority,
                    "preconditions": tc.preconditions,
                    "expectedResults": tc.expected_results,
                    "estimatedDuration": tc.estimated_duration,
                    "automationCandidate": tc.automation_candidate
                }
            })
        }).collect();

        // Create results directory structure for Allure
        let base_dir = Path::new(output_path).parent().unwrap_or(Path::new("."));
        let allure_dir = base_dir.join("allure-results");
        fs::create_dir_all(&allure_dir)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to create allure-results directory: {e}")))?;

        // Write individual test result files (Allure requirement)
        for result in allure_results.iter() {
            let test_file_path = allure_dir.join(format!("{}-result.json", Uuid::new_v4()));
            let json_content = serde_json::to_string_pretty(result)
                .map_err(|e| RigrError::FileWriteError(format!("Failed to serialize Allure result: {e}")))?;
            
            fs::write(&test_file_path, json_content)
                .map_err(|e| RigrError::FileWriteError(format!("Failed to write Allure result file: {e}")))?;
        }

        // Write environment properties
        let env_path = allure_dir.join("environment.properties");
        let env_content = format!(
            "Test.Tool=Rigr\n\
             Test.Generator=AI-Powered\n\
             Total.Test.Cases={}\n\
             Generated.Date={}\n",
            test_cases.len(),
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        fs::write(&env_path, env_content)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to write Allure environment file: {e}")))?;

        // Write categories for test classification
        let categories_path = allure_dir.join("categories.json");
        let categories = serde_json::json!([
            {
                "name": "Unit Tests",
                "matchedStatuses": ["unknown"],
                "messageRegex": ".*unit.*"
            },
            {
                "name": "Integration Tests", 
                "matchedStatuses": ["unknown"],
                "messageRegex": ".*integration.*"
            },
            {
                "name": "Performance Tests",
                "matchedStatuses": ["unknown"], 
                "messageRegex": ".*performance.*"
            },
            {
                "name": "Security Tests",
                "matchedStatuses": ["unknown"],
                "messageRegex": ".*security.*"
            }
        ]);
        let categories_content = serde_json::to_string_pretty(&categories)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to serialize Allure categories: {e}")))?;
        fs::write(&categories_path, categories_content)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to write Allure categories file: {e}")))?;

        Ok(())
    }

    pub fn export_to_junit_xml(test_cases: &[TestCase], output_path: &str) -> Result<(), RigrError> {
        let mut xml_content = String::new();
        xml_content.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        
        // Group test cases by category for test suites
        let mut suites: std::collections::HashMap<String, Vec<&TestCase>> = std::collections::HashMap::new();
        for test_case in test_cases {
            suites.entry(test_case.category.clone()).or_default().push(test_case);
        }

        xml_content.push_str(&format!(
            "<testsuites name=\"Rigr Generated Tests\" tests=\"{}\" time=\"0\" timestamp=\"{}\">\n",
            test_cases.len(),
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S")
        ));

        for (suite_name, suite_tests) in suites {
            xml_content.push_str(&format!(
                "  <testsuite name=\"{}\" tests=\"{}\" time=\"0\" timestamp=\"{}\">\n",
                suite_name,
                suite_tests.len(),
                chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S")
            ));

            for test_case in suite_tests {
                let class_name = test_case.category.replace(" ", "_");
                let method_name = test_case.test_name.replace(" ", "_").replace("(", "").replace(")", "");
                
                xml_content.push_str(&format!(
                    "    <testcase name=\"{}\" classname=\"{}.{}\" time=\"0\">\n",
                    test_case.test_name,
                    class_name,
                    method_name
                ));
                
                xml_content.push_str(&format!(
                    "      <properties>\n\
                     <property name=\"test_id\" value=\"{}\"/>\n\
                     <property name=\"priority\" value=\"{}\"/>\n\
                     <property name=\"test_type\" value=\"{}\"/>\n\
                     <property name=\"automation_candidate\" value=\"{}\"/>\n\
                     <property name=\"estimated_duration\" value=\"{}\"/>\n\
                     </properties>\n",
                    test_case.test_id,
                    test_case.priority,
                    test_case.test_type,
                    test_case.automation_candidate,
                    test_case.estimated_duration
                ));
                
                // Add test description and steps as system-out
                xml_content.push_str("      <system-out><![CDATA[\n");
                xml_content.push_str(&format!("Description: {}\n", test_case.description));
                xml_content.push_str(&format!("Preconditions: {}\n", test_case.preconditions));
                xml_content.push_str("Test Steps:\n");
                for (i, step) in test_case.test_steps.iter().enumerate() {
                    xml_content.push_str(&format!("{}. {}\n", i + 1, step));
                }
                xml_content.push_str(&format!("Expected Results: {}\n", test_case.expected_results));
                xml_content.push_str(&format!("Tags: {}\n", test_case.tags.join(", ")));
                xml_content.push_str("      ]]></system-out>\n");
                
                xml_content.push_str("    </testcase>\n");
            }
            
            xml_content.push_str("  </testsuite>\n");
        }
        
        xml_content.push_str("</testsuites>\n");

        fs::write(output_path, xml_content)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to write JUnit XML file: {e}")))?;

        Ok(())
    }

    pub fn export_to_github_issues_json(test_cases: &[TestCase], output_path: &str) -> Result<(), RigrError> {
        let github_issues: Vec<serde_json::Value> = test_cases.iter().map(|tc| {
            let mut body = String::new();
            body.push_str(&format!("## Test Case: {}\n\n", tc.test_name));
            body.push_str(&format!("**Test ID:** {}\n", tc.test_id));
            body.push_str(&format!("**Priority:** {}\n", tc.priority));
            body.push_str(&format!("**Test Type:** {}\n", tc.test_type));
            body.push_str(&format!("**Category:** {}\n\n", tc.category));
            body.push_str(&format!("### Description\n{}\n\n", tc.description));
            
            if !tc.preconditions.is_empty() {
                body.push_str(&format!("### Preconditions\n{}\n\n", tc.preconditions));
            }
            
            if !tc.test_steps.is_empty() {
                body.push_str("### Test Steps\n");
                for (i, step) in tc.test_steps.iter().enumerate() {
                    body.push_str(&format!("{}. {}\n", i + 1, step));
                }
                body.push('\n');
            }
            
            body.push_str(&format!("### Expected Results\n{}\n\n", tc.expected_results));
            
            if !tc.test_data.is_empty() {
                body.push_str(&format!("### Test Data\n{}\n\n", tc.test_data));
            }
            
            body.push_str("### Additional Information\n");
            body.push_str(&format!("- **Estimated Duration:** {}\n", tc.estimated_duration));
            body.push_str(&format!("- **Automation Candidate:** {}\n", if tc.automation_candidate { "✅ Yes" } else { "❌ No" }));
            body.push_str(&format!("- **Pass/Fail Criteria:** {}\n\n", tc.pass_fail_criteria));
            body.push_str("---\n*Generated by Rigr - AI-Powered Test Case Generator*");

            let mut labels = vec!["test-case".to_string(), tc.category.to_lowercase().replace(" ", "-")];
            labels.extend(tc.tags.iter().map(|tag| tag.to_lowercase().replace(" ", "-")).collect::<Vec<_>>());
            labels.push(format!("priority-{}", tc.priority.to_lowercase()));
            if tc.automation_candidate {
                labels.push("automation-candidate".to_string());
            }

            serde_json::json!({
                "title": format!("[Test Case] {}", tc.test_name),
                "body": body,
                "labels": labels,
                "assignees": [],
                "milestone": null,
                "meta": {
                    "test_id": tc.test_id,
                    "test_type": tc.test_type,
                    "priority": tc.priority,
                    "category": tc.category,
                    "automation_candidate": tc.automation_candidate,
                    "estimated_duration": tc.estimated_duration
                }
            })
        }).collect();

        let github_export = serde_json::json!({
            "metadata": {
                "generator": "Rigr AI-Powered Test Case Generator",
                "version": "1.0.0",
                "generated_at": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                "total_test_cases": test_cases.len(),
                "instructions": {
                    "manual_import": "Create issues manually by copying title and body from each test case",
                    "api_import": "Use GitHub API to create issues programmatically",
                    "bulk_import": "Use GitHub's issue import feature with CSV conversion"
                }
            },
            "issues": github_issues
        });

        let json_content = serde_json::to_string_pretty(&github_export)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to serialize GitHub Issues JSON: {e}")))?;

        fs::write(output_path, json_content)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to write GitHub Issues JSON file: {e}")))?;

        Ok(())
    }

    pub fn export_to_cucumber_feature(test_cases: &[TestCase], output_path: &str) -> Result<(), RigrError> {
        let _feature_content = String::new();
        
        // Group test cases by category for different feature files
        let mut features: std::collections::HashMap<String, Vec<&TestCase>> = std::collections::HashMap::new();
        for test_case in test_cases {
            features.entry(test_case.category.clone()).or_default().push(test_case);
        }

        // Create a feature directory
        let base_dir = Path::new(output_path).parent().unwrap_or(Path::new("."));
        let features_dir = base_dir.join("features");
        fs::create_dir_all(&features_dir)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to create features directory: {e}")))?;

        for (feature_name, feature_tests) in features {
            let mut feature_file_content = String::new();
            
            feature_file_content.push_str(&format!("Feature: {feature_name}\n"));
            feature_file_content.push_str("  As a tester\n");
            feature_file_content.push_str(&format!("  I want to verify {} functionality\n", feature_name.to_lowercase()));
            feature_file_content.push_str("  So that I can ensure the system works correctly\n\n");

            for test_case in feature_tests {
                feature_file_content.push_str(&format!("  @{} @{} @{}\n",
                    test_case.priority.to_lowercase(),
                    test_case.test_type.replace(" ", "-").to_lowercase(),
                    test_case.category.replace(" ", "-").to_lowercase()
                ));
                
                // Add custom tags
                for tag in &test_case.tags {
                    feature_file_content.push_str(&format!("  @{}\n", tag.replace(" ", "-").to_lowercase()));
                }
                
                feature_file_content.push_str(&format!("  Scenario: {}\n", test_case.test_name));
                
                if !test_case.preconditions.is_empty() {
                    feature_file_content.push_str(&format!("    Given {}\n", test_case.preconditions));
                } else {
                    feature_file_content.push_str("    Given the system is ready for testing\n");
                }
                
                for step in &test_case.test_steps {
                    // Convert steps to Gherkin format
                    if step.to_lowercase().contains("verify") || step.to_lowercase().contains("check") || step.to_lowercase().contains("ensure") {
                        feature_file_content.push_str(&format!("    Then {step}\n"));
                    } else if step.to_lowercase().contains("click") || step.to_lowercase().contains("enter") || step.to_lowercase().contains("select") {
                        feature_file_content.push_str(&format!("    When {step}\n"));
                    } else {
                        feature_file_content.push_str(&format!("    And {step}\n"));
                    }
                }
                
                if !test_case.expected_results.is_empty() {
                    feature_file_content.push_str(&format!("    Then {}\n", test_case.expected_results));
                }
                
                feature_file_content.push('\n');
            }

            // Write feature file
            let feature_filename = format!("{}.feature", feature_name.replace(" ", "_").to_lowercase());
            let feature_file_path = features_dir.join(feature_filename);
            fs::write(&feature_file_path, feature_file_content)
                .map_err(|e| RigrError::FileWriteError(format!("Failed to write Cucumber feature file: {e}")))?;
        }

        Ok(())
    }

    pub fn export_to_excel_xlsx(test_cases: &[TestCase], output_path: &str) -> Result<(), RigrError> {
        let mut workbook = Workbook::new();
        
        // Create main test cases worksheet
        let worksheet = workbook.add_worksheet().set_name("Test Cases")
            .map_err(|e| RigrError::FileWriteError(format!("Failed to create Excel worksheet: {e}")))?;

        // Create formats for styling
        let header_format = Format::new()
            .set_bold()
            .set_background_color("#4472C4")
            .set_font_color("#FFFFFF")
            .set_border(rust_xlsxwriter::FormatBorder::Thin);

        let cell_format = Format::new()
            .set_border(rust_xlsxwriter::FormatBorder::Thin)
            .set_text_wrap();

        let priority_high_format = Format::new()
            .set_background_color("#FFE6E6")
            .set_border(rust_xlsxwriter::FormatBorder::Thin);

        let priority_medium_format = Format::new()
            .set_background_color("#FFF2E6")
            .set_border(rust_xlsxwriter::FormatBorder::Thin);

        let priority_low_format = Format::new()
            .set_background_color("#E6F7FF")
            .set_border(rust_xlsxwriter::FormatBorder::Thin);

        // Write headers
        let headers = vec![
            "Test ID", "Test Name", "Test Type", "Priority", "Category", 
            "Description", "Preconditions", "Test Steps", "Expected Results",
            "Postconditions", "Test Data", "Pass/Fail Criteria", "Tags",
            "Estimated Duration", "Automation Candidate"
        ];

        for (col, header) in headers.iter().enumerate() {
            worksheet.write_with_format(0, col as u16, *header, &header_format)
                .map_err(|e| RigrError::FileWriteError(format!("Failed to write Excel header: {e}")))?;
        }

        // Set column widths
        worksheet.set_column_width(0, 10.0).unwrap(); // Test ID
        worksheet.set_column_width(1, 25.0).unwrap(); // Test Name
        worksheet.set_column_width(2, 15.0).unwrap(); // Test Type
        worksheet.set_column_width(3, 10.0).unwrap(); // Priority
        worksheet.set_column_width(4, 15.0).unwrap(); // Category
        worksheet.set_column_width(5, 40.0).unwrap(); // Description
        worksheet.set_column_width(6, 25.0).unwrap(); // Preconditions
        worksheet.set_column_width(7, 40.0).unwrap(); // Test Steps
        worksheet.set_column_width(8, 30.0).unwrap(); // Expected Results
        worksheet.set_column_width(9, 20.0).unwrap(); // Postconditions
        worksheet.set_column_width(10, 20.0).unwrap(); // Test Data
        worksheet.set_column_width(11, 25.0).unwrap(); // Pass/Fail Criteria
        worksheet.set_column_width(12, 20.0).unwrap(); // Tags
        worksheet.set_column_width(13, 15.0).unwrap(); // Duration
        worksheet.set_column_width(14, 15.0).unwrap(); // Automation

        // Write test case data
        for (row, test_case) in test_cases.iter().enumerate() {
            let excel_row = (row + 1) as u32;
            
            // Choose format based on priority
            let row_format = match test_case.priority.to_lowercase().as_str() {
                "critical" | "high" => &priority_high_format,
                "medium" => &priority_medium_format,
                "low" => &priority_low_format,
                _ => &cell_format,
            };

            let test_steps_joined = test_case.test_steps.iter()
                .enumerate()
                .map(|(i, step)| format!("{}. {}", i + 1, step))
                .collect::<Vec<_>>()
                .join("\n");
            
            let tags_joined = test_case.tags.join(", ");
            let automation_status = if test_case.automation_candidate { "Yes" } else { "No" };

            let row_data = vec![
                &test_case.test_id,
                &test_case.test_name,
                &test_case.test_type,
                &test_case.priority,
                &test_case.category,
                &test_case.description,
                &test_case.preconditions,
                &test_steps_joined,
                &test_case.expected_results,
                &test_case.postconditions,
                &test_case.test_data,
                &test_case.pass_fail_criteria,
                &tags_joined,
                &test_case.estimated_duration,
                automation_status,
            ];

            for (col, data) in row_data.iter().enumerate() {
                worksheet.write_with_format(excel_row, col as u16, *data, row_format)
                    .map_err(|e| RigrError::FileWriteError(format!("Failed to write Excel cell: {e}")))?;
            }
        }

        // Create summary worksheet
        let summary_worksheet = workbook.add_worksheet().set_name("Summary")
            .map_err(|e| RigrError::FileWriteError(format!("Failed to create summary worksheet: {e}")))?;

        // Summary statistics
        let total_tests = test_cases.len();
        let high_priority = test_cases.iter().filter(|tc| tc.priority.to_lowercase() == "high" || tc.priority.to_lowercase() == "critical").count();
        let automation_candidates = test_cases.iter().filter(|tc| tc.automation_candidate).count();
        
        let summary_data = [
            ("Total Test Cases", total_tests.to_string()),
            ("High Priority Tests", high_priority.to_string()),
            ("Automation Candidates", automation_candidates.to_string()),
            ("Generated Date", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()),
            ("Tool", "Rigr - AI-Powered Test Case Generator".to_string()),
        ];

        summary_worksheet.write_with_format(0, 0, "Test Case Summary", &header_format).unwrap();
        for (row, (label, value)) in summary_data.iter().enumerate() {
            summary_worksheet.write_with_format((row + 2) as u32, 0, *label, &cell_format).unwrap();
            summary_worksheet.write_with_format((row + 2) as u32, 1, value, &cell_format).unwrap();
        }

        // Save workbook
        workbook.save(output_path)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to save Excel file: {e}")))?;

        Ok(())
    }

    // Medium Priority Exports
    pub fn export_to_zephyr_json(test_cases: &[TestCase], output_path: &str) -> Result<(), RigrError> {
        let zephyr_tests: Vec<serde_json::Value> = test_cases.iter().map(|tc| {
            serde_json::json!({
                "name": tc.test_name,
                "objective": tc.description,
                "precondition": tc.preconditions,
                "estimatedTime": tc.estimated_duration,
                "priority": match tc.priority.to_lowercase().as_str() {
                    "critical" => "Highest",
                    "high" => "High",
                    "medium" => "Medium", 
                    "low" => "Low",
                    _ => "Medium"
                },
                "status": "Draft",
                "folder": tc.category,
                "labels": tc.tags,
                "testScript": {
                    "type": "STEP_BY_STEP",
                    "steps": tc.test_steps.iter().enumerate().map(|(idx, step)| {
                        serde_json::json!({
                            "orderId": idx + 1,
                            "step": step,
                            "data": tc.test_data,
                            "result": tc.expected_results
                        })
                    }).collect::<Vec<_>>()
                },
                "customFields": {
                    "testType": tc.test_type,
                    "testId": tc.test_id,
                    "automationCandidate": tc.automation_candidate,
                    "passFailCriteria": tc.pass_fail_criteria,
                    "postconditions": tc.postconditions
                }
            })
        }).collect();

        let zephyr_export = serde_json::json!({
            "project": {
                "key": "RIGR",
                "name": "Rigr Generated Tests"
            },
            "testCases": zephyr_tests,
            "metadata": {
                "generator": "Rigr AI-Powered Test Case Generator",
                "version": "1.0.0",
                "generatedAt": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                "totalTestCases": test_cases.len(),
                "importInstructions": "Import via Zephyr Scale API or manual entry"
            }
        });

        let json_content = serde_json::to_string_pretty(&zephyr_export)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to serialize Zephyr JSON: {e}")))?;

        fs::write(output_path, json_content)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to write Zephyr JSON file: {e}")))?;

        Ok(())
    }

    pub fn export_to_testlink_xml(test_cases: &[TestCase], output_path: &str) -> Result<(), RigrError> {
        let mut xml_content = String::new();
        xml_content.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml_content.push_str("<testsuite name=\"Rigr Generated Test Suite\">\n");
        
        for test_case in test_cases {
            xml_content.push_str(&format!(
                "  <testcase name=\"{}\" id=\"{}\">\n",
                test_case.test_name.replace('"', "&quot;"),
                test_case.test_id
            ));
            
            xml_content.push_str(&format!(
                "    <summary><![CDATA[{}]]></summary>\n",
                test_case.description
            ));
            
            xml_content.push_str(&format!(
                "    <preconditions><![CDATA[{}]]></preconditions>\n",
                test_case.preconditions
            ));
            
            xml_content.push_str("    <steps>\n");
            for (i, step) in test_case.test_steps.iter().enumerate() {
                xml_content.push_str(&format!(
                    "      <step>\n        <step_number>{}</step_number>\n        <actions><![CDATA[{}]]></actions>\n        <expectedresults><![CDATA[{}]]></expectedresults>\n      </step>\n",
                    i + 1,
                    step,
                    test_case.expected_results
                ));
            }
            xml_content.push_str("    </steps>\n");
            
            xml_content.push_str(&format!(
                "    <importance>{}</importance>\n",
                match test_case.priority.to_lowercase().as_str() {
                    "critical" => "3",
                    "high" => "3",
                    "medium" => "2",
                    "low" => "1",
                    _ => "2"
                }
            ));
            
            xml_content.push_str(&format!(
                "    <execution_type>{}</execution_type>\n",
                if test_case.automation_candidate { "2" } else { "1" } // 1=Manual, 2=Automated
            ));
            
            xml_content.push_str(&format!(
                "    <estimated_exec_duration>{}</estimated_exec_duration>\n",
                test_case.estimated_duration
            ));
            
            // Add custom fields
            xml_content.push_str("    <custom_fields>\n");
            xml_content.push_str(&format!("      <custom_field name=\"Test Type\" value=\"{}\"/>\n", test_case.test_type));
            xml_content.push_str(&format!("      <custom_field name=\"Category\" value=\"{}\"/>\n", test_case.category));
            xml_content.push_str(&format!("      <custom_field name=\"Tags\" value=\"{}\"/>\n", test_case.tags.join(", ")));
            xml_content.push_str("    </custom_fields>\n");
            
            xml_content.push_str("  </testcase>\n");
        }
        
        xml_content.push_str("</testsuite>\n");

        fs::write(output_path, xml_content)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to write TestLink XML file: {e}")))?;

        Ok(())
    }

    pub fn export_to_confluence_markup(test_cases: &[TestCase], output_path: &str) -> Result<(), RigrError> {
        let mut content = String::new();
        
        content.push_str("h1. Rigr Generated Test Cases\n\n");
        content.push_str(&format!("*Generated:* {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        content.push_str(&format!("*Total Test Cases:* {}\n\n", test_cases.len()));
        
        // Create summary table
        content.push_str("h2. Test Case Summary\n\n");
        content.push_str("||Test ID||Test Name||Priority||Type||Category||Automation||\n");
        
        for test_case in test_cases {
            content.push_str(&format!(
                "|{}|{}|{}|{}|{}|{}|\n",
                test_case.test_id,
                test_case.test_name,
                test_case.priority,
                test_case.test_type,
                test_case.category,
                if test_case.automation_candidate { "✓" } else { "✗" }
            ));
        }
        
        content.push('\n');
        
        // Detailed test cases
        content.push_str("h2. Detailed Test Cases\n\n");
        
        for test_case in test_cases.iter() {
            content.push_str(&format!("h3. {} - {}\n\n", test_case.test_id, test_case.test_name));
            
            content.push_str("||Property||Value||\n");
            content.push_str(&format!("|Test ID|{}|\n", test_case.test_id));
            content.push_str(&format!("|Priority|{}|\n", test_case.priority));
            content.push_str(&format!("|Type|{}|\n", test_case.test_type));
            content.push_str(&format!("|Category|{}|\n", test_case.category));
            content.push_str(&format!("|Estimated Duration|{}|\n", test_case.estimated_duration));
            content.push_str(&format!("|Automation Candidate|{}|\n", if test_case.automation_candidate { "Yes" } else { "No" }));
            content.push('\n');
            
            content.push_str("*Description:*\n");
            content.push_str(&format!("{}\n\n", test_case.description));
            
            if !test_case.preconditions.is_empty() {
                content.push_str("*Preconditions:*\n");
                content.push_str(&format!("{}\n\n", test_case.preconditions));
            }
            
            if !test_case.test_steps.is_empty() {
                content.push_str("*Test Steps:*\n");
                for step in test_case.test_steps.iter() {
                    content.push_str(&format!("# {step}\n"));
                }
                content.push('\n');
            }
            
            content.push_str("*Expected Results:*\n");
            content.push_str(&format!("{}\n\n", test_case.expected_results));
            
            if !test_case.postconditions.is_empty() {
                content.push_str("*Postconditions:*\n");
                content.push_str(&format!("{}\n\n", test_case.postconditions));
            }
            
            if !test_case.test_data.is_empty() {
                content.push_str("*Test Data:*\n");
                content.push_str(&format!("{{{{color:#666666}}}}{}{{{{color}}}}\n\n", test_case.test_data));
            }
            
            content.push_str("*Pass/Fail Criteria:*\n");
            content.push_str(&format!("{}\n\n", test_case.pass_fail_criteria));
            
            if !test_case.tags.is_empty() {
                content.push_str("*Tags:* ");
                for tag in &test_case.tags {
                    content.push_str(&format!("{{{tag}}} "));
                }
                content.push_str("\n\n");
            }
            
            content.push_str("----\n\n");
        }
        
        content.push_str("\n{info}Generated by Rigr - AI-Powered Test Case Generator{info}\n");

        fs::write(output_path, content)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to write Confluence markup file: {e}")))?;

        Ok(())
    }

    pub fn export_to_postman_collection(test_cases: &[TestCase], output_path: &str) -> Result<(), RigrError> {
        let collection_id = Uuid::new_v4().to_string();
        
        let postman_items: Vec<serde_json::Value> = test_cases.iter().map(|tc| {
            let item_id = Uuid::new_v4().to_string();
            
            // Convert test steps to Postman requests
            let requests: Vec<serde_json::Value> = tc.test_steps.iter().enumerate().map(|(i, step)| {
                serde_json::json!({
                    "name": format!("Step {} - {step}", i + 1),
                    "id": Uuid::new_v4().to_string(),
                    "request": {
                        "method": "GET",
                        "header": [],
                        "url": {
                            "raw": "{{base_url}}/test-endpoint",
                            "host": ["{{base_url}}"],
                            "path": ["test-endpoint"]
                        },
                        "description": step
                    },
                    "response": [],
                    "event": [
                        {
                            "listen": "test",
                            "script": {
                                "type": "text/javascript",
                                "exec": [
                                    format!("// Test: {}", tc.test_name),
                                    format!("// Expected: {}", tc.expected_results),
                                    "pm.test(\"Status code is 200\", function () {",
                                    "    pm.response.to.have.status(200);",
                                    "});"
                                ]
                            }
                        }
                    ]
                })
            }).collect();
            
            serde_json::json!({
                "name": tc.test_name,
                "id": item_id,
                "description": format!(
                    "**Test ID:** {}\n**Priority:** {}\n**Type:** {}\n\n**Description:**\n{}\n\n**Expected Results:**\n{}",
                    tc.test_id, tc.priority, tc.test_type, tc.description, tc.expected_results
                ),
                "item": requests,
                "event": [],
                "variable": [
                    {
                        "key": "test_id",
                        "value": tc.test_id,
                        "type": "string"
                    },
                    {
                        "key": "priority", 
                        "value": tc.priority,
                        "type": "string"
                    }
                ]
            })
        }).collect();
        
        let postman_collection = serde_json::json!({
            "info": {
                "name": "Rigr Generated API Tests",
                "description": "API test collection generated by Rigr AI-Powered Test Case Generator",
                "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json",
                "_postman_id": collection_id,
                "version": {
                    "major": 1,
                    "minor": 0,
                    "patch": 0
                }
            },
            "item": postman_items,
            "event": [],
            "variable": [
                {
                    "key": "base_url",
                    "value": "https://api.example.com",
                    "type": "string",
                    "description": "Base URL for API testing"
                }
            ],
            "auth": {
                "type": "noauth"
            }
        });

        let json_content = serde_json::to_string_pretty(&postman_collection)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to serialize Postman collection: {e}")))?;

        fs::write(output_path, json_content)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to write Postman collection file: {e}")))?;

        Ok(())
    }

    pub fn export_to_robot_framework(test_cases: &[TestCase], output_path: &str) -> Result<(), RigrError> {
        let mut robot_content = String::new();
        
        robot_content.push_str("*** Settings ***\n");
        robot_content.push_str("Documentation    Rigr Generated Robot Framework Test Suite\n");
        robot_content.push_str("Library          Collections\n");
        robot_content.push_str("Library          String\n");
        robot_content.push('\n');
        
        robot_content.push_str("*** Variables ***\n");
        robot_content.push_str("${GENERATED_BY}    Rigr AI-Powered Test Case Generator\n");
        robot_content.push_str(&format!("${{GENERATED_DATE}}    {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        robot_content.push_str(&format!("${{TOTAL_TESTS}}    {}\n", test_cases.len()));
        robot_content.push('\n');
        
        robot_content.push_str("*** Test Cases ***\n");
        
        for test_case in test_cases {
            // Convert test name to Robot Framework format
            let robot_test_name = test_case.test_name.replace(" ", "_").replace("-", "_");
            
            robot_content.push_str(&format!("{robot_test_name}\n"));
            robot_content.push_str(&format!("    [Documentation]    {}\n", test_case.description));
            robot_content.push_str(&format!("    [Tags]    {}\n", test_case.tags.join("    ")));
            robot_content.push_str(&format!("    {}    {}", test_case.category.replace(" ", "_").to_lowercase(), test_case.priority.to_lowercase()));
            if test_case.automation_candidate {
                robot_content.push_str("    automation");
            }
            robot_content.push('\n');
            
            // Add test metadata
            robot_content.push_str(&format!("    [Setup]    Log    Test ID: {}\n", test_case.test_id));
            
            // Add preconditions if any
            if !test_case.preconditions.is_empty() {
                robot_content.push_str(&format!("    Given    {}\n", test_case.preconditions));
            }
            
            // Convert test steps to Robot Framework keywords
            for (i, step) in test_case.test_steps.iter().enumerate() {
                let keyword = if step.to_lowercase().contains("verify") || step.to_lowercase().contains("check") {
                    "Then"
                } else if step.to_lowercase().contains("click") || step.to_lowercase().contains("enter") {
                    "When"
                } else if i == 0 {
                    "Given"
                } else {
                    "And"
                };
                
                robot_content.push_str(&format!("    {keyword}    {step}\n"));
            }
            
            // Add expected results verification
            if !test_case.expected_results.is_empty() {
                robot_content.push_str(&format!("    Then    {}\n", test_case.expected_results));
            }
            
            // Add postconditions if any
            if !test_case.postconditions.is_empty() {
                robot_content.push_str(&format!("    [Teardown]    {}\n", test_case.postconditions));
            } else {
                robot_content.push_str(&format!("    [Teardown]    Log    Test {} completed\n", test_case.test_id));
            }
            
            robot_content.push('\n');
        }
        
        robot_content.push_str("*** Keywords ***\n");
        robot_content.push_str("Given\n");
        robot_content.push_str("    [Arguments]    ${condition}\n");
        robot_content.push_str("    Log    Precondition: ${condition}\n");
        robot_content.push('\n');
        
        robot_content.push_str("When\n");
        robot_content.push_str("    [Arguments]    ${action}\n");
        robot_content.push_str("    Log    Action: ${action}\n");
        robot_content.push('\n');
        
        robot_content.push_str("Then\n");
        robot_content.push_str("    [Arguments]    ${expected}\n");
        robot_content.push_str("    Log    Verification: ${expected}\n");
        robot_content.push('\n');
        
        robot_content.push_str("And\n");
        robot_content.push_str("    [Arguments]    ${step}\n");
        robot_content.push_str("    Log    Additional step: ${step}\n");

        fs::write(output_path, robot_content)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to write Robot Framework file: {e}")))?;

        Ok(())
    }

    fn generate_import_instructions(output_path: &str, formats: &[ExportFormat]) -> Result<(), RigrError> {
        let mut instructions = String::new();
        instructions.push_str("# Test Case Import Instructions\n\n");
        instructions.push_str("This document provides step-by-step instructions for importing Rigr-generated test cases into popular test management tools.\n\n");

        for format in formats {
            match format {
                ExportFormat::CSV => {
                    instructions.push_str("## Generic CSV Import (Excel, Google Sheets, etc.)\n\n");
                    instructions.push_str("**File:** `rigr_test_cases.csv`\n\n");
                    instructions.push_str("### Steps:\n");
                    instructions.push_str("1. Open your spreadsheet application (Excel, Google Sheets, etc.)\n");
                    instructions.push_str("2. Import the CSV file using 'File > Import' or 'Data > Import'\n");
                    instructions.push_str("3. Ensure UTF-8 encoding is selected during import\n");
                    instructions.push_str("4. Review and adjust column mappings as needed\n");
                    instructions.push_str("5. Use filters and sorting to organize test cases by priority, category, or tags\n\n");
                },
                ExportFormat::TestRail => {
                    instructions.push_str("## TestRail Import\n\n");
                    instructions.push_str("**File:** `rigr_test_cases_testrail.csv`\n\n");
                    instructions.push_str("### Steps:\n");
                    instructions.push_str("1. Log into your TestRail instance\n");
                    instructions.push_str("2. Navigate to your project\n");
                    instructions.push_str("3. Go to 'Test Cases' section\n");
                    instructions.push_str("4. Click 'Import' button\n");
                    instructions.push_str("5. Select 'CSV' as import format\n");
                    instructions.push_str("6. Upload the `rigr_test_cases_testrail.csv` file\n");
                    instructions.push_str("7. Map the CSV columns to TestRail fields:\n");
                    instructions.push_str("   - ID → Test Case ID\n");
                    instructions.push_str("   - Title → Title\n");
                    instructions.push_str("   - Section → Section\n");
                    instructions.push_str("   - Priority → Priority\n");
                    instructions.push_str("   - Type → Type\n");
                    instructions.push_str("   - Preconditions → Preconditions\n");
                    instructions.push_str("   - Steps → Steps\n");
                    instructions.push_str("   - Expected Result → Expected Result\n");
                    instructions.push_str("8. Review preview and click 'Import'\n");
                    instructions.push_str("9. Organize test cases into test suites as needed\n\n");
                },
                ExportFormat::JiraXray => {
                    instructions.push_str("## Jira Xray Import\n\n");
                    instructions.push_str("**File:** `rigr_test_cases_xray.json`\n\n");
                    instructions.push_str("### Steps:\n");
                    instructions.push_str("1. Open Jira and navigate to your project\n");
                    instructions.push_str("2. Go to 'Apps' > 'Xray' > 'Import Tests'\n");
                    instructions.push_str("3. Select 'JSON' as import format\n");
                    instructions.push_str("4. Upload the `rigr_test_cases_xray.json` file\n");
                    instructions.push_str("5. Review the import preview\n");
                    instructions.push_str("6. Configure test execution settings if needed\n");
                    instructions.push_str("7. Click 'Import' to create test issues\n");
                    instructions.push_str("8. Organize tests into test sets for execution planning\n\n");
                },
                ExportFormat::AzureDevOps => {
                    instructions.push_str("## Azure DevOps Import\n\n");
                    instructions.push_str("**File:** `rigr_test_cases_azure.json`\n\n");
                    instructions.push_str("### Steps:\n");
                    instructions.push_str("1. Open Azure DevOps and navigate to your project\n");
                    instructions.push_str("2. Go to 'Test Plans' in the left navigation\n");
                    instructions.push_str("3. Create a new Test Plan or select existing one\n");
                    instructions.push_str("4. Click 'Import test cases' from the toolbar\n");
                    instructions.push_str("5. Select 'JSON' format and upload the file\n");
                    instructions.push_str("6. Map JSON fields to Azure DevOps work item fields\n");
                    instructions.push_str("7. Review import preview and adjust settings\n");
                    instructions.push_str("8. Click 'Import' to create test case work items\n");
                    instructions.push_str("9. Add test cases to test suites within your test plan\n\n");
                },
                ExportFormat::JSON => {
                    instructions.push_str("## Generic JSON Import\n\n");
                    instructions.push_str("**File:** `rigr_test_cases.json`\n\n");
                    instructions.push_str("### Use Cases:\n");
                    instructions.push_str("- Custom test management tool integrations\n");
                    instructions.push_str("- API-based imports\n");
                    instructions.push_str("- Data processing and analytics\n");
                    instructions.push_str("- Integration with CI/CD pipelines\n\n");
                    instructions.push_str("### Sample Python Script:\n");
                    instructions.push_str("```python\n");
                    instructions.push_str("import json\n");
                    instructions.push_str("with open('rigr_test_cases.json', 'r') as f:\n");
                    instructions.push_str("    test_cases = json.load(f)\n");
                    instructions.push_str("    for tc in test_cases:\n");
                    instructions.push_str("        print(f\"Test: {tc['test_name']} - {tc['priority']}\")\n");
                    instructions.push_str("```\n\n");
                },
                ExportFormat::Markdown => {
                    instructions.push_str("## Markdown Documentation\n\n");
                    instructions.push_str("**File:** `rigr_test_cases_consolidated.md`\n\n");
                    instructions.push_str("### Use Cases:\n");
                    instructions.push_str("- Documentation in wikis (Confluence, Notion, GitHub)\n");
                    instructions.push_str("- Review and approval processes\n");
                    instructions.push_str("- Test case documentation in repositories\n");
                    instructions.push_str("- Conversion to other formats (PDF, HTML)\n\n");
                },
                ExportFormat::XML => {
                    instructions.push_str("## XML Import (Coming Soon)\n\n");
                    instructions.push_str("XML export will support enterprise test management tools.\n\n");
                },
                // High Priority Formats
                ExportFormat::Allure => {
                    instructions.push_str("## Allure Framework Import\n\n");
                    instructions.push_str("**Directory:** `allure-results/`\n\n");
                    instructions.push_str("### Steps:\n");
                    instructions.push_str("1. Copy the generated `allure-results` directory to your project\n");
                    instructions.push_str("2. Install Allure command-line tool: `npm install -g allure-commandline`\n");
                    instructions.push_str("3. Generate report: `allure generate allure-results --clean -o allure-report`\n");
                    instructions.push_str("4. Open report: `allure open allure-report`\n");
                    instructions.push_str("5. For CI/CD integration, add allure-results to your test execution pipeline\n");
                    instructions.push_str("6. View detailed test results with trends and history\n\n");
                },
                ExportFormat::JUnitXML => {
                    instructions.push_str("## JUnit XML Import\n\n");
                    instructions.push_str("**File:** `rigr_test_cases_junit.xml`\n\n");
                    instructions.push_str("### Compatible Tools:\n");
                    instructions.push_str("- Jenkins, TeamCity, Azure DevOps, GitHub Actions\n");
                    instructions.push_str("- IntelliJ IDEA, Eclipse, Visual Studio\n");
                    instructions.push_str("- SonarQube, Codecov, Code Climate\n\n");
                    instructions.push_str("### Steps:\n");
                    instructions.push_str("1. Copy the XML file to your test results directory\n");
                    instructions.push_str("2. Configure your CI/CD tool to read JUnit XML format\n");
                    instructions.push_str("3. For Jenkins: Use 'Publish JUnit test result report' post-build action\n");
                    instructions.push_str("4. For GitHub Actions: Use test reporting actions that support JUnit XML\n");
                    instructions.push_str("5. View test results in your build reports\n\n");
                },
                ExportFormat::GitHubIssues => {
                    instructions.push_str("## GitHub Issues Import\n\n");
                    instructions.push_str("**File:** `rigr_test_cases_github_issues.json`\n\n");
                    instructions.push_str("### Manual Import:\n");
                    instructions.push_str("1. Open the JSON file and copy issue titles and bodies\n");
                    instructions.push_str("2. Create new issues in your GitHub repository\n");
                    instructions.push_str("3. Apply the suggested labels from the export\n\n");
                    instructions.push_str("### API Import (Bulk):\n");
                    instructions.push_str("1. Use GitHub API with personal access token\n");
                    instructions.push_str("2. Script to create issues programmatically:\n");
                    instructions.push_str("```bash\n");
                    instructions.push_str("# Example using curl\n");
                    instructions.push_str("curl -X POST -H \"Authorization: token YOUR_TOKEN\" \\\n");
                    instructions.push_str("  -d '{\"title\":\"Issue Title\",\"body\":\"Issue Body\"}' \\\n");
                    instructions.push_str("  https://api.github.com/repos/owner/repo/issues\n");
                    instructions.push_str("```\n\n");
                },
                ExportFormat::Cucumber => {
                    instructions.push_str("## Cucumber/Gherkin Import\n\n");
                    instructions.push_str("**Directory:** `features/`\n\n");
                    instructions.push_str("### Steps:\n");
                    instructions.push_str("1. Copy `.feature` files to your Cucumber project's features directory\n");
                    instructions.push_str("2. Implement step definitions for the generated steps\n");
                    instructions.push_str("3. Run with Cucumber: `cucumber features/`\n");
                    instructions.push_str("4. For Java: Place in `src/test/resources/features/`\n");
                    instructions.push_str("5. For JavaScript: Place in project root or specify path in config\n");
                    instructions.push_str("6. Update step definitions to match your application's actions\n\n");
                },
                ExportFormat::ExcelXLSX => {
                    instructions.push_str("## Excel/XLSX Import\n\n");
                    instructions.push_str("**File:** `rigr_test_cases.xlsx`\n\n");
                    instructions.push_str("### Steps:\n");
                    instructions.push_str("1. Open in Microsoft Excel, LibreOffice Calc, or Google Sheets\n");
                    instructions.push_str("2. Review test cases in 'Test Cases' worksheet\n");
                    instructions.push_str("3. Check summary statistics in 'Summary' worksheet\n");
                    instructions.push_str("4. Use filters to organize by priority, category, or automation candidate\n");
                    instructions.push_str("5. Export to CSV if needed for other tool imports\n");
                    instructions.push_str("6. Use color coding for easy visual organization\n\n");
                },
                // Medium Priority Formats
                ExportFormat::Zephyr => {
                    instructions.push_str("## Zephyr Scale Import\n\n");
                    instructions.push_str("**File:** `rigr_test_cases_zephyr.json`\n\n");
                    instructions.push_str("### Steps:\n");
                    instructions.push_str("1. Log into Jira with Zephyr Scale plugin\n");
                    instructions.push_str("2. Navigate to your project\n");
                    instructions.push_str("3. Go to 'Tests' > 'Import Tests'\n");
                    instructions.push_str("4. Select 'JSON' as import format\n");
                    instructions.push_str("5. Upload the Zephyr JSON file\n");
                    instructions.push_str("6. Map fields and configure import settings\n");
                    instructions.push_str("7. Review and import test cases\n");
                    instructions.push_str("8. Organize into test cycles for execution\n\n");
                },
                ExportFormat::TestLink => {
                    instructions.push_str("## TestLink Import\n\n");
                    instructions.push_str("**File:** `rigr_test_cases_testlink.xml`\n\n");
                    instructions.push_str("### Steps:\n");
                    instructions.push_str("1. Log into your TestLink installation\n");
                    instructions.push_str("2. Navigate to your test project\n");
                    instructions.push_str("3. Go to 'Test Specification' section\n");
                    instructions.push_str("4. Click 'Import Test Cases'\n");
                    instructions.push_str("5. Select 'XML' format and upload the file\n");
                    instructions.push_str("6. Choose target test suite or create new one\n");
                    instructions.push_str("7. Review import settings and execute\n");
                    instructions.push_str("8. Verify imported test cases and organize as needed\n\n");
                },
                ExportFormat::Confluence => {
                    instructions.push_str("## Confluence Import\n\n");
                    instructions.push_str("**File:** `rigr_test_cases_confluence.txt`\n\n");
                    instructions.push_str("### Steps:\n");
                    instructions.push_str("1. Create new page in Confluence\n");
                    instructions.push_str("2. Switch to 'Wiki Markup' editor mode\n");
                    instructions.push_str("3. Copy and paste the markup content\n");
                    instructions.push_str("4. Switch back to visual editor to preview\n");
                    instructions.push_str("5. Publish the page to share with team\n");
                    instructions.push_str("6. Use Confluence's table of contents macro for navigation\n");
                    instructions.push_str("7. Add page properties for better organization\n\n");
                },
                ExportFormat::PostmanCollections => {
                    instructions.push_str("## Postman Collections Import\n\n");
                    instructions.push_str("**File:** `rigr_test_cases_postman.json`\n\n");
                    instructions.push_str("### Steps:\n");
                    instructions.push_str("1. Open Postman application\n");
                    instructions.push_str("2. Click 'Import' button in the top-left\n");
                    instructions.push_str("3. Select 'Upload Files' and choose the JSON file\n");
                    instructions.push_str("4. Review collection structure and click 'Import'\n");
                    instructions.push_str("5. Set up environment variables (base_url, etc.)\n");
                    instructions.push_str("6. Update request URLs and parameters for your API\n");
                    instructions.push_str("7. Run collection to execute API tests\n");
                    instructions.push_str("8. Use Newman for CI/CD integration\n\n");
                },
                ExportFormat::RobotFramework => {
                    instructions.push_str("## Robot Framework Import\n\n");
                    instructions.push_str("**File:** `rigr_test_cases.robot`\n\n");
                    instructions.push_str("### Steps:\n");
                    instructions.push_str("1. Place the `.robot` file in your test directory\n");
                    instructions.push_str("2. Install Robot Framework: `pip install robotframework`\n");
                    instructions.push_str("3. Implement keyword libraries for your application\n");
                    instructions.push_str("4. Update test steps to use your custom keywords\n");
                    instructions.push_str("5. Run tests: `robot rigr_test_cases.robot`\n");
                    instructions.push_str("6. View reports in `report.html` and `log.html`\n");
                    instructions.push_str("7. Integrate with CI/CD pipelines for automated execution\n\n");
                },
            }
        }

        instructions.push_str("## General Tips\n\n");
        instructions.push_str("- **Review Before Import:** Always review test cases before importing\n");
        instructions.push_str("- **Backup:** Create backups of your test management tool before large imports\n");
        instructions.push_str("- **Test Small Batch:** Import a small batch first to verify formatting\n");
        instructions.push_str("- **Organize:** Plan your test suite structure before importing\n");
        instructions.push_str("- **Tags:** Use tags for better organization and filtering\n");
        instructions.push_str("- **Automation:** Mark test cases suitable for automation\n\n");
        
        instructions.push_str("---\n");
        instructions.push_str("Generated by Rigr - AI-Powered Test Case Generator\n");

        fs::write(output_path, instructions)
            .map_err(|e| RigrError::FileWriteError(format!("Failed to write import instructions: {e}")))?;

        Ok(())
    }
}