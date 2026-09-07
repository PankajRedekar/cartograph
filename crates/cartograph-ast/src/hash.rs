use sha2::{Digest, Sha256};

/// Compute a deterministic checksum for an AST node based on its normalized token stream.
/// Whitespace variations (extra indentation, newlines) are collapsed so cosmetic edits
/// do not change the semantic hash, while syntax or identifier changes produce a different hash.
pub fn compute_node_checksum(source: &str) -> String {
    let mut hasher = Sha256::new();
    for token in source.split_whitespace() {
        hasher.update(token.as_bytes());
        hasher.update(b" ");
    }
    format!("{:x}", hasher.finalize())
}

/// Compute a composite Merkle hash from a parent node's own tokens plus its child checksums.
pub fn compute_composite_hash(own_checksum: &str, child_checksums: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(own_checksum.as_bytes());
    for child in child_checksums {
        hasher.update(b"::");
        hasher.update(child.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_checksum_whitespace_insensitivity() {
        let code1 = "def login(user: str) -> bool:\n    return True";
        let code2 = "def login(user: str) -> bool:\n        return True";
        let code3 = "def login(user: str) -> bool: return True";

        assert_eq!(compute_node_checksum(code1), compute_node_checksum(code2));
        assert_eq!(compute_node_checksum(code1), compute_node_checksum(code3));
    }

    #[test]
    fn test_node_checksum_semantic_change() {
        let code1 = "def login(user: str) -> bool:\n    return True";
        let code2 = "def login(user: str) -> bool:\n    return False";

        assert_ne!(compute_node_checksum(code1), compute_node_checksum(code2));
    }

    #[test]
    fn test_composite_hash() {
        let parent = "hash_parent";
        let child1 = "hash_c1";
        let child2 = "hash_c2";

        let comp1 = compute_composite_hash(parent, &[child1, child2]);
        let comp2 = compute_composite_hash(parent, &[child2, child1]);

        assert_ne!(comp1, comp2); // child ordering matters
        assert_eq!(comp1, compute_composite_hash(parent, &[child1, child2]));
    }
}
