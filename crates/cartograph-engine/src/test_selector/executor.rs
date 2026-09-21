use cartograph_core::error::{CartographError, Result};
use cartograph_core::model::TestRunnerInvocation;
use colored::*;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    pub total_invocations: usize,
    pub successful_invocations: usize,
    pub failed_invocations: usize,
    pub exit_code: i32,
    pub duration_seconds: f64,
}

pub struct TestExecutor;

impl TestExecutor {
    /// Execute synthesized runner commands sequentially with live streaming and bail support
    pub fn execute(invocations: &[TestRunnerInvocation], bail: bool) -> Result<ExecutionSummary> {
        if invocations.is_empty() {
            println!("{}", "No tests selected to execute.".dimmed());
            return Ok(ExecutionSummary {
                total_invocations: 0,
                successful_invocations: 0,
                failed_invocations: 0,
                exit_code: 0,
                duration_seconds: 0.0,
            });
        }

        let start = Instant::now();
        let mut successful = 0;
        let mut failed = 0;
        let mut final_exit_code = 0;

        println!(
            "{}",
            format!(
                "🚀 Executing {} test runner invocation(s)...",
                invocations.len()
            )
            .bold()
            .cyan()
        );

        for (idx, invocation) in invocations.iter().enumerate() {
            println!(
                "\n{} [{}/{}] {}",
                "▶".bold().blue(),
                idx + 1,
                invocations.len(),
                invocation.display_command.bold()
            );

            let mut cmd = Command::new(&invocation.program);
            cmd.args(&invocation.args);
            cmd.current_dir(&invocation.working_dir);

            // Inherit stdout/stderr to stream live runner output to user's terminal
            cmd.stdout(std::process::Stdio::inherit());
            cmd.stderr(std::process::Stdio::inherit());

            let status = cmd.status().map_err(|e| {
                CartographError::Other(format!(
                    "Failed to launch process '{}': {e}",
                    invocation.program
                ))
            })?;

            if status.success() {
                successful += 1;
                println!(
                    "{} Invocations [{}/{}] passed.",
                    "✔".bold().green(),
                    idx + 1,
                    invocations.len()
                );
            } else {
                failed += 1;
                let code = status.code().unwrap_or(1);
                final_exit_code = code;
                println!(
                    "{} Invocation failed with exit code {}.",
                    "✖".bold().red(),
                    code
                );

                if bail {
                    println!(
                        "{}",
                        "🛑 --bail enabled: aborting remaining test runs immediately."
                            .bold()
                            .yellow()
                    );
                    break;
                }
            }
        }

        let duration = start.elapsed().as_secs_f64();

        Ok(ExecutionSummary {
            total_invocations: invocations.len(),
            successful_invocations: successful,
            failed_invocations: failed,
            exit_code: final_exit_code,
            duration_seconds: duration,
        })
    }
}
