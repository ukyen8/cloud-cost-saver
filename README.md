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

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.
## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.