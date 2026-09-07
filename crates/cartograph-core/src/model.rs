use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Function,
    Method,
    Class,
    Struct,
    Interface,
    Module,
    File,
    Route,
    Model,
    Component,
    Variable,
    Constant,
    Enum,
    Macro,
    #[serde(untagged)]
    Other(String),
}

impl NodeKind {
    pub fn as_str(&self) -> &str {
        match self {
            NodeKind::Function => "function",
            NodeKind::Method => "method",
            NodeKind::Class => "class",
            NodeKind::Struct => "struct",
            NodeKind::Interface => "interface",
            NodeKind::Module => "module",
            NodeKind::File => "file",
            NodeKind::Route => "route",
            NodeKind::Model => "model",
            NodeKind::Component => "component",
            NodeKind::Variable => "variable",
            NodeKind::Constant => "constant",
            NodeKind::Enum => "enum",
            NodeKind::Macro => "macro",
            NodeKind::Other(s) => s.as_str(),
        }
    }
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for NodeKind {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let kind = match s.to_lowercase().as_str() {
            "function" => NodeKind::Function,
            "method" => NodeKind::Method,
            "class" => NodeKind::Class,
            "struct" => NodeKind::Struct,
            "interface" => NodeKind::Interface,
            "module" => NodeKind::Module,
            "file" => NodeKind::File,
            "route" => NodeKind::Route,
            "model" => NodeKind::Model,
            "component" => NodeKind::Component,
            "variable" => NodeKind::Variable,
            "constant" => NodeKind::Constant,
            "enum" => NodeKind::Enum,
            "macro" => NodeKind::Macro,
            other => NodeKind::Other(other.to_string()),
        };
        Ok(kind)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Imports,
    Calls,
    Extends,
    Implements,
    Defines,
    References,
    Contains,
    Returns,
    ParameterOf,
    Tests,
    MapsTo,
    #[serde(untagged)]
    Other(String),
}

impl EdgeKind {
    pub fn as_str(&self) -> &str {
        match self {
            EdgeKind::Imports => "imports",
            EdgeKind::Calls => "calls",
            EdgeKind::Extends => "extends",
            EdgeKind::Implements => "implements",
            EdgeKind::Defines => "defines",
            EdgeKind::References => "references",
            EdgeKind::Contains => "contains",
            EdgeKind::Returns => "returns",
            EdgeKind::ParameterOf => "parameter_of",
            EdgeKind::Tests => "tests",
            EdgeKind::MapsTo => "maps_to",
            EdgeKind::Other(s) => s.as_str(),
        }
    }
}

impl fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for EdgeKind {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let kind = match s.to_lowercase().as_str() {
            "imports" => EdgeKind::Imports,
            "calls" => EdgeKind::Calls,
            "extends" => EdgeKind::Extends,
            "implements" => EdgeKind::Implements,
            "defines" => EdgeKind::Defines,
            "references" => EdgeKind::References,
            "contains" => EdgeKind::Contains,
            "returns" => EdgeKind::Returns,
            "parameter_of" => EdgeKind::ParameterOf,
            "tests" => EdgeKind::Tests,
            "maps_to" | "mapsto" => EdgeKind::MapsTo,
            other => EdgeKind::Other(other.to_string()),
        };
        Ok(kind)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub kind: NodeKind,
    pub filepath: String,
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_col: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_col: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docstring: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub source_id: String,
    pub target_id: String,
    pub kind: EdgeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

impl Default for Edge {
    fn default() -> Self {
        Self {
            source_id: String::new(),
            target_id: String::new(),
            kind: EdgeKind::Calls,
            metadata: None,
            confidence: Some(1.0),
            weight: Some(1.0),
        }
    }
}

impl Edge {
    pub fn new(source_id: impl Into<String>, target_id: impl Into<String>, kind: EdgeKind) -> Self {
        Self {
            source_id: source_id.into(),
            target_id: target_id.into(),
            kind,
            metadata: None,
            confidence: Some(1.0),
            weight: Some(1.0),
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub node: Node,
    pub score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub total_files: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}
