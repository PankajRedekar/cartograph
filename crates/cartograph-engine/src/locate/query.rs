use cartograph_core::{EdgeKind, NodeKind};
use regex::Regex;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryIntent {
    /// Exact canonical URI: cartograph://path::scope::name#kind
    CanonicalUri(String),
    /// Qualified symbol: scope + name (e.g. AuthService::verify_token, Order.save)
    QualifiedSymbol { scope: String, name: String },
    /// Exact unqualified identifier: RateCard, calculate_tax
    ExactSymbol(String),
    /// Scoped path prefix or glob: src/billing/*.rs, crates/engine/
    PathFilter(String),
    /// Natural language or multi-word search phrase: "accrual period update"
    SemanticIntent(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalMode {
    Radial,
    FlowChain,
}

impl FromStr for TraversalMode {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "flow" | "flowchain" | "chain" => Ok(TraversalMode::FlowChain),
            _ => Ok(TraversalMode::Radial),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalDirection {
    Both,
    DownstreamOnly,
    UpstreamOnly,
}

impl FromStr for TraversalDirection {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "down" | "downstream" | "outgoing" => Ok(TraversalDirection::DownstreamOnly),
            "up" | "upstream" | "incoming" => Ok(TraversalDirection::UpstreamOnly),
            _ => Ok(TraversalDirection::Both),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextDetail {
    Minimal,
    Signatures,
    Full,
}

impl FromStr for ContextDetail {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "minimal" | "min" | "tree" => Ok(ContextDetail::Minimal),
            "full" | "code" => Ok(ContextDetail::Full),
            _ => Ok(ContextDetail::Signatures),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Terminal,
    InteractiveTui,
    Markdown,
    Json,
    AgentXml,
}

impl FromStr for OutputFormat {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            "json" | "jsonl" => Ok(OutputFormat::Json),
            "xml" | "agent-xml" | "agent_xml" => Ok(OutputFormat::AgentXml),
            "interactive" | "tui" => Ok(OutputFormat::InteractiveTui),
            _ => Ok(OutputFormat::Terminal),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LocateCliOptions {
    pub kind: Option<String>,
    pub path: Option<String>,
    pub lang: Option<String>,
    pub depth: Option<usize>,
    pub flow: bool,
    pub direction: Option<String>,
    pub edges: Option<String>,
    pub min_confidence: Option<f64>,
    pub resolve_types: bool,
    pub detail: Option<String>,
    pub code: bool,
    pub context_lines: Option<usize>,
    pub max_tokens: Option<usize>,
    pub limit: Option<usize>,
    pub format: Option<String>,
    pub interactive: bool,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct InlineDirectives {
    pub kind: Option<String>,
    pub path: Option<String>,
    pub lang: Option<String>,
    pub min_confidence: Option<f64>,
    pub depth: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct LocateQuery {
    pub raw_query: String,
    pub clean_query: String,
    pub intent: QueryIntent,
    pub kind_filter: Option<Vec<NodeKind>>,
    pub path_filter: Option<String>,
    pub language_filter: Option<String>,
    pub depth: usize,
    pub mode: TraversalMode,
    pub direction: TraversalDirection,
    pub edge_kinds: Option<Vec<EdgeKind>>,
    pub min_confidence: f64,
    pub resolve_types: bool,
    pub detail: ContextDetail,
    pub include_code: bool,
    pub context_lines: usize,
    pub max_tokens: Option<usize>,
    pub limit: usize,
    pub format: OutputFormat,
}

impl Default for LocateQuery {
    fn default() -> Self {
        Self {
            raw_query: String::new(),
            clean_query: String::new(),
            intent: QueryIntent::ExactSymbol(String::new()),
            kind_filter: None,
            path_filter: None,
            language_filter: None,
            depth: 1,
            mode: TraversalMode::Radial,
            direction: TraversalDirection::Both,
            edge_kinds: None,
            min_confidence: 0.0,
            resolve_types: false,
            detail: ContextDetail::Signatures,
            include_code: false,
            context_lines: 3,
            max_tokens: None,
            limit: 5,
            format: OutputFormat::Terminal,
        }
    }
}

pub struct QueryParser;

impl QueryParser {
    /// Parse raw query string and CLI parameters into a validated LocateQuery
    pub fn parse(raw_query: &str, cli_options: LocateCliOptions) -> LocateQuery {
        let (clean_query, inline) = Self::extract_inline_directives(raw_query);
        let intent = Self::classify_intent(&clean_query);

        // Resolve kind filter (inline directive takes precedence over CLI flag)
        let kind_str = inline.kind.or(cli_options.kind);
        let kind_filter = kind_str.map(|k| {
            k.split(',')
                .map(|s| {
                    s.trim()
                        .parse::<NodeKind>()
                        .unwrap_or(NodeKind::Other(s.to_string()))
                })
                .collect()
        });

        // Path filter
        let path_filter = inline.path.or(cli_options.path);

        // Language filter
        let language_filter = inline.lang.or(cli_options.lang);

        // Depth (default 1)
        let depth = inline.depth.or(cli_options.depth).unwrap_or(1);

        // Traversal mode
        let mode = if cli_options.flow {
            TraversalMode::FlowChain
        } else {
            TraversalMode::Radial
        };

        // Direction
        let direction = cli_options
            .direction
            .as_deref()
            .and_then(|d| d.parse::<TraversalDirection>().ok())
            .unwrap_or(TraversalDirection::Both);

        // Edge kinds
        let edge_kinds = cli_options.edges.map(|e| {
            e.split(',')
                .map(|s| {
                    s.trim()
                        .parse::<EdgeKind>()
                        .unwrap_or(EdgeKind::Other(s.to_string()))
                })
                .collect()
        });

        // Minimum confidence
        let min_confidence = inline
            .min_confidence
            .or(cli_options.min_confidence)
            .unwrap_or(0.0);

        // Context detail
        let detail = if cli_options.code {
            ContextDetail::Full
        } else if let Some(ref d) = cli_options.detail {
            d.parse::<ContextDetail>()
                .unwrap_or(ContextDetail::Signatures)
        } else {
            ContextDetail::Signatures
        };
        let include_code = detail == ContextDetail::Full || cli_options.code;

        // Context lines
        let context_lines = cli_options.context_lines.unwrap_or(3);

        // Format
        let format = if cli_options.interactive {
            OutputFormat::InteractiveTui
        } else if let Some(ref f) = cli_options.format {
            f.parse::<OutputFormat>().unwrap_or(OutputFormat::Terminal)
        } else {
            OutputFormat::Terminal
        };

        LocateQuery {
            raw_query: raw_query.to_string(),
            clean_query,
            intent,
            kind_filter,
            path_filter,
            language_filter,
            depth,
            mode,
            direction,
            edge_kinds,
            min_confidence,
            resolve_types: cli_options.resolve_types,
            detail,
            include_code,
            context_lines,
            max_tokens: cli_options.max_tokens,
            limit: cli_options.limit.unwrap_or(5),
            format,
        }
    }

    /// Extract inline directives like `kind:function path:src/ min-conf:0.8`
    pub fn extract_inline_directives(input: &str) -> (String, InlineDirectives) {
        let re = Regex::new(r"(?i)\b(kind|path|lang|min-conf|min_confidence|depth):(\S+)").unwrap();
        let mut directives = InlineDirectives::default();

        for cap in re.captures_iter(input) {
            let key = cap[1].to_lowercase();
            let val = &cap[2];
            match key.as_str() {
                "kind" => directives.kind = Some(val.to_string()),
                "path" => directives.path = Some(val.to_string()),
                "lang" => directives.lang = Some(val.to_string()),
                "min-conf" | "min_confidence" => {
                    directives.min_confidence = val.parse::<f64>().ok();
                }
                "depth" => directives.depth = val.parse::<usize>().ok(),
                _ => {}
            }
        }

        let clean = re.replace_all(input, "").trim().to_string();
        (clean, directives)
    }

    /// Classify query intent
    pub fn classify_intent(query: &str) -> QueryIntent {
        let trimmed = query.trim();
        if trimmed.starts_with("cartograph://") {
            return QueryIntent::CanonicalUri(trimmed.to_string());
        }

        if trimmed.contains("::")
            || trimmed.contains('#')
            || (trimmed.contains('.')
                && !trimmed.contains(' ')
                && !trimmed.ends_with(".rs")
                && !trimmed.ends_with(".py")
                && !trimmed.ends_with(".ts")
                && !trimmed.ends_with(".js")
                && !trimmed.ends_with(".go"))
        {
            // Qualified symbol: split scope and name
            let parts: Vec<&str> = if trimmed.contains("::") {
                trimmed.split("::").collect()
            } else if trimmed.contains('#') {
                trimmed.split('#').collect()
            } else {
                trimmed.split('.').collect()
            };

            if parts.len() >= 2 {
                let scope = parts[..parts.len() - 1].join("::");
                let name = parts.last().unwrap_or(&"").to_string();
                return QueryIntent::QualifiedSymbol { scope, name };
            }
        }

        if (trimmed.contains('/') || trimmed.contains('*')) && !trimmed.contains(' ') {
            return QueryIntent::PathFilter(trimmed.to_string());
        }

        if trimmed.contains(' ') {
            let tokens = Self::tokenize_terms(trimmed);
            return QueryIntent::SemanticIntent(tokens);
        }

        QueryIntent::ExactSymbol(trimmed.to_string())
    }

    /// Split compound identifiers (CamelCase, snake_case, kebab-case)
    pub fn tokenize_terms(query: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        for word in query.split_whitespace() {
            // Split snake_case and kebab-case
            for part in word.split(&['-', '_', ':', '.'][..]) {
                if part.is_empty() {
                    continue;
                }
                // Split CamelCase: detect lowercase followed by uppercase
                let mut current = String::new();
                let mut prev_is_lower = false;

                for ch in part.chars() {
                    if ch.is_uppercase() && prev_is_lower && !current.is_empty() {
                        tokens.push(current.to_lowercase());
                        current = String::new();
                    }
                    prev_is_lower = ch.is_lowercase();
                    current.push(ch);
                }

                if !current.is_empty() {
                    tokens.push(current.to_lowercase());
                }
            }
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_classification() {
        assert_eq!(
            QueryParser::classify_intent("cartograph://src/a.rs::::foo#function"),
            QueryIntent::CanonicalUri("cartograph://src/a.rs::::foo#function".to_string())
        );

        match QueryParser::classify_intent("AuthService::verify_token") {
            QueryIntent::QualifiedSymbol { scope, name } => {
                assert_eq!(scope, "AuthService");
                assert_eq!(name, "verify_token");
            }
            other => panic!("Expected QualifiedSymbol, got {:?}", other),
        }

        match QueryParser::classify_intent("User#save") {
            QueryIntent::QualifiedSymbol { scope, name } => {
                assert_eq!(scope, "User");
                assert_eq!(name, "save");
            }
            other => panic!("Expected QualifiedSymbol, got {:?}", other),
        }

        assert_eq!(
            QueryParser::classify_intent("RateCard"),
            QueryIntent::ExactSymbol("RateCard".to_string())
        );

        assert_eq!(
            QueryParser::classify_intent("src/billing/*.rs"),
            QueryIntent::PathFilter("src/billing/*.rs".to_string())
        );

        match QueryParser::classify_intent("accrual period calculation") {
            QueryIntent::SemanticIntent(tokens) => {
                assert_eq!(tokens, vec!["accrual", "period", "calculation"]);
            }
            other => panic!("Expected SemanticIntent, got {:?}", other),
        }
    }

    #[test]
    fn test_extract_inline_directives() {
        let input = "verify_token kind:function path:src/auth min-conf:0.8 depth:2";
        let (clean, directives) = QueryParser::extract_inline_directives(input);
        assert_eq!(clean, "verify_token");
        assert_eq!(directives.kind, Some("function".to_string()));
        assert_eq!(directives.path, Some("src/auth".to_string()));
        assert_eq!(directives.min_confidence, Some(0.8));
        assert_eq!(directives.depth, Some(2));
    }

    #[test]
    fn test_camel_case_and_snake_case_tokenization() {
        assert_eq!(
            QueryParser::tokenize_terms("RateCard"),
            vec!["rate", "card"]
        );
        assert_eq!(
            QueryParser::tokenize_terms("calculate_accrual_period"),
            vec!["calculate", "accrual", "period"]
        );
        assert_eq!(
            QueryParser::tokenize_terms("UserProfile-view"),
            vec!["user", "profile", "view"]
        );
    }

    #[test]
    fn test_parse_with_cli_and_inline_options() {
        let raw = "process_payment kind:function";
        let cli = LocateCliOptions {
            code: true,
            max_tokens: Some(3000),
            flow: true,
            ..Default::default()
        };
        let query = QueryParser::parse(raw, cli);
        assert_eq!(query.clean_query, "process_payment");
        assert_eq!(query.mode, TraversalMode::FlowChain);
        assert_eq!(query.detail, ContextDetail::Full);
        assert_eq!(query.max_tokens, Some(3000));
        assert!(query.kind_filter.is_some());
    }
}
