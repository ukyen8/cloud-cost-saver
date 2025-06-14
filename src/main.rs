mod checker;

use std::process::ExitCode;

use crate::checker::Checker;
use crate::parsers::config::Config;
mod error_reporter;
mod parsers;
mod rules;
use crate::parsers::cfn::{parse_cloudformation, parse_samconfig};
use crate::parsers::iac::InfratructureTemplate;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    cloud_provider: String,

    #[arg(short, long)]
    template: String,

    #[arg(short, long, default_value = "default")]
    environment: String,

    #[arg(short, long)]
    samconfig: Option<String>,

    #[arg(short, long, default_value_t = String::from("./cloudsaving.yaml"))]
    config: String,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let cloud_provider = args.cloud_provider;
    let template_file = args.template;
    let config_file = args.config;
    let config = Config::load(&config_file).unwrap_or_else(|e| {
        eprintln!("Failed to load config: {e}");
        std::process::exit(1);
    });
    let environment = args.environment;
    let mut error_reporter = error_reporter::ErrorReporter::new(&template_file);

    if cloud_provider.as_str() == "aws" {
        let mut parsed_cfn =
            parse_cloudformation(&template_file).expect("Failed to parse CloudFormation template");
        if let Some(samconfig) = args.samconfig.as_deref() {
            let samconfig = parse_samconfig(samconfig).expect("Failed to parse samconfig");
            parsed_cfn.resolve_parameters(Some(&samconfig), environment.as_str());
        } else {
            parsed_cfn.resolve_parameters(None, environment.as_str());
        }
        let infra_template = InfratructureTemplate {
            cloudformation: Some(parsed_cfn),
        };

        let line_marker =
            parsers::get_yaml_line_marker(&template_file).expect("Failed to get YAML line marker");
        let mut checker = Checker::new(
            &config,
            &mut error_reporter,
            &infra_template,
            &line_marker,
            &environment,
        );
        checker.run_checks();
        if error_reporter.has_errors() {
            eprintln!("{}", error_reporter.render_errors());
            return ExitCode::FAILURE;
        }
    };
    ExitCode::SUCCESS
}
