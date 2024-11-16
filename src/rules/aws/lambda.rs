use crate::error_reporter::ErrorReporter;
use crate::parsers::config::{RuleConfig, RuleType};
use crate::parsers::iac::AWSResourceType;
use crate::parsers::iac::InfratructureTemplate;
use crate::parsers::LineMarker;
use crate::rules::violations::LambdaViolation;

pub fn check_lambda_missing_tag<'a, L: LineMarker>(
    infra_template: &InfratructureTemplate,
    rule_config: &RuleConfig,
    error_reporter: &mut ErrorReporter,
    line_marker: &'a L,
) {
    if let Some(cloudformation) = &infra_template.cloudformation {
        if let Some(resources) = &cloudformation.resources {
            for (key, resource) in resources {
                match &resource.type_ {
                    AWSResourceType::LambdaFunction => {
                        if let Some(properties) = &resource.properties {
                            if let Some(tags) = properties.get("Tags") {
                                if let Some(rule_type) =
                                    rule_config.rules.get(&RuleType::LambdaMissingTag)
                                {
                                    if let Some(target_tags) = rule_type.config_detail.get_values()
                                    {
                                        // Check if at least one tag is defined in the resource
                                        let tag_exists = target_tags.iter().any(|tag| {
                                            tags.as_mapping().map_or(false, |m| m.contains_key(tag))
                                        });

                                        if !tag_exists {
                                            error_reporter.add_error(
                                                Box::new(LambdaViolation::MissingTag),
                                                &key,
                                                line_marker
                                                    .get_resource_span(vec![
                                                        key,
                                                        "Properties",
                                                        "Tags",
                                                    ])
                                                    .copied(),
                                            );
                                        }
                                    }
                                } else {
                                    error_reporter.add_error(
                                        Box::new(LambdaViolation::MissingTag),
                                        &key,
                                        line_marker
                                            .get_resource_span(vec![key, "Properties", "Tags"])
                                            .copied(),
                                    );
                                }
                            } else {
                                error_reporter.add_error(
                                    Box::new(LambdaViolation::MissingTag),
                                    &key,
                                    line_marker.get_resource_span(vec![key]).copied(),
                                )
                            };
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}

pub fn check_lambda_architecture_arm<'a, L: LineMarker>(
    infra_template: &InfratructureTemplate,
    error_reporter: &mut ErrorReporter,
    line_marker: &'a L,
) {
    if let Some(cloudformation) = &infra_template.cloudformation {
        if let Some(resources) = &cloudformation.resources {
            for (key, resource) in resources {
                match &resource.type_ {
                    AWSResourceType::LambdaFunction => {
                        if let Some(properties) = &resource.properties {
                            if let Some(architectures) = properties.get("Architectures") {
                                if architectures
                                    .as_sequence()
                                    .map_or(true, |v| !v.iter().any(|arch| arch == "arm64"))
                                {
                                    error_reporter.add_error(
                                        Box::new(LambdaViolation::ARMArchitecture),
                                        &key,
                                        line_marker
                                            .get_resource_span(vec![
                                                key,
                                                "Properties",
                                                "Architectures",
                                            ])
                                            .copied(),
                                    );
                                }
                            } else {
                                error_reporter.add_error(
                                    Box::new(LambdaViolation::ARMArchitecture),
                                    &key,
                                    line_marker.get_resource_span(vec![key]).copied(),
                                );
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}

pub fn check_lambda_missing_log_group<'a, L: LineMarker>(
    infra_template: &InfratructureTemplate,
    error_reporter: &mut ErrorReporter,
    line_marker: &'a L,
) {
    if let Some(cloudformation) = &infra_template.cloudformation {
        if let Some(resources) = &cloudformation.resources {
            for (key, resource) in resources {
                match &resource.type_ {
                    AWSResourceType::LambdaFunction => {
                        if let Some(properties) = &resource.properties {
                            if let Some(logging_config) = properties.get("LoggingConfig") {
                                if !logging_config
                                    .as_mapping()
                                    .map_or(false, |m| m.contains_key("LogGroup"))
                                {
                                    error_reporter.add_error(
                                        Box::new(LambdaViolation::MissingLogGroup),
                                        &key,
                                        line_marker
                                            .get_resource_span(vec![
                                                key,
                                                "Properties",
                                                "LoggingConfig",
                                            ])
                                            .copied(),
                                    );
                                }
                            } else {
                                error_reporter.add_error(
                                    Box::new(LambdaViolation::MissingLogGroup),
                                    &key,
                                    line_marker.get_resource_span(vec![key]).copied(),
                                );
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}
