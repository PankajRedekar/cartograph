// crates/cartograph-cli/src/docs.rs
//! Automated CLI Reference Generator and Documentation Tooling for Cartograph.

use clap::Command;
use colored::Colorize;
use std::fs;
use std::path::Path;

/// Generate exhaustive MyST markdown manual pages for all Cartograph CLI commands.
pub fn generate_cli_reference<P: AsRef<Path>>(
    root_cmd: &Command,
    out_dir: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;

    let mut command_summaries = Vec::new();

    // Iterate through all top-level subcommands
    for subcmd in root_cmd.get_subcommands() {
        let name = subcmd.get_name();
        // Skip hidden commands
        if subcmd.is_hide_set() {
            continue;
        }

        let about = subcmd
            .get_about()
            .map(|s| s.to_string())
            .unwrap_or_default();
        command_summaries.push((name.to_string(), about.clone()));

        let markdown = generate_subcommand_markdown("cartograph", subcmd);
        let filename = format!("cartograph-{}.md", name);
        let file_path = out_dir.join(&filename);
        fs::write(&file_path, markdown)?;
        println!(
            "  {} Generated CLI reference page: {}",
            "✔".green(),
            filename.cyan()
        );
    }

    // Generate index.md for CLI reference
    let index_content = generate_cli_index_markdown(&command_summaries);
    let index_path = out_dir.join("index.md");
    fs::write(&index_path, index_content)?;
    println!(
        "  {} Generated CLI reference catalog: {}",
        "✔".green(),
        "index.md".cyan()
    );

    Ok(())
}

fn generate_subcommand_markdown(parent: &str, cmd: &Command) -> String {
    let name = cmd.get_name();
    let full_name = format!("{} {}", parent, name);
    let about = cmd.get_about().map(|s| s.to_string()).unwrap_or_default();
    let long_about = cmd
        .get_long_about()
        .map(|s| s.to_string())
        .unwrap_or_else(|| about.clone());

    let mut out = String::new();
    out.push_str(&format!("# `{}`\n\n", full_name));
    out.push_str(&format!("{}\n\n", long_about));

    // Synopsis / Usage
    out.push_str("## Synopsis\n\n```bash\n");
    let mut usage = format!("cartograph {}", name);
    let mut positional_args = Vec::new();
    let mut options = Vec::new();

    for arg in cmd.get_arguments() {
        if arg.is_hide_set() {
            continue;
        }
        if arg.is_positional() {
            positional_args.push(arg);
        } else {
            options.push(arg);
        }
    }

    if !options.is_empty() {
        usage.push_str(" [OPTIONS]");
    }
    for pos in &positional_args {
        let arg_name = pos.get_id().as_str();
        if pos.is_required_set() {
            usage.push_str(&format!(" <{}>", arg_name));
        } else {
            usage.push_str(&format!(" [<{}>]", arg_name));
        }
    }
    out.push_str(&usage);
    out.push_str("\n```\n\n");

    // Positional Arguments
    if !positional_args.is_empty() {
        out.push_str("## Arguments\n\n");
        out.push_str("| Argument | Description | Required |\n");
        out.push_str("| :--- | :--- | :--- |\n");
        for arg in positional_args {
            let id = arg.get_id().as_str();
            let help = arg.get_help().map(|s| s.to_string()).unwrap_or_default();
            let req = if arg.is_required_set() { "Yes" } else { "No" };
            out.push_str(&format!("| `<{}>` | {} | {} |\n", id, help, req));
        }
        out.push('\n');
    }

    // Options and Flags
    if !options.is_empty() {
        out.push_str("## Options & Flags\n\n");
        out.push_str("| Flag | Name | Value | Default | Description |\n");
        out.push_str("| :--- | :--- | :--- | :--- | :--- |\n");

        for opt in options {
            let short = opt
                .get_short()
                .map(|c| format!("`-{}`", c))
                .unwrap_or_else(|| "-".to_string());
            let long = opt
                .get_long()
                .map(|l| format!("`--{}`", l))
                .unwrap_or_else(|| "-".to_string());
            let val = opt
                .get_value_names()
                .map(|v| v.join(","))
                .unwrap_or_else(|| "flag".to_string());
            let default_val = opt
                .get_default_values()
                .iter()
                .map(|v| v.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join(", ");
            let def_str = if default_val.is_empty() {
                "-".to_string()
            } else {
                format!("`{}`", default_val)
            };
            let help = opt.get_help().map(|s| s.to_string()).unwrap_or_default();

            out.push_str(&format!(
                "| {} | {} | `{}` | {} | {} |\n",
                short, long, val, def_str, help
            ));
        }
        out.push('\n');
    }

    // Examples Section
    out.push_str("## Examples\n\n");
    out.push_str("::::{tab-set}\n");
    out.push_str(":::{tab-item} Basic Command\n");
    out.push_str(&format!("```bash\ncartograph {}\n```\n", name));
    out.push_str(":::\n\n");

    out.push_str(":::{tab-item} JSON Output\n");
    out.push_str(&format!("```bash\ncartograph {} --json\n```\n", name));
    out.push_str(":::\n");
    out.push_str("::::\n");

    out
}

fn generate_cli_index_markdown(commands: &[(String, String)]) -> String {
    let mut out = String::new();
    out.push_str("# 🛠️ CLI Command Reference\n\n");
    out.push_str(
        "Exhaustive reference documentation for all Cartograph CLI commands, options, flags, and exit codes.\n\n",
    );

    out.push_str("| Command | Description |\n");
    out.push_str("| :--- | :--- |\n");
    for (name, about) in commands {
        out.push_str(&format!(
            "| [`cartograph {}`](cartograph-{}.md) | {} |\n",
            name, name, about
        ));
    }
    out.push_str("\n\n```{toctree}\n:maxdepth: 1\n:hidden:\n\n");
    for (name, _) in commands {
        out.push_str(&format!("cartograph-{}\n", name));
    }
    out.push_str("```\n");

    out
}

/// Headless documentation emitter for AI coding agents.
pub fn emit_agent_docs(
    section: Option<&str>,
    max_tokens: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let max_tokens = max_tokens.unwrap_or(2500);

    match section {
        Some("locate") => {
            println!(
                r#"<cartograph_docs section="locate" budget="{max_tokens}">
  <summary>Localize and retrieve 1-turn AST code subgraphs in &lt;3ms.</summary>
  <usage>cartograph locate &lt;QUERY&gt; [--path &lt;PATH&gt;] [--kind &lt;KIND&gt;] [--depth &lt;D&gt;] [--format xml|json]</usage>
  <options>
    <option flag="--depth" type="int" default="1" desc="BFS traversal depth"/>
    <option flag="--flow" type="bool" desc="Enable end-to-end execution flow tracing"/>
    <option flag="--resolve-types" type="bool" desc="Inline parameter and return types"/>
    <option flag="--format" type="string" default="xml" desc="Output format: xml, json, terminal"/>
  </options>
</cartograph_docs>"#
            );
        }
        Some("impact") => {
            println!(
                r#"<cartograph_docs section="impact" budget="{max_tokens}">
  <summary>Analyze blast radius, breaking changes, and risk of code modifications in &lt;25ms.</summary>
  <usage>cartograph impact --symbol &lt;NAME&gt; [--depth &lt;D&gt;] [--fail-on &lt;LEVEL&gt;] [--format xml|json]</usage>
  <options>
    <option flag="--symbol" type="string" desc="Target symbol query"/>
    <option flag="--depth" type="int" default="3" desc="Maximum graph traversal depth"/>
    <option flag="--verify-semver" type="string" desc="Enforce SemVer policy (patch, minor, major)"/>
    <option flag="--fail-on" type="string" desc="Fail CI if risk meets threshold (low, medium, high, critical)"/>
  </options>
</cartograph_docs>"#
            );
        }
        Some("verify") => {
            println!(
                r#"<cartograph_docs section="verify" budget="{max_tokens}">
  <summary>Pre-commit closed-loop verification gate (syntax, arch, impact &amp; tests) in &lt;200ms.</summary>
  <usage>cartograph verify [--staged] [--branch &lt;RANGE&gt;] [--fail-fast] [--format xml|json]</usage>
  <options>
    <option flag="--staged" type="bool" default="true" desc="Verify Git staged modifications"/>
    <option flag="--branch" type="string" desc="Verify modifications against base branch"/>
    <option flag="--fail-fast" type="bool" desc="Halt immediately upon first failing check"/>
  </options>
</cartograph_docs>"#
            );
        }
        _ => {
            println!(
                r#"<cartograph_docs section="index" budget="{max_tokens}">
  <summary>Cartograph: Ultra-fast native AST code graph and developer intelligence engine.</summary>
  <commands count="14">
    <cmd name="locate" desc="AST symbol search replacing grep loops"/>
    <cmd name="impact" desc="Blast-radius calculation and breaking change detection"/>
    <cmd name="test-selector" desc="Predictive test selection and execution"/>
    <cmd name="routes" desc="Full-stack HTTP API route and ORM mapping"/>
    <cmd name="lint-arch" desc="Architectural boundary and circular dependency checker"/>
    <cmd name="complexity" desc="Cognitive complexity heatmap and God-class detection"/>
    <cmd name="deadcode" desc="Mark-and-sweep reachability and dead code pruning"/>
    <cmd name="ai-review" desc="Contract-aware PR reviews and patch application"/>
    <cmd name="extract-logic" desc="Domain business logic and Gherkin scenario extractor"/>
    <cmd name="diagnose" desc="Stack trace AST slicer and test reproducer generator"/>
    <cmd name="diagram" desc="Visual architecture and callgraph diagram exporter"/>
    <cmd name="skills" desc="Multi-IDE agent skill installer and doctor"/>
    <cmd name="verify" desc="Closed-loop pre-commit self-healing gate"/>
    <cmd name="config" desc="Configuration validation and profile manager"/>
  </commands>
</cartograph_docs>"#
            );
        }
    }

    Ok(())
}
