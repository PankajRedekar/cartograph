use regex::Regex;
use serde_json::Value;

/// Parsed crash input metadata and extracted raw frames.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCrash {
    pub crash_type: String,
    pub crash_message: String,
    pub frames: Vec<RawFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFrame {
    pub file_path: String,
    pub line_number: usize,
    pub column: Option<usize>,
    pub function_name: String,
    pub code_snippet: Option<String>,
}

/// Universal Polyglot Traceback & Crash Log Parser.
pub struct TracebackParser;

impl TracebackParser {
    /// Ingests arbitrary crash logs, stack traces, or Sentry JSON.
    pub fn parse(input: &str) -> ParsedCrash {
        let trimmed = input.trim();

        // 1. Try Sentry / APM JSON
        if ((trimmed.starts_with('{') && trimmed.ends_with('}'))
            || (trimmed.starts_with('[') && trimmed.ends_with(']')))
            && let Ok(json_val) = serde_json::from_str::<Value>(trimmed)
            && let Some(parsed) = Self::parse_sentry_json(&json_val)
        {
            return parsed;
        }

        // 2. Try Python traceback
        if (trimmed.contains("Traceback (most recent call last)") || trimmed.contains("File \""))
            && let Some(parsed) = Self::parse_python_traceback(trimmed)
        {
            return parsed;
        }

        // 3. Try Rust backtrace / panic
        if (trimmed.contains("panicked at") || trimmed.contains("stack backtrace:"))
            && let Some(parsed) = Self::parse_rust_backtrace(trimmed)
        {
            return parsed;
        }

        // 4. Try JS / TS stack trace
        if (trimmed.contains("    at ")
            || trimmed.contains("TypeError:")
            || trimmed.contains("Error:"))
            && let Some(parsed) = Self::parse_js_stacktrace(trimmed)
        {
            return parsed;
        }

        // 5. Try Go panic trace
        if (trimmed.contains("panic:") || trimmed.contains("goroutine "))
            && let Some(parsed) = Self::parse_go_panic(trimmed)
        {
            return parsed;
        }

        // 6. Try Java stack trace
        if (trimmed.contains("\tat ") || trimmed.contains("Exception in thread"))
            && let Some(parsed) = Self::parse_java_stacktrace(trimmed)
        {
            return parsed;
        }

        // 7. Generic fallback parser
        Self::parse_generic_trace(trimmed)
    }

    fn parse_sentry_json(val: &Value) -> Option<ParsedCrash> {
        let exception = val
            .get("exception")
            .and_then(|e| e.get("values"))
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .or_else(|| {
                val.get("exception")?
                    .as_object()
                    .map(|_| val.get("exception").unwrap())
            })?;

        let crash_type = exception
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("Error")
            .to_string();

        let crash_message = exception
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let frames_val = exception
            .get("stacktrace")
            .and_then(|s| s.get("frames"))
            .and_then(|f| f.as_array())?;

        let mut frames = Vec::new();
        for f in frames_val {
            let filename = f.get("filename").and_then(|s| s.as_str()).unwrap_or("");
            let lineno = f.get("lineno").and_then(|n| n.as_u64()).unwrap_or(1) as usize;
            let colno = f.get("colno").and_then(|n| n.as_u64()).map(|n| n as usize);
            let function = f
                .get("function")
                .and_then(|s| s.as_str())
                .unwrap_or("<unknown>")
                .to_string();
            let context_line = f
                .get("context_line")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string());

            if !filename.is_empty() {
                frames.push(RawFrame {
                    file_path: filename.to_string(),
                    line_number: lineno,
                    column: colno,
                    function_name: function,
                    code_snippet: context_line,
                });
            }
        }

        if !frames.is_empty() {
            Some(ParsedCrash {
                crash_type,
                crash_message,
                frames,
            })
        } else {
            None
        }
    }

    fn parse_python_traceback(text: &str) -> Option<ParsedCrash> {
        let frame_re = Regex::new(r#"File "([^"]+)", line (\d+)(?:, in (.+))?"#).ok()?;
        let lines: Vec<&str> = text.lines().collect();
        let mut frames = Vec::new();
        let mut crash_type = "RuntimeError".to_string();
        let mut crash_message = String::new();

        for i in 0..lines.len() {
            let line = lines[i];
            if let Some(caps) = frame_re.captures(line) {
                let file_path = caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let line_num: usize = caps
                    .get(2)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(1);
                let func_name = caps
                    .get(3)
                    .map(|m| m.as_str().trim().to_string())
                    .unwrap_or_else(|| "<module>".to_string());

                let code_snippet = if i + 1 < lines.len() {
                    let next = lines[i + 1].trim();
                    if !next.starts_with("File ") && !next.ends_with(':') {
                        Some(next.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };

                frames.push(RawFrame {
                    file_path,
                    line_number: line_num,
                    column: None,
                    function_name: func_name,
                    code_snippet,
                });
            } else if line.contains(':')
                && !line.trim().starts_with("File")
                && !line.starts_with("  ")
            {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 && !parts[0].contains('/') && !parts[0].contains('\\') {
                    let ctype = parts[0].trim();
                    if !ctype.is_empty() && ctype.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        crash_type = ctype.to_string();
                        crash_message = parts[1].trim().to_string();
                    }
                }
            }
        }

        if !frames.is_empty() {
            Some(ParsedCrash {
                crash_type,
                crash_message,
                frames,
            })
        } else {
            None
        }
    }

    fn parse_rust_backtrace(text: &str) -> Option<ParsedCrash> {
        let panic_re =
            Regex::new(r#"panicked at (?:'([^']+)', )?([^:]+):(\d+)(?::(\d+))?"#).ok()?;
        let at_re = Regex::new(r#"^\s*(?:\d+:)?\s*(?:at\s+)?([^\s:]+):(\d+)(?::(\d+))?"#).ok()?;
        let func_re = Regex::new(r#"^\s*\d+:\s*([^\s]+)"#).ok()?;

        let crash_type = "panic".to_string();
        let mut crash_message = String::new();
        let mut frames = Vec::new();

        let lines: Vec<&str> = text.lines().collect();

        // Check for panicked at line
        for line in &lines {
            if let Some(caps) = panic_re.captures(line) {
                crash_message = caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(|| "Explicit panic triggered".to_string());
                let file = caps
                    .get(2)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let line_num: usize = caps
                    .get(3)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(1);
                let col: Option<usize> = caps.get(4).and_then(|m| m.as_str().parse().ok());

                frames.push(RawFrame {
                    file_path: file,
                    line_number: line_num,
                    column: col,
                    function_name: "panic_site".to_string(),
                    code_snippet: None,
                });
                break;
            }
        }

        // Parse stack backtrace lines
        let mut current_func = "<unknown>".to_string();
        for line in &lines {
            if let Some(caps) = func_re.captures(line) {
                current_func = caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
            } else if let Some(caps) = at_re.captures(line) {
                let file = caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let line_num: usize = caps
                    .get(2)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(1);
                let col: Option<usize> = caps.get(3).and_then(|m| m.as_str().parse().ok());

                if !file.is_empty() && (file.ends_with(".rs") || !file.contains('/')) {
                    frames.push(RawFrame {
                        file_path: file,
                        line_number: line_num,
                        column: col,
                        function_name: current_func.clone(),
                        code_snippet: None,
                    });
                }
            }
        }

        if !frames.is_empty() {
            Some(ParsedCrash {
                crash_type,
                crash_message,
                frames,
            })
        } else {
            None
        }
    }

    fn parse_js_stacktrace(text: &str) -> Option<ParsedCrash> {
        let frame_re =
            Regex::new(r#"^\s*at (?:(.+?)\s+\((.+?):(\d+):(\d+)\)|(.+?):(\d+):(\d+))"#).ok()?;

        let mut crash_type = "Error".to_string();
        let mut crash_message = String::new();
        let mut frames = Vec::new();

        let lines: Vec<&str> = text.lines().collect();

        // First line often "TypeError: message" or "Error: message"
        if let Some(first_line) = lines.first()
            && first_line.contains(':')
            && !first_line.trim().starts_with("at ")
        {
            let parts: Vec<&str> = first_line.splitn(2, ':').collect();
            crash_type = parts[0].trim().to_string();
            if parts.len() > 1 {
                crash_message = parts[1].trim().to_string();
            }
        }

        for line in &lines {
            if let Some(caps) = frame_re.captures(line) {
                if let (Some(func), Some(file), Some(line_s), col_s) =
                    (caps.get(1), caps.get(2), caps.get(3), caps.get(4))
                {
                    let line_num: usize = line_s.as_str().parse().unwrap_or(1);
                    let col: Option<usize> = col_s.and_then(|c| c.as_str().parse().ok());
                    frames.push(RawFrame {
                        file_path: file.as_str().to_string(),
                        line_number: line_num,
                        column: col,
                        function_name: func.as_str().to_string(),
                        code_snippet: None,
                    });
                } else if let (Some(file), Some(line_s), col_s) =
                    (caps.get(5), caps.get(6), caps.get(7))
                {
                    let line_num: usize = line_s.as_str().parse().unwrap_or(1);
                    let col: Option<usize> = col_s.and_then(|c| c.as_str().parse().ok());
                    frames.push(RawFrame {
                        file_path: file.as_str().to_string(),
                        line_number: line_num,
                        column: col,
                        function_name: "<anonymous>".to_string(),
                        code_snippet: None,
                    });
                }
            }
        }

        if !frames.is_empty() {
            Some(ParsedCrash {
                crash_type,
                crash_message,
                frames,
            })
        } else {
            None
        }
    }

    fn parse_go_panic(text: &str) -> Option<ParsedCrash> {
        let panic_re = Regex::new(r#"panic:\s*(.+)"#).ok()?;
        let frame_loc_re = Regex::new(r#"^\s*(.+):(\d+)(?:\s+\+0x[0-9a-fA-F]+)?"#).ok()?;

        let crash_type = "panic".to_string();
        let mut crash_message = String::new();
        let mut frames = Vec::new();

        let lines: Vec<&str> = text.lines().collect();

        for i in 0..lines.len() {
            let line = lines[i];
            if let Some(caps) = panic_re.captures(line) {
                crash_message = caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
            } else if let Some(caps) = frame_loc_re.captures(line) {
                let file = caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let line_num: usize = caps
                    .get(2)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(1);
                let func_name = if i > 0 {
                    lines[i - 1].trim().to_string()
                } else {
                    "<unknown>".to_string()
                };

                frames.push(RawFrame {
                    file_path: file,
                    line_number: line_num,
                    column: None,
                    function_name: func_name,
                    code_snippet: None,
                });
            }
        }

        if !frames.is_empty() {
            Some(ParsedCrash {
                crash_type,
                crash_message,
                frames,
            })
        } else {
            None
        }
    }

    fn parse_java_stacktrace(text: &str) -> Option<ParsedCrash> {
        let ex_re = Regex::new(r#"^([a-zA-Z0-9_$.]+(?:Exception|Error))(?::\s*(.*))?"#).ok()?;
        let at_re =
            Regex::new(r#"^\s*at\s+([a-zA-Z0-9_$.]+)\.([a-zA-Z0-9_$<]+)\(([^:]+):(\d+)\)"#).ok()?;

        let mut crash_type = "Exception".to_string();
        let mut crash_message = String::new();
        let mut frames = Vec::new();

        let lines: Vec<&str> = text.lines().collect();

        for line in &lines {
            if let Some(caps) = ex_re.captures(line) {
                crash_type = caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                crash_message = caps
                    .get(2)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
            } else if let Some(caps) = at_re.captures(line) {
                let class_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let method_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let file_name = caps.get(3).map(|m| m.as_str()).unwrap_or("");
                let line_num: usize = caps
                    .get(4)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(1);

                frames.push(RawFrame {
                    file_path: file_name.to_string(),
                    line_number: line_num,
                    column: None,
                    function_name: format!("{class_name}.{method_name}"),
                    code_snippet: None,
                });
            }
        }

        if !frames.is_empty() {
            Some(ParsedCrash {
                crash_type,
                crash_message,
                frames,
            })
        } else {
            None
        }
    }

    fn parse_generic_trace(text: &str) -> ParsedCrash {
        let loc_re = Regex::new(r#"([a-zA-Z0-9_./\\-]+\.[a-zA-Z0-9]+):(\d+)(?::(\d+))?"#).unwrap();
        let mut frames = Vec::new();

        for line in text.lines() {
            if let Some(caps) = loc_re.captures(line) {
                let file = caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let line_num: usize = caps
                    .get(2)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(1);
                let col: Option<usize> = caps.get(3).and_then(|m| m.as_str().parse().ok());

                frames.push(RawFrame {
                    file_path: file,
                    line_number: line_num,
                    column: col,
                    function_name: "<unknown>".to_string(),
                    code_snippet: None,
                });
            }
        }

        ParsedCrash {
            crash_type: "Error".to_string(),
            crash_message: text
                .lines()
                .next()
                .unwrap_or("Unknown error")
                .trim()
                .to_string(),
            frames,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_python_traceback() {
        let trace = r#"
Traceback (most recent call last):
  File "src/main.py", line 12, in <module>
    app.run()
  File "src/app.py", line 45, in run
    service.process_order(order_id)
  File "src/services/order.py", line 88, in process_order
    user = repo.get_user(order.user_id)
  File "src/services/repo.py", line 22, in get_user
    return record["user"]
KeyError: 'user'
"#;

        let parsed = TracebackParser::parse(trace);
        assert_eq!(parsed.crash_type, "KeyError");
        assert_eq!(parsed.crash_message, "'user'");
        assert_eq!(parsed.frames.len(), 4);
        assert_eq!(parsed.frames[0].file_path, "src/main.py");
        assert_eq!(parsed.frames[0].line_number, 12);
        assert_eq!(parsed.frames[3].file_path, "src/services/repo.py");
        assert_eq!(parsed.frames[3].line_number, 22);
    }

    #[test]
    fn test_parse_js_stacktrace() {
        let trace = r#"
TypeError: Cannot read properties of undefined (reading 'calculateTotal')
    at processCart (/app/src/billing.ts:34:18)
    at checkout (/app/src/checkout.ts:12:5)
    at /app/src/server.ts:80:10
"#;

        let parsed = TracebackParser::parse(trace);
        assert_eq!(parsed.crash_type, "TypeError");
        assert_eq!(
            parsed.crash_message,
            "Cannot read properties of undefined (reading 'calculateTotal')"
        );
        assert_eq!(parsed.frames.len(), 3);
        assert_eq!(parsed.frames[0].file_path, "/app/src/billing.ts");
        assert_eq!(parsed.frames[0].line_number, 34);
        assert_eq!(parsed.frames[0].column, Some(18));
    }

    #[test]
    fn test_parse_rust_backtrace() {
        let trace = r#"
thread 'main' panicked at 'called `Option::unwrap()` on a `None` value', src/core/engine.rs:104:14
stack backtrace:
   0: std::panicking::begin_panic
   1: cartograph::core::engine::run
             at src/core/engine.rs:104:14
   2: cartograph::main
             at src/main.rs:25:5
"#;

        let parsed = TracebackParser::parse(trace);
        assert_eq!(parsed.crash_type, "panic");
        assert!(parsed.crash_message.contains("Option::unwrap()"));
        assert!(!parsed.frames.is_empty());
        assert_eq!(parsed.frames[0].file_path, "src/core/engine.rs");
        assert_eq!(parsed.frames[0].line_number, 104);
    }

    #[test]
    fn test_parse_sentry_json() {
        let sentry_json = r#"{
            "exception": {
                "values": [{
                    "type": "NullPointerException",
                    "value": "user object is null",
                    "stacktrace": {
                        "frames": [
                            {"filename": "app/controller.py", "lineno": 15, "function": "handle_request"},
                            {"filename": "app/service.py", "lineno": 42, "function": "get_profile"}
                        ]
                    }
                }]
            }
        }"#;

        let parsed = TracebackParser::parse(sentry_json);
        assert_eq!(parsed.crash_type, "NullPointerException");
        assert_eq!(parsed.crash_message, "user object is null");
        assert_eq!(parsed.frames.len(), 2);
        assert_eq!(parsed.frames[1].function_name, "get_profile");
    }
}
