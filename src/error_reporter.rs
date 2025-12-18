use crate::rules::violations::Violation;
pub struct ErrorReporter {
    errors: Vec<ErrorDetail>,
    file_path: String,
    current_environment: String,
    scanned_environments: std::collections::HashSet<String>,
}
use marked_yaml::Span;

#[derive(Debug)]
pub struct ErrorDetail {
    pub violation: Box<dyn Violation>,
    pub resource_name: String,
    pub span: Option<Span>,
    pub environment: String,
}

use askama::Template;
use std::collections::HashMap;

#[derive(Template)]
#[template(path = "report.html")]
struct ReportTemplate {
    generated_at: String,
    total_issues: usize,
    environments: Vec<EnvironmentView>,
}

struct EnvironmentView {
    name: String,
    resources: Vec<ResourceGroupView>,
    total_issues: usize,
}

struct ResourceGroupView {
    name: String,
    violations: Vec<ViolationView>,
}

struct ViolationView {
    code: String,
    message: String,
    file: String,
    line: Option<usize>,
}

impl ErrorDetail {
    pub fn new(
        violation: Box<dyn Violation>,
        resource_name: String,
        span: Option<Span>,
        environment: String,
    ) -> Self {
        Self {
            violation,
            resource_name,
            span,
            environment,
        }
    }
}

impl ErrorReporter {
    pub fn new(file_path: &str) -> Self {
        ErrorReporter {
            errors: Vec::new(),
            file_path: file_path.to_string(),
            current_environment: "default".to_string(),
            scanned_environments: std::collections::HashSet::new(),
        }
    }

    pub fn set_current_environment(&mut self, environment: &str) {
        self.current_environment = environment.to_string();
        self.scanned_environments.insert(environment.to_string());
    }

    pub fn add_error(
        &mut self,
        violation: Box<dyn Violation>,
        resource_name: &str,
        span: Option<Span>,
    ) {
        let error_detail = ErrorDetail::new(
            violation,
            resource_name.to_string(),
            span,
            self.current_environment.clone(),
        );
        self.errors.push(error_detail);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn render_errors(&self) -> String {
        self.errors
            .iter()
            .map(|e| {
                let span_info = if let Some(span) = &e.span {
                    if let Some(start) = span.start() {
                        format!("{}:{}", self.file_path, start.line() - 1)
                    } else {
                        self.file_path.clone()
                    }
                } else {
                    self.file_path.clone()
                };
                format!(
                    "{}:{}:{}\n{}\n",
                    e.violation.code(),
                    e.resource_name,
                    e.violation.message(),
                    span_info,
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn render_html(&self) -> String {
        let mut env_map: HashMap<String, HashMap<String, Vec<&ErrorDetail>>> = HashMap::new();

        // 1. Group by Environment -> Resource Name
        for error in &self.errors {
            env_map
                .entry(error.environment.clone())
                .or_default()
                .entry(error.resource_name.clone())
                .or_default()
                .push(error);
        }

        // 2. Transform into View Structs
        let mut environments: Vec<EnvironmentView> = self
            .scanned_environments
            .iter()
            .map(|env_name| {
                let resource_map = env_map.remove(env_name).unwrap_or_default();
                let mut resources: Vec<ResourceGroupView> = resource_map
                    .into_iter()
                    .map(|(res_name, errors)| {
                        let mut violations: Vec<ViolationView> = errors
                            .into_iter()
                            .map(|e| {
                                let line =
                                    e.span.as_ref().and_then(|s| s.start()).map(|p| p.line());
                                ViolationView {
                                    code: e.violation.code(),
                                    message: e.violation.message(),
                                    file: self.file_path.clone(),
                                    line,
                                }
                            })
                            .collect();

                        // Sort violations by Code
                        violations.sort_by(|a, b| a.code.cmp(&b.code));

                        ResourceGroupView {
                            name: res_name,
                            violations,
                        }
                    })
                    .collect();

                // Sort resources by Name
                resources.sort_by(|a, b| a.name.cmp(&b.name));

                let total_issues = resources.iter().map(|r| r.violations.len()).sum();

                EnvironmentView {
                    name: env_name.clone(),
                    resources,
                    total_issues,
                }
            })
            .collect();

        // Sort environments by Name (default first if possible, or usually alphabetical)
        environments.sort_by(|a, b| a.name.cmp(&b.name));

        let template = ReportTemplate {
            generated_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            total_issues: self.errors.len(),
            environments,
        };

        template
            .render()
            .unwrap_or_else(|e| format!("Error generating report: {}", e))
    }

    pub fn render_json(&self) -> String {
        use serde::Serialize;

        #[derive(Serialize)]
        struct JsonErrorDetail {
            code: String,
            message: String,
            resource: String,
            file: String,
            line: Option<usize>,
        }

        let json_errors: Vec<JsonErrorDetail> = self
            .errors
            .iter()
            .map(|e| {
                let line = e
                    .span
                    .as_ref()
                    .and_then(|s| s.start())
                    .map(|p| p.line() - 1);
                JsonErrorDetail {
                    code: e.violation.code(),
                    message: e.violation.message(),
                    resource: e.resource_name.clone(),
                    file: self.file_path.clone(),
                    line,
                }
            })
            .collect();

        serde_json::to_string_pretty(&json_errors).unwrap_or_else(|_| "[]".to_string())
    }
}
