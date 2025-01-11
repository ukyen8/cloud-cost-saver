use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct RuleTypeConfig {
    pub enabled: bool,
    #[serde(flatten)]
    pub config_detail: RuleTypeConfigDetail,
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub enum ThresholdValue {
    Int(u64),
    Float(f64),
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub enum RuleTypeConfigDetail {
    Value { value: String },
    Values { values: Vec<String> },
    Threshold { threshold: ThresholdValue },
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
                return Ok(RuleTypeConfigDetail::Threshold {
                    threshold: ThresholdValue::Int(threshold),
                });
            } else if let Some(threshold) = threshold.as_f64() {
                return Ok(RuleTypeConfigDetail::Threshold {
                    threshold: ThresholdValue::Float(threshold),
                });
            }
        } else if let Some(value) = map.get("value") {
            if let Some(value) = value.as_str() {
                return Ok(RuleTypeConfigDetail::Value {
                    value: value.to_string(),
                });
            }
        }

        Ok(RuleTypeConfigDetail::Simple)
    }
}

impl RuleTypeConfigDetail {
    pub fn get_value(&self) -> Option<&String> {
        if let RuleTypeConfigDetail::Value { value } = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn get_values(&self) -> Option<&Vec<String>> {
        if let RuleTypeConfigDetail::Values { values } = self {
            Some(values)
        } else {
            None
        }
    }

    pub fn get_threshold_int(&self) -> Option<u64> {
        if let RuleTypeConfigDetail::Threshold {
            threshold: ThresholdValue::Int(value),
        } = self
        {
            return Some(*value);
        }
        None
    }

    pub fn get_threshold_float(&self) -> Option<f64> {
        if let RuleTypeConfigDetail::Threshold {
            threshold: ThresholdValue::Float(value),
        } = self
        {
            return Some(*value);
        }
        None
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[allow(non_camel_case_types)]
pub enum RuleType {
    LAMBDA_001,
    LAMBDA_002,
    LAMBDA_003,
    LAMBDA_004,
    LAMBDA_005,
    LAMBDA_006,
    LAMBDA_007,
    CW_001,
    CW_002,
    CW_003,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleConfig {
    pub rules: HashMap<RuleType, RuleTypeConfig>,
    pub environments: HashMap<String, Option<HashMap<RuleType, RuleTypeConfig>>>,
}

impl RuleConfig {
    pub fn enabled(&self, violation: RuleType, environment: &str) -> bool {
        if let Some(env) = self.environments.get(environment) {
            if let Some(rules) = env {
                if let Some(rule) = rules.get(&violation) {
                    return rule.enabled;
                }
            }
        }
        false
    }

    pub fn set_rule(
        &mut self,
        rule: RuleType,
        enabled: bool,
        environment: &str,
        config_detail: Option<&RuleTypeConfigDetail>,
    ) {
        if let Some(env) = self.environments.get_mut(environment) {
            if let Some(rules) = env {
                if let Some(rule) = rules.get_mut(&rule) {
                    rule.enabled = enabled;
                    if config_detail.is_some() {
                        rule.config_detail = config_detail.cloned().unwrap();
                    }
                }
            }
        }
    }

    pub fn get_rule(&self, rule: RuleType, environment: &str) -> Option<&RuleTypeConfig> {
        if let Some(env) = self.environments.get(environment) {
            if let Some(rules) = env {
                return rules.get(&rule);
            }
        }
        None
    }
}

impl Default for RuleConfig {
    fn default() -> Self {
        let mut rules = HashMap::new();

        // Default configurations for each rule
        rules.insert(
            RuleType::LAMBDA_001,
            RuleTypeConfig {
                enabled: true,
                config_detail: RuleTypeConfigDetail::Simple,
            },
        );
        rules.insert(
            RuleType::LAMBDA_002,
            RuleTypeConfig {
                enabled: false,
                config_detail: RuleTypeConfigDetail::Simple,
            },
        );
        rules.insert(
            RuleType::LAMBDA_003,
            RuleTypeConfig {
                enabled: false,
                config_detail: RuleTypeConfigDetail::Values { values: vec![] },
            },
        );
        rules.insert(
            RuleType::LAMBDA_004,
            RuleTypeConfig {
                enabled: true,
                config_detail: RuleTypeConfigDetail::Threshold {
                    threshold: ThresholdValue::Int(0),
                },
            },
        );
        rules.insert(
            RuleType::LAMBDA_005,
            RuleTypeConfig {
                enabled: false,
                config_detail: RuleTypeConfigDetail::Value {
                    value: "INFO".to_string(),
                },
            },
        );
        rules.insert(
            RuleType::LAMBDA_006,
            RuleTypeConfig {
                enabled: true,
                config_detail: RuleTypeConfigDetail::Simple,
            },
        );
        rules.insert(
            RuleType::LAMBDA_007,
            RuleTypeConfig {
                enabled: false,
                config_detail: RuleTypeConfigDetail::Threshold {
                    threshold: ThresholdValue::Float(0.01),
                },
            },
        );
        rules.insert(
            RuleType::CW_001,
            RuleTypeConfig {
                enabled: true,
                config_detail: RuleTypeConfigDetail::Threshold {
                    threshold: ThresholdValue::Int(30),
                },
            },
        );
        rules.insert(
            RuleType::CW_002,
            RuleTypeConfig {
                enabled: true,
                config_detail: RuleTypeConfigDetail::Simple,
            },
        );
        rules.insert(
            RuleType::CW_003,
            RuleTypeConfig {
                enabled: false,
                config_detail: RuleTypeConfigDetail::Simple,
            },
        );
        let mut environments = HashMap::new();
        environments.insert("default".to_string(), Some(rules.clone()));

        RuleConfig {
            rules,
            environments,
        }
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
        let default_rules: HashMap<RuleType, RuleTypeConfig> = RuleConfig::default().rules;
        if let Some(ref mut cloudformation) = config.cloudformation {
            for (rule_name, default_rule) in default_rules {
                cloudformation
                    .rules
                    .entry(rule_name)
                    .or_insert(default_rule);
            }
        }

        if let Some(ref mut cloudformation) = config.cloudformation {
            // Create `default` environment
            cloudformation
                .environments
                .entry("default".to_string())
                .or_insert_with(|| Some(cloudformation.rules.clone()));
            for rules in cloudformation.environments.values_mut() {
                // No override rules, apply default rules
                if rules.is_none() {
                    rules.replace(cloudformation.rules.clone());
                } else {
                    // Merge default rules with the loaded configuration
                    if let Some(rules) = rules {
                        for (rule_name, default_rule) in &cloudformation.rules.clone() {
                            rules
                                .entry(rule_name.clone())
                                .or_insert_with(|| default_rule.clone());
                        }
                    }
                }
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

        assert!(!cloudformation.enabled(RuleType::LAMBDA_003, "default"));
        assert!(!cloudformation.enabled(RuleType::LAMBDA_002, "default"));
        assert!(cloudformation.enabled(RuleType::LAMBDA_001, "default"));
        assert!(cloudformation.enabled(RuleType::CW_001, "default"));
        assert!(!cloudformation.enabled(RuleType::CW_003, "default"));

        let cw_log_retention_policy = cloudformation.rules.get(&RuleType::CW_001).unwrap();
        assert_eq!(
            cw_log_retention_policy
                .config_detail
                .get_threshold_int()
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

        let lambda_architecture_arm = cloudformation.rules.get(&RuleType::LAMBDA_002).unwrap();
        assert!(lambda_architecture_arm.enabled);

        let lambda_missing_tag = cloudformation.rules.get(&RuleType::LAMBDA_003).unwrap();
        assert_eq!(
            lambda_missing_tag.config_detail.get_values().unwrap(),
            &vec!["tag1".to_string(), "tag2".to_string()]
        );

        let cw_log_retention_policy = cloudformation.rules.get(&RuleType::CW_001).unwrap();
        assert_eq!(
            cw_log_retention_policy
                .config_detail
                .get_threshold_int()
                .unwrap(),
            14
        );

        dbg!(&cloudformation.environments);
        let default_env = cloudformation
            .environments
            .get("default")
            .unwrap()
            .as_ref()
            .unwrap();
        assert_eq!(default_env.len(), cloudformation.rules.len());

        // No override rules, apply default rules
        let dev_env = cloudformation
            .environments
            .get("dev")
            .unwrap()
            .as_ref()
            .unwrap();
        assert!(dev_env
            .iter()
            .all(|(k, v)| default_env.get(k).map_or(false, |dv| dv == v)));

        let prod_env = cloudformation
            .environments
            .get("prod")
            .unwrap()
            .as_ref()
            .unwrap();
        let prod_cw002 = prod_env.get(&RuleType::CW_002).unwrap();
        assert!(!prod_cw002.enabled);
        let prod_lamnda003 = prod_env.get(&RuleType::LAMBDA_003).unwrap();
        assert_eq!(
            prod_lamnda003.config_detail.get_values().unwrap(),
            &vec!["tag3".to_string(), "tag4".to_string()]
        );
    }

    #[test]
    #[should_panic(expected = "missing field `enabled`")]
    fn test_invalid_config_missing_enabled() {
        let yaml = r#"
        cloudformation:
          rules:
            LAMBDA_002:
              threshold: 14
        "#;

        let _: Config = from_str(yaml).unwrap();
    }

    #[test]
    fn test_default_environment() {
        let config = Config::default();
        let cloudformation = config.cloudformation.unwrap();

        // Test with default environment
        assert!(cloudformation.enabled(RuleType::LAMBDA_001, "default"));
        assert!(!cloudformation.enabled(RuleType::LAMBDA_003, "default"));
    }
}
