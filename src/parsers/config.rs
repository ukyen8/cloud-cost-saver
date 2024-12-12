use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleTypeConfig {
    pub enabled: bool,
    #[serde(flatten)]
    pub config_detail: RuleTypeConfigDetail,
}

#[derive(Debug, Serialize)]
pub enum RuleTypeConfigDetail {
    Values { values: Vec<String> },
    Threshold { threshold: u64 },
    Simple,
}

impl<'de> Deserialize<'de> for RuleTypeConfigDetail {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map = HashMap::<String, serde_yaml::Value>::deserialize(deserializer)?;

        if let Some(values) = map.get("values") {
            if let Some(values) = values.as_sequence() {
                let values = values
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                return Ok(RuleTypeConfigDetail::Values { values });
            }
        } else if let Some(threshold) = map.get("threshold") {
            if let Some(threshold) = threshold.as_u64() {
                return Ok(RuleTypeConfigDetail::Threshold { threshold });
            }
        }

        Ok(RuleTypeConfigDetail::Simple)
    }
}

impl RuleTypeConfigDetail {
    pub fn get_values(&self) -> Option<&Vec<String>> {
        if let RuleTypeConfigDetail::Values { values } = self {
            Some(values)
        } else {
            None
        }
    }

    pub fn get_threshold(&self) -> Option<u64> {
        if let RuleTypeConfigDetail::Threshold { threshold } = self {
            Some(*threshold)
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RuleType {
    LambdaMissingTag,
    LambdaArchitectureARM,
    LambdaMissingLogGroup,
    CWLogRetentionPolicy,
    CWInfrequentAccessLogGroupClass,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleConfig {
    pub rules: HashMap<RuleType, RuleTypeConfig>,
}

impl RuleConfig {
    pub fn enabled(&self, violation: RuleType) -> bool {
        self.rules
            .get(&violation)
            .map_or(false, |rule| rule.enabled)
    }
}

impl Default for RuleConfig {
    fn default() -> Self {
        let mut rules = HashMap::new();

        // Default configurations for each rule
        rules.insert(RuleType::LambdaMissingTag, RuleTypeConfig {
            enabled: true,
            config_detail: RuleTypeConfigDetail::Values { values: vec![] },
        });
        rules.insert(RuleType::LambdaArchitectureARM, RuleTypeConfig {
            enabled: true,
            config_detail: RuleTypeConfigDetail::Simple,
        });
        rules.insert(RuleType::LambdaMissingLogGroup, RuleTypeConfig {
            enabled: true,
            config_detail: RuleTypeConfigDetail::Simple,
        });
        rules.insert(RuleType::CWLogRetentionPolicy, RuleTypeConfig {
            enabled: true,
            config_detail: RuleTypeConfigDetail::Threshold { threshold: 30 },
        });
        rules.insert(RuleType::CWInfrequentAccessLogGroupClass, RuleTypeConfig {
            enabled: false,
            config_detail: RuleTypeConfigDetail::Simple,
        });

        RuleConfig { rules }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub cloudformation: Option<RuleConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            cloudformation: Some(RuleConfig::default()),
        }
    }
}

impl Config {
    pub fn load(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read_to_string(file_path)?;
        let mut config: Config = serde_yaml::from_str(&data)?;

        // Merge default rules with the loaded configuration
        let default_rules = RuleConfig::default().rules;
        if let Some(ref mut cloudformation) = config.cloudformation {
            for (rule_name, default_rule) in default_rules {
                cloudformation
                    .rules
                    .entry(rule_name)
                    .or_insert(default_rule);
            }
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml::from_str;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.cloudformation.is_some());
        let cloudformation = config.cloudformation.unwrap();

        assert!(cloudformation.enabled(RuleType::LambdaMissingTag));
        assert!(cloudformation.enabled(RuleType::LambdaArchitectureARM));
        assert!(cloudformation.enabled(RuleType::LambdaMissingLogGroup));
        assert!(cloudformation.enabled(RuleType::CWLogRetentionPolicy));
        assert!(!cloudformation.enabled(RuleType::CWInfrequentAccessLogGroupClass));

        let cw_log_retention_policy = cloudformation
            .rules
            .get(&RuleType::CWLogRetentionPolicy)
            .unwrap();
        assert_eq!(
            cw_log_retention_policy
                .config_detail
                .get_threshold()
                .unwrap(),
            30
        );
    }

    #[test]
    fn test_load_config() {
        let file_path = "src/fixtures/.cloudsaving.yaml";
        let config = Config::load(file_path).unwrap();
        assert!(config.cloudformation.is_some());
        let cloudformation = config.cloudformation.unwrap();

        let lambda_architecture_arm = cloudformation
            .rules
            .get(&RuleType::LambdaArchitectureARM)
            .unwrap();
        assert_eq!(lambda_architecture_arm.enabled, true);

        let lambda_missing_tag = cloudformation
            .rules
            .get(&RuleType::LambdaMissingTag)
            .unwrap();
        assert_eq!(
            lambda_missing_tag.config_detail.get_values().unwrap(),
            &vec!["tag1".to_string(), "tag2".to_string()]
        );

        let cw_log_retention_policy = cloudformation
            .rules
            .get(&RuleType::CWLogRetentionPolicy)
            .unwrap();
        assert_eq!(
            cw_log_retention_policy
                .config_detail
                .get_threshold()
                .unwrap(),
            14
        );
    }

    #[test]
    #[should_panic(expected = "missing field `enabled`")]
    fn test_invalid_config_missing_enabled() {
        let yaml = r#"
        cloudformation:
          rules:
            LambdaMissingLogGroup:
              threshold: 14
        "#;

        let _: Config = from_str(yaml).unwrap();
    }
}
