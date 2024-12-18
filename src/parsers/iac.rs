use crate::parsers::cfn::CloudFormation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct InfratructureTemplate {
    pub cloudformation: Option<CloudFormation>,
}

#[allow(unused)]
pub trait IaCParameter {
    fn get_type(&self) -> String;
    fn get_default(&self) -> Option<String>;
    fn get_description(&self) -> Option<String>;
}

#[allow(unused)]
pub trait IaCMapping {
    fn get_value(&self, key1: &str, key2: &str) -> Option<serde_yaml::Value>;
}

#[allow(unused)]
pub trait IaCResource {
    fn get_type(&self) -> String;
    fn get_properties(&self) -> Option<HashMap<String, serde_yaml::Value>>;
}

#[allow(unused)]
pub trait IaCOutput {
    fn get_description(&self) -> Option<String>;
    fn get_value(&self) -> String;
    fn get_export(&self) -> Option<HashMap<String, String>>;
}

#[derive(Debug, Serialize)]
pub enum AWSResourceType {
    LambdaFunction,
    CloudWatch,
    Unknown(String),
}

impl<'de> Deserialize<'de> for AWSResourceType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let cfn_type = String::deserialize(deserializer)?;
        let resource_type = match cfn_type.to_uppercase().as_str() {
            "AWS::LAMBDA::FUNCTION" | "AWS::SERVERLESS::FUNCTION" => Self::LambdaFunction,
            "AWS::LOGS::LOGGROUP" => Self::CloudWatch,
            _ => Self::Unknown(cfn_type),
        };
        Ok(resource_type)
    }
}
