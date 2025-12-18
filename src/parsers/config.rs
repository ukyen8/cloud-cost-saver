use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

use std::hash::Hash;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct RuleTypeConfig {
    pub enabled: bool,
    #[serde(flatten)]
    pub config_detail: RuleTypeConfigDetail,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum ThresholdValue {
    Int(u64),
    Float(f64),
}

#[derive(Debug, PartialEq, Clone)]
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

impl Serialize for RuleTypeConfigDetail {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RuleTypeConfigDetail::Simple => {
                let map = serializer.serialize_map(Some(0))?;
                map.end()
            }
            RuleTypeConfigDetail::Value { value } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("value", value)?;
                map.end()
            }
            RuleTypeConfigDetail::Values { values } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("values", values)?;
                map.end()
            }
            RuleTypeConfigDetail::Threshold { threshold } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("threshold", threshold)?;
                map.end()
            }
        }
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

    LAMBDA_008,
    LAMBDA_009,
    CW_001,
    CW_002,
    CW_003,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    Minimal,
    Recommended,
    Strict,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleConfig {
    pub rules: HashMap<RuleType, RuleTypeConfig>,
    pub environments: HashMap<String, Option<HashMap<RuleType, RuleTypeConfig>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<Preset>,
}

impl RuleConfig {
    pub fn enabled(&self, violation: RuleType, environment: &str) -> bool {
        if let Some(rule) = self
            .environments
            .get(environment)
            .and_then(|env| env.as_ref())
            .and_then(|rules| rules.get(&violation))
        {
            return rule.enabled;
        }
        // Fallback to base rules
        self.rules
            .get(&violation)
            .map(|r| r.enabled)
            .unwrap_or(false)
    }

    pub fn get_rule(&self, rule: RuleType, environment: &str) -> Option<&RuleTypeConfig> {
        self.environments
            .get(environment)
            .and_then(|rules| rules.as_ref())
            .and_then(|rules| rules.get(&rule))
            .or_else(|| self.rules.get(&rule))
    }

    pub fn apply_preset(&mut self, preset: Preset) {
        self.preset = Some(preset);
        let mut new_rules = HashMap::new();

        // Helper to enable a rule simply
        let enable = |enabled: bool| RuleTypeConfig {
            enabled,
            config_detail: RuleTypeConfigDetail::Simple,
        };

        match preset {
            Preset::Minimal => {
                // Critical only
                new_rules.insert(RuleType::LAMBDA_001, enable(true)); // Log retention missing
                new_rules.insert(
                    RuleType::LAMBDA_004,
                    RuleTypeConfig {
                        // Async retries
                        enabled: true,
                        config_detail: RuleTypeConfigDetail::Threshold {
                            threshold: ThresholdValue::Int(0),
                        },
                    },
                );
                new_rules.insert(RuleType::LAMBDA_009, enable(true)); // Static concurrency
                new_rules.insert(RuleType::CW_002, enable(true)); // No retention policy

                // Explicitly disable others commonly on by default
                new_rules.insert(RuleType::LAMBDA_006, enable(false));
                new_rules.insert(RuleType::LAMBDA_008, enable(false));
                new_rules.insert(
                    RuleType::CW_001,
                    RuleTypeConfig {
                        enabled: false,
                        config_detail: RuleTypeConfigDetail::Threshold {
                            threshold: ThresholdValue::Int(30),
                        },
                    },
                );
            }
            Preset::Recommended => {
                // Minimal + High Value rules (Default-ish)
                new_rules.insert(RuleType::LAMBDA_001, enable(true));
                new_rules.insert(
                    RuleType::LAMBDA_004,
                    RuleTypeConfig {
                        enabled: true,
                        config_detail: RuleTypeConfigDetail::Threshold {
                            threshold: ThresholdValue::Int(0),
                        },
                    },
                );
                new_rules.insert(RuleType::LAMBDA_006, enable(true));
                new_rules.insert(RuleType::LAMBDA_008, enable(true));
                new_rules.insert(RuleType::LAMBDA_009, enable(true));
                new_rules.insert(
                    RuleType::CW_001,
                    RuleTypeConfig {
                        enabled: true,
                        config_detail: RuleTypeConfigDetail::Threshold {
                            threshold: ThresholdValue::Int(30),
                        },
                    },
                );
                new_rules.insert(RuleType::CW_002, enable(true));
            }
            Preset::Strict => {
                // Everything enabled
                new_rules.insert(RuleType::LAMBDA_001, enable(true));
                new_rules.insert(RuleType::LAMBDA_002, enable(true)); // ARM
                new_rules.insert(
                    RuleType::LAMBDA_003,
                    RuleTypeConfig {
                        // Tags
                        enabled: true,
                        config_detail: RuleTypeConfigDetail::Values {
                            values: vec!["CostCenter".to_string()],
                        },
                    },
                );
                new_rules.insert(
                    RuleType::LAMBDA_004,
                    RuleTypeConfig {
                        enabled: true,
                        config_detail: RuleTypeConfigDetail::Threshold {
                            threshold: ThresholdValue::Int(0),
                        },
                    },
                );
                new_rules.insert(
                    RuleType::LAMBDA_005,
                    RuleTypeConfig {
                        enabled: true,
                        config_detail: RuleTypeConfigDetail::Value {
                            value: "INFO".to_string(),
                        },
                    },
                );
                new_rules.insert(RuleType::LAMBDA_006, enable(true));
                new_rules.insert(
                    RuleType::LAMBDA_007,
                    RuleTypeConfig {
                        enabled: true,
                        config_detail: RuleTypeConfigDetail::Threshold {
                            threshold: ThresholdValue::Float(0.1),
                        },
                    },
                );
                new_rules.insert(RuleType::LAMBDA_008, enable(true));
                new_rules.insert(RuleType::LAMBDA_009, enable(true));
                new_rules.insert(
                    RuleType::CW_001,
                    RuleTypeConfig {
                        enabled: true,
                        config_detail: RuleTypeConfigDetail::Threshold {
                            threshold: ThresholdValue::Int(14),
                        },
                    },
                );
                new_rules.insert(RuleType::CW_002, enable(true));
                new_rules.insert(RuleType::CW_003, enable(true));
            }
        }

        // Apply new rules, merging with existing ones
        for (key, val) in new_rules {
            self.rules.insert(key.clone(), val.clone());
            // Also update all environments to ensure consistency
            for env_rules in self.environments.values_mut().flatten() {
                env_rules.insert(key.clone(), val.clone());
            }
        }
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
            RuleType::LAMBDA_008,
            RuleTypeConfig {
                enabled: true,
                config_detail: RuleTypeConfigDetail::Simple,
            },
        );
        rules.insert(
            RuleType::LAMBDA_009,
            RuleTypeConfig {
                enabled: true,
                config_detail: RuleTypeConfigDetail::Simple,
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
            preset: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub cloudformation: RuleConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        let cloudformation = config.cloudformation;

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
    fn test_default_environment() {
        let config = Config::default();
        let cloudformation = config.cloudformation;

        // Test with default environment
        assert!(cloudformation.enabled(RuleType::LAMBDA_001, "default"));
        assert!(!cloudformation.enabled(RuleType::LAMBDA_003, "default"));
    }
}
