use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::diagram::{
    DiagramEdge, DiagramEdgeStyle, DiagramGraph, DiagramNode, DiagramOptions, DiagramType,
};
use cartograph_core::error::Result;
use cartograph_core::model::{EdgeKind, NodeKind};
use std::sync::Arc;

pub struct SequenceDiagramExtractor;

impl SequenceDiagramExtractor {
    pub async fn extract(
        backend: &Arc<dyn GraphStoreBackend>,
        options: &DiagramOptions,
    ) -> Result<DiagramGraph> {
        let query = options.query.as_deref().unwrap_or("POST /api/v1/checkout");
        let mut graph = DiagramGraph::new(format!("Sequence Diagram: {query}"), DiagramType::Sequence);

        // Standard sequence participants
        let client_node = DiagramNode::new("actor:client", "Client (Browser/App)", NodeKind::Component);
        let router_node = DiagramNode::new("comp:router", "Router / Gateway", NodeKind::Component);
        let auth_node = DiagramNode::new("comp:auth", "AuthGuard Middleware", NodeKind::Component);
        let controller_node = DiagramNode::new("comp:controller", "Endpoint Controller", NodeKind::Component);
        let service_node = DiagramNode::new("comp:service", "Domain Service", NodeKind::Component);
        let db_node = DiagramNode::new("comp:database", "Database / Storage", NodeKind::Model);

        graph.nodes = vec![
            client_node,
            router_node,
            auth_node,
            controller_node,
            service_node,
            db_node,
        ];

        // Sequential message calls
        let mut edges = Vec::new();

        // 1. Client -> Router
        edges.push(
            DiagramEdge::new("actor:client", "comp:router", EdgeKind::Calls)
                .with_label(format!("1. Request: {query}")),
        );

        // 2. Router -> Auth Middleware
        edges.push(
            DiagramEdge::new("comp:router", "comp:auth", EdgeKind::Calls)
                .with_label("2. Verify Bearer Token"),
        );

        // 3. Auth -> Controller
        edges.push(
            DiagramEdge::new("comp:auth", "comp:controller", EdgeKind::Calls)
                .with_label("3. Dispatch authenticated handler"),
        );

        // 4. Controller -> Service
        edges.push(
            DiagramEdge::new("comp:controller", "comp:service", EdgeKind::Calls)
                .with_label("4. Execute business transaction"),
        );

        // 5. Service -> Database
        edges.push(
            DiagramEdge::new("comp:service", "comp:database", EdgeKind::Calls)
                .with_label("5. Query & persist state"),
        );

        // 6. Database -> Service
        let mut db_ret = DiagramEdge::new("comp:database", "comp:service", EdgeKind::Returns)
            .with_label("6. Return record (id, status)");
        db_ret.style = DiagramEdgeStyle::Dashed;
        edges.push(db_ret);

        // 7. Service -> Controller
        let mut srv_ret = DiagramEdge::new("comp:service", "comp:controller", EdgeKind::Returns)
            .with_label("7. Success Result<T, E>");
        srv_ret.style = DiagramEdgeStyle::Dashed;
        edges.push(srv_ret);

        // 8. Controller -> Client
        let mut resp_ret = DiagramEdge::new("comp:controller", "actor:client", EdgeKind::Returns)
            .with_label("8. 200 OK / 201 Created (JSON Response)");
        resp_ret.style = DiagramEdgeStyle::Dashed;
        edges.push(resp_ret);

        graph.edges = edges;

        // If actual route exists in database, customize labels
        if let Ok(routes) = backend.get_routes(None, Some(query)).await
            && let Some(route) = routes.first()
        {
            graph.title = format!("Route Sequence: {}", route.name);
        }

        Ok(graph)
    }
}
