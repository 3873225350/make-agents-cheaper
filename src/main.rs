use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value as JsonValue};
use sha2::{Digest, Sha256};
use toml::Value as TomlValue;

const EXPECTED_BASE_URL_ENV: &str = "MAKE_AGENTS_CHEAPER_EXPECTED_BASE_URL";

const WS_CONFIG: &str = r#"model_provider = "cache_router"
model = "gpt-5.4"
model_reasoning_effort = "xhigh"
plan_mode_reasoning_effort = "xhigh"
model_reasoning_summary = "none"
model_verbosity = "medium"
approval_policy = "never"
sandbox_mode = "danger-full-access"
suppress_unstable_features_warning = true

[model_providers.cache_router]
name = "OpenAI"
base_url = "https://router.example/v1"
wire_api = "responses"
requires_openai_auth = false
env_key = "CACHE_ROUTER_API_KEY"
supports_websockets = true

[features]
responses_websockets_v2 = true
"#;

const HTTP_CONFIG: &str = r#"model_provider = "cache_router"
model = "gpt-5.4"
model_reasoning_effort = "xhigh"
plan_mode_reasoning_effort = "xhigh"
model_reasoning_summary = "none"
model_verbosity = "medium"
approval_policy = "never"
sandbox_mode = "danger-full-access"

[model_providers.cache_router]
name = "OpenAI"
base_url = "https://router.example/v1"
wire_api = "responses"
requires_openai_auth = false
env_key = "CACHE_ROUTER_API_KEY"
"#;

const COMPACT_TEMPLATE: &str = r#"# Cache-Aware Reactivation Prefix

## Stable Repo Facts
- Repository:
- Primary language / framework:
- Stable architecture notes:

## Stable Operating Rules
- AGENTS.md / project rules:
- Do-not-touch areas:
- Validation expectations:

## Stable Tool And Harness Policy
- Keep tool schema order stable.
- Keep provider, model, transport, and session route stable during one task.
- Structure stable components first and dynamic components later.

## Current Objective
- Goal:
- Success criteria:

## Known Constraints
- Security / privacy:
- Compatibility:
- Product wording:

## Last Verified State
- Branch:
- Tests or checks last run:
- Known failures:

## Dynamic Recent Changes
- Files changed recently:
- Latest tool outputs:
- Open questions:

## Next Turn Request
- Start here:
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Finding {
    level: &'static str,
    message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PriceConfig {
    uncached_input_per_mtok: f64,
    cached_input_per_mtok: f64,
    output_per_mtok: f64,
}

#[derive(Debug, PartialEq)]
enum Command {
    AuditConfig {
        config: PathBuf,
    },
    PrintWsConfig,
    PrintHttpConfig,
    Fingerprint {
        input: PathBuf,
        previous: Option<PathBuf>,
    },
    ToolSchema {
        input: PathBuf,
        previous: Option<PathBuf>,
    },
    Breakpoints {
        input: PathBuf,
    },
    TraceImport {
        input: PathBuf,
        run_id: String,
        task_id: String,
        condition: String,
        slice: Option<String>,
        repeat_id: Option<u64>,
        phase: Option<String>,
        output: Option<PathBuf>,
        artifacts_dir: Option<PathBuf>,
        validation_path: Option<PathBuf>,
        validation_passed: Option<bool>,
        task_success: Option<bool>,
    },
    ClaudeJsonImport {
        input: PathBuf,
        run_id: String,
        task_id: String,
        condition: String,
        slice: Option<String>,
        repeat_id: Option<u64>,
        phase: Option<String>,
        output: Option<PathBuf>,
        validation_path: Option<PathBuf>,
        validation_passed: Option<bool>,
        task_success: Option<bool>,
    },
    Eval {
        baseline: PathBuf,
        candidate: PathBuf,
        prices: Option<PriceConfig>,
    },
    TaskReport {
        baseline: PathBuf,
        candidate: PathBuf,
    },
    AnalysisReport {
        baseline: PathBuf,
        candidate: PathBuf,
        output: Option<PathBuf>,
    },
    EvidenceDiff {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    InitExperiment {
        dir: PathBuf,
    },
    PilotPlan {
        manifest: PathBuf,
        task: String,
        experiment_dir: PathBuf,
        slice: Option<String>,
        repeats: u64,
    },
    RunPilot {
        manifest: PathBuf,
        task: String,
        experiment_dir: PathBuf,
        slice: Option<String>,
        repeats: u64,
        execute: bool,
    },
    MatrixPlan {
        manifest: PathBuf,
        experiment_dir: PathBuf,
        tasks: Option<String>,
        repeats: u64,
    },
    CompactTemplate,
    Help,
}

fn main() {
    let command = match parse_args(env::args().skip(1)) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("{message}");
            print_help();
            std::process::exit(2);
        }
    };

    let exit_code = match command {
        Command::Help => {
            print_help();
            0
        }
        Command::PrintWsConfig => {
            print!("{WS_CONFIG}");
            0
        }
        Command::PrintHttpConfig => {
            print!("{HTTP_CONFIG}");
            0
        }
        Command::AuditConfig { config } => run_config_report(&config).unwrap_or_else(print_error),
        Command::Fingerprint { input, previous } => {
            run_fingerprint_report(&input, previous.as_deref()).unwrap_or_else(print_error)
        }
        Command::ToolSchema { input, previous } => {
            run_tool_schema_report(&input, previous.as_deref()).unwrap_or_else(print_error)
        }
        Command::Breakpoints { input } => run_breakpoint_report(&input).unwrap_or_else(print_error),
        Command::TraceImport {
            input,
            run_id,
            task_id,
            condition,
            slice,
            repeat_id,
            phase,
            output,
            artifacts_dir,
            validation_path,
            validation_passed,
            task_success,
        } => run_trace_import(TraceImportOptions {
            input: &input,
            run_id: &run_id,
            task_id: &task_id,
            condition: &condition,
            slice: slice.as_deref(),
            repeat_id,
            phase: phase.as_deref(),
            output: output.as_deref(),
            artifacts_dir: artifacts_dir.as_deref(),
            validation_path: validation_path.as_deref(),
            validation_passed,
            task_success,
        })
        .unwrap_or_else(print_error),
        Command::ClaudeJsonImport {
            input,
            run_id,
            task_id,
            condition,
            slice,
            repeat_id,
            phase,
            output,
            validation_path,
            validation_passed,
            task_success,
        } => run_claude_json_import(ClaudeJsonImportOptions {
            input: &input,
            run_id: &run_id,
            task_id: &task_id,
            condition: &condition,
            slice: slice.as_deref(),
            repeat_id,
            phase: phase.as_deref(),
            output: output.as_deref(),
            validation_path: validation_path.as_deref(),
            validation_passed,
            task_success,
        })
        .unwrap_or_else(print_error),
        Command::Eval {
            baseline,
            candidate,
            prices,
        } => run_eval_report(&baseline, &candidate, prices).unwrap_or_else(print_error),
        Command::TaskReport {
            baseline,
            candidate,
        } => run_task_report(&baseline, &candidate).unwrap_or_else(print_error),
        Command::AnalysisReport {
            baseline,
            candidate,
            output,
        } => run_analysis_report(&baseline, &candidate, output.as_deref())
            .unwrap_or_else(print_error),
        Command::EvidenceDiff { input, output } => {
            run_evidence_diff(&input, output.as_deref()).unwrap_or_else(print_error)
        }
        Command::InitExperiment { dir } => run_init_experiment(&dir).unwrap_or_else(print_error),
        Command::PilotPlan {
            manifest,
            task,
            experiment_dir,
            slice,
            repeats,
        } => run_pilot_plan(&manifest, &task, &experiment_dir, slice.as_deref(), repeats)
            .unwrap_or_else(print_error),
        Command::RunPilot {
            manifest,
            task,
            experiment_dir,
            slice,
            repeats,
            execute,
        } => run_pilot(
            &manifest,
            &task,
            &experiment_dir,
            slice.as_deref(),
            repeats,
            execute,
        )
        .unwrap_or_else(print_error),
        Command::MatrixPlan {
            manifest,
            experiment_dir,
            tasks,
            repeats,
        } => run_matrix_plan(&manifest, &experiment_dir, tasks.as_deref(), repeats)
            .unwrap_or_else(print_error),
        Command::CompactTemplate => {
            print!("{COMPACT_TEMPLATE}");
            0
        }
    };

    std::process::exit(exit_code);
}

fn print_error(message: String) -> i32 {
    eprintln!("[ERROR] {message}");
    2
}

fn print_help() {
    println!(
        r#"make-agents-cheaper

Usage:
  make-agents-cheaper [--config PATH]
  make-agents-cheaper audit [--config PATH]
  make-agents-cheaper --print-ws-config
  make-agents-cheaper --print-http-config
  make-agents-cheaper fingerprint --input layers.json [--previous prev.json]
  make-agents-cheaper tool-schema --input tools.json [--previous prev.json]
  make-agents-cheaper breakpoints --input request.json
  make-agents-cheaper trace-import --input raw.jsonl --run-id RUN --task-id TASK --condition CONDITION \
    [--slice SLICE] [--repeat-id N] [--phase PHASE] [--output out.jsonl] \
    [--artifacts-dir runs/exp] [--validation-path path] [--validation-passed true|false] \
    [--task-success true|false]
  make-agents-cheaper claude-json-import --input result.json --run-id RUN --task-id TASK --condition CONDITION \
    [--slice SLICE] [--repeat-id N] [--phase PHASE] [--output out.jsonl] \
    [--validation-path path] [--validation-passed true|false] [--task-success true|false]
  make-agents-cheaper eval --baseline baseline.jsonl --candidate cache-friendly.jsonl \
    [--uncached-input-per-mtok USD --cached-input-per-mtok USD --output-per-mtok USD]
  make-agents-cheaper task-report --baseline baseline.jsonl --candidate cache-friendly.jsonl
  make-agents-cheaper analysis-report --baseline baseline.jsonl --candidate cache-friendly.jsonl \
    [--output report.md]
  make-agents-cheaper evidence-diff --input session.jsonl [--output code-changes.json]
  make-agents-cheaper init-experiment --dir runs/exp-name
  make-agents-cheaper pilot-plan --manifest suite.json --task TASK_ID --experiment-dir runs/exp-name \
    [--slice SLICE_ID] [--repeats N]
  make-agents-cheaper run-pilot --manifest suite.json --task TASK_ID --experiment-dir runs/exp-name \
    [--slice SLICE_ID] [--repeats N] [--execute true|false]
  make-agents-cheaper matrix-plan --manifest suite.json --experiment-dir runs/exp-name \
    [--tasks task-a,task-b] [--repeats N]
  make-agents-cheaper compact-template

Commands:
  audit             Inspect Codex config for cache-friendly settings
  fingerprint       Hash prompt/harness layers and report drift
  tool-schema       Hash and inspect tool schema stability
  breakpoints       Inspect cache_control breakpoint placement
  trace-import      Normalize a raw claude-trace JSONL run into eval JSONL
  claude-json-import Normalize Claude Code --output-format json into eval JSONL
  eval              Compare baseline vs cache-friendly JSONL runs
  task-report       Print per-task token usage from JSONL runs
  analysis-report   Write paper-facing aggregate, per-slice, and per-task Markdown tables
  evidence-diff     Extract tool-claimed code changes from session JSONL/JSON
  init-experiment   Create a reproducible experiment log directory
  pilot-plan        Print a ready-to-run paired pilot command plan from a task manifest
  run-pilot         Write a bounded pilot shell script, optionally execute it
  matrix-plan       Print full-matrix pilot-plan commands and expected run counts
  compact-template  Print a stable-first reactivation template

Options:
  --config PATH         Inspect a specific Codex config.toml
  --print-ws-config     Print recommended cache-aware router WebSocket config
  --print-http-config   Print recommended cache-aware router HTTP config
  -h, --help            Show this help"#
    );
}

fn parse_args<I>(args: I) -> Result<Command, String>
where
    I: IntoIterator,
    I::Item: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    if args.is_empty() {
        return Ok(Command::AuditConfig {
            config: default_config_path(),
        });
    }

    match args[0].as_str() {
        "-h" | "--help" | "help" => Ok(Command::Help),
        "--print-ws-config" => reject_extra(&args, Command::PrintWsConfig),
        "--print-http-config" => reject_extra(&args, Command::PrintHttpConfig),
        "--config" => Ok(Command::AuditConfig {
            config: one_path_arg(&args, "--config")?,
        }),
        "audit" => Ok(Command::AuditConfig {
            config: option_path(&args[1..], "--config")?.unwrap_or_else(default_config_path),
        }),
        "fingerprint" => Ok(Command::Fingerprint {
            input: required_path(&args[1..], "--input")?,
            previous: option_path(&args[1..], "--previous")?,
        }),
        "tool-schema" => Ok(Command::ToolSchema {
            input: required_path(&args[1..], "--input")?,
            previous: option_path(&args[1..], "--previous")?,
        }),
        "breakpoints" => Ok(Command::Breakpoints {
            input: required_path(&args[1..], "--input")?,
        }),
        "trace-import" => Ok(Command::TraceImport {
            input: required_path(&args[1..], "--input")?,
            run_id: required_string(&args[1..], "--run-id")?,
            task_id: required_string(&args[1..], "--task-id")?,
            condition: required_string(&args[1..], "--condition")?,
            slice: option_string(&args[1..], "--slice")?,
            repeat_id: option_u64(&args[1..], "--repeat-id")?,
            phase: option_string(&args[1..], "--phase")?,
            output: option_path(&args[1..], "--output")?,
            artifacts_dir: option_path(&args[1..], "--artifacts-dir")?,
            validation_path: option_path(&args[1..], "--validation-path")?,
            validation_passed: option_bool(&args[1..], "--validation-passed")?,
            task_success: option_bool(&args[1..], "--task-success")?,
        }),
        "claude-json-import" => Ok(Command::ClaudeJsonImport {
            input: required_path(&args[1..], "--input")?,
            run_id: required_string(&args[1..], "--run-id")?,
            task_id: required_string(&args[1..], "--task-id")?,
            condition: required_string(&args[1..], "--condition")?,
            slice: option_string(&args[1..], "--slice")?,
            repeat_id: option_u64(&args[1..], "--repeat-id")?,
            phase: option_string(&args[1..], "--phase")?,
            output: option_path(&args[1..], "--output")?,
            validation_path: option_path(&args[1..], "--validation-path")?,
            validation_passed: option_bool(&args[1..], "--validation-passed")?,
            task_success: option_bool(&args[1..], "--task-success")?,
        }),
        "eval" => Ok(Command::Eval {
            baseline: required_path(&args[1..], "--baseline")?,
            candidate: required_path(&args[1..], "--candidate")?,
            prices: price_config_from_args(&args[1..])?,
        }),
        "task-report" => Ok(Command::TaskReport {
            baseline: required_path(&args[1..], "--baseline")?,
            candidate: required_path(&args[1..], "--candidate")?,
        }),
        "analysis-report" => Ok(Command::AnalysisReport {
            baseline: required_path(&args[1..], "--baseline")?,
            candidate: required_path(&args[1..], "--candidate")?,
            output: option_path(&args[1..], "--output")?,
        }),
        "evidence-diff" => Ok(Command::EvidenceDiff {
            input: required_path(&args[1..], "--input")?,
            output: option_path(&args[1..], "--output")?,
        }),
        "init-experiment" => Ok(Command::InitExperiment {
            dir: required_path(&args[1..], "--dir")?,
        }),
        "pilot-plan" => Ok(Command::PilotPlan {
            manifest: required_path(&args[1..], "--manifest")?,
            task: required_string(&args[1..], "--task")?,
            experiment_dir: required_path(&args[1..], "--experiment-dir")?,
            slice: option_string(&args[1..], "--slice")?,
            repeats: option_u64(&args[1..], "--repeats")?.unwrap_or(1),
        }),
        "run-pilot" => Ok(Command::RunPilot {
            manifest: required_path(&args[1..], "--manifest")?,
            task: required_string(&args[1..], "--task")?,
            experiment_dir: required_path(&args[1..], "--experiment-dir")?,
            slice: option_string(&args[1..], "--slice")?,
            repeats: option_u64(&args[1..], "--repeats")?.unwrap_or(1),
            execute: option_bool(&args[1..], "--execute")?.unwrap_or(false),
        }),
        "matrix-plan" => Ok(Command::MatrixPlan {
            manifest: required_path(&args[1..], "--manifest")?,
            experiment_dir: required_path(&args[1..], "--experiment-dir")?,
            tasks: option_string(&args[1..], "--tasks")?,
            repeats: option_u64(&args[1..], "--repeats")?.unwrap_or(3),
        }),
        "compact-template" => reject_extra(&args, Command::CompactTemplate),
        other => Err(format!("unknown command or option: {other}")),
    }
}

fn reject_extra(args: &[String], command: Command) -> Result<Command, String> {
    if args.len() == 1 {
        Ok(command)
    } else {
        Err(format!("unexpected extra argument: {}", args[1]))
    }
}

fn one_path_arg(args: &[String], flag: &str) -> Result<PathBuf, String> {
    if args.len() != 2 {
        return Err(format!("{flag} requires exactly one path"));
    }
    Ok(PathBuf::from(&args[1]))
}

fn required_path(args: &[String], flag: &str) -> Result<PathBuf, String> {
    option_path(args, flag)?.ok_or_else(|| format!("{flag} is required"))
}

fn required_string(args: &[String], flag: &str) -> Result<String, String> {
    option_string(args, flag)?.ok_or_else(|| format!("{flag} is required"))
}

fn option_path(args: &[String], flag: &str) -> Result<Option<PathBuf>, String> {
    let mut found = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == flag {
            let Some(value) = args.get(i + 1) else {
                return Err(format!("{flag} requires a path"));
            };
            found = Some(PathBuf::from(value));
            i += 2;
        } else if args[i].starts_with("--") {
            let Some(_) = args.get(i + 1) else {
                return Err(format!("{} requires a value", args[i]));
            };
            i += 2;
        } else {
            return Err(format!("unexpected argument: {}", args[i]));
        }
    }
    Ok(found)
}

fn option_string(args: &[String], flag: &str) -> Result<Option<String>, String> {
    let mut found = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == flag {
            let Some(value) = args.get(i + 1) else {
                return Err(format!("{flag} requires a value"));
            };
            found = Some(value.clone());
            i += 2;
        } else if args[i].starts_with("--") {
            let Some(_) = args.get(i + 1) else {
                return Err(format!("{} requires a value", args[i]));
            };
            i += 2;
        } else {
            return Err(format!("unexpected argument: {}", args[i]));
        }
    }
    Ok(found)
}

fn option_u64(args: &[String], flag: &str) -> Result<Option<u64>, String> {
    let Some(value) = option_string(args, flag)? else {
        return Ok(None);
    };
    value
        .parse::<u64>()
        .map(Some)
        .map_err(|err| format!("{flag} must be a non-negative integer: {err}"))
}

fn option_bool(args: &[String], flag: &str) -> Result<Option<bool>, String> {
    let Some(value) = option_string(args, flag)? else {
        return Ok(None);
    };
    match value.as_str() {
        "true" => Ok(Some(true)),
        "false" => Ok(Some(false)),
        _ => Err(format!("{flag} must be true or false")),
    }
}

fn price_config_from_args(args: &[String]) -> Result<Option<PriceConfig>, String> {
    let uncached_input_per_mtok = option_f64(args, "--uncached-input-per-mtok")?;
    let cached_input_per_mtok = option_f64(args, "--cached-input-per-mtok")?;
    let output_per_mtok = option_f64(args, "--output-per-mtok")?;

    match (
        uncached_input_per_mtok,
        cached_input_per_mtok,
        output_per_mtok,
    ) {
        (None, None, None) => Ok(None),
        (Some(uncached_input_per_mtok), Some(cached_input_per_mtok), Some(output_per_mtok)) => {
            Ok(Some(PriceConfig {
                uncached_input_per_mtok,
                cached_input_per_mtok,
                output_per_mtok,
            }))
        }
        _ => Err(
            "cost estimation requires all three price flags: --uncached-input-per-mtok, --cached-input-per-mtok, and --output-per-mtok"
                .to_string(),
        ),
    }
}

fn option_f64(args: &[String], flag: &str) -> Result<Option<f64>, String> {
    let mut found = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == flag {
            let Some(value) = args.get(i + 1) else {
                return Err(format!("{flag} requires a value"));
            };
            let parsed = value
                .parse::<f64>()
                .map_err(|err| format!("{flag} must be a number: {err}"))?;
            found = Some(parsed);
            i += 2;
        } else if args[i].starts_with("--") {
            let Some(_) = args.get(i + 1) else {
                return Err(format!("{} requires a value", args[i]));
            };
            i += 2;
        } else {
            return Err(format!("unexpected argument: {}", args[i]));
        }
    }
    Ok(found)
}

fn default_config_path() -> PathBuf {
    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home).join(".codex").join("config.toml");
    }
    PathBuf::from(".codex").join("config.toml")
}

fn run_config_report(config_path: &Path) -> Result<i32, String> {
    if !config_path.exists() {
        println!("Make Agents Cheaper report");
        println!("Config: {}", config_path.display());
        println!();
        println!("[WARN] Config file does not exist.");
        println!("Tip: print a template with --print-ws-config or --print-http-config.");
        return Ok(1);
    }

    let raw = fs::read_to_string(config_path)
        .map_err(|err| format!("Could not read config file: {err}"))?;
    let config = raw
        .parse::<TomlValue>()
        .map_err(|err| format!("Could not parse TOML: {err}"))?;

    Ok(print_findings(config_path, &audit_config(&config)))
}

fn print_findings(config_path: &Path, findings: &[Finding]) -> i32 {
    let warnings = findings
        .iter()
        .filter(|finding| finding.level == "WARN")
        .count();

    println!("Make Agents Cheaper report");
    println!("Config: {}", config_path.display());
    println!();

    for finding in findings {
        println!("[{}] {}", finding.level, finding.message);
    }

    println!();
    if warnings > 0 {
        println!("Result: {warnings} cache-friendliness warning(s).");
        println!("Tip: print a known-good template with --print-ws-config or --print-http-config.");
        1
    } else {
        println!("Result: config looks prompt-cache friendly.");
        0
    }
}

fn audit_config(config: &TomlValue) -> Vec<Finding> {
    let mut findings = Vec::new();
    let Some(provider_name) = str_value(config, &["model_provider"]) else {
        findings.push(warn("No top-level model_provider is configured."));
        return findings;
    };

    findings.push(info(format!("Active provider: {provider_name}")));

    if let Some(model) = str_value(config, &["model"]) {
        findings.push(info(format!("Active model: {model}")));
    } else {
        findings.push(warn("No stable top-level model is configured."));
    }

    let provider_path = ["model_providers", provider_name];

    match str_value(config, &[provider_path[0], provider_path[1], "base_url"]) {
        Some(base_url) if base_url.trim().is_empty() => {
            findings.push(warn("Provider base_url is empty."));
        }
        Some(base_url) => match env::var(EXPECTED_BASE_URL_ENV) {
            Ok(expected_base_url) if base_url == expected_base_url => {
                findings.push(ok(format!(
                    "Provider base_url matches {EXPECTED_BASE_URL_ENV}."
                )));
            }
            Ok(expected_base_url) => {
                findings.push(warn(format!(
                    "Provider base_url is {:?}, not the value from {EXPECTED_BASE_URL_ENV}: {:?}.",
                    base_url, expected_base_url
                )));
            }
            Err(_) => {
                findings.push(ok(format!(
                    "Provider base_url is configured. Set {EXPECTED_BASE_URL_ENV} to verify a specific router endpoint."
                )));
            }
        },
        None => findings.push(warn("Provider base_url is missing.")),
    }

    match str_value(config, &[provider_path[0], provider_path[1], "wire_api"]) {
        Some("responses") => findings.push(ok(r#"wire_api = "responses" is configured."#)),
        _ => findings.push(warn(
            r#"wire_api is not "responses"; Codex cache behavior may be less stable."#,
        )),
    }

    let supports_ws = bool_value(
        config,
        &[provider_path[0], provider_path[1], "supports_websockets"],
    );
    let feature_ws = bool_value(config, &["features", "responses_websockets_v2"]);
    match (supports_ws, feature_ws) {
        (true, true) => findings.push(ok("WebSocket Responses mode is enabled.")),
        (true, false) | (false, true) => findings.push(warn(
            "WebSocket config is only partially enabled; check provider and features.",
        )),
        (false, false) => findings.push(info(
            "WebSocket mode is not enabled. HTTP can still work, but long-session continuity may be weaker.",
        )),
    }

    match str_value(config, &[provider_path[0], provider_path[1], "env_key"]) {
        Some(env_key) if env::var_os(env_key).is_some() => {
            findings.push(ok(format!("Environment variable {env_key} is set.")));
        }
        Some(env_key) => findings.push(warn(format!(
            "Environment variable {env_key} is not set in this shell."
        ))),
        None => findings.push(warn("Provider env_key is missing.")),
    }

    match (
        str_value(config, &["model_reasoning_effort"]),
        str_value(config, &["plan_mode_reasoning_effort"]),
    ) {
        (Some(effort), Some(plan_effort)) if effort != plan_effort => findings.push(info(
            "Model and plan-mode reasoning efforts differ; keep them stable within a task.",
        )),
        (Some(effort), _) => findings.push(ok(format!("Reasoning effort is stable: {effort}"))),
        _ => findings.push(info("No explicit reasoning effort configured.")),
    }

    if str_value(config, &["model_reasoning_summary"]) == Some("none") {
        findings.push(ok("Reasoning summaries are disabled."));
    }

    findings
}

fn run_fingerprint_report(input: &Path, previous: Option<&Path>) -> Result<i32, String> {
    let current = extract_layers(&read_json(input)?)?;
    let previous_layers = previous
        .map(read_json)
        .transpose()?
        .map(|value| extract_layers(&value))
        .transpose()?;

    println!("Prefix fingerprint report");
    println!("Input: {}", input.display());
    if let Some(previous_path) = previous {
        println!("Previous: {}", previous_path.display());
    }
    println!();
    println!("{:<28} {:<12} {}", "Layer", "Status", "Hash");

    for (name, value) in &current {
        let hash = short_hash_json(value);
        let status = previous_layers
            .as_ref()
            .and_then(|prev| prev.get(name))
            .map(|prev| {
                if short_hash_json(prev) == hash {
                    "stable"
                } else {
                    "changed"
                }
            })
            .unwrap_or("new");
        println!("{:<28} {:<12} {}", name, status, hash);
    }

    if let Some(previous_layers) = previous_layers.as_ref() {
        let missing: BTreeSet<_> = previous_layers
            .keys()
            .filter(|name| !current.contains_key(*name))
            .cloned()
            .collect();
        for name in missing {
            println!("{:<28} {:<12} -", name, "removed");
        }
    }

    println!();
    println!("Tip: changed stable-layer hashes point to prefix drift without printing private prompt text.");
    Ok(0)
}

fn run_tool_schema_report(input: &Path, previous: Option<&Path>) -> Result<i32, String> {
    let current_json = read_json(input)?;
    let current_tools = extract_tools(&current_json)?;
    let current_hash = short_hash_json(&JsonValue::Array(current_tools.clone()));
    let current_names = tool_names(&current_tools);
    let sorted_names = sorted_clone(&current_names);

    println!("Tool schema report");
    println!("Input: {}", input.display());
    println!();
    println!("Tool count: {}", current_tools.len());
    println!("Tool schema hash: {current_hash}");
    println!(
        "Tool order: {}",
        if current_names == sorted_names {
            "canonical by name"
        } else {
            "not sorted by name"
        }
    );

    if let Some(previous) = previous {
        let previous_json = read_json(previous)?;
        let previous_tools = extract_tools(&previous_json)?;
        let previous_hash = short_hash_json(&JsonValue::Array(previous_tools));
        println!("Previous: {}", previous.display());
        println!(
            "Drift: {}",
            if previous_hash == current_hash {
                "stable"
            } else {
                "changed"
            }
        );
    }

    if current_names != sorted_names {
        println!();
        println!("[WARN] Tool definitions are not in canonical name order.");
        println!("       Tool execution order can vary; tool definition order should stay stable.");
        return Ok(1);
    }

    Ok(0)
}

fn run_breakpoint_report(input: &Path) -> Result<i32, String> {
    let json = read_json(input)?;
    let blocks = extract_blocks(&json);
    let breakpoints: Vec<_> = blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| has_direct_cache_control(block))
        .collect();

    println!("Breakpoint-aware cache report");
    println!("Input: {}", input.display());
    println!();
    println!("Observed blocks: {}", blocks.len());
    println!("cache_control breakpoints: {}", breakpoints.len());

    if breakpoints.is_empty() {
        println!("[INFO] No explicit cache_control breakpoints found.");
        return Ok(0);
    }

    println!();
    println!("{:<12} {:<16} {}", "Index", "GapFromPrev", "Role");
    let mut previous_index = None;
    let mut over_20_risk = false;
    for (index, block) in breakpoints {
        let gap = previous_index.map(|prev| index - prev).unwrap_or(index);
        if gap > 20 {
            over_20_risk = true;
        }
        println!("{:<12} {:<16} {}", index, gap, block_role(block));
        previous_index = Some(index);
    }

    println!();
    if over_20_risk {
        println!("[WARN] At least one breakpoint gap exceeds 20 blocks.");
        println!(
            "       Some cache reads may fail if the provider only searches a bounded window."
        );
        Ok(1)
    } else {
        println!("Result: breakpoint gaps are within the 20-block heuristic.");
        Ok(0)
    }
}

fn run_eval_report(
    baseline: &Path,
    candidate: &Path,
    prices: Option<PriceConfig>,
) -> Result<i32, String> {
    let baseline_stats = RunStats::from_jsonl(baseline)?;
    let candidate_stats = RunStats::from_jsonl(candidate)?;

    println!("Cache-hit evaluation report");
    println!("Baseline: {}", baseline.display());
    println!("Candidate: {}", candidate.display());
    println!();
    println!("{:<28} {:>14} {:>14}", "Metric", "Baseline", "Candidate");
    println!(
        "{:<28} {:>13.2}% {:>13.2}%",
        "Cache hit rate",
        baseline_stats.cache_hit_rate() * 100.0,
        candidate_stats.cache_hit_rate() * 100.0
    );
    println!(
        "{:<28} {:>14} {:>14}",
        "Uncached input", baseline_stats.uncached_input, candidate_stats.uncached_input
    );
    println!(
        "{:<28} {:>14} {:>14}",
        "Output tokens", baseline_stats.output, candidate_stats.output
    );
    println!(
        "{:<28} {:>14} {:>14}",
        "Median TTFT ms",
        display_median(&baseline_stats.ttft_ms),
        display_median(&candidate_stats.ttft_ms)
    );
    println!(
        "{:<28} {:>14} {:>14}",
        "Median latency ms",
        display_median(&baseline_stats.total_latency_ms),
        display_median(&candidate_stats.total_latency_ms)
    );
    println!(
        "{:<28} {:>14} {:>14}",
        "Validation passed",
        format!(
            "{}/{}",
            baseline_stats.validation_passed, baseline_stats.records
        ),
        format!(
            "{}/{}",
            candidate_stats.validation_passed, candidate_stats.records
        )
    );
    println!(
        "{:<28} {:>14} {:>14}",
        "Task success",
        format!("{}/{}", baseline_stats.task_success, baseline_stats.records),
        format!(
            "{}/{}",
            candidate_stats.task_success, candidate_stats.records
        )
    );
    if baseline_stats.actual_cost_usd > 0.0 || candidate_stats.actual_cost_usd > 0.0 {
        println!(
            "{:<28} {:>14} {:>14}",
            "Observed cost",
            display_usd(baseline_stats.actual_cost_usd),
            display_usd(candidate_stats.actual_cost_usd)
        );
    }
    if let Some(prices) = prices {
        let baseline_cost = baseline_stats.estimated_actual_cost(prices);
        let candidate_cost = candidate_stats.estimated_actual_cost(prices);
        let baseline_full_cost = baseline_stats.estimated_full_uncached_cost(prices);
        let candidate_full_cost = candidate_stats.estimated_full_uncached_cost(prices);
        println!(
            "{:<28} {:>14} {:>14}",
            "Estimated cost",
            display_usd(baseline_cost),
            display_usd(candidate_cost)
        );
        println!(
            "{:<28} {:>14} {:>14}",
            "Full uncached cost",
            display_usd(baseline_full_cost),
            display_usd(candidate_full_cost)
        );
        println!(
            "{:<28} {:>14} {:>14}",
            "Cache savings",
            display_optional_percent(savings_ratio(baseline_cost, baseline_full_cost)),
            display_optional_percent(savings_ratio(candidate_cost, candidate_full_cost))
        );
    }

    println!();
    println!("Interpretation:");
    let uncached_ratio = ratio(
        candidate_stats.uncached_input,
        baseline_stats.uncached_input,
    );
    println!("  uncached input ratio: {}", display_ratio(uncached_ratio));
    println!("  cached tokens should go up, uncached paid input should go down, and task success should not regress.");
    if let Some(prices) = prices {
        let baseline_cost = baseline_stats.estimated_actual_cost(prices);
        let candidate_cost = candidate_stats.estimated_actual_cost(prices);
        println!(
            "  estimated cost ratio: {}",
            display_ratio_f64(ratio_f64(candidate_cost, baseline_cost))
        );
    } else if baseline_stats.actual_cost_usd > 0.0 || candidate_stats.actual_cost_usd > 0.0 {
        println!(
            "  observed cost ratio: {}",
            display_ratio_f64(ratio_f64(
                candidate_stats.actual_cost_usd,
                baseline_stats.actual_cost_usd
            ))
        );
    } else {
        println!("  price flags were not provided, so estimated dollar cost was not computed.");
    }
    if baseline_stats.cache_accounting_unobservable > 0
        || candidate_stats.cache_accounting_unobservable > 0
    {
        println!("  warning: at least one record marks cache_accounting_observable=false; do not claim token-cost savings for those records.");
    }
    Ok(0)
}

fn run_task_report(baseline: &Path, candidate: &Path) -> Result<i32, String> {
    let baseline_records = RunRecord::from_jsonl(baseline)?;
    let candidate_records = RunRecord::from_jsonl(candidate)?;
    let baseline_by_task = stats_by_task(&baseline_records);
    let candidate_by_task = stats_by_task(&candidate_records);

    let mut task_ids: BTreeSet<String> = BTreeSet::new();
    task_ids.extend(baseline_by_task.keys().cloned());
    task_ids.extend(candidate_by_task.keys().cloned());

    println!("Per-task token usage report");
    println!("Baseline: {}", baseline.display());
    println!("Candidate: {}", candidate.display());
    println!();
    println!(
        "{:<24} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>11} {:>11}",
        "Task",
        "B Hit%",
        "C Hit%",
        "B Unc",
        "C Unc",
        "B Cache",
        "C Cache",
        "B Out",
        "C Out",
        "B Success",
        "C Success"
    );

    for task_id in task_ids {
        let baseline_stats = baseline_by_task.get(&task_id).cloned().unwrap_or_default();
        let candidate_stats = candidate_by_task.get(&task_id).cloned().unwrap_or_default();
        println!(
            "{:<24} {:>8.2}% {:>8.2}% {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>11} {:>11}",
            truncate_for_table(&task_id, 24),
            baseline_stats.cache_hit_rate() * 100.0,
            candidate_stats.cache_hit_rate() * 100.0,
            baseline_stats.uncached_input,
            candidate_stats.uncached_input,
            baseline_stats.cached_input,
            candidate_stats.cached_input,
            baseline_stats.output,
            candidate_stats.output,
            format!("{}/{}", baseline_stats.task_success, baseline_stats.records),
            format!(
                "{}/{}",
                candidate_stats.task_success, candidate_stats.records
            )
        );
    }

    println!();
    println!("Token fields:");
    println!("  Unc = input_tokens - cached_input_tokens");
    println!("  Cache = cached_input_tokens");
    println!("  Out = output_tokens");
    Ok(0)
}

fn run_analysis_report(
    baseline: &Path,
    candidate: &Path,
    output: Option<&Path>,
) -> Result<i32, String> {
    let baseline_records = RunRecord::from_jsonl(baseline)?;
    let candidate_records = RunRecord::from_jsonl(candidate)?;
    let report =
        analysis_report_markdown(baseline, candidate, &baseline_records, &candidate_records);

    if let Some(output) = output {
        if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
            fs::create_dir_all(parent)
                .map_err(|err| format!("Could not create {}: {err}", parent.display()))?;
        }
        fs::write(output, &report)
            .map_err(|err| format!("Could not write {}: {err}", output.display()))?;
        println!("Wrote analysis report: {}", output.display());
    } else {
        print!("{report}");
    }

    Ok(0)
}

fn analysis_report_markdown(
    baseline_path: &Path,
    candidate_path: &Path,
    baseline_records: &[RunRecord],
    candidate_records: &[RunRecord],
) -> String {
    let baseline_all = RunStats::from_records(baseline_records);
    let candidate_all = RunStats::from_records(candidate_records);
    let baseline_successful = successful_records(baseline_records);
    let candidate_successful = successful_records(candidate_records);
    let baseline_successful_stats = RunStats::from_records(&baseline_successful);
    let candidate_successful_stats = RunStats::from_records(&candidate_successful);

    let mut report = String::new();
    report.push_str("# Paper-Facing Analysis Summary\n\n");
    report.push_str("## Inputs\n\n");
    let _ = writeln!(report, "- Baseline JSONL: `{}`", baseline_path.display());
    let _ = writeln!(report, "- Candidate JSONL: `{}`", candidate_path.display());
    report.push('\n');

    report.push_str("## Roles\n\n");
    report.push_str("- Development assistant: Codex\n");
    report.push_str("- Studied harness: Claude Code\n");
    report.push_str("- Backend route/model: MiMo, such as `mimo-v2.5-pro`\n");
    report.push_str("- Measurement layer: `make-agents-cheaper` audit/eval logs\n");
    report.push_str("- Reuse layer: skills are auxiliary runbooks, not primary evidence\n\n");

    report.push_str("## Aggregate Tables\n\n");
    report.push_str(
        "The all-runs table is the primary evidence surface because it keeps failures and anomalies in view.\n\n",
    );
    report.push_str("| Slice | Baseline records | Candidate records | Baseline hit | Candidate hit | Baseline uncached | Candidate uncached | Candidate/baseline uncached | Baseline success | Candidate success |\n");
    report.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    push_analysis_row(&mut report, "All runs", &baseline_all, &candidate_all);
    report.push('\n');

    report.push_str("The per-slice all-runs table keeps `control-steady` and `dynamic-drift` evidence separate.\n\n");
    report.push_str("| Slice | Baseline records | Candidate records | Baseline hit | Candidate hit | Baseline uncached | Candidate uncached | Candidate/baseline uncached | Baseline success | Candidate success |\n");
    report.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    let baseline_by_slice = stats_by_slice(baseline_records);
    let candidate_by_slice = stats_by_slice(candidate_records);
    let mut slice_ids: BTreeSet<String> = BTreeSet::new();
    slice_ids.extend(baseline_by_slice.keys().cloned());
    slice_ids.extend(candidate_by_slice.keys().cloned());
    for slice_id in slice_ids {
        let baseline_stats = baseline_by_slice
            .get(&slice_id)
            .cloned()
            .unwrap_or_default();
        let candidate_stats = candidate_by_slice
            .get(&slice_id)
            .cloned()
            .unwrap_or_default();
        push_analysis_row(
            &mut report,
            &markdown_cell(&slice_id),
            &baseline_stats,
            &candidate_stats,
        );
    }
    report.push('\n');

    report.push_str(
        "The successful-only table is diagnostic; it must not replace the all-runs table when making the main claim.\n\n",
    );
    report.push_str("| Slice | Baseline records | Candidate records | Baseline hit | Candidate hit | Baseline uncached | Candidate uncached | Candidate/baseline uncached | Baseline success | Candidate success |\n");
    report.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    push_analysis_row(
        &mut report,
        "Successful-only",
        &baseline_successful_stats,
        &candidate_successful_stats,
    );
    report.push('\n');

    report.push_str("## Per-Task All-Runs Table\n\n");
    report.push_str("| Task | B records | C records | B hit | C hit | B uncached | C uncached | B success | C success |\n");
    report.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    let baseline_by_task = stats_by_task(baseline_records);
    let candidate_by_task = stats_by_task(candidate_records);
    let mut task_ids: BTreeSet<String> = BTreeSet::new();
    task_ids.extend(baseline_by_task.keys().cloned());
    task_ids.extend(candidate_by_task.keys().cloned());
    for task_id in task_ids {
        let baseline_stats = baseline_by_task.get(&task_id).cloned().unwrap_or_default();
        let candidate_stats = candidate_by_task.get(&task_id).cloned().unwrap_or_default();
        let _ = writeln!(
            report,
            "| `{}` | {} | {} | {} | {} | {} | {} | {} | {} |",
            markdown_cell(&task_id),
            baseline_stats.records,
            candidate_stats.records,
            cache_hit_display(&baseline_stats),
            cache_hit_display(&candidate_stats),
            baseline_stats.uncached_input,
            candidate_stats.uncached_input,
            success_display(&baseline_stats),
            success_display(&candidate_stats)
        );
    }
    report.push('\n');

    report.push_str("## Quality And Accounting Checks\n\n");
    push_quality_gate(&mut report, &baseline_all, &candidate_all);
    let _ = writeln!(
        report,
        "- Baseline cache-accounting-unobservable records: {}/{}.",
        baseline_all.cache_accounting_unobservable, baseline_all.records
    );
    let _ = writeln!(
        report,
        "- Candidate cache-accounting-unobservable records: {}/{}.",
        candidate_all.cache_accounting_unobservable, candidate_all.records
    );
    if baseline_all.cache_accounting_unobservable > 0
        || candidate_all.cache_accounting_unobservable > 0
    {
        report.push_str("- Cache-accounting gate: blocked for affected records; do not claim token-cost savings from records where cache accounting is unobservable.\n");
    } else {
        report.push_str("- Cache-accounting gate: all loaded records expose cache accounting.\n");
    }
    let uncached_ratio = ratio(candidate_all.uncached_input, baseline_all.uncached_input);
    if matches!(uncached_ratio, Some(value) if value < 1.0) {
        report.push_str("- Savings gate: candidate all-runs uncached input is lower than baseline; pair this with the quality gate before making the primary claim.\n");
    } else {
        report.push_str("- Savings gate: candidate all-runs uncached input is not lower than baseline, or the baseline denominator is missing; do not make the primary savings claim.\n");
    }
    report.push('\n');

    push_behavioral_diagnostics(&mut report, baseline_records, candidate_records);

    report.push_str("## Interpretation Guardrails\n\n");
    report.push_str("- The main claim requires lower candidate uncached input and no task-success regression.\n");
    report.push_str("- Warm-up calls should be excluded from these JSONL files unless the paper explicitly labels a cold-start analysis.\n");
    report.push_str("- Successful-only numbers are useful for mechanism diagnosis but can hide quality regressions.\n");
    report.push_str("- If a row has `cache_accounting_observable=false`, report prefix stability or latency only as exploratory evidence for that row.\n\n");

    report.push_str("## Limitations To Carry Into Paper\n\n");
    report.push_str(
        "- Dataset size and task diversity are limited to the recorded benchmark suite.\n",
    );
    report.push_str("- The current studied path is one Claude Code harness and one MiMo-compatible model route, not all agents or providers.\n");
    report.push_str("- Quality failures, validation failures, and trace anomalies must remain in the all-runs accounting.\n");
    report.push_str("- The skill layer can support reproducibility, but the reported cache-hit evidence comes from audit/eval logs.\n\n");

    report.push_str("## Regeneration Commands\n\n");
    report.push_str("```bash\n");
    let _ = writeln!(
        report,
        "make-agents-cheaper eval --baseline {} --candidate {}",
        baseline_path.display(),
        candidate_path.display()
    );
    let _ = writeln!(
        report,
        "make-agents-cheaper task-report --baseline {} --candidate {}",
        baseline_path.display(),
        candidate_path.display()
    );
    let _ = writeln!(
        report,
        "make-agents-cheaper analysis-report --baseline {} --candidate {} --output runs/<experiment>/analysis-report.md",
        baseline_path.display(),
        candidate_path.display()
    );
    report.push_str("```\n");

    report
}

fn successful_records(records: &[RunRecord]) -> Vec<RunRecord> {
    records
        .iter()
        .filter(|record| record.validation_passed && record.task_success)
        .cloned()
        .collect()
}

fn push_analysis_row(report: &mut String, label: &str, baseline: &RunStats, candidate: &RunStats) {
    let _ = writeln!(
        report,
        "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
        label,
        baseline.records,
        candidate.records,
        cache_hit_display(baseline),
        cache_hit_display(candidate),
        baseline.uncached_input,
        candidate.uncached_input,
        display_ratio(ratio(candidate.uncached_input, baseline.uncached_input)),
        success_display(baseline),
        success_display(candidate)
    );
}

fn push_quality_gate(report: &mut String, baseline: &RunStats, candidate: &RunStats) {
    match (
        ratio(baseline.task_success, baseline.records),
        ratio(candidate.task_success, candidate.records),
    ) {
        (Some(baseline_rate), Some(candidate_rate))
            if candidate_rate + f64::EPSILON < baseline_rate =>
        {
            report.push_str("- Quality gate: blocked; candidate task-success rate is lower than baseline in all-runs accounting.\n");
        }
        (Some(_), Some(_)) => {
            report.push_str("- Quality gate: no aggregate task-success regression is visible in all-runs accounting; still inspect the per-task table.\n");
        }
        _ => {
            report.push_str(
                "- Quality gate: inconclusive because at least one side has no records.\n",
            );
        }
    }
}

fn push_behavioral_diagnostics(
    report: &mut String,
    baseline_records: &[RunRecord],
    candidate_records: &[RunRecord],
) {
    let mut baseline_by_key: BTreeMap<(String, String, u64), &RunRecord> = BTreeMap::new();
    for record in baseline_records {
        baseline_by_key.insert(
            (
                record.task_id.clone(),
                record.slice.clone(),
                record.repeat_id.unwrap_or(0),
            ),
            record,
        );
    }

    let mut rows = Vec::new();
    for candidate in candidate_records {
        let key = (
            candidate.task_id.clone(),
            candidate.slice.clone(),
            candidate.repeat_id.unwrap_or(0),
        );
        let Some(baseline) = baseline_by_key.get(&key) else {
            continue;
        };
        let baseline_uncached = baseline
            .input_tokens
            .saturating_sub(baseline.cached_input_tokens);
        let candidate_uncached = candidate
            .input_tokens
            .saturating_sub(candidate.cached_input_tokens);
        let mut notes = Vec::new();
        if let (Some(baseline_turns), Some(candidate_turns)) =
            (baseline.turns_to_completion, candidate.turns_to_completion)
        {
            if candidate_turns >= baseline_turns.saturating_mul(2)
                && candidate_turns >= baseline_turns + 3
            {
                notes.push("candidate used many more agent turns");
            }
        }
        if candidate_uncached > baseline_uncached + baseline_uncached / 4 {
            notes.push("candidate uncached input increased");
        }
        if candidate.output_tokens > baseline.output_tokens + baseline.output_tokens / 2 {
            notes.push("candidate output increased");
        }
        if !notes.is_empty() {
            rows.push((
                baseline,
                candidate,
                baseline_uncached,
                candidate_uncached,
                notes,
            ));
        }
    }

    report.push_str("## Behavioral Diagnostics\n\n");
    if rows.is_empty() {
        report.push_str("- No paired behavioral outlier was detected from turns, uncached input, or output-token deltas.\n\n");
        return;
    }

    report.push_str("These rows flag cases where the candidate may have changed agent behavior, not just prefix cache shape.\n\n");
    report.push_str("| Candidate run | Task | Slice | Repeat | Baseline turns | Candidate turns | Baseline uncached | Candidate uncached | Baseline out | Candidate out | Note |\n");
    report.push_str("| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |\n");
    for (baseline, candidate, baseline_uncached, candidate_uncached, notes) in rows {
        let _ = writeln!(
            report,
            "| `{}` | `{}` | `{}` | {} | {} | {} | {} | {} | {} | {} | {} |",
            markdown_cell(&candidate.run_id),
            markdown_cell(&candidate.task_id),
            markdown_cell(&candidate.slice),
            candidate.repeat_id.unwrap_or(0),
            option_u64_display(baseline.turns_to_completion),
            option_u64_display(candidate.turns_to_completion),
            baseline_uncached,
            candidate_uncached,
            baseline.output_tokens,
            candidate.output_tokens,
            notes.join("; ")
        );
    }
    report.push('\n');
}

fn cache_hit_display(stats: &RunStats) -> String {
    display_optional_percent(ratio(stats.cached_input, stats.input))
}

fn success_display(stats: &RunStats) -> String {
    format!("{}/{}", stats.task_success, stats.records)
}

fn option_u64_display(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "n/a".to_string())
}

fn markdown_cell(value: &str) -> String {
    value.replace('|', "\\|")
}

struct TraceImportOptions<'a> {
    input: &'a Path,
    run_id: &'a str,
    task_id: &'a str,
    condition: &'a str,
    slice: Option<&'a str>,
    repeat_id: Option<u64>,
    phase: Option<&'a str>,
    output: Option<&'a Path>,
    artifacts_dir: Option<&'a Path>,
    validation_path: Option<&'a Path>,
    validation_passed: Option<bool>,
    task_success: Option<bool>,
}

struct ClaudeJsonImportOptions<'a> {
    input: &'a Path,
    run_id: &'a str,
    task_id: &'a str,
    condition: &'a str,
    slice: Option<&'a str>,
    repeat_id: Option<u64>,
    phase: Option<&'a str>,
    output: Option<&'a Path>,
    validation_path: Option<&'a Path>,
    validation_passed: Option<bool>,
    task_success: Option<bool>,
}

#[derive(Debug, Clone)]
struct TracePair {
    request: JsonValue,
    response: JsonValue,
}

#[derive(Debug, Default, Clone)]
struct TraceUsage {
    input_tokens: u64,
    cached_input_tokens: u64,
    cache_creation_input_tokens: u64,
    output_tokens: u64,
    actual_cost_usd: f64,
    observable: bool,
}

fn run_trace_import(options: TraceImportOptions<'_>) -> Result<i32, String> {
    let entries = read_jsonl_values(options.input)?;
    let pair = select_trace_pair(&entries).ok_or_else(|| {
        format!(
            "No request/response pair found in {}",
            options.input.display()
        )
    })?;
    let model = pair
        .request
        .get("model")
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown");
    let usage = trace_usage(&pair.response, model);
    let validation_passed = options.validation_passed.unwrap_or(false);
    let task_success = options.task_success.unwrap_or(validation_passed);
    let total_latency_ms = first_u64_by_keys(
        &entries,
        &[
            "duration_ms",
            "durationMs",
            "total_latency_ms",
            "totalLatencyMs",
        ],
    );
    let ttft_ms = first_u64_by_keys(&entries, &["ttft_ms", "ttftMs", "time_to_first_token_ms"]);

    let mut record = serde_json::Map::new();
    record.insert("task_id".to_string(), json!(options.task_id));
    record.insert("run_id".to_string(), json!(options.run_id));
    record.insert("condition".to_string(), json!(options.condition));
    if let Some(slice) = options.slice {
        record.insert("slice".to_string(), json!(slice));
    }
    if let Some(repeat_id) = options.repeat_id {
        record.insert("repeat_id".to_string(), json!(repeat_id));
    }
    if let Some(phase) = options.phase {
        record.insert("phase".to_string(), json!(phase));
    }
    record.insert("agent".to_string(), json!("claude_code"));
    record.insert("model".to_string(), json!(model));
    record.insert("transport".to_string(), json!("claude_trace"));
    record.insert("input_tokens".to_string(), json!(usage.input_tokens));
    record.insert(
        "cached_input_tokens".to_string(),
        json!(usage.cached_input_tokens),
    );
    record.insert(
        "cache_creation_input_tokens".to_string(),
        json!(usage.cache_creation_input_tokens),
    );
    record.insert(
        "uncached_input_tokens".to_string(),
        json!(usage.input_tokens.saturating_sub(usage.cached_input_tokens)),
    );
    record.insert("output_tokens".to_string(), json!(usage.output_tokens));
    if let Some(ttft_ms) = ttft_ms {
        record.insert("ttft_ms".to_string(), json!(ttft_ms));
    }
    if let Some(total_latency_ms) = total_latency_ms {
        record.insert("total_latency_ms".to_string(), json!(total_latency_ms));
    }
    if usage.actual_cost_usd > 0.0 {
        record.insert("actual_cost_usd".to_string(), json!(usage.actual_cost_usd));
    }
    record.insert("validation_passed".to_string(), json!(validation_passed));
    record.insert("task_success".to_string(), json!(task_success));
    record.insert(
        "cache_accounting_observable".to_string(),
        json!(usage.observable),
    );
    record.insert(
        "trace_path".to_string(),
        json!(options.input.display().to_string()),
    );
    if let Some(validation_path) = options.validation_path {
        record.insert(
            "validation_path".to_string(),
            json!(validation_path.display().to_string()),
        );
    }
    record.insert(
        "anomaly".to_string(),
        json!(if usage.observable {
            ""
        } else {
            "usage fields missing from trace"
        }),
    );

    if let Some(artifacts_dir) = options.artifacts_dir {
        write_trace_artifacts(artifacts_dir, options.run_id, &pair, &mut record)?;
    }

    let line = serde_json::to_string(&JsonValue::Object(record))
        .map_err(|err| format!("Could not serialize normalized trace row: {err}"))?;
    if let Some(output) = options.output {
        append_jsonl(output, &line)?;
        println!("Appended normalized trace row to {}", output.display());
    } else {
        println!("{line}");
    }
    Ok(0)
}

fn run_claude_json_import(options: ClaudeJsonImportOptions<'_>) -> Result<i32, String> {
    let response = read_json(options.input)?;
    let model = direct_json_model(&response).unwrap_or("unknown");
    let mut usage = trace_usage(&response, model);
    if usage.actual_cost_usd == 0.0 {
        usage.actual_cost_usd =
            json_f64_any(&response, &["total_cost_usd", "totalCostUsd"]).unwrap_or(0.0);
    }
    let validation_passed = options.validation_passed.unwrap_or(false);
    let task_success = options.task_success.unwrap_or(validation_passed);
    let values = vec![response.clone()];
    let total_latency_ms = first_u64_by_keys(
        &values,
        &[
            "duration_ms",
            "durationMs",
            "total_latency_ms",
            "totalLatencyMs",
        ],
    );
    let model_latency_ms = first_u64_by_keys(&values, &["duration_api_ms", "durationApiMs"]);

    let mut record = serde_json::Map::new();
    record.insert("task_id".to_string(), json!(options.task_id));
    record.insert("run_id".to_string(), json!(options.run_id));
    record.insert("condition".to_string(), json!(options.condition));
    if let Some(slice) = options.slice {
        record.insert("slice".to_string(), json!(slice));
    }
    if let Some(repeat_id) = options.repeat_id {
        record.insert("repeat_id".to_string(), json!(repeat_id));
    }
    if let Some(phase) = options.phase {
        record.insert("phase".to_string(), json!(phase));
    }
    record.insert("agent".to_string(), json!("claude_code"));
    record.insert("model".to_string(), json!(model));
    record.insert("transport".to_string(), json!("claude_code_json"));
    record.insert("input_tokens".to_string(), json!(usage.input_tokens));
    record.insert(
        "cached_input_tokens".to_string(),
        json!(usage.cached_input_tokens),
    );
    record.insert(
        "cache_creation_input_tokens".to_string(),
        json!(usage.cache_creation_input_tokens),
    );
    record.insert(
        "uncached_input_tokens".to_string(),
        json!(usage.input_tokens.saturating_sub(usage.cached_input_tokens)),
    );
    record.insert("output_tokens".to_string(), json!(usage.output_tokens));
    if let Some(total_latency_ms) = total_latency_ms {
        record.insert("total_latency_ms".to_string(), json!(total_latency_ms));
    }
    if let Some(model_latency_ms) = model_latency_ms {
        record.insert("model_latency_ms".to_string(), json!(model_latency_ms));
    }
    if let Some(num_turns) = response.get("num_turns").and_then(JsonValue::as_u64) {
        record.insert("turns_to_completion".to_string(), json!(num_turns));
    }
    if usage.actual_cost_usd > 0.0 {
        record.insert("actual_cost_usd".to_string(), json!(usage.actual_cost_usd));
    }
    record.insert("validation_passed".to_string(), json!(validation_passed));
    record.insert("task_success".to_string(), json!(task_success));
    record.insert(
        "cache_accounting_observable".to_string(),
        json!(usage.observable),
    );
    record.insert("request_shape_observable".to_string(), json!(false));
    record.insert(
        "trace_path".to_string(),
        json!(options.input.display().to_string()),
    );
    record.insert(
        "raw_json_path".to_string(),
        json!(options.input.display().to_string()),
    );
    if let Some(validation_path) = options.validation_path {
        record.insert(
            "validation_path".to_string(),
            json!(validation_path.display().to_string()),
        );
    }
    record.insert(
        "anomaly".to_string(),
        json!(if usage.observable {
            "direct-json fallback: request/layer/tool artifacts unavailable"
        } else {
            "usage fields missing from Claude JSON"
        }),
    );

    let line = serde_json::to_string(&JsonValue::Object(record))
        .map_err(|err| format!("Could not serialize normalized Claude JSON row: {err}"))?;
    if let Some(output) = options.output {
        append_jsonl(output, &line)?;
        println!(
            "Appended normalized Claude JSON row to {}",
            output.display()
        );
    } else {
        println!("{line}");
    }
    Ok(0)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EvidenceCodeChange {
    event_index: usize,
    timestamp: Option<String>,
    tool_name: Option<String>,
    call_id: Option<String>,
    title: String,
    diff: EvidenceDiff,
    before: String,
    after: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EvidenceDiff {
    kind: String,
    files: Vec<String>,
    additions: u64,
    deletions: u64,
    summary: String,
    preview: String,
}

fn run_evidence_diff(input: &Path, output: Option<&Path>) -> Result<i32, String> {
    let events = read_session_events(input)?;
    let code_changes = extract_evidence_diffs(&events);
    let value = evidence_diff_output(input, &code_changes);
    let text = serde_json::to_string_pretty(&value)
        .map_err(|err| format!("Could not serialize evidence diff output: {err}"))?;
    if let Some(output) = output {
        if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
            fs::create_dir_all(parent)
                .map_err(|err| format!("Could not create {}: {err}", parent.display()))?;
        }
        fs::write(output, format!("{text}\n"))
            .map_err(|err| format!("Could not write {}: {err}", output.display()))?;
        println!("Wrote evidence diff: {}", output.display());
    } else {
        println!("{text}");
    }
    Ok(0)
}

fn read_session_events(input: &Path) -> Result<Vec<JsonValue>, String> {
    if let Ok(values) = read_jsonl_values(input) {
        if !values.is_empty() {
            return Ok(values);
        }
    }

    let value = read_json(input)?;
    if let Some(array) = value.as_array() {
        return Ok(array.clone());
    }
    for key in ["events", "session", "replay", "history", "messages"] {
        if let Some(array) = value.get(key).and_then(JsonValue::as_array) {
            return Ok(array.clone());
        }
    }
    Ok(vec![value])
}

fn extract_evidence_diffs(events: &[JsonValue]) -> Vec<EvidenceCodeChange> {
    let mut changes = Vec::new();
    for (event_index, event) in events.iter().enumerate() {
        if !is_toolish_event(event) {
            continue;
        }
        let mut seen = BTreeSet::new();
        for text in diff_candidate_texts(event) {
            let Some(diff) = parse_diff_metadata(&text) else {
                continue;
            };
            if !seen.insert(diff.preview.clone()) {
                continue;
            }
            let (before, after) = diff_before_after(&text);
            let tool_name = event_string(event, &["tool_name", "toolName", "name"]);
            let title = format!("{} evidence diff", tool_name.as_deref().unwrap_or("tool"));
            changes.push(EvidenceCodeChange {
                event_index,
                timestamp: event_string(event, &["timestamp", "time", "created_at", "createdAt"]),
                tool_name,
                call_id: event_string(event, &["call_id", "callId", "tool_call_id", "id"]),
                title,
                diff,
                before: truncate_chars(&before, 6000),
                after: truncate_chars(&after, 6000),
            });
        }
    }
    changes
}

fn evidence_diff_output(input: &Path, changes: &[EvidenceCodeChange]) -> JsonValue {
    let code_changes = changes
        .iter()
        .map(|change| {
            json!({
                "event_index": change.event_index,
                "timestamp": change.timestamp,
                "tool_name": change.tool_name,
                "call_id": change.call_id,
                "title": change.title,
                "summary": change.diff.summary,
                "diff": {
                    "kind": change.diff.kind,
                    "files": change.diff.files,
                    "additions": change.diff.additions,
                    "deletions": change.diff.deletions,
                    "summary": change.diff.summary,
                    "preview": change.diff.preview,
                    "before": change.before,
                    "after": change.after
                }
            })
        })
        .collect::<Vec<_>>();
    json!({
        "evidence_scope": "session_tool_record",
        "source": input.display().to_string(),
        "limitations": [
            "This evidence diff is reconstructed from session/tool records.",
            "It answers what a tool call claimed to change at that time.",
            "It is not the current filesystem diff against git baseline."
        ],
        "code_changes": code_changes
    })
}

fn is_toolish_event(event: &JsonValue) -> bool {
    if let Some(kind) = event_string(event, &["type", "event", "event_type", "eventType"]) {
        let kind = kind.to_ascii_lowercase();
        if kind.contains("tool") || kind.contains("function_call") || kind.contains("function") {
            return true;
        }
    }
    event.get("tool_name").is_some()
        || event.get("toolName").is_some()
        || event.get("tool_call").is_some()
        || event.get("tool_call_id").is_some()
        || event.get("function_call").is_some()
        || event.get("function_call_output").is_some()
}

fn diff_candidate_texts(event: &JsonValue) -> Vec<String> {
    let mut texts = Vec::new();
    collect_diff_candidate_texts(event, false, &mut texts);
    texts
}

fn collect_diff_candidate_texts(
    value: &JsonValue,
    in_interesting_field: bool,
    texts: &mut Vec<String>,
) {
    match value {
        JsonValue::String(text) => {
            if in_interesting_field {
                texts.push(text.clone());
            }
        }
        JsonValue::Array(array) => {
            for item in array {
                collect_diff_candidate_texts(item, in_interesting_field, texts);
            }
        }
        JsonValue::Object(object) => {
            for (key, child) in object {
                let interesting = in_interesting_field || is_diff_evidence_field(key);
                collect_diff_candidate_texts(child, interesting, texts);
            }
        }
        _ => {}
    }
}

fn is_diff_evidence_field(key: &str) -> bool {
    matches!(
        key,
        "input"
            | "arguments"
            | "args"
            | "output"
            | "result"
            | "message"
            | "text"
            | "patch"
            | "diff"
    )
}

fn parse_diff_metadata(text: &str) -> Option<EvidenceDiff> {
    let kind = if is_apply_patch(text) {
        "apply_patch"
    } else if is_unified_diff(text) {
        "unified_diff"
    } else {
        return None;
    };
    let mut files = extract_diff_files(text, kind);
    files.sort();
    files.dedup();
    let (additions, deletions) = count_diff_changes(text);
    let summary = format!(
        "{kind}: {} files +{} -{}",
        files.len(),
        additions,
        deletions
    );
    Some(EvidenceDiff {
        kind: kind.to_string(),
        files,
        additions,
        deletions,
        summary,
        preview: truncate_chars(text.trim(), 12_000),
    })
}

fn is_apply_patch(text: &str) -> bool {
    text.contains("*** Begin Patch") && text.contains("*** End Patch")
}

fn is_unified_diff(text: &str) -> bool {
    text.contains("diff --git ")
        || (text.lines().any(|line| line.starts_with("--- "))
            && text.lines().any(|line| line.starts_with("+++ "))
            && text.lines().any(|line| line.starts_with("@@")))
}

fn extract_diff_files(text: &str, kind: &str) -> Vec<String> {
    let mut files = Vec::new();
    for line in text.lines() {
        if kind == "apply_patch" {
            for prefix in ["*** Add File: ", "*** Update File: ", "*** Delete File: "] {
                if let Some(path) = line.strip_prefix(prefix) {
                    files.push(path.trim().to_string());
                }
            }
        }
        if let Some(rest) = line.strip_prefix("diff --git ") {
            let parts = rest.split_whitespace().collect::<Vec<_>>();
            if let Some(path) = parts.get(1).or_else(|| parts.first()) {
                files.push(strip_diff_path_prefix(path).to_string());
            }
        } else if let Some(path) = line.strip_prefix("--- ") {
            let path = strip_diff_path_prefix(path.trim());
            if path != "/dev/null" {
                files.push(path.to_string());
            }
        } else if let Some(path) = line.strip_prefix("+++ ") {
            let path = strip_diff_path_prefix(path.trim());
            if path != "/dev/null" {
                files.push(path.to_string());
            }
        }
    }
    files
}

fn strip_diff_path_prefix(path: &str) -> &str {
    path.strip_prefix("a/")
        .or_else(|| path.strip_prefix("b/"))
        .unwrap_or(path)
}

fn count_diff_changes(text: &str) -> (u64, u64) {
    let mut additions = 0;
    let mut deletions = 0;
    for line in text.lines() {
        if is_diff_count_header(line) {
            continue;
        }
        if line.starts_with('+') {
            additions += 1;
        } else if line.starts_with('-') {
            deletions += 1;
        }
    }
    (additions, deletions)
}

fn is_diff_count_header(line: &str) -> bool {
    line.starts_with("+++ ")
        || line.starts_with("--- ")
        || line.starts_with("diff --git ")
        || line.starts_with("index ")
        || line.starts_with("@@")
        || line.starts_with("*** ")
}

fn diff_before_after(text: &str) -> (String, String) {
    let mut before = Vec::new();
    let mut after = Vec::new();
    for line in text.lines() {
        if should_skip_excerpt_line(line) {
            continue;
        }
        if let Some(path) = apply_patch_file_marker(line) {
            let marker = format!("# {path}");
            before.push(marker.clone());
            after.push(marker);
        } else if line.starts_with("@@") {
            before.push(line.to_string());
            after.push(line.to_string());
        } else if let Some(content) = line.strip_prefix('+') {
            after.push(content.to_string());
        } else if let Some(content) = line.strip_prefix('-') {
            before.push(content.to_string());
        } else if let Some(content) = line.strip_prefix(' ') {
            before.push(content.to_string());
            after.push(content.to_string());
        } else {
            before.push(line.to_string());
            after.push(line.to_string());
        }
    }
    (before.join("\n"), after.join("\n"))
}

fn should_skip_excerpt_line(line: &str) -> bool {
    line.starts_with("diff --git ")
        || line.starts_with("index ")
        || line.starts_with("--- ")
        || line.starts_with("+++ ")
        || line == "*** Begin Patch"
        || line == "*** End Patch"
}

fn apply_patch_file_marker(line: &str) -> Option<&str> {
    for prefix in ["*** Add File: ", "*** Update File: ", "*** Delete File: "] {
        if let Some(path) = line.strip_prefix(prefix) {
            return Some(path.trim());
        }
    }
    None
}

fn event_string(value: &JsonValue, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(text) = find_key_recursive(value, key).and_then(JsonValue::as_str) {
            return Some(text.to_string());
        }
    }
    None
}

fn direct_json_model(value: &JsonValue) -> Option<&str> {
    find_key_recursive(value, "model")
        .and_then(JsonValue::as_str)
        .or_else(|| {
            find_key_recursive(value, "modelUsage")
                .and_then(JsonValue::as_object)
                .and_then(|object| object.keys().next().map(String::as_str))
        })
}

fn read_jsonl_values(path: &Path) -> Result<Vec<JsonValue>, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("Could not read {}: {err}", path.display()))?;
    let mut values = Vec::new();
    for (index, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let value: JsonValue = serde_json::from_str(line).map_err(|err| {
            format!(
                "Could not parse {} line {}: {err}",
                path.display(),
                index + 1
            )
        })?;
        values.push(value);
    }
    Ok(values)
}

fn select_trace_pair(entries: &[JsonValue]) -> Option<TracePair> {
    let mut selected = None;
    for entry in entries {
        if !looks_like_model_request(entry) {
            continue;
        }
        let request = trace_request_body(entry).unwrap_or_else(|| JsonValue::Null);
        let response = trace_response_body(entry).unwrap_or_else(|| JsonValue::Null);
        if !request.is_null() || !response.is_null() {
            selected = Some(TracePair { request, response });
        }
    }
    selected
}

fn looks_like_model_request(entry: &JsonValue) -> bool {
    trace_url(entry)
        .map(|url| {
            url.contains("/v1/messages")
                || url.contains("/anthropic")
                || url.contains("/chat/completions")
        })
        .unwrap_or(true)
}

fn trace_url(entry: &JsonValue) -> Option<&str> {
    get_path(entry, &["request", "url"])
        .or_else(|| get_path(entry, &["url"]))
        .or_else(|| get_path(entry, &["requestUrl"]))
        .and_then(JsonValue::as_str)
}

fn trace_request_body(entry: &JsonValue) -> Option<JsonValue> {
    first_body_candidate(
        entry,
        &[
            &["request", "body_json"],
            &["request", "body"],
            &["request", "bodyRaw"],
            &["request", "body_raw"],
            &["request_body"],
            &["requestBody"],
            &["body"],
        ],
    )
}

fn trace_response_body(entry: &JsonValue) -> Option<JsonValue> {
    first_body_candidate(
        entry,
        &[
            &["response", "body_json"],
            &["response", "body"],
            &["response", "bodyRaw"],
            &["response", "body_raw"],
            &["response_body"],
            &["responseBody"],
            &["body"],
            &["body_raw"],
        ],
    )
}

fn first_body_candidate(entry: &JsonValue, paths: &[&[&str]]) -> Option<JsonValue> {
    for path in paths {
        if let Some(value) = get_path(entry, path).and_then(parse_body_value) {
            return Some(value);
        }
    }
    None
}

fn parse_body_value(value: &JsonValue) -> Option<JsonValue> {
    match value {
        JsonValue::Object(_) | JsonValue::Array(_) => Some(value.clone()),
        JsonValue::String(text) => parse_body_text(text),
        _ => None,
    }
}

fn parse_body_text(text: &str) -> Option<JsonValue> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(value) = serde_json::from_str::<JsonValue>(trimmed) {
        return Some(value);
    }
    parse_sse_body(trimmed)
}

fn parse_sse_body(text: &str) -> Option<JsonValue> {
    let mut last_json = None;
    for line in text.lines() {
        let line = line.trim();
        let Some(data) = line.strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();
        if data.is_empty() || data == "[DONE]" {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<JsonValue>(data) {
            if value.get("usage").is_some() {
                return Some(value);
            }
            last_json = Some(value);
        }
    }
    last_json
}

fn get_path<'a>(value: &'a JsonValue, path: &[&str]) -> Option<&'a JsonValue> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}

fn trace_usage(response: &JsonValue, model: &str) -> TraceUsage {
    if let Some(usage) = model_usage(response, model) {
        return usage;
    }
    if let Some(usage_value) = find_key_recursive(response, "usage") {
        if let Some(usage) = usage_from_usage_object(usage_value) {
            return usage;
        }
    }
    TraceUsage::default()
}

fn model_usage(response: &JsonValue, model: &str) -> Option<TraceUsage> {
    let model_usage = find_key_recursive(response, "modelUsage")?.as_object()?;
    let usage = model_usage
        .get(model)
        .or_else(|| model_usage.values().next())?;
    let input_tokens = json_u64_any(usage, &["inputTokens", "input_tokens"])?;
    let cached_input_tokens =
        json_u64_any(usage, &["cacheReadInputTokens", "cache_read_input_tokens"]).unwrap_or(0);
    let cache_creation_input_tokens = json_u64_any(
        usage,
        &["cacheCreationInputTokens", "cache_creation_input_tokens"],
    )
    .unwrap_or(0);
    Some(TraceUsage {
        input_tokens: input_tokens + cached_input_tokens + cache_creation_input_tokens,
        cached_input_tokens,
        cache_creation_input_tokens,
        output_tokens: json_u64_any(usage, &["outputTokens", "output_tokens"]).unwrap_or(0),
        actual_cost_usd: json_f64_any(usage, &["costUSD", "cost_usd"]).unwrap_or(0.0),
        observable: true,
    })
}

fn usage_from_usage_object(usage: &JsonValue) -> Option<TraceUsage> {
    if let Some(prompt_tokens) = json_u64_any(usage, &["prompt_tokens", "promptTokens"]) {
        let cached_input_tokens = usage
            .get("prompt_tokens_details")
            .or_else(|| usage.get("promptTokensDetails"))
            .and_then(|details| json_u64_any(details, &["cached_tokens", "cachedTokens"]))
            .unwrap_or(0);
        return Some(TraceUsage {
            input_tokens: prompt_tokens,
            cached_input_tokens,
            cache_creation_input_tokens: 0,
            output_tokens: json_u64_any(usage, &["completion_tokens", "completionTokens"])
                .unwrap_or(0),
            actual_cost_usd: 0.0,
            observable: true,
        });
    }

    let base_input = json_u64_any(usage, &["input_tokens", "inputTokens"])?;
    let cached_input_tokens =
        json_u64_any(usage, &["cache_read_input_tokens", "cacheReadInputTokens"]).unwrap_or(0);
    let cache_creation_input_tokens = json_u64_any(
        usage,
        &["cache_creation_input_tokens", "cacheCreationInputTokens"],
    )
    .unwrap_or(0);
    Some(TraceUsage {
        input_tokens: base_input + cached_input_tokens + cache_creation_input_tokens,
        cached_input_tokens,
        cache_creation_input_tokens,
        output_tokens: json_u64_any(usage, &["output_tokens", "outputTokens"]).unwrap_or(0),
        actual_cost_usd: json_f64_any(usage, &["cost_usd", "costUSD"]).unwrap_or(0.0),
        observable: true,
    })
}

fn find_key_recursive<'a>(value: &'a JsonValue, key: &str) -> Option<&'a JsonValue> {
    match value {
        JsonValue::Object(object) => {
            if let Some(found) = object.get(key) {
                return Some(found);
            }
            object
                .values()
                .find_map(|value| find_key_recursive(value, key))
        }
        JsonValue::Array(array) => array
            .iter()
            .find_map(|value| find_key_recursive(value, key)),
        _ => None,
    }
}

fn json_u64_any(value: &JsonValue, keys: &[&str]) -> Option<u64> {
    keys.iter().find_map(|key| json_u64(value, key))
}

fn json_f64_any(value: &JsonValue, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|key| json_f64(value, key))
}

fn first_u64_by_keys(values: &[JsonValue], keys: &[&str]) -> Option<u64> {
    for value in values.iter().rev() {
        for key in keys {
            if let Some(found) = find_key_recursive(value, key).and_then(JsonValue::as_u64) {
                return Some(found);
            }
        }
    }
    None
}

fn write_trace_artifacts(
    artifacts_dir: &Path,
    run_id: &str,
    pair: &TracePair,
    record: &mut serde_json::Map<String, JsonValue>,
) -> Result<(), String> {
    let request_path = artifacts_dir
        .join("requests")
        .join(format!("{run_id}.request.json"));
    let trace_path = artifacts_dir
        .join("traces")
        .join(format!("{run_id}.response.json"));
    let layers_path = artifacts_dir
        .join("layers")
        .join(format!("{run_id}.layers.json"));
    let tools_path = artifacts_dir
        .join("tools")
        .join(format!("{run_id}.tools.json"));

    write_json_pretty(&request_path, &pair.request)?;
    write_json_pretty(&trace_path, &pair.response)?;

    let layers = json!({
        "model": pair.request.get("model").cloned().unwrap_or(JsonValue::Null),
        "system": pair.request.get("system").cloned().unwrap_or(JsonValue::Null),
        "messages": pair.request.get("messages").cloned().unwrap_or(JsonValue::Null)
    });
    write_json_pretty(&layers_path, &layers)?;

    let tools = pair
        .request
        .get("tools")
        .cloned()
        .unwrap_or_else(|| JsonValue::Array(Vec::new()));
    write_json_pretty(&tools_path, &tools)?;

    record.insert(
        "request_path".to_string(),
        json!(request_path.display().to_string()),
    );
    record.insert(
        "layers_path".to_string(),
        json!(layers_path.display().to_string()),
    );
    record.insert(
        "tools_path".to_string(),
        json!(tools_path.display().to_string()),
    );
    record.insert(
        "normalized_trace_path".to_string(),
        json!(trace_path.display().to_string()),
    );
    Ok(())
}

fn write_json_pretty(path: &Path, value: &JsonValue) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Could not create {}: {err}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(value)
        .map_err(|err| format!("Could not serialize {}: {err}", path.display()))?;
    fs::write(path, format!("{text}\n"))
        .map_err(|err| format!("Could not write {}: {err}", path.display()))
}

fn append_jsonl(path: &Path, line: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Could not create {}: {err}", parent.display()))?;
    }
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| format!("Could not open {}: {err}", path.display()))?;
    writeln!(file, "{line}").map_err(|err| format!("Could not append {}: {err}", path.display()))
}

fn run_matrix_plan(
    manifest_path: &Path,
    experiment_dir: &Path,
    task_filter: Option<&str>,
    repeats: u64,
) -> Result<i32, String> {
    if repeats == 0 {
        return Err("--repeats must be at least 1".to_string());
    }

    let manifest = read_json(manifest_path)?;
    let tasks = selected_manifest_task_ids(&manifest, task_filter)?;
    let conditions = pilot_conditions(&manifest)?;
    let slices = pilot_slices(&manifest, None)?;
    let minimum_repeats = manifest
        .get("minimum_measured_repeats")
        .and_then(JsonValue::as_u64)
        .unwrap_or(3);
    let measured_runs =
        tasks.len() as u64 * conditions.len() as u64 * slices.len() as u64 * repeats;
    let warm_up_runs = measured_runs;

    println!("Full matrix command plan");
    println!("Manifest: {}", manifest_path.display());
    println!("Experiment dir: {}", experiment_dir.display());
    println!("Tasks: {}", tasks.len());
    println!("Conditions: {}", conditions.len());
    println!("Slices: {}", slices.len());
    println!("Repeats per task/condition/slice: {repeats}");
    println!("Warm-up calls: {warm_up_runs}");
    println!("Measured calls: {measured_runs}");
    println!("Validation logs expected: {measured_runs}");
    if repeats < minimum_repeats {
        println!(
            "Warning: manifest minimum measured repeats is {minimum_repeats}; this plan is a smaller pilot."
        );
    }
    println!();
    println!("Selected tasks:");
    for task in &tasks {
        println!("  - {task}");
    }
    println!();
    println!("Setup:");
    println!(
        "cargo run --quiet -- init-experiment --dir {}",
        shell_word(&experiment_dir.display().to_string())
    );
    println!();
    println!("Generate task/slice command plans:");
    for task in &tasks {
        for slice in &slices {
            println!(
                "cargo run --quiet -- pilot-plan --manifest {} --task {} --experiment-dir {} --slice {} --repeats {} > {}",
                shell_word(&manifest_path.display().to_string()),
                shell_word(task),
                shell_word(&experiment_dir.display().to_string()),
                shell_word(&slice.id),
                repeats,
                shell_word(&format!(
                    "{}/notes/plan-{}-{}.sh",
                    experiment_dir.display(),
                    task,
                    slice.id
                ))
            );
        }
    }
    println!();
    println!("After all measured runs:");
    let baseline_path = experiment_dir.join("baseline.jsonl");
    let candidate_path = experiment_dir.join("cache-friendly.jsonl");
    println!(
        "cargo run --quiet -- eval --baseline {} --candidate {}",
        shell_word(&baseline_path.display().to_string()),
        shell_word(&candidate_path.display().to_string())
    );
    println!(
        "cargo run --quiet -- task-report --baseline {} --candidate {}",
        shell_word(&baseline_path.display().to_string()),
        shell_word(&candidate_path.display().to_string())
    );
    println!(
        "cargo run --quiet -- analysis-report --baseline {} --candidate {} --output {}",
        shell_word(&baseline_path.display().to_string()),
        shell_word(&candidate_path.display().to_string()),
        shell_word(
            &experiment_dir
                .join("analysis-report.md")
                .display()
                .to_string()
        )
    );
    Ok(0)
}

fn selected_manifest_task_ids(
    manifest: &JsonValue,
    task_filter: Option<&str>,
) -> Result<Vec<String>, String> {
    let all = manifest_task_ids(manifest)?;
    let Some(filter) = task_filter else {
        return Ok(all);
    };
    let available: BTreeSet<_> = all.iter().cloned().collect();
    let mut selected = Vec::new();
    for task in filter
        .split(',')
        .map(str::trim)
        .filter(|task| !task.is_empty())
    {
        if !available.contains(task) {
            return Err(format!("task {task} was not found in manifest"));
        }
        selected.push(task.to_string());
    }
    selected.sort();
    selected.dedup();
    if selected.is_empty() {
        return Err("--tasks did not name any tasks".to_string());
    }
    Ok(selected)
}

fn manifest_task_ids(manifest: &JsonValue) -> Result<Vec<String>, String> {
    let tasks = manifest
        .get("tasks")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| "manifest is missing a tasks array".to_string())?;
    let mut ids = Vec::new();
    for task in tasks {
        let id = task
            .get("id")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| "task is missing id".to_string())?;
        ids.push(id.to_string());
    }
    if ids.is_empty() {
        return Err("manifest has no tasks".to_string());
    }
    Ok(ids)
}

fn run_pilot_plan(
    manifest_path: &Path,
    task_id: &str,
    experiment_dir: &Path,
    slice_filter: Option<&str>,
    repeats: u64,
) -> Result<i32, String> {
    if repeats == 0 {
        return Err("--repeats must be at least 1".to_string());
    }

    let manifest = read_json(manifest_path)?;
    let fixture_path = manifest
        .get("fixture")
        .and_then(|fixture| fixture.get("path"))
        .and_then(JsonValue::as_str)
        .unwrap_or("runs/fixtures/real-coding-v2");
    let model = pilot_model(&manifest);
    let minimum_repeats = manifest
        .get("minimum_measured_repeats")
        .and_then(JsonValue::as_u64)
        .unwrap_or(3);
    let task = pilot_task(&manifest, task_id)?;
    let conditions = pilot_conditions(&manifest)?;
    let slices = pilot_slices(&manifest, slice_filter)?;

    println!("Pilot run command plan");
    println!("Manifest: {}", manifest_path.display());
    println!("Experiment dir: {}", experiment_dir.display());
    println!("Fixture: {fixture_path}");
    println!("Task: {task_id}");
    println!("Validation: {}", task.validation);
    println!("Model: {model}");
    println!("Pilot repeats: {repeats}");
    println!("Full-matrix minimum measured repeats: {minimum_repeats}");
    println!();
    println!("Prerequisites:");
    println!("  - claude is authenticated and points at the fixed provider/route.");
    println!("  - The current plan uses direct Claude JSON capture; claude-trace is not required.");
    println!("  - The printed commands use --permission-mode bypassPermissions inside the ignored fixture.");
    println!("  - Run this from the repository root.");
    println!();
    println!("Setup:");
    println!(
        "cargo run --quiet -- init-experiment --dir {}",
        shell_word(&experiment_dir.display().to_string())
    );
    println!("REPO_ROOT=\"$(pwd)\"");
    if experiment_dir.is_absolute() {
        println!(
            "EXP_DIR={}",
            shell_word(&experiment_dir.display().to_string())
        );
    } else {
        println!("EXP_DIR=\"$REPO_ROOT/{}\"", experiment_dir.display());
    }
    if Path::new(fixture_path).is_absolute() {
        println!("FIXTURE={}", shell_word(fixture_path));
    } else {
        println!("FIXTURE=\"$REPO_ROOT/{fixture_path}\"");
    }
    println!("mkdir -p \"$EXP_DIR/prompts\" \"$EXP_DIR/drift\" \"$EXP_DIR/raw/claude-json\" \"$EXP_DIR/validation\"");
    for (index, prompt) in task.prompt_turns.iter().enumerate() {
        let turn = index + 1;
        println!("cat > \"$EXP_DIR/prompts/{task_id}-turn{turn}.txt\" <<'PROMPT_EOF'");
        println!("{prompt}");
        println!("PROMPT_EOF");
    }

    println!();
    println!("Run order:");
    for slice in slices {
        println!();
        println!("# Slice: {}", slice.id);
        for repeat in 1..=repeats {
            println!("# Repeat: {repeat}");
            for condition in &conditions {
                print_pilot_call(&task, &slice, condition, repeat, "warm-up", &model, false);
            }
            for condition in &conditions {
                print_pilot_call(&task, &slice, condition, repeat, "measured", &model, true);
            }
        }
    }

    println!();
    println!("After measured runs:");
    println!("  - Normalize each measured run with claude-json-import, for example:");
    println!("    cargo run --quiet -- claude-json-import \\");
    println!("      --input \"$EXP_DIR/raw/claude-json/<run_id>.json\" \\");
    println!("      --run-id <run_id> \\");
    println!("      --task-id {} \\", task.id);
    println!("      --condition <baseline|cache-friendly> \\");
    println!("      --slice <control-steady|dynamic-drift> \\");
    println!("      --repeat-id <n> \\");
    println!("      --phase measured \\");
    println!("      --output \"$EXP_DIR/<baseline|cache-friendly>.jsonl\" \\");
    println!("      --validation-path \"$EXP_DIR/validation/<run_id>.txt\" \\");
    println!("      --validation-passed <true|false> \\");
    println!("      --task-success <true|false>");
    println!("  - Run:");
    println!("    cargo run --quiet -- eval --baseline \"$EXP_DIR/baseline.jsonl\" --candidate \"$EXP_DIR/cache-friendly.jsonl\"");
    println!("    cargo run --quiet -- task-report --baseline \"$EXP_DIR/baseline.jsonl\" --candidate \"$EXP_DIR/cache-friendly.jsonl\"");
    println!("    cargo run --quiet -- analysis-report --baseline \"$EXP_DIR/baseline.jsonl\" --candidate \"$EXP_DIR/cache-friendly.jsonl\" --output \"$EXP_DIR/analysis-report.md\"");
    Ok(0)
}

fn run_pilot(
    manifest_path: &Path,
    task_id: &str,
    experiment_dir: &Path,
    slice_filter: Option<&str>,
    repeats: u64,
    execute: bool,
) -> Result<i32, String> {
    if repeats == 0 {
        return Err("--repeats must be at least 1".to_string());
    }

    let manifest = read_json(manifest_path)?;
    let fixture_path = manifest
        .get("fixture")
        .and_then(|fixture| fixture.get("path"))
        .and_then(JsonValue::as_str)
        .unwrap_or("runs/fixtures/real-coding-v2");
    let model = pilot_model(&manifest);
    let task = pilot_task(&manifest, task_id)?;
    let conditions = pilot_conditions(&manifest)?;
    let slices = pilot_slices(&manifest, slice_filter)?;

    fs::create_dir_all(experiment_dir.join("notes")).map_err(|err| {
        format!(
            "Could not create {}: {err}",
            experiment_dir.join("notes").display()
        )
    })?;
    let script_path = experiment_dir.join("notes").join("run-pilot.sh");
    let script = pilot_shell_script(
        manifest_path,
        experiment_dir,
        fixture_path,
        &model,
        &task,
        &conditions,
        &slices,
        repeats,
    );
    fs::write(&script_path, script)
        .map_err(|err| format!("Could not write {}: {err}", script_path.display()))?;

    println!("Pilot runner script written");
    println!("Script: {}", script_path.display());
    println!("Task: {}", task.id);
    println!("Repeats: {repeats}");
    println!(
        "Slices: {}",
        slices
            .iter()
            .map(|slice| slice.id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!();
    println!("Default behavior is safe: this command only writes the script.");
    println!("Run manually with:");
    println!("  bash {}", shell_word(&script_path.display().to_string()));
    println!();
    println!(
        "To execute from the CLI, pass --execute true. This may call Claude and incur model cost."
    );

    if !Path::new(fixture_path).exists() {
        println!();
        println!(
            "[WARN] Fixture path does not exist locally: {}",
            fixture_path
        );
        println!("       The script is still useful as a reproducible runner, but execution will fail until the fixture exists.");
    }

    if !execute {
        return Ok(0);
    }

    println!();
    println!("Executing pilot script...");
    let status = ProcessCommand::new("bash")
        .arg(&script_path)
        .status()
        .map_err(|err| format!("Could not execute {}: {err}", script_path.display()))?;
    Ok(status.code().unwrap_or(1))
}

fn pilot_shell_script(
    manifest_path: &Path,
    experiment_dir: &Path,
    fixture_path: &str,
    model: &str,
    task: &PilotTask,
    conditions: &[PilotCondition],
    slices: &[PilotSlice],
    repeats: u64,
) -> String {
    let mut script = String::new();
    let repo_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    script.push_str("#!/usr/bin/env bash\n");
    script.push_str("set -euo pipefail\n\n");
    let _ = writeln!(
        script,
        "REPO_ROOT=\"${{REPO_ROOT:-{}}}\"",
        shell_word(&repo_root.display().to_string())
    );
    if experiment_dir.is_absolute() {
        let _ = writeln!(
            script,
            "EXP_DIR={}",
            shell_word(&experiment_dir.display().to_string())
        );
    } else {
        let _ = writeln!(
            script,
            "EXP_DIR=\"$REPO_ROOT/{}\"",
            experiment_dir.display()
        );
    }
    if Path::new(fixture_path).is_absolute() {
        let _ = writeln!(script, "FIXTURE={}", shell_word(fixture_path));
    } else {
        let _ = writeln!(script, "FIXTURE=\"$REPO_ROOT/{fixture_path}\"");
    }
    script.push_str(
        "export GIT_CEILING_DIRECTORIES=\"${GIT_CEILING_DIRECTORIES:-$REPO_ROOT/runs}\"\n",
    );
    let _ = writeln!(
        script,
        "MANIFEST={}",
        shell_word(&manifest_path.display().to_string())
    );
    let _ = writeln!(script, "TASK_ID={}", shell_word(&task.id));
    let _ = writeln!(script, "MODEL={}", shell_word(model));
    script.push('\n');
    script.push_str("if [[ ! -d \"$FIXTURE\" ]]; then\n");
    script.push_str("  echo \"Fixture directory is missing: $FIXTURE\" >&2\n");
    script.push_str("  exit 2\n");
    script.push_str("fi\n\n");
    script.push_str("if [[ ! -f \"$EXP_DIR/manifest.json\" ]]; then\n");
    script.push_str("  cargo run --quiet -- init-experiment --dir \"$EXP_DIR\"\n");
    script.push_str("fi\n");
    script.push_str("mkdir -p \"$EXP_DIR/prompts\" \"$EXP_DIR/drift\" \"$EXP_DIR/raw/claude-json\" \"$EXP_DIR/validation\" \"$EXP_DIR/notes\"\n");
    script.push_str("for existing in \"$EXP_DIR/baseline.jsonl\" \"$EXP_DIR/cache-friendly.jsonl\" \"$EXP_DIR/notes/run-log.tsv\"; do\n");
    script.push_str("  if [[ -s \"$existing\" ]]; then\n");
    script.push_str(
        "    echo \"Refusing to overwrite non-empty experiment artifact: $existing\" >&2\n",
    );
    script.push_str("    exit 2\n");
    script.push_str("  fi\n");
    script.push_str("done\n");
    script.push_str("printf 'run_id\\tclaude_status\\tvalidation_status\\tcondition\\tslice\\trepeat\\tphase\\n' > \"$EXP_DIR/notes/run-log.tsv\"\n\n");

    for (index, prompt) in task.prompt_turns.iter().enumerate() {
        let turn = index + 1;
        let _ = writeln!(
            script,
            "cat > \"$EXP_DIR/prompts/{}-turn{turn}.txt\" <<'PROMPT_EOF'",
            task.id
        );
        script.push_str(prompt);
        if !prompt.ends_with('\n') {
            script.push('\n');
        }
        script.push_str("PROMPT_EOF\n\n");
    }

    script.push_str("ensure_fixture_git() {\n");
    script.push_str("  local top\n");
    script.push_str(
        "  top=\"$(cd \"$FIXTURE\" && git rev-parse --show-toplevel 2>/dev/null || true)\"\n",
    );
    script.push_str("  if [[ \"$top\" != \"$FIXTURE\" ]]; then\n");
    script.push_str("    (cd \"$FIXTURE\" && git init -q)\n");
    script.push_str("    (cd \"$FIXTURE\" && git add Cargo.toml Cargo.lock README.md src tests examples task-reset.sh task-validate.sh)\n");
    script.push_str("    (cd \"$FIXTURE\" && git -c user.name='make-agents-cheaper' -c user.email='make-agents-cheaper@example.invalid' commit -q -m 'fixture baseline' >/dev/null 2>&1 || true)\n");
    script.push_str("  fi\n");
    script.push_str("  local exclude=\"$FIXTURE/.git/info/exclude\"\n");
    script.push_str("  for pattern in '.baseline/' '.claude-trace/' 'target/'; do\n");
    script.push_str("    grep -qxF \"$pattern\" \"$exclude\" 2>/dev/null || printf '%s\\n' \"$pattern\" >> \"$exclude\"\n");
    script.push_str("  done\n");
    script.push_str("}\n\n");
    script.push_str("ensure_fixture_git\n\n");

    script.push_str("reset_fixture() {\n");
    script.push_str("  (cd \"$FIXTURE\" && bash task-reset.sh \"$TASK_ID\")\n");
    script.push_str(
        "  rm -f \"$FIXTURE/.cache-drift-probe\" \"$FIXTURE/.local-git-status-snapshot\"\n",
    );
    script.push_str("  rm -rf \"$FIXTURE/.local-memory\"\n");
    script.push_str("}\n\n");

    script.push_str("condition_flags() {\n");
    script.push_str("  case \"$1\" in\n");
    for condition in conditions {
        let flags = condition
            .flags
            .iter()
            .map(|flag| shell_word(flag))
            .collect::<Vec<_>>()
            .join(" ");
        let _ = writeln!(
            script,
            "    {}) printf '%s\\n' {} ;;",
            condition.id,
            shell_word(&flags)
        );
    }
    script.push_str("    *) echo \"unknown condition: $1\" >&2; return 2 ;;\n");
    script.push_str("  esac\n");
    script.push_str("}\n\n");

    script.push_str("apply_drift() {\n");
    script.push_str("  case \"$1\" in\n");
    for slice in slices {
        let _ = writeln!(script, "    {})", slice.id);
        if slice.drift_actions.is_empty() {
            script.push_str("      ;;\n");
        } else {
            for action in &slice.drift_actions {
                let _ = writeln!(script, "      {action}");
            }
            script.push_str("      ;;\n");
        }
    }
    script.push_str("    *) echo \"unknown slice: $1\" >&2; return 2 ;;\n");
    script.push_str("  esac\n");
    script.push_str("}\n\n");

    script.push_str("run_one() {\n");
    script.push_str("  local condition=\"$1\"\n");
    script.push_str("  local slice=\"$2\"\n");
    script.push_str("  local repeat=\"$3\"\n");
    script.push_str("  local phase=\"$4\"\n");
    script.push_str("  local run_id=\"${TASK_ID}-${slice}-${condition}-r${repeat}-${phase}\"\n");
    script.push_str("  local validation_status=\"NA\"\n");
    script.push_str("  local prompt_file=\"$EXP_DIR/prompts/${TASK_ID}-turn1.txt\"\n");
    script.push_str("  local flag_string\n");
    script.push_str("  if [[ ! -s \"$prompt_file\" ]]; then\n");
    script.push_str("    echo \"Prompt file is missing or empty: $prompt_file\" >&2\n");
    script.push_str("    return 2\n");
    script.push_str("  fi\n");
    script.push_str("  flag_string=\"$(condition_flags \"$condition\")\"\n");
    script.push_str("  local extra=()\n");
    script.push_str("  if [[ -n \"$flag_string\" ]]; then\n");
    script.push_str("    read -r -a extra <<< \"$flag_string\"\n");
    script.push_str("  fi\n");
    script.push_str("  reset_fixture\n");
    script.push_str("  if [[ \"$phase\" == \"measured\" ]]; then\n");
    script.push_str("    (cd \"$FIXTURE\" && apply_drift \"$slice\")\n");
    script.push_str("    (cd \"$FIXTURE\" && git status --short > \"$EXP_DIR/drift/${run_id}.git-status.txt\") || true\n");
    script.push_str("  fi\n");
    script.push_str("  echo \"BEGIN $run_id\"\n");
    script.push_str("  set +e\n");
    script.push_str("  (cd \"$FIXTURE\" && claude -p --model \"$MODEL\" --output-format json --no-session-persistence --permission-mode bypassPermissions \"${extra[@]}\" \"$(cat \"$prompt_file\")\") > \"$EXP_DIR/raw/claude-json/${run_id}.json\" 2> \"$EXP_DIR/raw/claude-json/${run_id}.stderr.txt\"\n");
    script.push_str("  local claude_status=$?\n");
    script.push_str("  if [[ \"$phase\" == \"measured\" && \"$claude_status\" -eq 0 ]]; then\n");
    let _ = writeln!(
        script,
        "    (cd \"$FIXTURE\" && {}) > \"$EXP_DIR/validation/${{run_id}}.txt\" 2>&1",
        task.validation
    );
    script.push_str("    validation_status=$?\n");
    script.push_str("  fi\n");
    script.push_str("  set -e\n");
    script.push_str("  printf '%s\\t%s\\t%s\\t%s\\t%s\\t%s\\t%s\\n' \"$run_id\" \"$claude_status\" \"$validation_status\" \"$condition\" \"$slice\" \"$repeat\" \"$phase\" >> \"$EXP_DIR/notes/run-log.tsv\"\n");
    script.push_str("  echo \"END $run_id claude_status=$claude_status validation_status=$validation_status\"\n");
    script.push_str("  reset_fixture\n");
    script.push_str("}\n\n");

    for slice in slices {
        for repeat in 1..=repeats {
            for condition in conditions {
                let _ = writeln!(
                    script,
                    "run_one {} {} {repeat} warm-up",
                    shell_word(&condition.id),
                    shell_word(&slice.id)
                );
            }
            for condition in conditions {
                let _ = writeln!(
                    script,
                    "run_one {} {} {repeat} measured",
                    shell_word(&condition.id),
                    shell_word(&slice.id)
                );
            }
        }
    }

    script.push_str("\n: > \"$EXP_DIR/baseline.jsonl\"\n");
    script.push_str(": > \"$EXP_DIR/cache-friendly.jsonl\"\n");
    script.push_str("while IFS=$'\\t' read -r run_id claude_status validation_status condition slice repeat phase; do\n");
    script.push_str("  [[ \"$run_id\" == \"run_id\" ]] && continue\n");
    script.push_str("  [[ \"$phase\" == \"measured\" ]] || continue\n");
    script.push_str("  [[ \"$claude_status\" == \"0\" ]] || continue\n");
    script.push_str("  output=\"$EXP_DIR/baseline.jsonl\"\n");
    script.push_str("  [[ \"$condition\" == \"cache-friendly\" ]] && output=\"$EXP_DIR/cache-friendly.jsonl\"\n");
    script.push_str("  passed=false\n");
    script.push_str("  [[ \"$validation_status\" == \"0\" ]] && passed=true\n");
    script.push_str("  cargo run --quiet -- claude-json-import \\\n");
    script.push_str("    --input \"$EXP_DIR/raw/claude-json/${run_id}.json\" \\\n");
    script.push_str("    --run-id \"$run_id\" \\\n");
    script.push_str("    --task-id \"$TASK_ID\" \\\n");
    script.push_str("    --condition \"$condition\" \\\n");
    script.push_str("    --slice \"$slice\" \\\n");
    script.push_str("    --repeat-id \"$repeat\" \\\n");
    script.push_str("    --phase \"$phase\" \\\n");
    script.push_str("    --validation-path \"$EXP_DIR/validation/${run_id}.txt\" \\\n");
    script.push_str("    --validation-passed \"$passed\" \\\n");
    script.push_str("    --task-success \"$passed\" \\\n");
    script.push_str("    --output \"$output\" >/dev/null\n");
    script.push_str("done < \"$EXP_DIR/notes/run-log.tsv\"\n\n");
    script.push_str("cargo run --quiet -- eval --baseline \"$EXP_DIR/baseline.jsonl\" --candidate \"$EXP_DIR/cache-friendly.jsonl\" | tee \"$EXP_DIR/eval.txt\"\n");
    script.push_str("cargo run --quiet -- task-report --baseline \"$EXP_DIR/baseline.jsonl\" --candidate \"$EXP_DIR/cache-friendly.jsonl\" | tee \"$EXP_DIR/task-report.txt\"\n");
    script.push_str("cargo run --quiet -- analysis-report --baseline \"$EXP_DIR/baseline.jsonl\" --candidate \"$EXP_DIR/cache-friendly.jsonl\" --output \"$EXP_DIR/analysis-report.md\"\n");
    script
}

#[derive(Debug, Clone)]
struct PilotTask {
    id: String,
    validation: String,
    prompt_turns: Vec<String>,
}

#[derive(Debug, Clone)]
struct PilotCondition {
    id: String,
    flags: Vec<String>,
}

#[derive(Debug, Clone)]
struct PilotSlice {
    id: String,
    drift_actions: Vec<String>,
}

fn pilot_model(manifest: &JsonValue) -> String {
    manifest
        .get("object_of_study")
        .and_then(|object| object.get("backend_route_model"))
        .and_then(JsonValue::as_str)
        .and_then(|model| {
            if model.contains("mimo-v2.5-pro") {
                Some("mimo-v2.5-pro")
            } else {
                model.split_whitespace().next()
            }
        })
        .unwrap_or("mimo-v2.5-pro")
        .to_string()
}

fn pilot_task(manifest: &JsonValue, task_id: &str) -> Result<PilotTask, String> {
    let tasks = manifest
        .get("tasks")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| "manifest is missing a tasks array".to_string())?;
    for task in tasks {
        if task.get("id").and_then(JsonValue::as_str) == Some(task_id) {
            let validation = task
                .get("validation")
                .and_then(JsonValue::as_str)
                .ok_or_else(|| format!("task {task_id} is missing validation"))?
                .to_string();
            let prompt_turns = task
                .get("prompt_turns")
                .and_then(JsonValue::as_array)
                .ok_or_else(|| format!("task {task_id} is missing prompt_turns"))?
                .iter()
                .map(|prompt| {
                    prompt
                        .as_str()
                        .map(str::to_string)
                        .ok_or_else(|| format!("task {task_id} has a non-string prompt"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            if prompt_turns.is_empty() {
                return Err(format!("task {task_id} has no prompt turns"));
            }
            return Ok(PilotTask {
                id: task_id.to_string(),
                validation,
                prompt_turns,
            });
        }
    }
    Err(format!("task {task_id} was not found in manifest"))
}

fn pilot_conditions(manifest: &JsonValue) -> Result<Vec<PilotCondition>, String> {
    let conditions = manifest
        .get("conditions")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| "manifest is missing a conditions array".to_string())?;
    let mut out = Vec::new();
    for condition in conditions {
        let id = condition
            .get("id")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| "condition is missing id".to_string())?
            .to_string();
        let flags = condition
            .get("claude_flags")
            .and_then(JsonValue::as_array)
            .map(|flags| {
                flags
                    .iter()
                    .map(|flag| {
                        flag.as_str()
                            .map(str::to_string)
                            .ok_or_else(|| format!("condition {id} has a non-string flag"))
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();
        out.push(PilotCondition { id, flags });
    }
    if out.is_empty() {
        return Err("manifest has no conditions".to_string());
    }
    Ok(out)
}

fn pilot_slices(
    manifest: &JsonValue,
    slice_filter: Option<&str>,
) -> Result<Vec<PilotSlice>, String> {
    let slices = manifest
        .get("slices")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| "manifest is missing a slices array".to_string())?;
    let mut out = Vec::new();
    for slice in slices {
        let id = slice
            .get("id")
            .and_then(JsonValue::as_str)
            .ok_or_else(|| "slice is missing id".to_string())?;
        if slice_filter.is_some_and(|wanted| wanted != id) {
            continue;
        }
        let drift_actions = slice
            .get("drift_actions")
            .and_then(JsonValue::as_array)
            .map(|actions| {
                actions
                    .iter()
                    .map(|action| {
                        action
                            .as_str()
                            .map(str::to_string)
                            .ok_or_else(|| format!("slice {id} has a non-string drift action"))
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();
        out.push(PilotSlice {
            id: id.to_string(),
            drift_actions,
        });
    }
    if out.is_empty() {
        return Err(match slice_filter {
            Some(slice) => format!("slice {slice} was not found in manifest"),
            None => "manifest has no slices".to_string(),
        });
    }
    Ok(out)
}

fn print_pilot_call(
    task: &PilotTask,
    slice: &PilotSlice,
    condition: &PilotCondition,
    repeat: u64,
    phase: &str,
    model: &str,
    validate: bool,
) {
    let run_id = format!(
        "{}-{}-{}-r{}-{}",
        task.id, slice.id, condition.id, repeat, phase
    );
    println!();
    println!("# run_id={run_id}");
    println!("(");
    println!("  cd \"$FIXTURE\"");
    println!("  bash task-reset.sh {}", shell_word(&task.id));
    if phase == "measured" && !slice.drift_actions.is_empty() {
        println!("  mkdir -p \"$EXP_DIR/drift\"");
        for action in &slice.drift_actions {
            println!("  {action}");
        }
        println!("  git status --short > \"$EXP_DIR/drift/{run_id}.git-status.txt\"");
    }
    println!("  mkdir -p \"$EXP_DIR/raw/claude-json\"");
    println!("  claude -p \\");
    println!("    --model {} \\", shell_word(model));
    println!("    --output-format json \\");
    println!("    --no-session-persistence \\");
    println!("    --permission-mode bypassPermissions \\");
    for flag in &condition.flags {
        println!("    {} \\", shell_word(flag));
    }
    println!(
        "    \"$(cat \"$EXP_DIR/prompts/{}-turn1.txt\")\" \\",
        task.id
    );
    println!("    > \"$EXP_DIR/raw/claude-json/{run_id}.json\" \\");
    println!("    2> \"$EXP_DIR/raw/claude-json/{run_id}.stderr.txt\"");
    if validate {
        println!(
            "  {} > \"$EXP_DIR/validation/{run_id}.txt\" 2>&1",
            task.validation
        );
    }
    println!(")");
}

fn shell_word(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ':'))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn run_init_experiment(dir: &Path) -> Result<i32, String> {
    fs::create_dir_all(dir)
        .map_err(|err| format!("Could not create experiment dir {}: {err}", dir.display()))?;
    for child in [
        "raw/claude-trace",
        "raw/claude-json",
        "traces",
        "layers",
        "requests",
        "tools",
        "validation",
        "prompts",
        "drift",
        "notes",
    ] {
        fs::create_dir_all(dir.join(child)).map_err(|err| {
            format!(
                "Could not create experiment subdir {}: {err}",
                dir.join(child).display()
            )
        })?;
    }

    write_new_file(&dir.join("manifest.json"), &experiment_manifest_template())?;
    write_new_file(&dir.join("baseline.jsonl"), "")?;
    write_new_file(&dir.join("cache-friendly.jsonl"), "")?;
    write_new_file(&dir.join("notes.md"), EXPERIMENT_NOTES_TEMPLATE)?;
    write_new_file(&dir.join("README.md"), EXPERIMENT_README_TEMPLATE)?;

    println!("Experiment log directory initialized");
    println!("Dir: {}", dir.display());
    println!();
    println!("Next:");
    println!("  1. Fill manifest.json before running tasks.");
    println!("  2. Save direct Claude JSON outputs under raw/claude-json/.");
    println!(
        "  3. Save validation logs under validation/ and optional trace artifacts under their matching subdirs."
    );
    println!(
        "  4. Append one JSON object per model call to baseline.jsonl and cache-friendly.jsonl."
    );
    println!("  5. Compare with:");
    println!(
        "     make-agents-cheaper eval --baseline {} --candidate {}",
        dir.join("baseline.jsonl").display(),
        dir.join("cache-friendly.jsonl").display()
    );
    Ok(0)
}

fn write_new_file(path: &Path, contents: &str) -> Result<(), String> {
    if path.exists() {
        return Err(format!(
            "{} already exists; refusing to overwrite experiment log files",
            path.display()
        ));
    }
    fs::write(path, contents).map_err(|err| format!("Could not write {}: {err}", path.display()))
}

fn experiment_manifest_template() -> String {
    format!(
        r#"{{
  "schema_version": 1,
  "created_unix_seconds": {},
  "experiment_id": "",
  "agent": "claude_code",
  "model": "mimo-v2.5-pro",
  "provider_or_router": "",
  "repository": "",
  "repo_commit": "",
  "task_suite": "",
  "baseline_condition": "ordinary_setup",
  "candidate_condition": "cache_friendly",
  "fixed_variables": {{
    "same_model": true,
    "same_repo_snapshot": true,
    "same_task_prompts": true,
    "same_validation_commands": true
  }},
  "cache_friendly_policy": [
    "do_not_switch_model_mid_session",
    "do_not_change_mcp_or_hooks_mid_session",
    "keep_tool_schema_stable",
    "structure_stable_components_first",
    "record_cache_control_breakpoints",
    "record_cached_and_uncached_input"
  ],
  "notes": ""
}}
"#,
        unix_seconds()
    )
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

const EXPERIMENT_NOTES_TEMPLATE: &str = r#"# Experiment Notes

## Goal

Verify whether the cache-friendly condition increases prompt cache hit rate and reduces paid uncached input without reducing task success.

## Conditions

Baseline:

- ordinary setup

Cache-friendly:

- same model
- stable provider / route
- stable tool schema
- stable prompt prefix
- no mid-session MCP or hook changes
- cache-aware compact / reactivation when needed

## Task Log

| Task ID | Condition | Run ID | Validation | Success | Notes |
| --- | --- | --- | --- | --- | --- |

## Trace Log

| Run ID | Trace Path | Request Path | Layers Path | Tools Path | Notes |
| --- | --- | --- | --- | --- | --- |

## Observations

- Cache hit rate:
- Uncached paid input:
- TTFT / latency:
- Task success:
- Regressions:
"#;

const EXPERIMENT_README_TEMPLATE: &str = r#"# Experiment Log

This directory is an append-only experiment log for `make-agents-cheaper`.

Required files:

- `manifest.json`: fixed experiment metadata.
- `baseline.jsonl`: one JSON object per baseline model call.
- `cache-friendly.jsonl`: one JSON object per cache-friendly model call.
- `raw/claude-json/`: direct Claude Code JSON outputs for the current experiment path.
- `raw/claude-trace/`: optional raw request/response captures if trace capture is explicitly used.
- `traces/`: normalized response or trace exports.
- `requests/`: request JSON exports for breakpoint analysis.
- `layers/`: prompt/harness layer exports for fingerprint analysis.
- `tools/`: tool schema exports.
- `validation/`: validation command stdout/stderr logs.
- `prompts/`: exact prompt text used for each run.
- `drift/`: dynamic-drift probe snapshots.
- `notes.md`: human-readable run notes.

Record every run before interpreting results. Do not rely on memory.

Minimum JSONL record:

```json
{
  "task_id": "docs-token-accounting",
  "run_id": "docs-token-accounting-cache-friendly-drift-r1-measured",
  "condition": "cache-friendly",
  "slice": "dynamic-drift",
  "repeat_id": 1,
  "phase": "measured",
  "turn_index": 1,
  "agent": "claude_code",
  "model": "mimo-v2.5-pro",
  "transport": "claude_code_json",
  "input_tokens": 82000,
  "cached_input_tokens": 76000,
  "cache_creation_input_tokens": 0,
  "uncached_input_tokens": 6000,
  "output_tokens": 3000,
  "ttft_ms": 1200,
  "total_latency_ms": 24000,
  "tool_calls": 5,
  "validation_command": "bash task-validate.sh docs-token-accounting",
  "validation_passed": true,
  "task_success": true,
  "cache_accounting_observable": true,
  "trace_path": "raw/claude-json/docs-token-accounting-cache-friendly-drift-r1-measured.json",
  "raw_json_path": "raw/claude-json/docs-token-accounting-cache-friendly-drift-r1-measured.json",
  "request_shape_observable": false,
  "validation_path": "validation/docs-token-accounting-cache-friendly-drift-r1-measured.txt",
  "anomaly": ""
}
```
"#;

fn read_json(path: &Path) -> Result<JsonValue, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("Could not read {}: {err}", path.display()))?;
    serde_json::from_str(&raw)
        .map_err(|err| format!("Could not parse JSON {}: {err}", path.display()))
}

fn extract_layers(value: &JsonValue) -> Result<BTreeMap<String, JsonValue>, String> {
    let source = value.get("layers").unwrap_or(value);
    let Some(object) = source.as_object() else {
        return Ok(BTreeMap::from([("root".to_string(), source.clone())]));
    };
    Ok(object
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect())
}

fn extract_tools(value: &JsonValue) -> Result<Vec<JsonValue>, String> {
    let tools = value.get("tools").unwrap_or(value);
    let Some(array) = tools.as_array() else {
        return Err("expected JSON array or object with a tools array".to_string());
    };
    Ok(array.clone())
}

fn tool_names(tools: &[JsonValue]) -> Vec<String> {
    tools
        .iter()
        .enumerate()
        .map(|(index, tool)| {
            tool.get("name")
                .and_then(JsonValue::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| format!("unnamed_{index}"))
        })
        .collect()
}

fn sorted_clone(values: &[String]) -> Vec<String> {
    let mut sorted = values.to_vec();
    sorted.sort();
    sorted
}

fn extract_blocks(value: &JsonValue) -> Vec<JsonValue> {
    if let Some(blocks) = value.get("blocks").and_then(JsonValue::as_array) {
        return blocks.clone();
    }

    let mut blocks = Vec::new();
    if let Some(system) = value.get("system") {
        push_blockish(system, &mut blocks);
    }
    if let Some(tools) = value.get("tools") {
        push_blockish(tools, &mut blocks);
    }
    if let Some(messages) = value.get("messages") {
        push_blockish(messages, &mut blocks);
    }

    if blocks.is_empty() {
        push_blockish(value, &mut blocks);
    }
    blocks
}

fn push_blockish(value: &JsonValue, blocks: &mut Vec<JsonValue>) {
    match value {
        JsonValue::Array(array) => {
            for item in array {
                push_blockish(item, blocks);
            }
        }
        JsonValue::Object(object) => {
            blocks.push(value.clone());
            if let Some(content) = object.get("content") {
                push_blockish(content, blocks);
            }
        }
        _ => blocks.push(value.clone()),
    }
}

fn has_direct_cache_control(value: &JsonValue) -> bool {
    value
        .as_object()
        .map(|object| object.contains_key("cache_control"))
        .unwrap_or(false)
}

fn block_role(value: &JsonValue) -> String {
    value
        .get("role")
        .or_else(|| value.get("type"))
        .and_then(JsonValue::as_str)
        .unwrap_or("block")
        .to_string()
}

fn canonical_json(value: &JsonValue) -> JsonValue {
    match value {
        JsonValue::Array(array) => JsonValue::Array(array.iter().map(canonical_json).collect()),
        JsonValue::Object(object) => {
            let mut sorted = serde_json::Map::new();
            for (key, value) in object.iter().collect::<BTreeMap<_, _>>() {
                sorted.insert(key.clone(), canonical_json(value));
            }
            JsonValue::Object(sorted)
        }
        _ => value.clone(),
    }
}

fn short_hash_json(value: &JsonValue) -> String {
    let canonical =
        serde_json::to_string(&canonical_json(value)).unwrap_or_else(|_| "null".to_string());
    short_hash(canonical.as_bytes())
}

fn short_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::new();
    for byte in digest.iter().take(8) {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[derive(Debug, Default, Clone)]
struct RunStats {
    records: u64,
    input: u64,
    cached_input: u64,
    uncached_input: u64,
    output: u64,
    ttft_ms: Vec<u64>,
    total_latency_ms: Vec<u64>,
    validation_passed: u64,
    task_success: u64,
    cache_accounting_unobservable: u64,
    actual_cost_usd: f64,
}

impl RunStats {
    fn from_jsonl(path: &Path) -> Result<Self, String> {
        Ok(Self::from_records(&RunRecord::from_jsonl(path)?))
    }

    fn from_records(records: &[RunRecord]) -> Self {
        let mut stats = Self::default();
        for record in records {
            stats.add_record(record);
        }
        stats
    }

    fn add_record(&mut self, record: &RunRecord) {
        self.records += 1;
        self.input += record.input_tokens;
        self.cached_input += record.cached_input_tokens;
        self.uncached_input += record
            .input_tokens
            .saturating_sub(record.cached_input_tokens);
        self.output += record.output_tokens;
        if let Some(ttft) = record.ttft_ms {
            self.ttft_ms.push(ttft);
        }
        if let Some(latency) = record.total_latency_ms {
            self.total_latency_ms.push(latency);
        }
        if record.validation_passed {
            self.validation_passed += 1;
        }
        if record.task_success {
            self.task_success += 1;
        }
        if !record.cache_accounting_observable {
            self.cache_accounting_unobservable += 1;
        }
        self.actual_cost_usd += record.actual_cost_usd;
    }

    fn cache_hit_rate(&self) -> f64 {
        ratio(self.cached_input, self.input).unwrap_or(0.0)
    }

    fn estimated_actual_cost(&self, prices: PriceConfig) -> f64 {
        cost_per_mtok(
            self.uncached_input,
            prices.uncached_input_per_mtok,
            self.cached_input,
            prices.cached_input_per_mtok,
            self.output,
            prices.output_per_mtok,
        )
    }

    fn estimated_full_uncached_cost(&self, prices: PriceConfig) -> f64 {
        cost_per_mtok(
            self.input,
            prices.uncached_input_per_mtok,
            0,
            prices.cached_input_per_mtok,
            self.output,
            prices.output_per_mtok,
        )
    }
}

#[derive(Debug, Clone)]
struct RunRecord {
    run_id: String,
    task_id: String,
    slice: String,
    repeat_id: Option<u64>,
    input_tokens: u64,
    cached_input_tokens: u64,
    output_tokens: u64,
    turns_to_completion: Option<u64>,
    ttft_ms: Option<u64>,
    total_latency_ms: Option<u64>,
    validation_passed: bool,
    task_success: bool,
    cache_accounting_observable: bool,
    actual_cost_usd: f64,
}

impl RunRecord {
    fn from_jsonl(path: &Path) -> Result<Vec<Self>, String> {
        let raw = fs::read_to_string(path)
            .map_err(|err| format!("Could not read {}: {err}", path.display()))?;
        let mut records = Vec::new();
        for (index, line) in raw.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let value: JsonValue = serde_json::from_str(line).map_err(|err| {
                format!(
                    "Could not parse {} line {}: {err}",
                    path.display(),
                    index + 1
                )
            })?;
            records.push(Self::from_json(&value));
        }
        Ok(records)
    }

    fn from_json(value: &JsonValue) -> Self {
        Self {
            run_id: value
                .get("run_id")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown")
                .to_string(),
            task_id: value
                .get("task_id")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown")
                .to_string(),
            slice: value
                .get("slice")
                .and_then(JsonValue::as_str)
                .unwrap_or("unspecified")
                .to_string(),
            repeat_id: json_u64(value, "repeat_id").or_else(|| json_u64(value, "turn_index")),
            input_tokens: json_u64(value, "input_tokens").unwrap_or(0),
            cached_input_tokens: json_u64(value, "cached_input_tokens").unwrap_or(0),
            output_tokens: json_u64(value, "output_tokens").unwrap_or(0),
            turns_to_completion: json_u64(value, "turns_to_completion"),
            ttft_ms: json_u64(value, "ttft_ms"),
            total_latency_ms: json_u64(value, "total_latency_ms"),
            validation_passed: json_bool(value, "validation_passed").unwrap_or(false),
            task_success: json_bool(value, "task_success").unwrap_or(false),
            cache_accounting_observable: json_bool(value, "cache_accounting_observable")
                .unwrap_or(true),
            actual_cost_usd: json_f64(value, "actual_cost_usd").unwrap_or(0.0),
        }
    }
}

fn stats_by_task(records: &[RunRecord]) -> BTreeMap<String, RunStats> {
    let mut by_task = BTreeMap::new();
    for record in records {
        by_task
            .entry(record.task_id.clone())
            .or_insert_with(RunStats::default)
            .add_record(record);
    }
    by_task
}

fn stats_by_slice(records: &[RunRecord]) -> BTreeMap<String, RunStats> {
    let mut by_slice = BTreeMap::new();
    for record in records {
        by_slice
            .entry(record.slice.clone())
            .or_insert_with(RunStats::default)
            .add_record(record);
    }
    by_slice
}

fn cost_per_mtok(
    uncached_input_tokens: u64,
    uncached_input_per_mtok: f64,
    cached_input_tokens: u64,
    cached_input_per_mtok: f64,
    output_tokens: u64,
    output_per_mtok: f64,
) -> f64 {
    let input_cost = uncached_input_tokens as f64 * uncached_input_per_mtok;
    let cached_cost = cached_input_tokens as f64 * cached_input_per_mtok;
    let output_cost = output_tokens as f64 * output_per_mtok;
    (input_cost + cached_cost + output_cost) / 1_000_000.0
}

fn json_u64(value: &JsonValue, key: &str) -> Option<u64> {
    value.get(key)?.as_u64()
}

fn json_bool(value: &JsonValue, key: &str) -> Option<bool> {
    value.get(key)?.as_bool()
}

fn json_f64(value: &JsonValue, key: &str) -> Option<f64> {
    value.get(key)?.as_f64()
}

fn ratio(numerator: u64, denominator: u64) -> Option<f64> {
    if denominator == 0 {
        None
    } else {
        Some(numerator as f64 / denominator as f64)
    }
}

fn ratio_f64(numerator: f64, denominator: f64) -> Option<f64> {
    if denominator <= f64::EPSILON {
        None
    } else {
        Some(numerator / denominator)
    }
}

fn savings_ratio(actual: f64, full: f64) -> Option<f64> {
    ratio_f64(actual, full).map(|ratio| 1.0 - ratio)
}

fn display_ratio(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.3}x"))
        .unwrap_or_else(|| "n/a".to_string())
}

fn display_ratio_f64(value: Option<f64>) -> String {
    display_ratio(value)
}

fn truncate_for_table(value: &str, width: usize) -> String {
    if value.chars().count() <= width {
        value.to_string()
    } else {
        let mut truncated: String = value.chars().take(width.saturating_sub(1)).collect();
        truncated.push('~');
        truncated
    }
}

fn truncate_chars(value: &str, width: usize) -> String {
    if value.chars().count() <= width {
        value.to_string()
    } else {
        let mut truncated: String = value.chars().take(width.saturating_sub(13)).collect();
        truncated.push_str("\n...[truncated]");
        truncated
    }
}

fn display_usd(value: f64) -> String {
    format!("${value:.6}")
}

fn display_optional_percent(value: Option<f64>) -> String {
    value
        .map(|value| format!("{:.2}%", value * 100.0))
        .unwrap_or_else(|| "n/a".to_string())
}

fn display_median(values: &[u64]) -> String {
    median(values)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "n/a".to_string())
}

fn median(values: &[u64]) -> Option<u64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable_by(|left, right| {
        if left == right {
            Ordering::Equal
        } else if left < right {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });
    Some(sorted[sorted.len() / 2])
}

fn str_value<'a>(value: &'a TomlValue, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

fn bool_value(value: &TomlValue, path: &[&str]) -> bool {
    let mut current = value;
    for key in path {
        let Some(next) = current.get(*key) else {
            return false;
        };
        current = next;
    }
    current.as_bool().unwrap_or(false)
}

fn ok(message: impl Into<String>) -> Finding {
    Finding {
        level: "OK",
        message: message.into(),
    }
}

fn info(message: impl Into<String>) -> Finding {
    Finding {
        level: "INFO",
        message: message.into(),
    }
}

fn warn(message: impl Into<String>) -> Finding {
    Finding {
        level: "WARN",
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD_CONFIG: &str = r#"model_provider = "cache_router"
model = "gpt-5.4"
model_reasoning_effort = "xhigh"
plan_mode_reasoning_effort = "xhigh"
model_reasoning_summary = "none"
model_verbosity = "medium"

[model_providers.cache_router]
name = "OpenAI"
base_url = "https://router.example/v1"
wire_api = "responses"
requires_openai_auth = false
env_key = "CACHE_ROUTER_API_KEY"
supports_websockets = true

[features]
responses_websockets_v2 = true
"#;

    const BAD_CONFIG: &str = r#"model_provider = "openai"
model = "gpt-5.4"

[model_providers.openai]
name = "OpenAI"
base_url = "https://api.openai.com"
wire_api = "chat"
env_key = "OPENAI_API_KEY"
"#;

    #[test]
    fn parses_basic_args() {
        let command = parse_args(["--config", "custom.toml"]).unwrap();
        assert_eq!(
            command,
            Command::AuditConfig {
                config: PathBuf::from("custom.toml")
            }
        );

        let command = parse_args(["--print-ws-config"]).unwrap();
        assert_eq!(command, Command::PrintWsConfig);
    }

    #[test]
    fn parses_new_commands() {
        let command = parse_args([
            "fingerprint",
            "--input",
            "now.json",
            "--previous",
            "prev.json",
        ])
        .unwrap();
        assert_eq!(
            command,
            Command::Fingerprint {
                input: PathBuf::from("now.json"),
                previous: Some(PathBuf::from("prev.json"))
            }
        );

        let command =
            parse_args(["eval", "--baseline", "a.jsonl", "--candidate", "b.jsonl"]).unwrap();
        assert_eq!(
            command,
            Command::Eval {
                baseline: PathBuf::from("a.jsonl"),
                candidate: PathBuf::from("b.jsonl"),
                prices: None
            }
        );

        let command = parse_args([
            "task-report",
            "--baseline",
            "a.jsonl",
            "--candidate",
            "b.jsonl",
        ])
        .unwrap();
        assert_eq!(
            command,
            Command::TaskReport {
                baseline: PathBuf::from("a.jsonl"),
                candidate: PathBuf::from("b.jsonl")
            }
        );

        let command = parse_args([
            "analysis-report",
            "--baseline",
            "a.jsonl",
            "--candidate",
            "b.jsonl",
            "--output",
            "report.md",
        ])
        .unwrap();
        assert_eq!(
            command,
            Command::AnalysisReport {
                baseline: PathBuf::from("a.jsonl"),
                candidate: PathBuf::from("b.jsonl"),
                output: Some(PathBuf::from("report.md"))
            }
        );

        let command = parse_args([
            "evidence-diff",
            "--input",
            "session.jsonl",
            "--output",
            "code-changes.json",
        ])
        .unwrap();
        assert_eq!(
            command,
            Command::EvidenceDiff {
                input: PathBuf::from("session.jsonl"),
                output: Some(PathBuf::from("code-changes.json"))
            }
        );

        let command = parse_args([
            "eval",
            "--baseline",
            "a.jsonl",
            "--candidate",
            "b.jsonl",
            "--uncached-input-per-mtok",
            "2.0",
            "--cached-input-per-mtok",
            "0.2",
            "--output-per-mtok",
            "8.0",
        ])
        .unwrap();
        assert_eq!(
            command,
            Command::Eval {
                baseline: PathBuf::from("a.jsonl"),
                candidate: PathBuf::from("b.jsonl"),
                prices: Some(PriceConfig {
                    uncached_input_per_mtok: 2.0,
                    cached_input_per_mtok: 0.2,
                    output_per_mtok: 8.0
                })
            }
        );

        let command = parse_args(["init-experiment", "--dir", "runs/demo"]).unwrap();
        assert_eq!(
            command,
            Command::InitExperiment {
                dir: PathBuf::from("runs/demo")
            }
        );

        let command = parse_args([
            "pilot-plan",
            "--manifest",
            "suite.json",
            "--task",
            "docs-token-accounting",
            "--experiment-dir",
            "runs/pilot",
            "--slice",
            "dynamic-drift",
            "--repeats",
            "2",
        ])
        .unwrap();
        assert_eq!(
            command,
            Command::PilotPlan {
                manifest: PathBuf::from("suite.json"),
                task: "docs-token-accounting".to_string(),
                experiment_dir: PathBuf::from("runs/pilot"),
                slice: Some("dynamic-drift".to_string()),
                repeats: 2
            }
        );

        let command = parse_args([
            "run-pilot",
            "--manifest",
            "suite.json",
            "--task",
            "docs-token-accounting",
            "--experiment-dir",
            "runs/pilot",
            "--slice",
            "dynamic-drift",
            "--repeats",
            "2",
            "--execute",
            "true",
        ])
        .unwrap();
        assert_eq!(
            command,
            Command::RunPilot {
                manifest: PathBuf::from("suite.json"),
                task: "docs-token-accounting".to_string(),
                experiment_dir: PathBuf::from("runs/pilot"),
                slice: Some("dynamic-drift".to_string()),
                repeats: 2,
                execute: true
            }
        );

        let command = parse_args([
            "trace-import",
            "--input",
            "raw.jsonl",
            "--run-id",
            "run-1",
            "--task-id",
            "docs-token-accounting",
            "--condition",
            "baseline",
            "--output",
            "baseline.jsonl",
            "--validation-passed",
            "true",
        ])
        .unwrap();
        assert_eq!(
            command,
            Command::TraceImport {
                input: PathBuf::from("raw.jsonl"),
                run_id: "run-1".to_string(),
                task_id: "docs-token-accounting".to_string(),
                condition: "baseline".to_string(),
                slice: None,
                repeat_id: None,
                phase: None,
                output: Some(PathBuf::from("baseline.jsonl")),
                artifacts_dir: None,
                validation_path: None,
                validation_passed: Some(true),
                task_success: None
            }
        );

        let command = parse_args([
            "claude-json-import",
            "--input",
            "result.json",
            "--run-id",
            "run-1",
            "--task-id",
            "docs-token-accounting",
            "--condition",
            "cache-friendly",
            "--slice",
            "dynamic-drift",
            "--repeat-id",
            "1",
            "--phase",
            "measured",
            "--output",
            "cache-friendly.jsonl",
            "--validation-passed",
            "true",
        ])
        .unwrap();
        assert_eq!(
            command,
            Command::ClaudeJsonImport {
                input: PathBuf::from("result.json"),
                run_id: "run-1".to_string(),
                task_id: "docs-token-accounting".to_string(),
                condition: "cache-friendly".to_string(),
                slice: Some("dynamic-drift".to_string()),
                repeat_id: Some(1),
                phase: Some("measured".to_string()),
                output: Some(PathBuf::from("cache-friendly.jsonl")),
                validation_path: None,
                validation_passed: Some(true),
                task_success: None
            }
        );

        let command = parse_args([
            "matrix-plan",
            "--manifest",
            "suite.json",
            "--experiment-dir",
            "runs/matrix",
            "--tasks",
            "docs-token-accounting,config-warning-rule",
            "--repeats",
            "3",
        ])
        .unwrap();
        assert_eq!(
            command,
            Command::MatrixPlan {
                manifest: PathBuf::from("suite.json"),
                experiment_dir: PathBuf::from("runs/matrix"),
                tasks: Some("docs-token-accounting,config-warning-rule".to_string()),
                repeats: 3
            }
        );
    }

    #[test]
    fn good_config_has_no_cache_warnings_when_env_is_present() {
        env::remove_var(EXPECTED_BASE_URL_ENV);
        env::set_var("CACHE_ROUTER_API_KEY", "redacted");
        let config = GOOD_CONFIG.parse::<TomlValue>().unwrap();
        let findings = audit_config(&config);
        assert!(!findings.iter().any(|finding| finding.level == "WARN"));
    }

    #[test]
    fn bad_config_warns_about_cache_unfriendly_settings() {
        env::remove_var("OPENAI_API_KEY");
        let config = BAD_CONFIG.parse::<TomlValue>().unwrap();
        let findings = audit_config(&config);
        assert!(findings.iter().any(|finding| finding.level == "WARN"));
        assert!(findings
            .iter()
            .any(|finding| finding.message.contains("not \"responses\"")));
    }

    #[test]
    fn canonical_hash_ignores_object_key_order() {
        let a: JsonValue = serde_json::from_str(r#"{"b":2,"a":1}"#).unwrap();
        let b: JsonValue = serde_json::from_str(r#"{"a":1,"b":2}"#).unwrap();
        assert_eq!(short_hash_json(&a), short_hash_json(&b));
    }

    #[test]
    fn extracts_tools_from_object() {
        let value: JsonValue =
            serde_json::from_str(r#"{"tools":[{"name":"b"},{"name":"a"}]}"#).unwrap();
        let tools = extract_tools(&value).unwrap();
        assert_eq!(tool_names(&tools), vec!["b".to_string(), "a".to_string()]);
    }

    #[test]
    fn finds_direct_breakpoints() {
        let value: JsonValue = serde_json::from_str(
            r#"{"blocks":[{"type":"system","cache_control":{"type":"ephemeral"}},{"type":"text"}]}"#,
        )
        .unwrap();
        let blocks = extract_blocks(&value);
        assert_eq!(blocks.len(), 2);
        assert!(has_direct_cache_control(&blocks[0]));
    }

    #[test]
    fn extracts_apply_patch_evidence_diff() {
        let event = json!({
            "type": "tool_call",
            "timestamp": "2026-05-12T00:00:00Z",
            "tool_name": "apply_patch",
            "call_id": "call-1",
            "input": "*** Begin Patch\n*** Update File: src/lib.rs\n@@\n-old\n+new\n context\n*** End Patch\n"
        });
        let changes = extract_evidence_diffs(&[event]);
        assert_eq!(changes.len(), 1);
        let change = &changes[0];
        assert_eq!(change.event_index, 0);
        assert_eq!(change.tool_name.as_deref(), Some("apply_patch"));
        assert_eq!(change.diff.kind, "apply_patch");
        assert_eq!(change.diff.files, vec!["src/lib.rs".to_string()]);
        assert_eq!(change.diff.additions, 1);
        assert_eq!(change.diff.deletions, 1);
        assert!(change.before.contains("old"));
        assert!(change.after.contains("new"));
        assert!(change.before.contains("# src/lib.rs"));
    }

    #[test]
    fn extracts_unified_diff_evidence_diff_without_counting_headers() {
        let event = json!({
            "event": "function_call_output",
            "name": "Bash",
            "id": "call-2",
            "result": {
                "text": "diff --git a/README.md b/README.md\nindex 111..222 100644\n--- a/README.md\n+++ b/README.md\n@@ -1,2 +1,2 @@\n-old title\n+new title\n unchanged\n"
            }
        });
        let changes = extract_evidence_diffs(&[event]);
        assert_eq!(changes.len(), 1);
        let change = &changes[0];
        assert_eq!(change.diff.kind, "unified_diff");
        assert_eq!(change.diff.files, vec!["README.md".to_string()]);
        assert_eq!(change.diff.additions, 1);
        assert_eq!(change.diff.deletions, 1);
        assert!(change.before.contains("old title"));
        assert!(change.after.contains("new title"));
        assert!(!change.before.contains("+++ b/README.md"));
    }

    #[test]
    fn computes_run_stats() {
        let dir = env::temp_dir();
        let path = dir.join("make-agents-cheaper-test.jsonl");
        fs::write(
            &path,
            r#"{"input_tokens":100,"cached_input_tokens":60,"output_tokens":10,"validation_passed":true,"task_success":true,"actual_cost_usd":0.1}
{"input_tokens":200,"cached_input_tokens":100,"output_tokens":20,"validation_passed":false,"task_success":false,"cache_accounting_observable":false,"actual_cost_usd":0.2}
"#,
        )
        .unwrap();
        let stats = RunStats::from_jsonl(&path).unwrap();
        assert_eq!(stats.records, 2);
        assert_eq!(stats.uncached_input, 140);
        assert_eq!(stats.validation_passed, 1);
        assert_eq!(stats.task_success, 1);
        assert_eq!(stats.cache_accounting_unobservable, 1);
        assert!((stats.actual_cost_usd - 0.3).abs() < 0.000001);
        let prices = PriceConfig {
            uncached_input_per_mtok: 2.0,
            cached_input_per_mtok: 0.2,
            output_per_mtok: 8.0,
        };
        assert!((stats.estimated_actual_cost(prices) - 0.000552).abs() < 0.000001);
        let records = RunRecord::from_jsonl(&path).unwrap();
        let by_task = stats_by_task(&records);
        assert_eq!(by_task.get("unknown").unwrap().records, 2);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn analysis_report_distinguishes_successful_only() {
        let baseline_records = vec![
            RunRecord::from_json(&json!({
                "task_id": "docs-token-accounting",
                "input_tokens": 100,
                "cached_input_tokens": 20,
                "output_tokens": 10,
                "validation_passed": true,
                "task_success": true
            })),
            RunRecord::from_json(&json!({
                "task_id": "config-warning-rule",
                "input_tokens": 100,
                "cached_input_tokens": 0,
                "output_tokens": 10,
                "validation_passed": false,
                "task_success": false
            })),
        ];
        let candidate_records = vec![
            RunRecord::from_json(&json!({
                "task_id": "docs-token-accounting",
                "input_tokens": 100,
                "cached_input_tokens": 80,
                "output_tokens": 10,
                "validation_passed": true,
                "task_success": true
            })),
            RunRecord::from_json(&json!({
                "task_id": "config-warning-rule",
                "input_tokens": 100,
                "cached_input_tokens": 70,
                "output_tokens": 10,
                "validation_passed": true,
                "task_success": false,
                "cache_accounting_observable": false
            })),
        ];

        let report = analysis_report_markdown(
            Path::new("baseline.jsonl"),
            Path::new("cache-friendly.jsonl"),
            &baseline_records,
            &candidate_records,
        );

        assert!(report.contains("# Paper-Facing Analysis Summary"));
        assert!(report.contains("Development assistant: Codex"));
        assert!(report
            .contains("| All runs | 2 | 2 | 10.00% | 75.00% | 180 | 50 | 0.278x | 1/2 | 1/2 |"));
        assert!(report.contains(
            "| Successful-only | 1 | 1 | 20.00% | 80.00% | 80 | 20 | 0.250x | 1/1 | 1/1 |"
        ));
        assert!(report.contains("cache-accounting-unobservable records: 1/2"));
    }

    #[test]
    fn imports_claude_trace_usage() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = env::temp_dir().join(format!(
            "make-agents-cheaper-trace-test-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let raw_path = dir.join("raw.jsonl");
        let out_path = dir.join("baseline.jsonl");
        fs::write(
            &raw_path,
            r#"{"request":{"url":"https://api.example/v1/messages","body":{"model":"mimo-v2.5-pro","system":[{"type":"text","text":"rules"}],"tools":[{"name":"read"}],"messages":[{"role":"user","content":"hi"}]}},"response":{"body":{"usage":{"input_tokens":100,"cache_read_input_tokens":40,"cache_creation_input_tokens":10,"output_tokens":20}}},"duration_ms":1234}"#,
        )
        .unwrap();

        let code = run_trace_import(TraceImportOptions {
            input: &raw_path,
            run_id: "run-1",
            task_id: "docs-token-accounting",
            condition: "baseline",
            slice: Some("dynamic-drift"),
            repeat_id: Some(1),
            phase: Some("measured"),
            output: Some(&out_path),
            artifacts_dir: Some(&dir),
            validation_path: Some(Path::new("validation/run-1.txt")),
            validation_passed: Some(true),
            task_success: None,
        })
        .unwrap();

        assert_eq!(code, 0);
        let records = RunRecord::from_jsonl(&out_path).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].input_tokens, 150);
        assert_eq!(records[0].cached_input_tokens, 40);
        assert_eq!(records[0].output_tokens, 20);
        assert!(records[0].validation_passed);
        assert!(dir.join("requests").join("run-1.request.json").exists());
        assert!(dir.join("traces").join("run-1.response.json").exists());
        assert!(dir.join("layers").join("run-1.layers.json").exists());
        assert!(dir.join("tools").join("run-1.tools.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn imports_claude_json_usage() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = env::temp_dir().join(format!(
            "make-agents-cheaper-claude-json-test-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let raw_path = dir.join("result.json");
        let out_path = dir.join("cache-friendly.jsonl");
        fs::write(
            &raw_path,
            r#"{
  "type": "result",
  "subtype": "success",
  "duration_ms": 15762,
  "duration_api_ms": 15255,
  "num_turns": 4,
  "total_cost_usd": 0.075144,
  "modelUsage": {
    "mimo-v2.5-pro": {
      "inputTokens": 1271,
      "outputTokens": 541,
      "cacheReadInputTokens": 110528,
      "cacheCreationInputTokens": 0,
      "costUSD": 0.075144
    }
  }
}"#,
        )
        .unwrap();

        let code = run_claude_json_import(ClaudeJsonImportOptions {
            input: &raw_path,
            run_id: "run-1",
            task_id: "docs-token-accounting",
            condition: "cache-friendly",
            slice: Some("dynamic-drift"),
            repeat_id: Some(1),
            phase: Some("measured"),
            output: Some(&out_path),
            validation_path: Some(Path::new("validation/run-1.txt")),
            validation_passed: Some(true),
            task_success: None,
        })
        .unwrap();

        assert_eq!(code, 0);
        let raw = fs::read_to_string(&out_path).unwrap();
        let value: JsonValue = serde_json::from_str(raw.trim()).unwrap();
        let records = RunRecord::from_jsonl(&out_path).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].input_tokens, 111799);
        assert_eq!(records[0].cached_input_tokens, 110528);
        assert_eq!(records[0].output_tokens, 541);
        assert!(records[0].validation_passed);
        assert_eq!(
            value.get("transport").and_then(JsonValue::as_str),
            Some("claude_code_json")
        );
        assert_eq!(
            value
                .get("request_shape_observable")
                .and_then(JsonValue::as_bool),
            Some(false)
        );
        assert_eq!(
            value.get("model").and_then(JsonValue::as_str),
            Some("mimo-v2.5-pro")
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn init_experiment_creates_log_scaffold_and_refuses_overwrite() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = env::temp_dir().join(format!(
            "make-agents-cheaper-init-test-{}-{nanos}",
            std::process::id()
        ));

        let code = run_init_experiment(&dir).unwrap();
        assert_eq!(code, 0);
        assert!(dir.join("manifest.json").exists());
        assert!(dir.join("baseline.jsonl").exists());
        assert!(dir.join("cache-friendly.jsonl").exists());
        assert!(dir.join("traces").is_dir());
        assert!(dir.join("raw").join("claude-trace").is_dir());
        assert!(dir.join("raw").join("claude-json").is_dir());
        assert!(dir.join("requests").is_dir());
        assert!(dir.join("layers").is_dir());
        assert!(dir.join("tools").is_dir());
        assert!(dir.join("validation").is_dir());
        assert!(dir.join("prompts").is_dir());
        assert!(dir.join("drift").is_dir());

        assert!(run_init_experiment(&dir).is_err());

        let _ = fs::remove_dir_all(dir);
    }
}
