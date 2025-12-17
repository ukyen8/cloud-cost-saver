use crate::rules::violations::Violation;
pub struct ErrorReporter {
    errors: Vec<ErrorDetail>,
    file_path: String,
}
use marked_yaml::Span;

#[derive(Debug)]
pub struct ErrorDetail {
    pub violation: Box<dyn Violation>,
    pub resource_name: String,
    pub span: Option<Span>,
}

use askama::Template;
use std::collections::HashMap;

#[derive(Template)]
#[template(path = "report.html")]
struct ReportTemplate {
    generated_at: String,
    total_issues: usize,
    resources: Vec<ResourceGroupView>,
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
    pub fn new(violation: Box<dyn Violation>, resource_name: String, span: Option<Span>) -> Self {
        Self {
            violation,
            resource_name,
            span,
        }
    }
}

impl ErrorReporter {
    pub fn new(file_path: &str) -> Self {
        ErrorReporter {
            errors: Vec::new(),
            file_path: file_path.to_string(),
        }
    }

    pub fn add_error(
        &mut self,
        violation: Box<dyn Violation>,
        resource_name: &str,
        span: Option<Span>,
    ) {
        let error_detail = ErrorDetail::new(violation, resource_name.to_string(), span);
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
        let mut grouped_errors: HashMap<String, Vec<&ErrorDetail>> = HashMap::new();

        for error in &self.errors {
            grouped_errors
                .entry(error.resource_name.clone())
                .or_default()
                .push(error);
        }

        let mut resources: Vec<ResourceGroupView> = grouped_errors
            .into_iter()
            .map(|(name, errors)| {
                let mut violations: Vec<ViolationView> = errors
                    .into_iter()
                    .map(|e| {
                        let line = e.span.as_ref().and_then(|s| s.start()).map(|p| p.line());
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

                ResourceGroupView { name, violations }
            })
            .collect();

        // Sort resources alphabetically
        resources.sort_by(|a, b| a.name.cmp(&b.name));

        let template = ReportTemplate {
            generated_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            total_issues: self.errors.len(),
            resources,
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
