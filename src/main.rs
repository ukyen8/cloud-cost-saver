mod checker;

use std::process::ExitCode;

use crate::checker::Checker;
use crate::parsers::config::{Config, RuleConfig};
mod error_reporter;
mod parsers;
mod rules;
use crate::parsers::cfn::{parse_cloudformation, parse_samconfig};
use clap::{Parser, Subcommand, CommandFactory};
use dialoguer::Confirm;
use std::fs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    #[clap(flatten)]
    scan_args: ScanArgs,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init,
    Scan(ScanArgs),
}

#[derive(clap::Args, Debug, Clone)]
struct ScanArgs {
    #[arg(short, long)]
    template: Option<String>,

    #[arg(short, long, default_value = "default")]
    environment: String,

    #[arg(short, long)]
    samconfig: Option<String>,

    #[arg(short, long, default_value_t = String::from("./cloudsaving.yaml"))]
    config: String,

    #[arg(long, default_value = "text")]
    format: String,

    #[arg(short, long)]
    preset: Option<crate::parsers::config::Preset>,
}

fn main() -> ExitCode {
    let args = Args::parse();

    match args.command {
        Some(Commands::Init) => run_init(),
        Some(Commands::Scan(scan_args)) => run_scan(scan_args),
        None => {
            let _ = Args::command().print_help();
            ExitCode::FAILURE
        }
    }
}

fn run_init() -> ExitCode {
    println!("Welcome to CloudSaver initialization!");

    let env_name: String = dialoguer::Input::new()
        .with_prompt("Enter the name of your environment (e.g. production)")
        .interact()
        .unwrap();

    let strict_mode = Confirm::new()
        .with_prompt("Enable strict mode? (Enables all rules)")
        .default(false)
        .interact()
        .unwrap();
    
    // We can now use presets for init too!
    let preset = if strict_mode { Some(crate::parsers::config::Preset::Strict) } else { Some(crate::parsers::config::Preset::Recommended) };

    let mut rule_config = RuleConfig::default();
    if let Some(p) = preset {
        rule_config.apply_preset(p);
    }
    
    // Default environment always created with empty overrides (inherits base rules)
    rule_config.environments.insert("default".to_string(), None);

    // User environment created with empty overrides (inherits base rules)
    // Avoid cloning full rule set
    if env_name != "default" {
        rule_config.environments.insert(env_name, None);
    }

    let config = Config {
        cloudformation: rule_config,
    };

    let yaml_string = serde_yaml::to_string(&config).expect("Failed to serialize config");
    fs::write("cloudsaving.yaml", yaml_string).expect("Failed to write config file");

    println!("Configuration saved to cloudsaving.yaml");
    ExitCode::SUCCESS
}

fn run_scan(args: ScanArgs) -> ExitCode {
    let template_file = args.template.expect("Template file is required for scan");
    let config_file = args.config;
    let mut config = Config::load(&config_file).unwrap_or_else(|e| {
        eprintln!("Failed to load config: {e}");
        std::process::exit(1);
    });
    
    // CLI override for preset
    if let Some(preset) = args.preset {
        config.cloudformation.apply_preset(preset);
    }
    
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
        } else {
            eprintln!("{}", error_reporter.render_errors());
        }
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
