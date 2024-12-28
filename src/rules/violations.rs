use std::fmt::Debug;
use strum::Display;
use strum_macros::EnumIter;
pub trait Violation: std::fmt::Debug {
    fn message(&self) -> String;
    fn code(&self) -> String;
}

#[derive(EnumIter, Debug, Display)]
pub enum LambdaViolation {
    MissingLogGroup,
    ARMArchitecture,
    MissingTag,
    MaximumRetryAttempts,
    PowertoolsLogLevel,
}

impl Violation for LambdaViolation {
    fn message(&self) -> String {
        match self {
            LambdaViolation::ARMArchitecture => {
                "Consider using ARM architecture. \
                Lambda functions on ARM can be up to 20% cheaper than equivalent x86 functions.".to_string()
            }
            LambdaViolation::MissingLogGroup => {
                "Lambda function creates a log group automatically when invoked for the first time with no expiry. \
                Please explicitly create a log group with a retention policy.".to_string()
            }
            LambdaViolation::MissingTag => {
                "The Lambda function is missing a tag. \
                Tags are useful for budgeting and identifying areas for cost optimization.".to_string()
            },
            LambdaViolation::MaximumRetryAttempts => {
                "Asynchronously invoked Lambda functions have a default maximum retry attempts set to 2. \
                Consider setting the maximum retry attempts to 0 to prevent unnecessary retries. \
                For example, if your Lambda function is invoked via an SQS queue with 3 retries, \
                a failure event may result in up to 9 retries.".to_string()
            },
            LambdaViolation::PowertoolsLogLevel => {
                "Set the POWERTOOLS_LOG_LEVEL environment variable to appropriate logging levels \
                for different environments when using AWS Lambda Powertools. \
                This helps in reducing logging costs.".to_string()
            }
        }
    }

    fn code(&self) -> String {
        match self {
            LambdaViolation::MissingLogGroup => "LAMBDA-001".to_string(),
            LambdaViolation::ARMArchitecture => "LAMBDA-002".to_string(),
            LambdaViolation::MissingTag => "LAMBDA-003".to_string(),
            LambdaViolation::MaximumRetryAttempts => "LAMBDA-004".to_string(),
            LambdaViolation::PowertoolsLogLevel => "LAMBDA-005".to_string(),
        }
    }
}

#[derive(EnumIter, Debug, Display, PartialEq)]
pub enum CloudWatchViolation {
    LogRetentionTooLong,
    NoLogRetention,
    InfrequentAccessLogGroupClass,
}

impl Violation for CloudWatchViolation {
    fn message(&self) -> String {
        match self {
            CloudWatchViolation::LogRetentionTooLong => {
                "The log group retention period is too long. Consider reducing it to save costs and improve log management efficiency.".to_string()
            }
            CloudWatchViolation::NoLogRetention => {
                "The log group has no retention policy. Consider setting a retention policy to save costs and improve log management efficiency.".to_string()
            }
            CloudWatchViolation::InfrequentAccessLogGroupClass => {
                "The log group is using STANDARD class. Consider using INFREQUENT_ACCESS to save costs.".to_string()
            }
        }
    }

    fn code(&self) -> String {
        match self {
            CloudWatchViolation::LogRetentionTooLong => "CW-001".to_string(),
            CloudWatchViolation::NoLogRetention => "CW-002".to_string(),
            CloudWatchViolation::InfrequentAccessLogGroupClass => "CW-003".to_string(),
        }
    }
}
