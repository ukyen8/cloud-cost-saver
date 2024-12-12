use crate::error_reporter::ErrorReporter;
use crate::parsers::config::{RuleConfig, RuleType};
use crate::parsers::iac::AWSResourceType;
use crate::parsers::iac::InfratructureTemplate;
use crate::parsers::LineMarker;
use crate::rules::violations::CloudWatchViolation;

pub fn check_cloudwatch_log_group_retention<'a, L: LineMarker>(
    infra_template: &InfratructureTemplate,
    rule_config: &RuleConfig,
    error_reporter: &mut ErrorReporter,
    line_marker: &'a L,
) {
    if let Some(cloudformation) = &infra_template.cloudformation {
        if let Some(resources) = &cloudformation.resources {
            for (key, resource) in resources {
                match &resource.type_ {
                    AWSResourceType::CloudWatch => {
                        if let Some(properties) = &resource.properties {
                            if let Some(retention) = properties.get("RetentionInDays") {
                                if let Some(log_retention_days) =
                                    rule_config.rules.get(&RuleType::CWLogRetentionPolicy)
                                {
                                    if let Some(threshold) =
                                        log_retention_days.config_detail.get_threshold()
                                    {
                                        // Check if the retention period is longer than the threshold
                                        if retention.as_u64().map_or(true, |v| v > threshold) {
                                            error_reporter.add_error(
                                                Box::new(CloudWatchViolation::LogRetentionTooLong),
                                                &key,
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
                                error_reporter.add_error(
                                    Box::new(CloudWatchViolation::NoLogRetention),
                                    &key,
                                    line_marker
                                        .get_resource_span(vec![key, "Properties"])
                                        .copied(),
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
