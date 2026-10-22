use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::diagram::{
    DiagramAttribute, DiagramEdge, DiagramEdgeStyle, DiagramGraph, DiagramNode, DiagramOptions,
    DiagramType,
};
use cartograph_core::error::Result;
use cartograph_core::model::{EdgeKind, NodeKind};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct EntityRelationshipExtractor;

impl EntityRelationshipExtractor {
    pub async fn extract(
        backend: &Arc<dyn GraphStoreBackend>,
        options: &DiagramOptions,
    ) -> Result<DiagramGraph> {
        let mut graph = DiagramGraph::new(
            "Entity Relationship Diagram",
            DiagramType::EntityRelationship,
        );

        // 1. Fetch all candidate data models (Struct, Class, Model, Interface)
        let file_hashes = backend.get_all_file_hashes().await?;
        let mut model_nodes: Vec<cartograph_core::model::Node> = Vec::new();

        for file in file_hashes.keys() {
            let nodes = backend.get_nodes_by_filepath(file).await?;
            for node in nodes {
                if matches!(
                    node.kind,
                    NodeKind::Model | NodeKind::Struct | NodeKind::Class | NodeKind::Interface
                ) {
                    // Check if it's a domain entity or schema model
                    let is_entity = node.kind == NodeKind::Model
                        || node.filepath.contains("model")
                        || node.filepath.contains("schema")
                        || node.filepath.contains("entity")
                        || node.filepath.contains("entities")
                        || node.name.ends_with("Model")
                        || node.name.ends_with("Entity")
                        || node.name.ends_with("Schema")
                        || node.name.ends_with("Record")
                        || node.name.ends_with("DTO");

                    if is_entity || node_has_fields(&node) {
                        model_nodes.push(node);
                    }
                }
            }
        }

        // If query was specified, focus on that model or related models
        if let Some(query) = &options.query {
            let filtered: Vec<_> = model_nodes
                .iter()
                .filter(|n| n.name.to_lowercase().contains(&query.to_lowercase()))
                .cloned()
                .collect();
            if !filtered.is_empty() {
                model_nodes = filtered;
            }
        }

        if model_nodes.is_empty() {
            graph.title = "Entity Relationship Diagram (No domain models found)".to_string();
            return Ok(graph);
        }

        // Entity name -> DiagramNode
        let mut entity_map: HashMap<String, DiagramNode> = HashMap::new();
        let entity_names: HashSet<String> = model_nodes.iter().map(|n| n.name.clone()).collect();

        for node in &model_nodes {
            let mut dnode = DiagramNode::new(&node.id, &node.name, node.kind.clone());
            dnode.filepath = node.filepath.clone();
            dnode.line_span = (
                node.start_line.unwrap_or(1),
                node.end_line.unwrap_or(node.start_line.unwrap_or(1)),
            );

            // Extract fields / attributes
            dnode.attributes = extract_attributes(node, &entity_names);
            entity_map.insert(node.name.clone(), dnode);
        }

        // Discover relationships between entities
        let mut edges: Vec<DiagramEdge> = Vec::new();
        let mut edge_keys: HashSet<(String, String)> = HashSet::new();

        for source_node in entity_map.values() {
            for attr in &source_node.attributes {
                if let Some(target_name) = &attr.foreign_target
                    && let Some(target_node) = entity_map.get(target_name)
                {
                    let key = (source_node.id.clone(), target_node.id.clone());
                    if edge_keys.insert(key) {
                        let mut edge = DiagramEdge::new(
                            &source_node.id,
                            &target_node.id,
                            EdgeKind::References,
                        );

                        let is_collection = attr.data_type.starts_with("Vec<")
                            || attr.data_type.starts_with("List[")
                            || attr.data_type.ends_with("[]")
                            || attr.data_type.starts_with("Array<");

                        if is_collection {
                            edge.label = Some(format!("1:N ({})", attr.name));
                            edge.style = DiagramEdgeStyle::Solid;
                        } else if attr.is_fk {
                            edge.label = Some(format!("N:1 ({})", attr.name));
                            edge.style = DiagramEdgeStyle::Dashed;
                        } else {
                            edge.label = Some(format!("1:1 ({})", attr.name));
                            edge.style = DiagramEdgeStyle::Solid;
                        }

                        edges.push(edge);
                    }
                }
            }
        }

        // Also discover any structural reference edges present in graph database
        let all_ids: Vec<String> = entity_map.values().map(|n| n.id.clone()).collect();
        if let Ok(raw_edges) = backend.get_edges_for_nodes(&all_ids, 0.5).await {
            let id_set: HashSet<String> = all_ids.into_iter().collect();
            for re in raw_edges {
                if id_set.contains(&re.source_id) && id_set.contains(&re.target_id) {
                    let key = (re.source_id.clone(), re.target_id.clone());
                    if edge_keys.insert(key) {
                        let mut edge = DiagramEdge::new(&re.source_id, &re.target_id, re.kind);
                        edge.label = Some("1:N".to_string());
                        edge.style = DiagramEdgeStyle::Solid;
                        edges.push(edge);
                    }
                }
            }
        }

        graph.nodes = entity_map.into_values().collect();
        graph.edges = edges;

        // Pruning if exceeding max_nodes
        super::pruner::prune_graph_centrality(&mut graph, None, options.max_nodes);

        Ok(graph)
    }
}

/// Heuristically extract attributes/fields from docstrings, signatures, or metadata
fn extract_attributes(
    node: &cartograph_core::model::Node,
    entity_names: &HashSet<String>,
) -> Vec<DiagramAttribute> {
    let mut attrs = Vec::new();

    // Check if docstring or signature has field list
    let mut content = format!(
        "{}\n{}",
        node.signature.as_deref().unwrap_or(""),
        node.docstring.as_deref().unwrap_or("")
    );

    if let Some(ref meta) = node.metadata
        && let Some(fields_val) = meta.get("fields")
        && let Some(fields_str) = fields_val.as_str()
    {
        for part in fields_str.split(',') {
            content.push('\n');
            content.push_str(part.trim());
        }
    }

    // Default primary key
    attrs.push(DiagramAttribute {
        name: "id".to_string(),
        data_type: "uuid".to_string(),
        is_pk: true,
        is_fk: false,
        is_nullable: false,
        foreign_target: None,
    });

    // Parse potential field lines
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub ") || trimmed.starts_with("let ") || trimmed.contains(':') {
            let clean = trimmed.trim_start_matches("pub ").trim_end_matches(',');
            if let Some((field_name, field_type)) = clean.split_once(':') {
                let name = field_name
                    .trim()
                    .trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                let raw_type = field_type.trim();

                if name.is_empty() || name == "id" || name.starts_with("//") {
                    continue;
                }

                let is_nullable = raw_type.contains("Option<")
                    || raw_type.contains("Optional[")
                    || raw_type.ends_with('?')
                    || raw_type.contains("null");

                // Infer foreign key reference
                let mut foreign_target = None;
                let is_fk = name.ends_with("_id") || name.ends_with("Id") || name.ends_with("_fk");

                if is_fk {
                    let base = name
                        .trim_end_matches("_id")
                        .trim_end_matches("Id")
                        .trim_end_matches("_fk");
                    for entity in entity_names {
                        if entity.eq_ignore_ascii_case(base) {
                            foreign_target = Some(entity.clone());
                            break;
                        }
                    }
                }

                // Or direct type reference to another entity
                if foreign_target.is_none() {
                    for entity in entity_names {
                        if raw_type.contains(entity) {
                            foreign_target = Some(entity.clone());
                            break;
                        }
                    }
                }

                attrs.push(DiagramAttribute {
                    name: name.to_string(),
                    data_type: raw_type.to_string(),
                    is_pk: false,
                    is_fk,
                    is_nullable,
                    foreign_target,
                });
            }
        }
    }

    attrs
}

fn node_has_fields(node: &cartograph_core::model::Node) -> bool {
    node.signature
        .as_deref()
        .map(|s| s.contains('{') || s.contains("struct ") || s.contains("class "))
        .unwrap_or(false)
}
