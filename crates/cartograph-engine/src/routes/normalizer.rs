use cartograph_core::model::{ParamLocation, RouteParameter};

/// Convert framework-specific route parameters (:id, <int:id>, [id]) to canonical RFC 6570 {id} format
pub fn normalize_route_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "/" {
        return "/".to_string();
    }

    // Strip optional rails format extension: (.:format)
    let cleaned = trimmed.replace("(.:format)", "");

    // Split into path segments
    let segments: Vec<&str> = cleaned.split('/').filter(|s| !s.is_empty()).collect();
    let mut normalized_segments = Vec::new();

    for seg in segments {
        let mut s = seg.to_string();

        // 1. Next.js App / Pages Router: [[...slug]], [...slug], [id]
        if s.starts_with('[') && s.ends_with(']') {
            let inner = s.trim_matches(['[', ']']).trim_start_matches("...");
            s = format!("{{{inner}}}");
        }
        // 2. Flask / Django typed parameter: <int:id>, <str:name>, <slug:slug>, <id>
        else if s.starts_with('<') && s.ends_with('>') {
            let inner = s.trim_matches(['<', '>']);
            let param_name = if let Some(idx) = inner.find(':') {
                &inner[idx + 1..]
            } else {
                inner
            };
            s = format!("{{{param_name}}}");
        }
        // 3. Express / Koa / Rails colon parameter: :id or :userId
        else if s.starts_with(':') {
            let param_name = s.trim_start_matches(':');
            s = format!("{{{param_name}}}");
        }

        normalized_segments.push(s);
    }

    let mut result = format!("/{}", normalized_segments.join("/"));
    if result.is_empty() {
        result = "/".to_string();
    }
    result
}

/// Extract path parameters as RouteParameter instances from a route pattern
pub fn extract_route_parameters(path: &str) -> Vec<RouteParameter> {
    let normalized = normalize_route_path(path);
    let mut params = Vec::new();

    let mut inside = false;
    let mut current = String::new();

    for c in normalized.chars() {
        if c == '{' {
            inside = true;
            current.clear();
        } else if c == '}' {
            if inside && !current.is_empty() {
                params.push(RouteParameter {
                    name: current.clone(),
                    location: ParamLocation::Path,
                    type_hint: None,
                    required: true,
                    default_value: None,
                    description: None,
                });
            }
            inside = false;
            current.clear();
        } else if inside {
            current.push(c);
        }
    }

    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_express_routes() {
        assert_eq!(normalize_route_path("/users/:id"), "/users/{id}");
        assert_eq!(normalize_route_path("users/:userId/books/:bookId/"), "/users/{userId}/books/{bookId}");
        assert_eq!(normalize_route_path("//api///v1//:id"), "/api/v1/{id}");
    }

    #[test]
    fn test_normalize_flask_and_django_routes() {
        assert_eq!(normalize_route_path("/users/<int:user_id>"), "/users/{user_id}");
        assert_eq!(normalize_route_path("/posts/<slug:post_slug>/comments"), "/posts/{post_slug}/comments");
        assert_eq!(normalize_route_path("/items/<item_id>"), "/items/{item_id}");
    }

    #[test]
    fn test_normalize_nextjs_routes() {
        assert_eq!(normalize_route_path("/api/users/[id]"), "/api/users/{id}");
        assert_eq!(normalize_route_path("/api/docs/[...slug]"), "/api/docs/{slug}");
        assert_eq!(normalize_route_path("/api/optional/[[...param]]"), "/api/optional/{param}");
    }

    #[test]
    fn test_normalize_rails_routes() {
        assert_eq!(normalize_route_path("/invoices/:id(.:format)"), "/invoices/{id}");
        assert_eq!(normalize_route_path("/invoices"), "/invoices");
        assert_eq!(normalize_route_path(""), "/");
        assert_eq!(normalize_route_path("/"), "/");
    }

    #[test]
    fn test_extract_parameters() {
        let params = extract_route_parameters("/orgs/{org_id}/teams/{team_id}");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "org_id");
        assert_eq!(params[0].location, ParamLocation::Path);
        assert_eq!(params[0].required, true);
        assert_eq!(params[1].name, "team_id");
    }
}
