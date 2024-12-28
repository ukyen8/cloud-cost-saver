use crate::error_reporter::ErrorReporter;
use crate::parsers::config::{RuleConfig, RuleType};
use crate::parsers::iac::AWSResourceType;
use crate::parsers::iac::InfratructureTemplate;
use crate::parsers::LineMarker;
use crate::rules::violations::CloudWatchViolation;

pub fn check_cloudwatch_log_group_retention<L: LineMarker>(
    infra_template: &InfratructureTemplate,
    rule_config: &RuleConfig,
    error_reporter: &mut ErrorReporter,
    line_marker: &L,
) {
    if let Some(cloudformation) = &infra_template.cloudformation {
        if let Some(resources) = &cloudformation.resources {
            for (key, resource) in resources {
                if let AWSResourceType::CloudWatch = &resource.type_ {
                    if let Some(properties) = &resource.properties {
                        if let Some(retention) = properties.get("RetentionInDays") {
                            if !rule_config.enabled(RuleType::CW_001) {
                                continue;
                            }
                            if let Some(log_retention_days) =
                                rule_config.rules.get(&RuleType::CW_001)
                            {
                                if let Some(threshold) =
                                    log_retention_days.config_detail.get_threshold_int()
                                {
                                    // Check if the retention period is longer than the threshold
                                    if retention.as_u64().is_none_or(|v| v > threshold) {
                                        error_reporter.add_error(
                                            Box::new(CloudWatchViolation::LogRetentionTooLong),
                                            key,
                                            line_marker
                                                .get_resource_span(vec![
                                                    key,
                                                    "Properties",
                                                    "RetentionInDays",
                                                ])
                                                .copied(),
                                        );
                                    }
                                }
                            }
                        } else {
                            if !rule_config.enabled(RuleType::CW_002) {
                                continue;
                            }
                            error_reporter.add_error(
                                Box::new(CloudWatchViolation::NoLogRetention),
                                key,
                                line_marker
                                    .get_resource_span(vec![key, "Properties"])
                                    .copied(),
                            );
                        }
                    }
                }
            }
        }
    }
}

pub fn check_cloudwatch_log_group_class<L: LineMarker>(
    infra_template: &InfratructureTemplate,
    error_reporter: &mut ErrorReporter,
    line_marker: &L,
) {
    if let Some(cloudformation) = &infra_template.cloudformation {
        if let Some(resources) = &cloudformation.resources {
            for (key, resource) in resources {
                if let AWSResourceType::CloudWatch = &resource.type_ {
                    if let Some(properties) = &resource.properties {
                        if let Some(log_group_class) = properties.get("LogGroupClass") {
                            if log_group_class.as_str() == Some("STANDARD") {
                                error_reporter.add_error(
                                    Box::new(CloudWatchViolation::InfrequentAccessLogGroupClass),
                                    key,
                                    line_marker
                                        .get_resource_span(vec![key, "Properties", "LogGroupClass"])
                                        .copied(),
                                );
                            }
                        } else {
                            error_reporter.add_error(
                                Box::new(CloudWatchViolation::InfrequentAccessLogGroupClass),
                                key,
                                line_marker
                                    .get_resource_span(vec![key, "Properties"])
                                    .copied(),
                            );
                        }
                    }
                }
            }
        }
    }
}
