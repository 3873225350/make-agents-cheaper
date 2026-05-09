use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use toml::Value as TomlValue;

const XAI_BASE_URL: &str = "https://api.xairouter.com";

const WS_CONFIG: &str = r#"model_provider = "xai"
model = "gpt-5.4"
model_reasoning_effort = "xhigh"
plan_mode_reasoning_effort = "xhigh"
model_reasoning_summary = "none"
model_verbosity = "medium"
approval_policy = "never"
sandbox_mode = "danger-full-access"
suppress_unstable_features_warning = true

[model_providers.xai]
name = "OpenAI"
base_url = "https://api.xairouter.com"
wire_api = "responses"
requires_openai_auth = false
env_key = "XAI_API_KEY"
supports_websockets = true

[features]
responses_websockets_v2 = true
"#;

const HTTP_CONFIG: &str = r#"model_provider = "xai"
model = "gpt-5.4"
model_reasoning_effort = "xhigh"
plan_mode_reasoning_effort = "xhigh"
model_reasoning_summary = "none"
model_verbosity = "medium"
approval_policy = "never"
sandbox_mode = "danger-full-access"

[model_providers.xai]
name = "OpenAI"
base_url = "https://api.xairouter.com"
wire_api = "responses"
requires_openai_auth = false
env_key = "XAI_API_KEY"
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
    Eval {
        baseline: PathBuf,
        candidate: PathBuf,
        prices: Option<PriceConfig>,
    },
    TaskReport {
        baseline: PathBuf,
        candidate: PathBuf,
    },
    InitExperiment {
        dir: PathBuf,
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
        Command::Eval {
            baseline,
            candidate,
            prices,
        } => run_eval_report(&baseline, &candidate, prices).unwrap_or_else(print_error),
        Command::TaskReport {
            baseline,
            candidate,
        } => run_task_report(&baseline, &candidate).unwrap_or_else(print_error),
        Command::InitExperiment { dir } => run_init_experiment(&dir).unwrap_or_else(print_error),
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
  make-agents-cheaper eval --baseline baseline.jsonl --candidate cache-friendly.jsonl \
    [--uncached-input-per-mtok USD --cached-input-per-mtok USD --output-per-mtok USD]
  make-agents-cheaper task-report --baseline baseline.jsonl --candidate cache-friendly.jsonl
  make-agents-cheaper init-experiment --dir runs/exp-name
  make-agents-cheaper compact-template

Commands:
  audit             Inspect Codex config for cache-friendly settings
  fingerprint       Hash prompt/harness layers and report drift
  tool-schema       Hash and inspect tool schema stability
  breakpoints       Inspect cache_control breakpoint placement
  eval              Compare baseline vs cache-friendly JSONL runs
  task-report       Print per-task token usage from JSONL runs
  init-experiment   Create a reproducible experiment log directory
  compact-template  Print a stable-first reactivation template

Options:
  --config PATH         Inspect a specific Codex config.toml
  --print-ws-config     Print recommended XAI Router WebSocket config
  --print-http-config   Print recommended XAI Router HTTP config
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
        "eval" => Ok(Command::Eval {
            baseline: required_path(&args[1..], "--baseline")?,
            candidate: required_path(&args[1..], "--candidate")?,
            prices: price_config_from_args(&args[1..])?,
        }),
        "task-report" => Ok(Command::TaskReport {
            baseline: required_path(&args[1..], "--baseline")?,
            candidate: required_path(&args[1..], "--candidate")?,
        }),
        "init-experiment" => Ok(Command::InitExperiment {
            dir: required_path(&args[1..], "--dir")?,
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
        Some(base_url) if base_url == XAI_BASE_URL => {
            findings.push(ok("Provider base_url points to XAI Router."));
        }
        Some(base_url) => {
            findings.push(warn(format!(
                "Provider base_url is {:?}, not {:?}.",
                base_url, XAI_BASE_URL
            )));
        }
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

fn run_init_experiment(dir: &Path) -> Result<i32, String> {
    fs::create_dir_all(dir)
        .map_err(|err| format!("Could not create experiment dir {}: {err}", dir.display()))?;
    for child in ["traces", "layers", "requests", "tools", "notes"] {
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
    println!("  2. Save raw traces under traces/.");
    println!("  3. Save layer exports under layers/ and request exports under requests/.");
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
- `traces/`: raw agent or proxy traces.
- `requests/`: request JSON exports for breakpoint analysis.
- `layers/`: prompt/harness layer exports for fingerprint analysis.
- `tools/`: tool schema exports.
- `notes.md`: human-readable run notes.

Record every run before interpreting results. Do not rely on memory.

Minimum JSONL record:

```json
{
  "task_id": "docs-cache-hit-section",
  "run_id": "2026-05-09-cache-friendly-01",
  "condition": "cache_friendly",
  "turn_index": 1,
  "agent": "claude_code",
  "model": "mimo-v2.5-pro",
  "transport": "anthropic_messages",
  "input_tokens": 82000,
  "cached_input_tokens": 76000,
  "output_tokens": 3000,
  "ttft_ms": 1200,
  "total_latency_ms": 24000,
  "tool_calls": 5,
  "validation_command": "cargo test --locked",
  "validation_passed": true,
  "task_success": true,
  "trace_path": "traces/cache-friendly-01.json",
  "request_path": "requests/cache-friendly-01.request.json",
  "layers_path": "layers/cache-friendly-01.layers.json",
  "tools_path": "tools/cache-friendly-01.tools.json"
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
    task_id: String,
    input_tokens: u64,
    cached_input_tokens: u64,
    output_tokens: u64,
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
            task_id: value
                .get("task_id")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown")
                .to_string(),
            input_tokens: json_u64(value, "input_tokens").unwrap_or(0),
            cached_input_tokens: json_u64(value, "cached_input_tokens").unwrap_or(0),
            output_tokens: json_u64(value, "output_tokens").unwrap_or(0),
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

    const GOOD_CONFIG: &str = r#"model_provider = "xai"
model = "gpt-5.4"
model_reasoning_effort = "xhigh"
plan_mode_reasoning_effort = "xhigh"
model_reasoning_summary = "none"
model_verbosity = "medium"

[model_providers.xai]
name = "OpenAI"
base_url = "https://api.xairouter.com"
wire_api = "responses"
requires_openai_auth = false
env_key = "XAI_API_KEY"
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
    }

    #[test]
    fn good_config_has_no_cache_warnings_when_env_is_present() {
        env::set_var("XAI_API_KEY", "redacted");
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
        assert!(dir.join("requests").is_dir());
        assert!(dir.join("layers").is_dir());
        assert!(dir.join("tools").is_dir());

        assert!(run_init_experiment(&dir).is_err());

        let _ = fs::remove_dir_all(dir);
    }
}
