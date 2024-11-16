use crate::parsers::iac::{AWSResourceType, IaCMapping, IaCOutput, IaCParameter, IaCResource};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CloudFormation {
    #[serde(rename = "AWSTemplateFormatVersion")]
    pub awstemplate_format_version: Option<String>,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "Parameters")]
    pub parameters: Option<HashMap<String, Parameter>>,
    #[serde(rename = "Mappings")]
    pub mappings: Option<HashMap<String, Mapping>>,
    #[serde(rename = "Globals")]
    pub globals: Option<Globals>,
    #[serde(rename = "Resources")]
    pub resources: Option<IndexMap<String, Resource>>,
    #[serde(rename = "Outputs")]
    pub outputs: Option<HashMap<String, Output>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameter {
    #[serde(rename = "Type")]
    type_: String,
    #[serde(rename = "Default")]
    default: Option<serde_yaml::Value>,
    #[serde(rename = "Description")]
    description: Option<String>,
    #[serde(flatten)]
    other: HashMap<String, serde_yaml::Value>, // Extra fields can be captured here
}

impl IaCParameter for Parameter {
    fn get_type(&self) -> String {
        self.type_.clone()
    }

    fn get_default(&self) -> Option<String> {
        self.default
            .as_ref()
            .and_then(|v| v.as_str().map(String::from))
    }

    fn get_description(&self) -> Option<String> {
        self.description.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mapping {
    #[serde(flatten)]
    map: HashMap<String, HashMap<String, String>>,
}

impl IaCMapping for Mapping {
    fn get_value(&self, key1: &str, key2: &str) -> Option<String> {
        self.map.get(key1).and_then(|v| v.get(key2).cloned())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    #[serde(rename = "description")]
    description: String,
    #[serde(rename = "Value")]
    value: String,
    #[serde(rename = "Export")]
    export: Option<HashMap<String, String>>,
}

impl IaCOutput for Output {
    fn get_description(&self) -> Option<String> {
        Some(self.description.clone())
    }

    fn get_value(&self) -> String {
        self.value.clone()
    }

    fn get_export(&self) -> Option<HashMap<String, String>> {
        self.export.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Globals {
    #[serde(rename = "Function")]
    function: Option<HashMap<String, serde_yaml::Value>>,
    #[serde(rename = "Api")]
    api: Option<HashMap<String, serde_yaml::Value>>,
    #[serde(rename = "HttpApi")]
    http_api: Option<HashMap<String, serde_yaml::Value>>,
    #[serde(rename = "StateMachine")]
    state_machine: Option<HashMap<String, serde_yaml::Value>>,
    #[serde(rename = "SimpleTable")]
    simple_table: Option<HashMap<String, serde_yaml::Value>>,
}

#[derive(Debug, Serialize)]
pub struct Resource {
    #[serde(rename = "Type")]
    pub type_: AWSResourceType,
    #[serde(rename = "Properties")]
    pub properties: Option<HashMap<String, serde_yaml::Value>>, // Properties vary by resource type
    #[serde(flatten)]
    pub other: HashMap<String, serde_yaml::Value>, // Capture additional resource attributes if needed
}

impl IaCResource for Resource {
    fn get_type(&self) -> String {
        match &self.type_ {
            AWSResourceType::LambdaFunction => "AWS::Lambda::Function".to_string(),
            AWSResourceType::CloudWatch => "AWS::Logs::LogGroup".to_string(),
            AWSResourceType::Unknown(t) => t.clone(),
        }
    }

    fn get_properties(&self) -> Option<HashMap<String, serde_yaml::Value>> {
        self.properties.clone()
    }
}

impl<'de> Deserialize<'de> for Resource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize into an intermediate representation to extract the type first
        let mut map = IndexMap::<String, serde_yaml::Value>::deserialize(deserializer)?;

        // Extract the `Type` field and deserialize it into `ResourceType`
        let resource_type_str = map
            .shift_remove("Type")
            .ok_or_else(|| serde::de::Error::missing_field("Type"))?;
        let resource_type: AWSResourceType =
            serde_yaml::from_value(resource_type_str).map_err(serde::de::Error::custom)?;

        // Extract `Properties` if present
        let properties = map
            .shift_remove("Properties")
            .and_then(|v| v.as_mapping().cloned())
            .map(|mapping| {
                mapping
                    .iter()
                    .map(|(k, v)| (k.as_str().unwrap().to_string(), v.clone()))
                    .collect::<HashMap<String, serde_yaml::Value>>()
            });

        // Construct the Resource with the remaining fields in `other`
        Ok(Resource {
            type_: resource_type,
            properties: properties,
            other: map.into_iter().collect(),
        })
    }
}
pub(crate) fn parse_cloudformation(
    file_path: &str,
) -> Result<CloudFormation, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(file_path)?;
    let cloudformation: CloudFormation = serde_yaml::from_str(&data)?;
    Ok(cloudformation)
}
