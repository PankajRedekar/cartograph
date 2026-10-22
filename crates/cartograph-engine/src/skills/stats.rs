use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub command: String,
    pub tokens_saved: usize,
    pub file_reads_avoided: usize,
    pub tests_avoided: usize,
    pub test_minutes_saved: f64,
    pub regressions_caught: usize,
    pub latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoiStatsReport {
    pub total_agent_invocations: usize,
    pub prompt_tokens_saved: usize,
    pub estimated_cost_usd_saved: f64,
    pub full_file_reads_avoided: usize,
    pub tests_avoided: usize,
    pub test_time_saved_minutes: f64,
    pub pre_commit_regressions_caught: usize,
    pub average_latency_ms: f64,
}

impl RoiStatsReport {
    pub fn format_text(&self) -> String {
        format!(
            "┌────────────────────────────────────────────────────────────────────────┐\n\
             │               📊 Cartograph Agent Efficiency & ROI Report              │\n\
             ├────────────────────────────────────────────────────────────────────────┤\n\
             │ • Total Agent Invocations:     {:>6} commands                         │\n\
             │ • Prompt Tokens Saved:         {:>6} tokens (~${:.2} in API costs)    │\n\
             │ • Full-File Reads Avoided:     {:>6} redundant file reads             │\n\
             │ • Unaffected Tests Avoided:    {:>6} test cases (saved ~{:.1} mins)  │\n\
             │ • Contract Regressions Caught: {:>6} breaking changes prevented       │\n\
             │ • Average CLI Latency:         {:>6.1}ms per query                      │\n\
             └────────────────────────────────────────────────────────────────────────┘",
            self.total_agent_invocations,
            self.prompt_tokens_saved,
            self.estimated_cost_usd_saved,
            self.full_file_reads_avoided,
            self.tests_avoided,
            self.test_time_saved_minutes,
            self.pre_commit_regressions_caught,
            self.average_latency_ms
        )
    }

    pub fn format_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn format_xml(&self) -> String {
        format!(
            "<agent_roi_stats recorded_sessions=\"{}\">\n  \
               <token_savings total_tokens_saved=\"{}\" estimated_cost_usd=\"{:.2}\" />\n  \
               <productivity full_file_reads_avoided=\"{}\" tests_avoided=\"{}\" test_execution_time_saved_minutes=\"{:.1}\" />\n  \
               <quality_gates pre_commit_regressions_caught=\"{}\" />\n  \
               <performance average_cli_latency_ms=\"{:.1}\" />\n\
             </agent_roi_stats>",
            self.total_agent_invocations,
            self.prompt_tokens_saved,
            self.estimated_cost_usd_saved,
            self.full_file_reads_avoided,
            self.tests_avoided,
            self.test_time_saved_minutes,
            self.pre_commit_regressions_caught,
            self.average_latency_ms
        )
    }
}

pub struct TelemetryRecorder;

impl TelemetryRecorder {
    fn open_db(root: &Path) -> rusqlite::Result<rusqlite::Connection> {
        let db_dir = root.join(".cartograph");
        let _ = fs::create_dir_all(&db_dir);
        let db_path = db_dir.join("telemetry.db");
        let conn = rusqlite::Connection::open(&db_path)?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS telemetry_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                command TEXT NOT NULL,
                tokens_saved INTEGER NOT NULL,
                file_reads_avoided INTEGER NOT NULL,
                tests_avoided INTEGER NOT NULL,
                test_minutes_saved REAL NOT NULL,
                regressions_caught INTEGER NOT NULL,
                latency_ms REAL NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_telemetry_timestamp ON telemetry_events(timestamp);",
        )?;

        Ok(conn)
    }

    pub fn record_event(root: &Path, event: &TelemetryEvent) -> rusqlite::Result<()> {
        let conn = Self::open_db(root)?;
        conn.execute(
            "INSERT INTO telemetry_events (command, tokens_saved, file_reads_avoided, tests_avoided, test_minutes_saved, regressions_caught, latency_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                event.command,
                event.tokens_saved as i64,
                event.file_reads_avoided as i64,
                event.tests_avoided as i64,
                event.test_minutes_saved,
                event.regressions_caught as i64,
                event.latency_ms,
            ],
        )?;
        Ok(())
    }

    pub fn get_roi_stats(root: &Path, since: Option<&str>) -> rusqlite::Result<RoiStatsReport> {
        let conn = Self::open_db(root)?;

        let time_filter = match since {
            Some("24h") => "WHERE timestamp >= datetime('now', '-1 day')",
            Some("7d") => "WHERE timestamp >= datetime('now', '-7 days')",
            Some("30d") => "WHERE timestamp >= datetime('now', '-30 days')",
            _ => "",
        };

        let sql = format!(
            "SELECT count(*),
                    coalesce(sum(tokens_saved), 0),
                    coalesce(sum(file_reads_avoided), 0),
                    coalesce(sum(tests_avoided), 0),
                    coalesce(sum(test_minutes_saved), 0.0),
                    coalesce(sum(regressions_caught), 0),
                    coalesce(avg(latency_ms), 0.0)
             FROM telemetry_events {}",
            time_filter
        );

        let report = conn.query_row(&sql, [], |row| {
            let total_invocations: i64 = row.get(0)?;
            let total_tokens: i64 = row.get(1)?;
            let file_reads: i64 = row.get(2)?;
            let tests_avoided: i64 = row.get(3)?;
            let minutes_saved: f64 = row.get(4)?;
            let regressions: i64 = row.get(5)?;
            let avg_latency: f64 = row.get(6)?;

            let estimated_usd = (total_tokens as f64 / 1_000_000.0) * 3.0; // ~$3/M tokens

            Ok(RoiStatsReport {
                total_agent_invocations: total_invocations as usize,
                prompt_tokens_saved: total_tokens as usize,
                estimated_cost_usd_saved: estimated_usd,
                full_file_reads_avoided: file_reads as usize,
                tests_avoided: tests_avoided as usize,
                test_time_saved_minutes: minutes_saved,
                pre_commit_regressions_caught: regressions as usize,
                average_latency_ms: avg_latency,
            })
        })?;

        Ok(report)
    }

    pub fn reset(root: &Path) -> rusqlite::Result<()> {
        let conn = Self::open_db(root)?;
        conn.execute("DELETE FROM telemetry_events", [])?;
        conn.execute("VACUUM", [])?;
        Ok(())
    }
}
