use super::normalizer::normalize_route_path;

/// Combine a base router prefix with a subpath ensuring canonical formatting
pub fn join_route_prefixes(prefix: &str, subpath: &str) -> String {
    let p = prefix.trim();
    let s = subpath.trim();

    if p.is_empty() || p == "/" {
        return normalize_route_path(s);
    }
    if s.is_empty() || s == "/" {
        return normalize_route_path(p);
    }

    let p_clean = p.trim_matches('/');
    let s_clean = s.trim_matches('/');

    normalize_route_path(&format!("/{p_clean}/{s_clean}"))
}

/// A node in the hierarchical route prefix tree
#[derive(Debug, Clone, Default)]
pub struct PrefixTree {
    pub prefix: String,
    pub children: Vec<PrefixTree>,
}

impl PrefixTree {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: normalize_route_path(prefix),
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child_prefix: &str) -> &mut PrefixTree {
        let combined = join_route_prefixes(&self.prefix, child_prefix);
        let child = PrefixTree {
            prefix: combined,
            children: Vec::new(),
        };
        self.children.push(child);
        self.children.last_mut().unwrap()
    }

    pub fn resolve_path(&self, subpath: &str) -> String {
        join_route_prefixes(&self.prefix, subpath)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_route_prefixes() {
        assert_eq!(join_route_prefixes("/api/v1", "/users"), "/api/v1/users");
        assert_eq!(join_route_prefixes("/api/v1/", "users/"), "/api/v1/users");
        assert_eq!(join_route_prefixes("/", "/users"), "/users");
        assert_eq!(join_route_prefixes("", "users"), "/users");
        assert_eq!(join_route_prefixes("/api", ":id"), "/api/{id}");
    }

    #[test]
    fn test_prefix_tree_nesting() {
        let mut root = PrefixTree::new("/api");
        let v1 = root.add_child("v1");
        assert_eq!(v1.prefix, "/api/v1");
        assert_eq!(v1.resolve_path("/checkout"), "/api/v1/checkout");
    }
}
