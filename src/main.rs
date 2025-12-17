mod checker;

use std::process::ExitCode;

use crate::checker::Checker;
use crate::parsers::config::{Config, RuleConfig};
mod error_reporter;
mod parsers;
mod rules;
use crate::parsers::cfn::{parse_cloudformation, parse_samconfig};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Scan(ScanArgs),
}

#[derive(clap::Args, Debug, Clone)]
struct ScanArgs {
    #[arg(short, long)]
    template: String,

    #[arg(short, long, default_value = "default")]
    environment: String,

    #[arg(short, long)]
    samconfig: Option<String>,

    #[arg(long, default_value = "text")]
    format: String,
}

fn main() -> ExitCode {
    let args = Args::parse();

    match args.command {
        Commands::Scan(scan_args) => run_scan(scan_args),
    }
}

fn run_scan(args: ScanArgs) -> ExitCode {
    let template_file = args.template;

    // Default to Strict preset (all rules enabled)
    let mut rule_config = RuleConfig::default();
    rule_config.apply_preset(crate::parsers::config::Preset::Strict);
    let config = Config {
        cloudformation: rule_config,
    };

    let environment = args.environment;
    let mut error_reporter = error_reporter::ErrorReporter::new(&template_file);

    let mut parsed_cfn =
        parse_cloudformation(&template_file).expect("Failed to parse CloudFormation template");
    if let Some(samconfig) = args.samconfig.as_deref() {
        let samconfig = parse_samconfig(samconfig).expect("Failed to parse samconfig");
        parsed_cfn.resolve_parameters(Some(&samconfig), environment.as_str());
    } else {
        parsed_cfn.resolve_parameters(None, environment.as_str());
    }
    let line_marker =
        parsers::get_yaml_line_marker(&template_file).expect("Failed to get YAML line marker");
    let mut checker = Checker::new(
        &config,
        &mut error_reporter,
        &parsed_cfn,
        &line_marker,
        &environment,
    );
    checker.run_checks();
    if error_reporter.has_errors() {
        if args.format == "json" {
            println!("{}", error_reporter.render_json());
        } else if args.format == "html" {
            // HTML rendering will be implemented in error_reporter
            println!("{}", error_reporter.render_html());
        } else {
            eprintln!("{}", error_reporter.render_errors());
        }
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
