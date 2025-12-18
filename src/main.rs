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

    let mut error_reporter = error_reporter::ErrorReporter::new(&template_file);
    let line_marker =
        parsers::get_yaml_line_marker(&template_file).expect("Failed to get YAML line marker");

    // Determine environments
    let samconfig_obj = if let Some(path) = &args.samconfig {
        Some(parse_samconfig(path).expect("Failed to parse samconfig"))
    } else {
        None
    };

    let environments: Vec<String> = if let Some(ref sc) = samconfig_obj {
        // Collect all environment names from samconfig
        let mut envs: Vec<String> = sc.environments.keys().cloned().collect();
        // Sort for consistent execution order
        envs.sort();
        envs
    } else {
        // Fallback to single environment from args
        vec![args.environment.clone()]
    };

    for env in environments {
        error_reporter.set_current_environment(&env);

        let mut parsed_cfn =
            parse_cloudformation(&template_file).expect("Failed to parse CloudFormation template");

        parsed_cfn.resolve_parameters(samconfig_obj.as_ref(), &env);
        parsed_cfn.apply_conditions();

        let mut checker = Checker::new(
            &config,
            &mut error_reporter,
            &parsed_cfn,
            &line_marker,
            &env,
        );
        checker.run_checks();
    }

    if error_reporter.has_errors() {
        if args.format == "json" {
            println!("{}", error_reporter.render_json());
        } else if args.format == "html" {
            println!("{}", error_reporter.render_html());
        } else {
            eprintln!("{}", error_reporter.render_errors());
        }
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
