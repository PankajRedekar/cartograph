use std::fmt;
use std::str::FromStr;

use crate::error::CartographError;
use crate::model::NodeKind;

pub const URI_SCHEME: &str = "cartograph://";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalUri {
    pub filepath: String,
    pub scope: String,
    pub name: String,
    pub kind: NodeKind,
}

impl CanonicalUri {
    pub fn new(
        filepath: impl Into<String>,
        scope: impl Into<String>,
        name: impl Into<String>,
        kind: NodeKind,
    ) -> Self {
        Self {
            filepath: filepath.into(),
            scope: scope.into(),
            name: name.into(),
            kind,
        }
    }

    pub fn to_uri(&self) -> String {
        build_uri(&self.filepath, &self.scope, &self.name, &self.kind)
    }
}

impl fmt::Display for CanonicalUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_uri())
    }
}

impl FromStr for CanonicalUri {
    type Err = CartographError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_uri(s)
    }
}

pub fn build_uri(filepath: &str, scope: &str, name: &str, kind: &NodeKind) -> String {
    format!(
        "{URI_SCHEME}{}::{}::{}#{}",
        filepath,
        scope,
        name,
        kind.as_str()
    )
}

pub fn parse_uri(uri: &str) -> Result<CanonicalUri, CartographError> {
    if !uri.starts_with(URI_SCHEME) {
        return Err(CartographError::InvalidUri(format!(
            "URI does not start with '{URI_SCHEME}': {uri}"
        )));
    }

    let without_scheme = &uri[URI_SCHEME.len()..];
    let (body, kind_part) = without_scheme.split_once('#').ok_or_else(|| {
        CartographError::InvalidUri(format!("Missing kind delimiter '#' in URI: {uri}"))
    })?;

    if kind_part.is_empty() {
        return Err(CartographError::InvalidUri(format!(
            "Empty kind in URI: {uri}"
        )));
    }

    let kind: NodeKind = kind_part.parse().map_err(|_| {
        CartographError::InvalidUri(format!("Invalid kind in URI: {kind_part}"))
    })?;

    let parts: Vec<&str> = body.split("::").collect();
    if parts.len() != 3 {
        return Err(CartographError::InvalidUri(format!(
            "Expected 3 parts separated by '::' (<filepath>::<scope>::<name>), found {} parts in: {uri}",
            parts.len()
        )));
    }

    let filepath = parts[0].to_string();
    let scope = parts[1].to_string();
    let name = parts[2].to_string();

    if filepath.is_empty() {
        return Err(CartographError::InvalidUri(format!(
            "Filepath cannot be empty in URI: {uri}"
        )));
    }

    if name.is_empty() {
        return Err(CartographError::InvalidUri(format!(
            "Symbol name cannot be empty in URI: {uri}"
        )));
    }

    Ok(CanonicalUri {
        filepath,
        scope,
        name,
        kind,
    })
}
