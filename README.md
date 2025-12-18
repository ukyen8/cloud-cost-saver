# Cloud Cost Saver

A Rust-based tool for analyzing infrastructure-as-code templates (AWS CloudFormation) to identify cost-saving opportunities. It parses your templates, evaluates them against a set of cost optimization rules, and generates actionable reports.

> **Note**: This project is actively developed.

## Features

-   **CloudFormation Analysis**: Deep scanning of `template.yaml` files.
-   **Multi-Environment Support**: Reads `samconfig.toml` to understand deployment environments (e.g., `default`, `prod`) and evaluates parameter overrides and conditions specific to each environment.
-   **HTML Reporting**: Generates interactive HTML reports with environment selectors and dynamic issue filtering.
-   **Strict by Default**: Applies a comprehensive set of rules to catch potential savings.
-   **Condition Logic**: Supports `!If`, `!Equals`, `!Not`, and `!Ref` conditions to accurately reflect what resources are actually created in each environment.

## Installation

### Option 1: Download Binary

You don't need Rust to use Cloud Cost Saver. Simply download the latest binary for your operating system from the [Releases](https://github.com/ukyen8/cloud-cost-saver/releases) page.

1.  Download the binary/executable.
2.  Rename it to `ccs` (or `ccs.exe` on Windows).
3.  Add it to your PATH or run it directly.

```sh
# Example on macOS/Linux
./ccs scan -t template.yaml
```

### Option 2: Build from Source

If you have Rust installed:

```sh
git clone https://github.com/yourusername/cloud-cost-saver.git
cd cloud-cost-saver
cargo build --release
# Binary will be in target/release/ccs
```

## Usage

CloudSaver scans an AWS CloudFormation template and optional SAM configuration to report violations.

> **Tip**: If you are running from source code, replace `ccs` with `cargo run --` in the examples below.

### Basic Scan

Analyze a template using the `scan` command:

```sh
ccs scan -t template.yaml
```

### Multi-Environment Scan with HTML Report

To scan all environments defined in your `samconfig.toml` and generate an interactive HTML report:

```sh
ccs scan \
  --template src/fixtures/aws/cfn-advanced-cost.yaml \
  --samconfig src/fixtures/aws/samconfig.toml \
  --format html > report.html
```

Open `report.html` in your browser to inspect findings across different environments.

### CLI Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--template` | `-t` | Path to CloudFormation template | **Required** |
| `--samconfig` | | Path to `samconfig.toml` | None |
| `--environment` | `-e` | Fallback environment name if no config | `default` |
| `--format` | | Output format (`text`, `json`, `html`) | `text` |

## Violations

The following rules are enforced to help validata and optimize costs:

### Lambda

| Code | Description |
|------|-------------|
| `LAMBDA-001` | **Log Group Retention**: Ensure Log Groups are explicitly created with a retention policy to avoid infinite storage costs. |
| `LAMBDA-002` | **Architecture**: Suggests using **ARM64** architecture for up to 20% cost savings. |
| `LAMBDA-003` | **Tagging**: Checks for missing tags (e.g., `CostCenter`) crucial for cost allocation. |
| `LAMBDA-004` | **Async Retries**: Suggests reducing maximum retry attempts for async invocations to prevent wastage during failure loops. |
| `LAMBDA-005` | **Powertools Log Level**: Verify `POWERTOOLS_LOG_LEVEL` is set appropriate for the environment. |
| `LAMBDA-006` | **Log Events**: Suggests disabling `POWERTOOLS_LOGGER_LOG_EVENT` in production to reduce log data volume. |
| `LAMBDA-007` | **Sample Rate**: Suggests setting `POWERTOOLS_LOGGER_SAMPLE_RATE` to sample logs rather than logging everything. |
| `LAMBDA-008` | **VPC Gateway Endpoints**: Ensures Lambda functions in VPCs have access to S3/DynamoDB Gateway Endpoints to avoid expensive NAT Gateway charges. |
| `LAMBDA-009` | **Static Concurrency**: Detects static Provisioned Concurrency without AutoScaling, which leads to idle costs. |

### CloudWatch

| Code | Description |
|------|-------------|
| `CW-001` | **Retention Period**: warns if log retention is set to extremely long periods (e.g., > 30 days) for high-volume logs. |
| `CW-002` | **Missing Retention**: Warns if a Log Group has no retention policy at all. |
| `CW-003` | **Log Class**: Suggests using `INFREQUENT_ACCESS` log class for cost savings where appropriate. |

## Configuration

CloudSaver now relies on **presets** and standard **AWS SAM configuration**:

-   **Rules**: By default, the `Strict` preset is applied, enabling all available rules.
-   **Environments**: Defined in `samconfig.toml`. CloudSaver parses parameter overrides and applies them to the template to handle environment-specific logic (like conditional resources).

## GitHub Action Usage

Integrate CloudSaver into your CI/CD pipeline:

```yaml
- name: Run Cloud Cost Saver
  uses: ./
  with:
    template: template.yaml
    samconfig: samconfig.toml
    format: html
```

## Contributing

Contributions are welcome! Please open an issue or submit a PR.

## License

MIT License.