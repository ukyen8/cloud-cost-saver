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
cargo run -- aws --template src/sample-cfn.yaml --config .cloudsaving.yaml
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.