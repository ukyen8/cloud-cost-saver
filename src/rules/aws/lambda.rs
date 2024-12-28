use crate::error_reporter::ErrorReporter;
use crate::parsers::config::{RuleConfig, RuleType};
use crate::parsers::iac::AWSResourceType;
use crate::parsers::iac::InfratructureTemplate;
use crate::parsers::LineMarker;
use crate::rules::violations::LambdaViolation;

pub fn check_lambda_missing_tag<L: LineMarker>(
    infra_template: &InfratructureTemplate,
    rule_config: &RuleConfig,
    error_reporter: &mut ErrorReporter,
    line_marker: &L,
) {
    if let Some(cloudformation) = &infra_template.cloudformation {
        if let Some(resources) = &cloudformation.resources {
            for (key, resource) in resources {
                if let AWSResourceType::LambdaFunction | AWSResourceType::LambdaServerlessFunction =
                    &resource.type_
                {
                    if let Some(properties) = &resource.properties {
                        if let Some(tags) = properties.get("Tags") {
                            if let Some(rule_type) = rule_config.rules.get(&RuleType::LAMBDA_003) {
                                if let Some(target_tags) = rule_type.config_detail.get_values() {
                                    // Check if at least one tag is defined in the resource
                                    let tag_exists = target_tags.iter().any(|target_tag| {
                                        tags.as_sequence().is_some_and(|seq| {
                                            seq.iter().any(|tag_mapping| {
                                                tag_mapping
                                                    .as_mapping()
                                                    .is_some_and(|m| m.contains_key(target_tag))
                                            })
                                        })
                                    });

                                    if !tag_exists {
                                        error_reporter.add_error(
                                            Box::new(LambdaViolation::MissingTag),
                                            key,
                                            line_marker
                                                .get_resource_span(vec![key, "Properties", "Tags"])
                                                .copied(),
                                        );
                                    }
                                }
                            } else {
                                error_reporter.add_error(
                                    Box::new(LambdaViolation::MissingTag),
                                    key,
                                    line_marker
                                        .get_resource_span(vec![key, "Properties", "Tags"])
                                        .copied(),
                                );
                            }
                        } else {
                            error_reporter.add_error(
                                Box::new(LambdaViolation::MissingTag),
                                key,
                                line_marker.get_resource_span(vec![key]).copied(),
                            )
                        };
                    }
                }
            }
        }
    }
}

pub fn check_lambda_architecture_arm<L: LineMarker>(
    infra_template: &InfratructureTemplate,
    error_reporter: &mut ErrorReporter,
    line_marker: &L,
) {
    if let Some(cloudformation) = &infra_template.cloudformation {
        if let Some(resources) = &cloudformation.resources {
            for (key, resource) in resources {
                if let AWSResourceType::LambdaFunction | AWSResourceType::LambdaServerlessFunction =
                    &resource.type_
                {
                    if let Some(properties) = &resource.properties {
                        if let Some(architectures) = properties.get("Architectures") {
                            if architectures
                                .as_sequence()
                                .is_none_or(|v| !v.iter().any(|arch| arch == "arm64"))
                            {
                                error_reporter.add_error(
                                    Box::new(LambdaViolation::ARMArchitecture),
                                    key,
                                    line_marker
                                        .get_resource_span(vec![key, "Properties", "Architectures"])
                                        .copied(),
                                );
                            }
                        } else {
                            error_reporter.add_error(
                                Box::new(LambdaViolation::ARMArchitecture),
                                key,
                                line_marker.get_resource_span(vec![key]).copied(),
                            );
                        }
                    }
                }
            }
        }
    }
}

pub fn check_lambda_missing_log_group<L: LineMarker>(
    infra_template: &InfratructureTemplate,
    error_reporter: &mut ErrorReporter,
    line_marker: &L,
) {
    if let Some(cloudformation) = &infra_template.cloudformation {
        if let Some(resources) = &cloudformation.resources {
            for (key, resource) in resources {
                if let AWSResourceType::LambdaFunction | AWSResourceType::LambdaServerlessFunction =
                    &resource.type_
                {
                    if let Some(properties) = &resource.properties {
                        if let Some(logging_config) = properties.get("LoggingConfig") {
                            if !logging_config
                                .as_mapping()
                                .is_some_and(|m| m.contains_key("LogGroup"))
                            {
                                error_reporter.add_error(
                                    Box::new(LambdaViolation::MissingLogGroup),
                                    key,
                                    line_marker
                                        .get_resource_span(vec![key, "Properties", "LoggingConfig"])
                                        .copied(),
                                );
                            }
                        } else {
                            error_reporter.add_error(
                                Box::new(LambdaViolation::MissingLogGroup),
                                key,
                                line_marker.get_resource_span(vec![key]).copied(),
                            );
                        }
                    }
                }
            }
        }
    }
}

pub fn check_lambda_maxmimum_retry_attempts<L: LineMarker>(
    infra_template: &InfratructureTemplate,
    rule_config: &RuleConfig,
    error_reporter: &mut ErrorReporter,
    line_marker: &L,
) {
    if let Some(cloudformation) = &infra_template.cloudformation {
        // Fetch threshold from the rule configuration
        let max_retry_attempts_config = rule_config
            .rules
            .get(&RuleType::LAMBDA_004)
            .and_then(|rule_type| rule_type.config_detail.get_threshold_int())
            .unwrap_or(0);

        if let Some(resources) = &cloudformation.resources {
            for (key, resource) in resources {
                if let AWSResourceType::LambdaServerlessFunction = &resource.type_ {
                    if let Some(properties) = &resource.properties {
                        // Fetch threshold from the rule configuration
                        if let Some(event_invoke_config) = properties.get("EventInvokeConfig") {
                            if let Some(maximum_retry_attempts) =
                                event_invoke_config.get("MaximumRetryAttempts")
                            {
                                if maximum_retry_attempts
                                    .as_u64()
                                    .is_none_or(|v| v != max_retry_attempts_config)
                                {
                                    error_reporter.add_error(
                                        Box::new(LambdaViolation::MaximumRetryAttempts),
                                        key,
                                        line_marker
                                            .get_resource_span(vec![
                                                key,
                                                "Properties",
                                                "EventInvokeConfig",
                                            ])
                                            .copied(),
                                    );
                                } else {
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn check_lambda_powertools_environment_variables<L: LineMarker>(
    infra_template: &InfratructureTemplate,
    rule_config: &RuleConfig,
    error_reporter: &mut ErrorReporter,
    line_marker: &L,
) {
    if let Some(cloudformation) = &infra_template.cloudformation {
        if let Some(resources) = &cloudformation.resources {
            for (key, resource) in resources {
                if let AWSResourceType::LambdaFunction | AWSResourceType::LambdaServerlessFunction =
                    &resource.type_
                {
                    if let Some(variables) = resource
                        .properties
                        .as_ref()
                        .and_then(|props| props.get("Environment"))
                        .and_then(|env| env.get("Variables"))
                    {
                        if let Some(rule_type) = rule_config.rules.get(&RuleType::LAMBDA_005) {
                            if rule_type.enabled {
                                if let Some(target_log_level) = rule_type.config_detail.get_value()
                                {
                                    if let Some(powertools_log_level) =
                                        variables.get("POWERTOOLS_LOG_LEVEL")
                                    {
                                        if Some(target_log_level.as_str())
                                            != powertools_log_level.as_str()
                                        {
                                            error_reporter.add_error(
                                                Box::new(LambdaViolation::PowertoolsLogLevel),
                                                key,
                                                line_marker
                                                    .get_resource_span(vec![
                                                        key,
                                                        "Properties",
                                                        "Environment",
                                                        "Variables",
                                                    ])
                                                    .copied(),
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(rule_type) = rule_config.rules.get(&RuleType::LAMBDA_006) {
                            if rule_type.enabled {
                                if let Some(powertools_logger_log_event) =
                                    variables.get("POWERTOOLS_LOGGER_LOG_EVENT")
                                {
                                    if powertools_logger_log_event.as_bool().unwrap_or(false) {
                                        error_reporter.add_error(
                                            Box::new(LambdaViolation::PowertoolsLoggerLogEvent),
                                            key,
                                            line_marker
                                                .get_resource_span(vec![
                                                    key,
                                                    "Properties",
                                                    "Environment",
                                                    "Variables",
                                                ])
                                                .copied(),
                                        );
                                    }
                                }
                            }
                        }

                        if let Some(rule_type) = rule_config.rules.get(&RuleType::LAMBDA_007) {
                            if rule_type.enabled {
                                // Fetch threshold from the rule configuration
                                dbg!(&rule_config.rules);
                                let powertools_logger_sample_rate_config = rule_config
                                    .rules
                                    .get(&RuleType::LAMBDA_007)
                                    .and_then(|rule_type| {
                                        rule_type.config_detail.get_threshold_float()
                                    })
                                    .unwrap_or(1.0);
                                if let Some(powertools_logger_sample_rate) =
                                    variables.get("POWERTOOLS_LOGGER_SAMPLE_RATE")
                                {
                                    if powertools_logger_sample_rate
                                        != powertools_logger_sample_rate_config
                                    {
                                        error_reporter.add_error(
                                            Box::new(LambdaViolation::PowertoolsLoggerSampleRate),
                                            key,
                                            line_marker
                                                .get_resource_span(vec![
                                                    key,
                                                    "Properties",
                                                    "Environment",
                                                    "Variables",
                                                ])
                                                .copied(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
