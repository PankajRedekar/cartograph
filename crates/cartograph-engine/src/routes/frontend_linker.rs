use cartograph_core::model::{Edge, EdgeKind, HttpMethod, Node, NodeKind};
use serde_json::json;
use std::str::FromStr;
use super::normalizer::normalize_route_path;

/// Normalizes a frontend template URL (e.g. `/api/users/${id}`) into a comparable pattern
pub fn normalize_frontend_template_url(raw_url: &str) -> String {
    let mut cleaned = raw_url
        .trim()
        .trim_matches('`')
        .trim_matches('\'')
        .trim_matches('"')
        .trim()
        .to_string();

    // Replace JS template literals: `/api/users/${id}` -> `/api/users/{id}`
    while let Some(start) = cleaned.find("${") {
        if let Some(end) = cleaned[start..].find('}') {
            let full_end = start + end;
            let param_name = cleaned[start + 2..full_end].trim();
            // Simplify nested expressions: user.id -> id
            let clean_param = if let Some(dot_idx) = param_name.rfind('.') {
                &param_name[dot_idx + 1..]
            } else {
                param_name
            };
            cleaned.replace_range(start..=full_end, &format!("{{{clean_param}}}"));
        } else {
            break;
        }
    }

    normalize_route_path(&cleaned)
}

/// Matches a normalized frontend URL against a server route pattern
pub fn matches_route_pattern(client_url: &str, server_pattern: &str) -> bool {
    let norm_client = normalize_frontend_template_url(client_url);
    let norm_server = normalize_route_path(server_pattern);

    if norm_client == norm_server {
        return true;
    }

    let client_segs: Vec<&str> = norm_client.split('/').filter(|s| !s.is_empty()).collect();
    let server_segs: Vec<&str> = norm_server.split('/').filter(|s| !s.is_empty()).collect();

    if client_segs.len() != server_segs.len() {
        return false;
    }

    for (c, s) in client_segs.iter().zip(server_segs.iter()) {
        let c_is_param = c.starts_with('{') && c.ends_with('}');
        let s_is_param = s.starts_with('{') && s.ends_with('}');

        if c_is_param || s_is_param {
            continue;
        }

        if c != s {
            return false;
        }
    }

    true
}

/// Links frontend UI components to backend route nodes based on URL patterns and methods
pub struct FrontendRouteLinker;

impl FrontendRouteLinker {
    pub fn link_call_site(
        caller_component_id: &str,
        client_method: &HttpMethod,
        client_url_pattern: &str,
        available_routes: &[Node],
    ) -> Option<Edge> {
        let mut best_match: Option<(&Node, f64)> = None;

        for route_node in available_routes {
            if route_node.kind != NodeKind::Route {
                continue;
            }

            let meta = match route_node.metadata.as_ref() {
                Some(m) => m,
                None => continue,
            };

            let server_method_str = meta.get("http_method").and_then(|v| v.as_str()).unwrap_or("ALL");
            let server_method = HttpMethod::from_str(server_method_str).unwrap_or(HttpMethod::All);

            // Verify method compatibility
            let methods_compatible = server_method == HttpMethod::All
                || *client_method == HttpMethod::All
                || server_method == *client_method;

            if !methods_compatible {
                continue;
            }

            let raw_pattern = meta.get("raw_pattern").and_then(|v| v.as_str()).unwrap_or(&route_node.name);

            if matches_route_pattern(client_url_pattern, raw_pattern) {
                let is_exact = normalize_frontend_template_url(client_url_pattern) == normalize_route_path(raw_pattern);
                let confidence = if is_exact { 1.0 } else { 0.9 };

                if best_match.as_ref().map_or(true, |(_, current_conf)| confidence > *current_conf) {
                    best_match = Some((route_node, confidence));
                }
            }
        }

        if let Some((matched_route, confidence)) = best_match {
            let mut edge = Edge::new(caller_component_id, &matched_route.id, EdgeKind::Calls);
            edge.metadata = Some(json!({
                "fidelity": "definite",
                "resolution": "frontend_360_link",
                "confidence": confidence,
                "client_method": client_method.as_str(),
                "client_url": client_url_pattern,
            }));
            Some(edge)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_frontend_template() {
        assert_eq!(
            normalize_frontend_template_url("/api/users/${user.id}/profile"),
            "/api/users/{id}/profile"
        );
        assert_eq!(
            normalize_frontend_template_url("`/api/items/${itemId}`"),
            "/api/items/{itemId}"
        );
    }

    #[test]
    fn test_matches_route_pattern() {
        assert!(matches_route_pattern("/api/users/${id}", "/api/users/:id"));
        assert!(matches_route_pattern("/api/users/123", "/api/users/{id}"));
        assert!(matches_route_pattern("/api/v1/orders", "/api/v1/orders"));
        assert!(!matches_route_pattern("/api/v1/orders", "/api/v1/users"));
        assert!(!matches_route_pattern("/api/users/123/edit", "/api/users/{id}"));
    }

    #[test]
    fn test_link_call_site() {
        let route_node = Node {
            id: "cartograph://routes.ts::::GET%20%2Fapi%2Fusers%2F%7Bid%7D#route".to_string(),
            name: "GET /api/users/{id}".to_string(),
            kind: NodeKind::Route,
            filepath: "routes.ts".to_string(),
            scope: "routes".to_string(),
            start_line: Some(10),
            start_col: Some(1),
            end_line: Some(10),
            end_col: Some(20),
            signature: Some("GET /api/users/{id}".to_string()),
            docstring: None,
            checksum: None,
            metadata: Some(json!({
                "http_method": "GET",
                "raw_pattern": "/api/users/:id",
                "framework": "express",
            })),
        };

        let edge = FrontendRouteLinker::link_call_site(
            "cartograph://UserCard.tsx::::UserCard#component",
            &HttpMethod::Get,
            "/api/users/${id}",
            &[route_node],
        );

        assert!(edge.is_some());
        let e = edge.unwrap();
        assert_eq!(e.source_id, "cartograph://UserCard.tsx::::UserCard#component");
        assert_eq!(e.target_id, "cartograph://routes.ts::::GET%20%2Fapi%2Fusers%2F%7Bid%7D#route");
        assert_eq!(e.kind, EdgeKind::Calls);
    }
}
