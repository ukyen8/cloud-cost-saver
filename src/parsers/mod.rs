pub(crate) mod cfn;
pub(crate) mod config;
pub(crate) mod iac;
use marked_yaml::{parse_yaml, Node, Span};
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
    let node = parse_yaml(0, &doc).expect("Failed to parse YAML");
    Ok(YamlLineMarker::new(node))
}
