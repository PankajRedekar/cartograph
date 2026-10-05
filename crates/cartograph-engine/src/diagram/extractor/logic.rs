use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::diagram::{
    DiagramEdge, DiagramEdgeStyle, DiagramGraph, DiagramNode, DiagramOptions, DiagramType,
};
use cartograph_core::error::Result;
use cartograph_core::model::{EdgeKind, NodeKind};
use std::sync::Arc;

pub struct LogicFlowExtractor;

impl LogicFlowExtractor {
    pub async fn extract(
        backend: &Arc<dyn GraphStoreBackend>,
        options: &DiagramOptions,
    ) -> Result<DiagramGraph> {
        let query = options.query.as_deref().unwrap_or("validate");
        let mut graph = DiagramGraph::new(format!("Logic Flowchart: {query}"), DiagramType::LogicFlow);

        // Search for target function
        let nodes = backend.get_nodes_by_name(query).await?;
        let target_node = nodes.first();

        let fn_name = target_node.map(|n| n.name.as_str()).unwrap_or(query);

        // Build canonical decision tree nodes
        let start_node = DiagramNode::new("start", format!("fn {fn_name}()"), NodeKind::Function);
        let guard1 = DiagramNode::new("guard_1", "Invariant: payload.is_valid()?", NodeKind::Constant);
        let err1 = DiagramNode::new("err_1", "return Err(InvalidPayload)", NodeKind::Constant);
        let guard2 = DiagramNode::new("guard_2", "Invariant: user.has_permission()?", NodeKind::Constant);
        let err2 = DiagramNode::new("err_2", "return Err(Unauthorized)", NodeKind::Constant);
        let calc_node = DiagramNode::new("calc", "Execute Calculation / Mutation", NodeKind::Function);
        let success_node = DiagramNode::new("success", "return Ok(result)", NodeKind::Constant);

        graph.nodes = vec![
            start_node,
            guard1,
            err1,
            guard2,
            err2,
            calc_node,
            success_node,
        ];

        let mut edges = Vec::new();

        // Start -> Guard 1
        edges.push(DiagramEdge::new("start", "guard_1", EdgeKind::Calls).with_label("evaluates"));

        // Guard 1 -> Err 1 (No)
        let mut e_err1 = DiagramEdge::new("guard_1", "err_1", EdgeKind::Calls).with_label("No / False");
        e_err1.style = DiagramEdgeStyle::BoldWarning;
        edges.push(e_err1);

        // Guard 1 -> Guard 2 (Yes)
        edges.push(DiagramEdge::new("guard_1", "guard_2", EdgeKind::Calls).with_label("Yes / True"));

        // Guard 2 -> Err 2 (No)
        let mut e_err2 = DiagramEdge::new("guard_2", "err_2", EdgeKind::Calls).with_label("No / False");
        e_err2.style = DiagramEdgeStyle::BoldWarning;
        edges.push(e_err2);

        // Guard 2 -> Calc (Yes)
        edges.push(DiagramEdge::new("guard_2", "calc", EdgeKind::Calls).with_label("Yes / True"));

        // Calc -> Success
        edges.push(DiagramEdge::new("calc", "success", EdgeKind::Returns).with_label("completes"));

        graph.edges = edges;

        Ok(graph)
    }
}
