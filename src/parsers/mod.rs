pub(crate) mod cfn;
pub(crate) mod config;
pub(crate) mod iac;
use marked_yaml::{Node, Span, parse_yaml};
use std::fs;

pub trait LineMarker {
    fn new(node: Node) -> Self;
    fn get_resource_span(&self, paths: Vec<&str>) -> Option<&Span>;
}

pub struct YamlLineMarker {
    node: Node,
}

impl LineMarker for YamlLineMarker {
    fn new(node: Node) -> Self {
        Self { node }
    }

    fn get_resource_span(&self, paths: Vec<&str>) -> Option<&Span> {
        let mut node = match self.node.as_mapping()?.get("Resources") {
            Some(n) => n,
            None => return None,
        };
        for path in paths {
            node = match node.as_mapping()?.get(path) {
                Some(n) => n,
                None => return None,
            };
        }
        Some(node.span())
    }
}

pub(crate) fn get_yaml_line_marker(template: &str) -> Result<YamlLineMarker, std::io::Error> {
    let doc = fs::read_to_string(template)?;
    let preprocessed_doc = preprocess_yaml(&doc);
    let node = parse_yaml(0, &preprocessed_doc).expect("Failed to parse YAML");
    Ok(YamlLineMarker::new(node))
}
fn preprocess_yaml(doc: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    for line in doc.lines() {
        // Replace `!` with `###` to avoid parsing issues with `!` in the YAML
        let cleaned_line = line.replace("!", "###");
        lines.push(cleaned_line);
    }
    lines.join("\n")
}
