use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use crate::model::{EdgeKind, NodeKind};

/// Canonical diagram archetypes supported by Cartograph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagramType {
    Architecture,
    CallGraph,
    EntityRelationship,
    Sequence,
    LogicFlow,
    DiffImpact,
    ConceptCluster,
}

impl DiagramType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagramType::Architecture => "architecture",
            DiagramType::CallGraph => "callgraph",
            DiagramType::EntityRelationship => "er",
            DiagramType::Sequence => "sequence",
            DiagramType::LogicFlow => "logic",
            DiagramType::DiffImpact => "diff",
            DiagramType::ConceptCluster => "concept",
        }
    }
}

impl fmt::Display for DiagramType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for DiagramType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "architecture" | "arch" | "modules" | "module" => Ok(DiagramType::Architecture),
            "callgraph" | "call" | "calls" | "flow" => Ok(DiagramType::CallGraph),
            "er" | "data" | "models" | "schema" => Ok(DiagramType::EntityRelationship),
            "sequence" | "seq" | "trace" | "route" => Ok(DiagramType::Sequence),
            "logic" | "flowchart" | "decision" | "rules" => Ok(DiagramType::LogicFlow),
            "diff" | "impact" | "pr" => Ok(DiagramType::DiffImpact),
            "concept" | "domain" | "aggregate" => Ok(DiagramType::ConceptCluster),
            other => Err(format!("Unknown diagram type: '{}'. Valid types: architecture, callgraph, er, sequence, logic, diff, concept", other)),
        }
    }
}

/// Supported diagram export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagramFormat {
    Mermaid,
    Svg,
    Html,
    Text,
    D2,
    Dot,
    PlantUml,
    Json,
    Xml,
}

impl DiagramFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagramFormat::Mermaid => "mermaid",
            DiagramFormat::Svg => "svg",
            DiagramFormat::Html => "html",
            DiagramFormat::Text => "text",
            DiagramFormat::D2 => "d2",
            DiagramFormat::Dot => "dot",
            DiagramFormat::PlantUml => "plantuml",
            DiagramFormat::Json => "json",
            DiagramFormat::Xml => "xml",
        }
    }

    pub fn file_extension(&self) -> &'static str {
        match self {
            DiagramFormat::Mermaid => "mmd",
            DiagramFormat::Svg => "svg",
            DiagramFormat::Html => "html",
            DiagramFormat::Text => "txt",
            DiagramFormat::D2 => "d2",
            DiagramFormat::Dot => "dot",
            DiagramFormat::PlantUml => "puml",
            DiagramFormat::Json => "json",
            DiagramFormat::Xml => "xml",
        }
    }
}

impl fmt::Display for DiagramFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for DiagramFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "mermaid" | "mmd" | "md" => Ok(DiagramFormat::Mermaid),
            "svg" => Ok(DiagramFormat::Svg),
            "html" => Ok(DiagramFormat::Html),
            "text" | "ascii" | "ansi" | "terminal" => Ok(DiagramFormat::Text),
            "d2" => Ok(DiagramFormat::D2),
            "dot" | "graphviz" => Ok(DiagramFormat::Dot),
            "plantuml" | "puml" => Ok(DiagramFormat::PlantUml),
            "json" => Ok(DiagramFormat::Json),
            "xml" => Ok(DiagramFormat::Xml),
            other => Err(format!("Unknown diagram format: '{}'. Valid formats: mermaid, svg, html, text, d2, dot, plantuml, json, xml", other)),
        }
    }
}

/// Visual theme for SVG, HTML, and terminal rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagramTheme {
    Dark,
    Light,
    HighContrast,
}

impl DiagramTheme {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagramTheme::Dark => "dark",
            DiagramTheme::Light => "light",
            DiagramTheme::HighContrast => "high_contrast",
        }
    }
}

impl fmt::Display for DiagramTheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for DiagramTheme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "dark" => Ok(DiagramTheme::Dark),
            "light" => Ok(DiagramTheme::Light),
            "high_contrast" | "contrast" => Ok(DiagramTheme::HighContrast),
            other => Err(format!("Unknown theme: '{}'. Valid themes: dark, light, high_contrast", other)),
        }
    }
}

/// Direction for call-graph and dependency traversals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraversalDirection {
    Upstream,
    Downstream,
    Both,
}

impl TraversalDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            TraversalDirection::Upstream => "upstream",
            TraversalDirection::Downstream => "downstream",
            TraversalDirection::Both => "both",
        }
    }
}

impl fmt::Display for TraversalDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for TraversalDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "upstream" | "up" | "callers" | "in" => Ok(TraversalDirection::Upstream),
            "downstream" | "down" | "callees" | "out" => Ok(TraversalDirection::Downstream),
            "both" | "bidirectional" | "all" => Ok(TraversalDirection::Both),
            other => Err(format!("Unknown direction: '{}'. Valid directions: upstream, downstream, both", other)),
        }
    }
}

/// User options configuring diagram generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagramOptions {
    pub query: Option<String>,
    pub diagram_type: Option<DiagramType>,
    pub format: DiagramFormat,
    pub theme: DiagramTheme,
    pub depth: usize,
    pub direction: TraversalDirection,
    pub max_nodes: usize,
    pub collapse_modules: bool,
    pub include_tests: bool,
    pub include_stdlib: bool,
    pub focus_symbol: Option<String>,
    pub concept: Option<String>,
    pub branch: Option<String>,
    pub staged: bool,
    pub out_path: Option<String>,
    pub open_in_browser: bool,
    pub interactive: bool,
}

impl Default for DiagramOptions {
    fn default() -> Self {
        Self {
            query: None,
            diagram_type: None,
            format: DiagramFormat::Mermaid,
            theme: DiagramTheme::Dark,
            depth: 2,
            direction: TraversalDirection::Both,
            max_nodes: 35,
            collapse_modules: true,
            include_tests: false,
            include_stdlib: false,
            focus_symbol: None,
            concept: None,
            branch: None,
            staged: false,
            out_path: None,
            open_in_browser: false,
            interactive: false,
        }
    }
}

/// Node representation in the diagram IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagramNode {
    pub id: String,
    pub label: String,
    pub subtitle: Option<String>,
    pub kind: NodeKind,
    pub filepath: String,
    pub line_span: (u32, u32),
    pub cluster_id: Option<String>,
    pub attributes: Vec<DiagramAttribute>,
    pub metrics: DiagramNodeMetrics,
    pub diff_status: Option<DiagramDiffStatus>,
    pub layout: LayoutBox,
}

impl DiagramNode {
    pub fn new(id: impl Into<String>, label: impl Into<String>, kind: NodeKind) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            subtitle: None,
            kind,
            filepath: String::new(),
            line_span: (0, 0),
            cluster_id: None,
            attributes: Vec::new(),
            metrics: DiagramNodeMetrics::default(),
            diff_status: None,
            layout: LayoutBox::default(),
        }
    }
}

/// Attribute/field description for ER or Class nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagramAttribute {
    pub name: String,
    pub data_type: String,
    pub is_pk: bool,
    pub is_fk: bool,
    pub is_nullable: bool,
    pub foreign_target: Option<String>,
}

/// Metrics overlaid on nodes for heatmaps and risk assessment
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DiagramNodeMetrics {
    pub cognitive_complexity: u32,
    pub cyclomatic_complexity: u32,
    pub blast_risk_score: u32,
    pub afferent_coupling: usize,
    pub efferent_coupling: usize,
    pub git_churn_count: usize,
    pub has_test_coverage: bool,
}

/// Diff status for PR blast-radius visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagramDiffStatus {
    Added,
    Modified,
    Deleted,
    BlastRadiusHazard,
}

/// Edge representation in the diagram IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagramEdge {
    pub source_id: String,
    pub target_id: String,
    pub kind: EdgeKind,
    pub label: Option<String>,
    pub style: DiagramEdgeStyle,
    pub is_cycle_cut: bool,
    pub waypoints: Vec<Point2D>,
}

impl DiagramEdge {
    pub fn new(source_id: impl Into<String>, target_id: impl Into<String>, kind: EdgeKind) -> Self {
        Self {
            source_id: source_id.into(),
            target_id: target_id.into(),
            kind,
            label: None,
            style: DiagramEdgeStyle::Solid,
            is_cycle_cut: false,
            waypoints: Vec::new(),
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_style(mut self, style: DiagramEdgeStyle) -> Self {
        self.style = style;
        self
    }
}

/// Visual style applied to diagram edges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagramEdgeStyle {
    Solid,
    Dashed,
    Dotted,
    BoldWarning,
}

/// Subgraph cluster grouping related nodes (e.g. packages, modules, layers)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagramCluster {
    pub id: String,
    pub label: String,
    pub parent_cluster_id: Option<String>,
    pub bounds: LayoutBox,
}

impl DiagramCluster {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            parent_cluster_id: None,
            bounds: LayoutBox::default(),
        }
    }
}

/// 2D bounding box for node/cluster layout
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct LayoutBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// 2D coordinate point for edge Bézier curve routing
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

/// Complete graph data model passed to renderers and layout engines
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagramGraph {
    pub title: String,
    pub diagram_type: DiagramType,
    pub nodes: Vec<DiagramNode>,
    pub edges: Vec<DiagramEdge>,
    pub clusters: Vec<DiagramCluster>,
    pub total_unpruned_nodes: usize,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl DiagramGraph {
    pub fn new(title: impl Into<String>, diagram_type: DiagramType) -> Self {
        Self {
            title: title.into(),
            diagram_type,
            nodes: Vec::new(),
            edges: Vec::new(),
            clusters: Vec::new(),
            total_unpruned_nodes: 0,
            metadata: HashMap::new(),
        }
    }

    pub fn find_node(&self, id: &str) -> Option<&DiagramNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn find_node_mut(&mut self, id: &str) -> Option<&mut DiagramNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagram_type_parsing() {
        assert_eq!("architecture".parse::<DiagramType>().unwrap(), DiagramType::Architecture);
        assert_eq!("arch".parse::<DiagramType>().unwrap(), DiagramType::Architecture);
        assert_eq!("callgraph".parse::<DiagramType>().unwrap(), DiagramType::CallGraph);
        assert_eq!("er".parse::<DiagramType>().unwrap(), DiagramType::EntityRelationship);
        assert_eq!("sequence".parse::<DiagramType>().unwrap(), DiagramType::Sequence);
        assert_eq!("logic".parse::<DiagramType>().unwrap(), DiagramType::LogicFlow);
        assert_eq!("diff".parse::<DiagramType>().unwrap(), DiagramType::DiffImpact);
        assert_eq!("concept".parse::<DiagramType>().unwrap(), DiagramType::ConceptCluster);
    }

    #[test]
    fn test_diagram_format_parsing() {
        assert_eq!("mermaid".parse::<DiagramFormat>().unwrap(), DiagramFormat::Mermaid);
        assert_eq!("svg".parse::<DiagramFormat>().unwrap(), DiagramFormat::Svg);
        assert_eq!("html".parse::<DiagramFormat>().unwrap(), DiagramFormat::Html);
        assert_eq!("text".parse::<DiagramFormat>().unwrap(), DiagramFormat::Text);
        assert_eq!("d2".parse::<DiagramFormat>().unwrap(), DiagramFormat::D2);
        assert_eq!("dot".parse::<DiagramFormat>().unwrap(), DiagramFormat::Dot);
        assert_eq!("plantuml".parse::<DiagramFormat>().unwrap(), DiagramFormat::PlantUml);
        assert_eq!("json".parse::<DiagramFormat>().unwrap(), DiagramFormat::Json);
        assert_eq!("xml".parse::<DiagramFormat>().unwrap(), DiagramFormat::Xml);
    }

    #[test]
    fn test_diagram_graph_serialization() {
        let mut graph = DiagramGraph::new("Test Architecture", DiagramType::Architecture);
        let mut node = DiagramNode::new("srv1", "OrderService", NodeKind::Struct);
        node.filepath = "src/order.rs".to_string();
        node.metrics.cognitive_complexity = 8;
        graph.nodes.push(node);

        let edge = DiagramEdge::new("ctrl1", "srv1", EdgeKind::Calls).with_label("invokes");
        graph.edges.push(edge);

        let json = serde_json::to_string(&graph).expect("Must serialize to JSON");
        let roundtrip: DiagramGraph = serde_json::from_str(&json).expect("Must deserialize from JSON");
        assert_eq!(roundtrip.title, "Test Architecture");
        assert_eq!(roundtrip.nodes.len(), 1);
        assert_eq!(roundtrip.edges.len(), 1);
        assert_eq!(roundtrip.edges[0].label.as_deref(), Some("invokes"));
    }
}
