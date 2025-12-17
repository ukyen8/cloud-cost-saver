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

### Scan a Template
To analyze a CloudFormation template, use the `scan` command:

```sh
cargo run -- scan --template src/fixtures/aws/cfn-testing.yaml --environment default --samconfig src/fixtures/aws/samconfig.toml
```

You can also output the results in JSON format:

```sh
cargo run -- scan --template src/fixtures/aws/cfn-testing.yaml --format json
```

### HTML Reporting
Generate a detailed HTML report categorized by resource type:

```sh
cargo run -- scan --template src/fixtures/aws/cfn-testing.yaml --format html > report.html
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

This section lists the various violations that this tool can detect in AWS CloudFormation templates. By default, all rules are enabled.

#### Lambda
| Error Code | Description |
|------------|-------------|
| LAMBDA-001 | Lambda function creates a log group automatically when invoked for the first time with no expiry Please explicitly create a log group with a retention policy.|
| LAMBDA-002 | Consider using ARM architecture. Lambda functions on ARM can be up to 20% cheaper than equivalent x86 functions. |
| LAMBDA-003 | The Lambda function is missing a tag. Tags are useful for budgeting and identifying areas for cost optimization. |
| LAMBDA-004 | Asynchronously invoked Lambda functions have a default maximum retry attempts set to 2. Consider setting the maximum retry attempts to 0 to prevent unnecessary retries. For example, if your Lambda function is invoked via an SQS queue with 3 retries, a failure event may result in up to 9 retries. |
| LAMBDA-005 | Set the POWERTOOLS_LOG_LEVEL environment variable to appropriate logging levels for different environments when using AWS Lambda Powertools. This helps in reducing logging costs. |
| LAMBDA-006 | Logging every incoming event may significantly increase cloud costs. Consider disabling POWERTOOLS_LOGGER_LOG_EVENT in the production environment to help reduce logging expenses. |
| LAMBDA-007 | Set the POWERTOOLS_LOGGER_SAMPLE_RATE environment variable to a value between 0 and 1 to sample logs and reduce logging costs when using AWS Lambda Powertools. |
| LAMBDA-008 | Ensure Lambda functions in a VPC have compatible Gateway Endpoints (S3/DynamoDB) to avoid expensive NAT Gateway data processing charges. |
| LAMBDA-009 | Detect Static Provisioned Concurrency without AutoScaling. Suggests using Application Auto Scaling to optimize costs during low-traffic periods. |

#### CloudWatch

| Error Code | Description |
|------------|-------------|
| CW-001 | The log group retention period is too long. Consider reducing it to save costs and improve log management efficiency. |
| CW-002 | The log group has no retention policy. Consider setting a retention policy to save costs and improve log management efficiency. |
| CW-003 | The log group is using STANDARD class. Consider using INFREQUENT_ACCESS to save costs. |

## GitHub Action Usage

You can use Cloud Cost Saver as a GitHub Action to automatically analyze your AWS CloudFormation templates for cost optimization in your CI/CD pipeline.

### Basic Usage

Add the following step to your workflow YAML (e.g., `.github/workflows/cloud_cost_saver.yml`):

```yaml
- name: Run Cloud Cost Saver
  uses: ./
  with:
    template: src/fixtures/aws/cfn-testing-pass.yaml
    environment: default
    samconfig: src/fixtures/aws/samconfig.toml
```

### Inputs

| Name           | Description                                              | Required | Example                                      |
|----------------|----------------------------------------------------------|----------|----------------------------------------------|
| template       | Path to the CloudFormation template to analyze           | Yes      | src/fixtures/aws/cfn-testing-pass.yaml        |
| environment    | Environment name for rule overrides                      | No       | default                                      |
| samconfig      | Path to your AWS SAM config file                         | No       | src/fixtures/aws/samconfig.toml               |
| format         | Output format (text, json, html)                         | No       | text                                         |

### Example Workflow

```yaml
name: Cloud Cost Saver

on:
  push:
    branches:
      - main
  pull_request:

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Cloud Cost Saver
        uses: ./
        with:
          template: src/fixtures/aws/cfn-testing-pass.yaml
          environment: default
          samconfig: src/fixtures/aws/samconfig.toml
```

This will run the Cloud Cost Saver action on every push to `main` and on pull requests, analyzing your CloudFormation template for cost-saving opportunities.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.