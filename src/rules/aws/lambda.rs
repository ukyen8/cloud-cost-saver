use crate::error_reporter::ErrorReporter;
use crate::parsers::cfn::{AWSResourceType, CloudFormation};
use crate::parsers::config::{RuleConfig, RuleType};
use crate::parsers::LineMarker;
use crate::rules::violations::LambdaViolation;

pub fn check_lambda_missing_tag<L: LineMarker>(
    cloudformation: &CloudFormation,
    rule_config: &RuleConfig,
    error_reporter: &mut ErrorReporter,
    line_marker: &L,
) {
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

pub fn check_lambda_architecture_arm<L: LineMarker>(
    cloudformation: &CloudFormation,
    error_reporter: &mut ErrorReporter,
    line_marker: &L,
) {
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

pub fn check_lambda_missing_log_group<L: LineMarker>(
    cloudformation: &CloudFormation,
    error_reporter: &mut ErrorReporter,
    line_marker: &L,
) {
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

pub fn check_lambda_maxmimum_retry_attempts<L: LineMarker>(
    cloudformation: &CloudFormation,
    rule_config: &RuleConfig,
    error_reporter: &mut ErrorReporter,
    line_marker: &L,
) {
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

pub fn check_lambda_powertools_environment_variables<L: LineMarker>(
    cloudformation: &CloudFormation,
    rule_config: &RuleConfig,
    error_reporter: &mut ErrorReporter,
    line_marker: &L,
    environment: &str,
) {
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
                    if let Some(rule_type) = rule_config.get_rule(RuleType::LAMBDA_005, environment)
                    {
                        if rule_type.enabled {
                            if let Some(target_log_level) = rule_type.config_detail.get_value() {
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

                    if let Some(rule_type) = rule_config.get_rule(RuleType::LAMBDA_006, environment)
                    {
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

                    if let Some(rule_type) = rule_config.get_rule(RuleType::LAMBDA_007, environment)
                    {
                        if rule_type.enabled {
                            // Fetch threshold from the rule configuration
                            let powertools_logger_sample_rate_config = rule_config
                                .rules
                                .get(&RuleType::LAMBDA_007)
                                .and_then(|rule_type| rule_type.config_detail.get_threshold_float())
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



// LAMBDA-008: Check for VPC Gateway Endpoints
pub fn check_lambda_vpc_gateway_endpoints<L: LineMarker>(
    cloudformation: &CloudFormation,
    error_reporter: &mut ErrorReporter,
    line_marker: &L,
) {
    if let Some(resources) = &cloudformation.resources {
        // First check if there are any Lambdas in a VPC
        let mut vpc_lambda_found = false;
        let mut violation_candidates = Vec::new();

        for (key, resource) in resources {
            if let AWSResourceType::LambdaFunction | AWSResourceType::LambdaServerlessFunction =
                &resource.type_
            {
                if let Some(properties) = &resource.properties {
                     if properties.contains_key("VpcConfig") {
                        vpc_lambda_found = true;
                        violation_candidates.push(key);
                     }
                }
            }
        }

        if !vpc_lambda_found {
            return;
        }

        // Now check if there are any Gateway Endpoints for S3 or DynamoDB
        let mut endpoints_found = false;
        for resource in resources.values() {
             if let AWSResourceType::EC2VPCEndpoint = &resource.type_ {
                 if let Some(properties) = &resource.properties {
                     let service_name = properties
                        .get("ServiceName")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                     
                     let type_ = properties
                        .get("VpcEndpointType")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Gateway"); 

                     if (service_name.contains("s3") || service_name.contains("dynamodb")) 
                        && type_ == "Gateway" {
                         endpoints_found = true;
                         break;
                     }
                 }
             }
        }

        if !endpoints_found {
            // Updated to be a warning / acknowledgement of cross-stack resources
            for key in violation_candidates {
                error_reporter.add_error(
                    Box::new(LambdaViolation::NoVPCGatewayEndpoint),
                    key,
                    line_marker
                        .get_resource_span(vec![key, "Properties", "VpcConfig"])
                        .copied(),
                );
            }
        }
    }
}

// LAMBDA-009: Check for Static Provisioned Concurrency without AutoScaling
pub fn check_lambda_provisioned_concurrency_autoscaling<L: LineMarker>(
    cloudformation: &CloudFormation,
    error_reporter: &mut ErrorReporter,
    line_marker: &L,
) {
     if let Some(resources) = &cloudformation.resources {
        // Find Lambdas/Aliases with ProvisionedConcurrency > 0
        let mut static_provisioning = Vec::new();

        for (key, resource) in resources {
             if let AWSResourceType::LambdaFunction | AWSResourceType::LambdaServerlessFunction =
                &resource.type_
             {
                 if let Some(properties) = &resource.properties {
                     if let Some(conf) = properties.get("ProvisionedConcurrencyConfig") {
                         if let Some(val) = conf.get("ProvisionedConcurrentExecutions") {
                             if val.as_u64().unwrap_or(0) > 0 {
                                 static_provisioning.push(key);
                             }
                         }
                     }
                 }
             }
        }

        // Find ScalableTargets for Lambda ProvisionedConcurrency
        let mut scaled_resource_ids = Vec::new();
        for resource in resources.values() {
             if let AWSResourceType::ApplicationAutoScalingScalableTarget = &resource.type_ {
                 if let Some(properties) = &resource.properties {
                     let dimension = properties
                        .get("ScalableDimension")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                     
                     if dimension == "lambda:function:ProvisionedConcurrency" {
                         if let Some(resource_id) = properties.get("ResourceId") {
                             match resource_id {
                                 serde_yaml::Value::String(s) => {
                                     scaled_resource_ids.push(s.clone());
                                 }
                                 serde_yaml::Value::Tagged(tagged) => {
                                     if tagged.tag.to_string() == "!Sub" {
                                          if let serde_yaml::Value::String(s) = &tagged.value {
                                              scaled_resource_ids.push(s.clone());
                                          } else if let serde_yaml::Value::Sequence(seq) = &tagged.value {
                                              if let Some(serde_yaml::Value::String(s)) = seq.first() {
                                                  scaled_resource_ids.push(s.clone());
                                              }
                                          }
                                     }
                                 }
                                 _ => {}
                             }
                         }
                     }
                 }
             }
        }

        for key in static_provisioning {
            // Check if the Lambda's Logical ID (key) is referenced in any ScalableTarget's ResourceId.
            // This handles:
            // - !Sub "function:${MyLambda}:provisioned" (contains "MyLambda")
            // - "function:MyLambda:provisioned" (contains "MyLambda")
            let is_scaled = scaled_resource_ids.iter().any(|rid| rid.contains(key));

            if !is_scaled {
                 error_reporter.add_error(
                    Box::new(LambdaViolation::StaticProvisionedConcurrencyWithoutAutoScaling),
                    key,
                    line_marker
                        .get_resource_span(vec![key, "Properties", "ProvisionedConcurrencyConfig"])
                        .copied(),
                );
            }
        }
    }
}
