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
            .rev()
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
                "{}: {}: {}\n{}\n",
                e.violation.code(),
                e.resource_name,
                e.violation.message(),
                span_info,
            )
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}
