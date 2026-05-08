use std::env;
use std::fs;
use std::path::PathBuf;

use toml::Value;

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct Finding {
    level: &'static str,
    message: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Args {
    config: Option<PathBuf>,
    print_ws_config: bool,
    print_http_config: bool,
    help: bool,
}

fn main() {
    let args = match parse_args(env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            print_help();
            std::process::exit(2);
        }
    };

    if args.help {
        print_help();
        return;
    }

    if args.print_ws_config {
        print!("{WS_CONFIG}");
        return;
    }

    if args.print_http_config {
        print!("{HTTP_CONFIG}");
        return;
    }

    let config_path = args.config.unwrap_or_else(default_config_path);
    let exit_code = match run_report(&config_path) {
        Ok(code) => code,
        Err(message) => {
            println!("Make Agents Cheaper report");
            println!("Config: {}", config_path.display());
            println!();
            println!("[ERROR] {message}");
            2
        }
    };

    std::process::exit(exit_code);
}

fn print_help() {
    println!(
        "make-agents-cheaper\n\n\
         Usage:\n\
           make-agents-cheaper [--config PATH]\n\
           make-agents-cheaper --print-ws-config\n\
           make-agents-cheaper --print-http-config\n\n\
         Options:\n\
           --config PATH         Inspect a specific Codex config.toml\n\
           --print-ws-config     Print recommended XAI Router WebSocket config\n\
           --print-http-config   Print recommended XAI Router HTTP config\n\
           -h, --help            Show this help"
    );
}

fn parse_args<I>(args: I) -> Result<Args, String>
where
    I: IntoIterator,
    I::Item: Into<String>,
{
    let mut parsed = Args::default();
    let mut iter = args.into_iter().map(Into::into);

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--config" => {
                let Some(path) = iter.next() else {
                    return Err("--config requires a path".to_string());
                };
                parsed.config = Some(PathBuf::from(path));
            }
            "--print-ws-config" => parsed.print_ws_config = true,
            "--print-http-config" => parsed.print_http_config = true,
            "-h" | "--help" => parsed.help = true,
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    if parsed.print_ws_config && parsed.print_http_config {
        return Err("choose only one template flag".to_string());
    }

    Ok(parsed)
}

fn default_config_path() -> PathBuf {
    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home).join(".codex").join("config.toml");
    }
    PathBuf::from(".codex").join("config.toml")
}

fn run_report(config_path: &PathBuf) -> Result<i32, String> {
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
        .parse::<Value>()
        .map_err(|err| format!("Could not parse TOML: {err}"))?;

    Ok(print_findings(config_path, &audit_config(&config)))
}

fn print_findings(config_path: &PathBuf, findings: &[Finding]) -> i32 {
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

fn audit_config(config: &Value) -> Vec<Finding> {
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

fn str_value<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

fn bool_value(value: &Value, path: &[&str]) -> bool {
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
        let args = parse_args(["--config", "custom.toml"]).unwrap();
        assert_eq!(args.config, Some(PathBuf::from("custom.toml")));

        let args = parse_args(["--print-ws-config"]).unwrap();
        assert!(args.print_ws_config);
    }

    #[test]
    fn rejects_conflicting_template_args() {
        let err = parse_args(["--print-ws-config", "--print-http-config"]).unwrap_err();
        assert!(err.contains("choose only one"));
    }

    #[test]
    fn good_config_has_no_cache_warnings_when_env_is_present() {
        env::set_var("XAI_API_KEY", "redacted");
        let config = GOOD_CONFIG.parse::<Value>().unwrap();
        let findings = audit_config(&config);
        assert!(!findings.iter().any(|finding| finding.level == "WARN"));
    }

    #[test]
    fn bad_config_warns_about_cache_unfriendly_settings() {
        env::remove_var("OPENAI_API_KEY");
        let config = BAD_CONFIG.parse::<Value>().unwrap();
        let findings = audit_config(&config);
        assert!(findings.iter().any(|finding| finding.level == "WARN"));
        assert!(findings
            .iter()
            .any(|finding| finding.message.contains("not \"responses\"")));
    }
}
