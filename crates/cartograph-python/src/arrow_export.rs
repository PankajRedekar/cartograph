use pyo3::prelude::*;
use pyo3::types::PyDict;
use cartograph_core::model::{ComplexityRecord, Edge, Node};

pub fn nodes_to_columnar_dict<'py>(
    py: Python<'py>,
    nodes: &[Node],
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new_bound(py);
    let mut ids = Vec::with_capacity(nodes.len());
    let mut names = Vec::with_capacity(nodes.len());
    let mut kinds = Vec::with_capacity(nodes.len());
    let mut filepaths = Vec::with_capacity(nodes.len());
    let mut scopes = Vec::with_capacity(nodes.len());
    let mut start_lines = Vec::with_capacity(nodes.len());
    let mut end_lines = Vec::with_capacity(nodes.len());
    let mut signatures = Vec::with_capacity(nodes.len());

    for n in nodes {
        ids.push(n.id.as_str());
        names.push(n.name.as_str());
        kinds.push(n.kind.as_str());
        filepaths.push(n.filepath.as_str());
        scopes.push(n.scope.as_str());
        start_lines.push(n.start_line);
        end_lines.push(n.end_line);
        signatures.push(n.signature.as_deref());
    }

    dict.set_item("id", ids)?;
    dict.set_item("name", names)?;
    dict.set_item("kind", kinds)?;
    dict.set_item("filepath", filepaths)?;
    dict.set_item("scope", scopes)?;
    dict.set_item("start_line", start_lines)?;
    dict.set_item("end_line", end_lines)?;
    dict.set_item("signature", signatures)?;

    Ok(dict)
}

pub fn edges_to_columnar_dict<'py>(
    py: Python<'py>,
    edges: &[Edge],
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new_bound(py);
    let mut sources = Vec::with_capacity(edges.len());
    let mut targets = Vec::with_capacity(edges.len());
    let mut kinds = Vec::with_capacity(edges.len());

    for e in edges {
        sources.push(e.source_id.as_str());
        targets.push(e.target_id.as_str());
        kinds.push(e.kind.as_str());
    }

    dict.set_item("source", sources)?;
    dict.set_item("target", targets)?;
    dict.set_item("kind", kinds)?;

    Ok(dict)
}

pub fn complexity_to_columnar_dict<'py>(
    py: Python<'py>,
    records: &[ComplexityRecord],
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new_bound(py);
    let mut symbols = Vec::with_capacity(records.len());
    let mut files = Vec::with_capacity(records.len());
    let mut start_lines = Vec::with_capacity(records.len());
    let mut end_lines = Vec::with_capacity(records.len());
    let mut cyclomatics = Vec::with_capacity(records.len());
    let mut cognitives = Vec::with_capacity(records.len());
    let mut locs = Vec::with_capacity(records.len());
    let mut scores = Vec::with_capacity(records.len());
    let mut danger_zones = Vec::with_capacity(records.len());

    for r in records {
        symbols.push(r.name.as_str());
        files.push(r.filepath.as_str());
        start_lines.push(r.start_line);
        end_lines.push(r.end_line);
        cyclomatics.push(r.complexity.cyclomatic);
        cognitives.push(r.complexity.cognitive);
        locs.push(r.complexity.loc);
        scores.push(r.hotspot_score);
        danger_zones.push(r.test_deficit.is_danger_zone);
    }

    dict.set_item("symbol_name", symbols)?;
    dict.set_item("filepath", files)?;
    dict.set_item("start_line", start_lines)?;
    dict.set_item("end_line", end_lines)?;
    dict.set_item("cyclomatic", cyclomatics)?;
    dict.set_item("cognitive", cognitives)?;
    dict.set_item("loc", locs)?;
    dict.set_item("hotspot_score", scores)?;
    dict.set_item("is_danger_zone", danger_zones)?;

    Ok(dict)
}
