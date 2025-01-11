use crate::error_reporter::ErrorReporter;
use crate::parsers::config::{Config, RuleType};
use crate::parsers::iac::InfratructureTemplate;
use crate::parsers::LineMarker;
use crate::rules::aws;

pub(crate) struct Checker<'a, L: LineMarker + 'a> {
    config: &'a Config,
    error_reporter: &'a mut ErrorReporter,
    infra_template: &'a InfratructureTemplate,
    line_marker: &'a L,
    environment: &'a str,
}

impl<'a, L: LineMarker + 'a> Checker<'a, L> {
    pub(crate) fn new(
        config: &'a Config,
        error_reporter: &'a mut ErrorReporter,
        infra_template: &'a InfratructureTemplate,
        line_marker: &'a L,
        environment: &'a str,
    ) -> Checker<'a, L> {
        Checker {
            config,
            error_reporter,
            infra_template,
            line_marker,
            environment,
        }
    }

    pub(crate) fn run_checks(&mut self) {
        if let Some(rule_config) = &self.config.cloudformation {
            if rule_config.enabled(RuleType::LAMBDA_003, self.environment) {
                aws::lambda::check_lambda_missing_tag(
                    self.infra_template,
                    rule_config,
                    self.error_reporter,
                    self.line_marker,
                );
            }
            if rule_config.enabled(RuleType::LAMBDA_002, self.environment) {
                aws::lambda::check_lambda_architecture_arm(
                    self.infra_template,
                    self.error_reporter,
                    self.line_marker,
                );
            }

            if rule_config.enabled(RuleType::LAMBDA_001, self.environment) {
                aws::lambda::check_lambda_missing_log_group(
                    self.infra_template,
                    self.error_reporter,
                    self.line_marker,
                );
            }

            if rule_config.enabled(RuleType::LAMBDA_004, self.environment) {
                aws::lambda::check_lambda_maxmimum_retry_attempts(
                    self.infra_template,
                    rule_config,
                    self.error_reporter,
                    self.line_marker,
                );
            }

            if rule_config.enabled(RuleType::LAMBDA_005, self.environment)
                || rule_config.enabled(RuleType::LAMBDA_006, self.environment)
                || rule_config.enabled(RuleType::LAMBDA_007, self.environment)
            {
                aws::lambda::check_lambda_powertools_environment_variables(
                    self.infra_template,
                    rule_config,
                    self.error_reporter,
                    self.line_marker,
                    self.environment,
                );
            }

            if rule_config.enabled(RuleType::CW_001, self.environment)
                || rule_config.enabled(RuleType::CW_002, self.environment)
            {
                aws::cloudwatch::check_cloudwatch_log_group_retention(
                    self.infra_template,
                    rule_config,
                    self.error_reporter,
                    self.line_marker,
                    self.environment,
                );
            }

            if rule_config.enabled(RuleType::CW_003, self.environment) {
                aws::cloudwatch::check_cloudwatch_log_group_class(
                    self.infra_template,
                    self.error_reporter,
                    self.line_marker,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests_cfn {
    use crate::rules::violations::Violation;

    struct ExpectedViolation {
        code: String,
        resource: String,
        message: String,
    }

    impl ExpectedViolation {
        fn new(violation: &dyn Violation, resource: &str) -> Self {
            Self {
                code: violation.code(),
                resource: resource.to_string(),
                message: violation.message(),
            }
        }
    }

    struct ExpectedViolations(Vec<ExpectedViolation>);

    impl ExpectedViolations {
        fn new(violations: Vec<ExpectedViolation>) -> Self {
            Self(violations)
        }

        fn assert_all_match(&self, actual: &str) {
            let actual_lines: Vec<&str> = actual
                .lines()
                .filter(|line| !line.is_empty() && !line.contains("src/fixtures/aws/"))
                .collect();
            for (i, expected) in self.0.iter().enumerate() {
                let expected_str = format!(
                    "{}:{}:{}",
                    expected.code, expected.resource, expected.message
                );
                let actual_line = actual_lines.get(i).unwrap_or(&"");
                assert_eq!(
                    actual_line, &expected_str,
                    "\nExpected: {}\nActual: {}",
                    expected_str, actual_line
                );
            }
        }
    }

    mod cfn_tests {
        use super::{ExpectedViolation, ExpectedViolations};
        use crate::checker::Checker;
        use crate::error_reporter::ErrorReporter;
        use crate::parsers::cfn::{parse_cloudformation, CloudFormation};
        use crate::parsers::config::{
            Config, RuleConfig, RuleType, RuleTypeConfigDetail, ThresholdValue,
        };
        use crate::parsers::get_yaml_line_marker;
        use crate::parsers::iac::InfratructureTemplate;
        use crate::parsers::YamlLineMarker;
        use crate::rules::violations::{CloudWatchViolation, LambdaViolation};
        use rstest::*;

        #[fixture]
        fn test_config() -> Config {
            let mut default_rule_config = RuleConfig::default();
            for rule in default_rule_config.rules.values_mut() {
                rule.enabled = false;
            }
            default_rule_config.environments.insert(
                "default".to_string(),
                Some(default_rule_config.rules.clone()),
            );

            Config {
                cloudformation: Some(default_rule_config),
            }
        }

        fn get_error_reporter(path: &str) -> ErrorReporter {
            ErrorReporter::new(&format!("src/fixtures/aws/{}", path))
        }

        fn get_cloudformation(path: &str) -> CloudFormation {
            let mut parsed_cfn =
                parse_cloudformation(&format!("src/fixtures/aws/{}", path)).unwrap();
            parsed_cfn.resolve_parameters(None, "default");
            parsed_cfn
        }

        fn get_line_marker(path: &str) -> YamlLineMarker {
            get_yaml_line_marker(&format!("src/fixtures/aws/{}", path)).unwrap()
        }

        fn enable_rule(
            config: &mut Config,
            rule_type: RuleType,
            config_detail: Option<RuleTypeConfigDetail>,
        ) {
            if let Some(cloudformation) = &mut config.cloudformation {
                cloudformation.set_rule(rule_type, true, "default", config_detail.as_ref());
            }
        }

        struct TestContext {
            config: Config,
            error_reporter: ErrorReporter,
            infra_template: InfratructureTemplate,
            line_marker: YamlLineMarker,
        }

        impl TestContext {
            fn new(
                template_name: &str,
                rule_type: RuleType,
                config_detail: Option<RuleTypeConfigDetail>,
            ) -> Self {
                let mut config = test_config(); // Use existing fixture
                enable_rule(&mut config, rule_type, config_detail);

                Self {
                    config,
                    error_reporter: get_error_reporter(template_name),
                    infra_template: InfratructureTemplate {
                        cloudformation: Some(get_cloudformation(template_name)),
                    },
                    line_marker: get_line_marker(template_name),
                }
            }

            fn create_checker(&mut self) -> Checker<'_, YamlLineMarker> {
                Checker::new(
                    &self.config,
                    &mut self.error_reporter,
                    &self.infra_template,
                    &self.line_marker,
                    "default",
                )
            }
        }

        #[fixture]
        fn setup_checker() -> impl Fn(&str, RuleType, Option<RuleTypeConfigDetail>) -> TestContext {
            |template_name, rule_type, config_detail: Option<RuleTypeConfigDetail>| {
                TestContext::new(template_name, rule_type, config_detail)
            }
        }

        #[rstest]
        #[case(
            "cfn-testing.yaml",
            RuleType::LAMBDA_003,
            Some(RuleTypeConfigDetail::Values {
                values: vec![String::from("tag1")]
            }),
            LambdaViolation::MissingTag
        )]
        #[case(
            "cfn-testing.yaml",
            RuleType::LAMBDA_002,
            None,
            LambdaViolation::ARMArchitecture
        )]
        #[case(
            "cfn-testing.yaml",
            RuleType::LAMBDA_001,
            None,
            LambdaViolation::MissingLogGroup
        )]
        fn test_lambda_001_002_003(
            #[case] template_name: &str,
            #[case] rule_type: RuleType,
            #[case] config_detail: Option<RuleTypeConfigDetail>,
            #[case] violation: LambdaViolation,
            setup_checker: impl Fn(&str, RuleType, Option<RuleTypeConfigDetail>) -> TestContext,
        ) {
            let mut context = setup_checker(template_name, rule_type, config_detail);
            let mut checker = context.create_checker();
            checker.run_checks();

            let expected = ExpectedViolations::new(vec![
                ExpectedViolation::new(&violation, "MyLambdaFunction"),
                ExpectedViolation::new(&violation, "MyLambdaFunction2"),
            ]);
            expected.assert_all_match(&context.error_reporter.render_errors());
        }

        #[rstest]
        #[case(
            "cfn-lambda-examples.yaml",
            RuleType::LAMBDA_004,
            Some(RuleTypeConfigDetail::Threshold { threshold: ThresholdValue::Int(0) }),
            LambdaViolation::MaximumRetryAttempts
        )]
        fn test_lambda_004(
            #[case] template_name: &str,
            #[case] rule_type: RuleType,
            #[case] config_detail: Option<RuleTypeConfigDetail>,
            #[case] violation: LambdaViolation,
            setup_checker: impl Fn(&str, RuleType, Option<RuleTypeConfigDetail>) -> TestContext,
        ) {
            let mut context = setup_checker(template_name, rule_type, config_detail);
            let mut checker = context.create_checker();
            checker.run_checks();

            let expected = ExpectedViolations::new(vec![
                ExpectedViolation::new(&violation, "MyLambdaFunction"),
                ExpectedViolation::new(&violation, "MyLambdaFunction3"),
            ]);
            expected.assert_all_match(&context.error_reporter.render_errors());
        }

        #[rstest]
        #[case(
            "cfn-lambda-examples.yaml",
            RuleType::LAMBDA_007,
            Some(RuleTypeConfigDetail::Threshold { threshold: ThresholdValue::Float(0.5) }),
            LambdaViolation::PowertoolsLoggerSampleRate
        )]
        fn test_lambda_005_006_007(
            #[case] template_name: &str,
            #[case] rule_type: RuleType,
            #[case] config_detail: Option<RuleTypeConfigDetail>,
            #[case] violation: LambdaViolation,
            setup_checker: impl Fn(&str, RuleType, Option<RuleTypeConfigDetail>) -> TestContext,
        ) {
            let mut context = setup_checker(template_name, rule_type, config_detail);
            let mut checker = context.create_checker();
            checker.run_checks();

            let expected = ExpectedViolations::new(vec![ExpectedViolation::new(
                &violation,
                "MyLambdaFunction",
            )]);
            expected.assert_all_match(&context.error_reporter.render_errors());
        }

        #[rstest]
        #[case(
            "cfn-testing.yaml",
            RuleType::CW_001,
            Some(RuleTypeConfigDetail::Threshold { threshold: ThresholdValue::Int(14) }),
            CloudWatchViolation::LogRetentionTooLong,
        )]
        #[case(
            "cfn-testing.yaml",
            RuleType::CW_002,
            None,
            CloudWatchViolation::NoLogRetention
        )]
        fn test_cw_001_002(
            #[case] template_name: &str,
            #[case] rule_type: RuleType,
            #[case] config_detail: Option<RuleTypeConfigDetail>,
            #[case] violation: CloudWatchViolation,
            setup_checker: impl Fn(&str, RuleType, Option<RuleTypeConfigDetail>) -> TestContext,
        ) {
            let mut context = setup_checker(template_name, rule_type, config_detail);
            let mut checker = context.create_checker();
            checker.run_checks();

            let log_group_name = if violation == CloudWatchViolation::LogRetentionTooLong {
                "MyLogGroup"
            } else {
                "MyLogGroup2"
            };
            let expected =
                ExpectedViolations::new(vec![ExpectedViolation::new(&violation, log_group_name)]);
            expected.assert_all_match(&context.error_reporter.render_errors());
        }

        #[rstest]
        #[case(
            "cfn-testing.yaml",
            RuleType::CW_003,
            None,
            CloudWatchViolation::InfrequentAccessLogGroupClass
        )]
        fn test_cw_003(
            #[case] template_name: &str,
            #[case] rule_type: RuleType,
            #[case] config_detail: Option<RuleTypeConfigDetail>,
            #[case] violation: CloudWatchViolation,
            setup_checker: impl Fn(&str, RuleType, Option<RuleTypeConfigDetail>) -> TestContext,
        ) {
            let mut context = setup_checker(template_name, rule_type, config_detail);
            let mut checker = context.create_checker();
            checker.run_checks();

            let expected = ExpectedViolations::new(vec![
                ExpectedViolation::new(&violation, "MyLogGroup"),
                ExpectedViolation::new(&violation, "MyLogGroup2"),
            ]);
            expected.assert_all_match(&context.error_reporter.render_errors());
        }
    }
}
