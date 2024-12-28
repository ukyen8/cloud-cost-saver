pub(crate) mod cfn;
pub(crate) mod config;
pub(crate) mod iac;
use marked_yaml::{parse_yaml, Node, Span};
use regex::Regex;
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
        let mut node = self.node.as_mapping()?.get("Resources")?;
        for path in paths {
            node = node.as_mapping()?.get(path)?;
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
    let re = Regex::new(r"Fn::\w+:").unwrap();
    for line in doc.lines() {
        // Replace `!` with `###` to avoid parsing issues with `!` in the YAML
        let mut cleaned_line = line.replace("!", "###");
        // Replace `Fn::xxx:` with another string to avoid parsing issues with `Fn::xxx:`
        cleaned_line = re.replace_all(&cleaned_line, "###REPLACED").to_string();
        lines.push(cleaned_line);
    }
    lines.join("\n")
}
