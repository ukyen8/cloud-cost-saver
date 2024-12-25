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

#### CloudWatch

| Error Code | Description | Default enabled |
|------------|-------------|-----------------|
| CW-001 | The log group retention period is too long. Consider reducing it to save costs and improve log management efficiency. | false |
| CW-002 | The log group has no retention policy. Consider setting a retention policy to save costs and improve log management efficiency. | true |
| CW-003 | The log group is using STANDARD class. Consider using INFREQUENT_ACCESS to save costs. | false |

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.