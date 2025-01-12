# Cloud Cost Saver

A Rust-based tool for analyzing infrastructure-as-code templates, such as CloudFormation, and providing actionable suggestions to optimize cloud costs.

**Note: This project is ongoing and actively developed.**

## Features

- Analyze CloudFormation templates
- Identify cost-saving opportunities
- Provide actionable suggestions
- Currently supports AWS CloudFormation

## Installation

To install Cloud Cost Saver, clone the repository from GitHub:

```sh
git clone https://github.com/yourusername/cloud-cost-saver.git
cd cloud-cost-saver
```

## Usage

To analyze a CloudFormation template, use the following command:

```sh
cargo run -- aws --template src/fixtures/aws/cfn-testing.yaml --environment default --samconfig src/fixtures/aws/samconfig.toml --config .cloudsaving.yaml
```

## Example Output

When you run the analysis, you might see output similar to the following:

```
LAMBDA-002: MyLambdaFunction2: Consider using ARM architecture. Lambda functions on ARM can be up to 20% cheaper than equivalent x86 functions.
src/fixtures/aws/cfn-testing.yaml:20
```

In this output:
- `LAMBDA-002` is the error code.
- `MyLambdaFunction2` is the resource name.
- The text following the resource name is the issue description.
- `src/fixtures/aws/cfn-testing.yaml:20` is the file path and line number where the issue was found.

## Violations

### AWS CloudFormation

This section lists the various violations that this tool can detect in AWS CloudFormation templates. Each violation is identified by an error code and includes a description of the issue and whether it is enabled by default.

#### Lambda
| Error Code | Description | Default enabled |
|------------|-------------|-----------------|
| LAMBDA-001 | Lambda function creates a log group automatically when invoked for the first time with no expiry Please explicitly create a log group with a retention policy.| true |
| LAMBDA-002 | Consider using ARM architecture. Lambda functions on ARM can be up to 20% cheaper than equivalent x86 functions. | false |
| LAMBDA-003 | The Lambda function is missing a tag. Tags are useful for budgeting and identifying areas for cost optimization. | false |
| LAMBDA-004 | Asynchronously invoked Lambda functions have a default maximum retry attempts set to 2. Consider setting the maximum retry attempts to 0 to prevent unnecessary retries. For example, if your Lambda function is invoked via an SQS queue with 3 retries, a failure event may result in up to 9 retries. | true |
| LAMBDA-005 | Set the POWERTOOLS_LOG_LEVEL environment variable to appropriate logging levels for different environments when using AWS Lambda Powertools. This helps in reducing logging costs. | false |
| LAMBDA-006 | Logging every incoming event may significantly increase cloud costs. Consider disabling POWERTOOLS_LOGGER_LOG_EVENT in the production environment to help reduce logging expenses. | true |
| LAMBDA-007 | Set the POWERTOOLS_LOGGER_SAMPLE_RATE environment variable to a value between 0 and 1 to sample logs and reduce logging costs when using AWS Lambda Powertools. | false |

#### CloudWatch

| Error Code | Description | Default enabled |
|------------|-------------|-----------------|
| CW-001 | The log group retention period is too long. Consider reducing it to save costs and improve log management efficiency. | false |
| CW-002 | The log group has no retention policy. Consider setting a retention policy to save costs and improve log management efficiency. | true |
| CW-003 | The log group is using STANDARD class. Consider using INFREQUENT_ACCESS to save costs. | false |

## Configuration

### AWS CloudFormation

To configure the Cloud Cost Saver for AWS CloudFormation, create a `.cloudsaving.yaml` file in the root of your project. This file should contain the following settings:

```yaml
cloudformation:
    rules:
        LAMBDA_001:
            enabled: true
        LAMBDA_002:
            enabled: true
        LAMBDA_003:
            enabled: true
            values:
                - tag1
                - tag2
        LAMBDA_004:
            enabled: true
            threshold: 0
        LAMBDA_005:
            enabled: true
            value: "ERROR"
        CW_001:
            enabled: true
            threshold: 14
        CW_002:
            enabled: true
        CW_003:
            enabled: false
    environments:
        dev:
        sandbox:
            LAMBDA_002:
                enabled: false
            CW_003:
                enabled: true
        prod:
            LAMBDA_003:
                enabled: true
                values:
                    - tag3
                    - tag4
            CW_002:
                enabled: false
```

In this configuration:
- `LAMBDA_001`, `LAMBDA_002`, and `LAMBDA_003` are enabled, with `LAMBDA_003` requiring specific tags (`tag1` and `tag2`) and `LAMBDA_004` with a threshold of 0 for retry attempts.
- `CW_001` is enabled with a threshold of 14 days for log retention.
- `CW_002` is enabled to ensure log groups have a retention policy.
- `CW_003` is disabled, meaning it will not check for the use of the `INFREQUENT_ACCESS` class for log groups.
- Environments `dev`, `sandbox`, and `prod` are defined with specific rule configurations.

A `default` environment will be automatically created. The rules defined under environments will override the default rules.

### Rules configuration table

The `.cloudsaving.yaml` file allows you to customize the behavior of the Cloud Cost Saver tool. The below table lists all the rules configurations, specifying whether they are simple (only need to specify enabled or not), value, values, or threshold.

| Rule Type | Configuration Type | Description |
|-----------|--------------------|-------------|
| LAMBDA_001 | Simple             | Enabled or not |
| LAMBDA_002 | Simple             | Enabled or not |
| LAMBDA_003 | Values             | List of required tags |
| LAMBDA_004 | Threshold          | Lambda maximum retry attempts |
| LAMBDA_005 | Value              | POWERTOOLS_LOG_LEVEL value |
| LAMBDA_006 | Simple             | Enable to check if POWERTOOLS_LOGGER_LOG_EVENT is set to false |
| LAMBDA_007 | Threshold          | Set sample rate with POWERTOOLS_LOGGER_SAMPLE_RATE |
| CW_001     | Threshold          | Log retention period in days |
| CW_002     | Simple             | Enabled or not |
| CW_003     | Simple             | Enabled or not |

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.