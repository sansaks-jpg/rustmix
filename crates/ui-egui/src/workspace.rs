//! Validate untrusted docking topology before handing it to the docking library.
use super::*;
use serde_json::Value;
use std::collections::HashSet;
fn bad() -> Error {
    Error::Invalid("Invalid saved panel layout; reset to Production".into())
}
pub(crate) fn decode(json: &str) -> Result<DockState<Panel>> {
    if json.len() > 65536 {
        return Err(bad());
    }
    let value: Value = serde_json::from_str(json)?;
    let surfaces = value
        .get("surfaces")
        .and_then(Value::as_array)
        .ok_or_else(bad)?;
    if surfaces.is_empty() || surfaces.len() > 5 {
        return Err(bad());
    }
    if surfaces.first().and_then(|s| s.get("Main")).is_none() {
        return Err(bad());
    }
    let mut seen = HashSet::new();
    for surface in surfaces {
        if surface.as_str() == Some("Empty") {
            continue;
        }
        let tree = surface
            .get("Main")
            .or_else(|| {
                surface
                    .get("Window")
                    .and_then(Value::as_array)
                    .and_then(|a| a.first())
            })
            .ok_or_else(bad)?;
        let nodes = tree
            .get("nodes")
            .and_then(Value::as_array)
            .ok_or_else(bad)?;
        if nodes.len() > 31 {
            return Err(bad());
        }
        for (index, node) in nodes.iter().enumerate() {
            if node.as_str() == Some("Empty") {
                continue;
            }
            if index > 0 {
                let parent = nodes.get((index - 1) / 2).ok_or_else(bad)?;
                if parent.get("Vertical").is_none() && parent.get("Horizontal").is_none() {
                    return Err(bad());
                }
            }
            if let Some(leaf) = node.get("Leaf") {
                let tabs = leaf.get("tabs").and_then(Value::as_array).ok_or_else(bad)?;
                let active = leaf.get("active").and_then(Value::as_u64).ok_or_else(bad)?;
                if tabs.is_empty() || tabs.len() > 4 || active >= tabs.len() as u64 {
                    return Err(bad());
                }
                for tab in tabs {
                    let name = tab.as_str().ok_or_else(bad)?;
                    if !matches!(name, "Inputs" | "Audio" | "Overlays")
                        || !seen.insert(name.to_owned())
                    {
                        return Err(bad());
                    }
                }
            } else if let Some(split) = node.get("Vertical").or_else(|| node.get("Horizontal")) {
                let fraction = split
                    .get("fraction")
                    .and_then(Value::as_f64)
                    .ok_or_else(bad)?;
                if !(0.0..=1.0).contains(&fraction) {
                    return Err(bad());
                }
                for child in [index * 2 + 1, index * 2 + 2] {
                    if nodes.get(child).is_none_or(|n| n.as_str() == Some("Empty")) {
                        return Err(bad());
                    }
                }
            } else {
                return Err(bad());
            }
        }
        if let Some(focus) = tree.get("focused_node").and_then(Value::as_u64)
            && nodes
                .get(focus as usize)
                .and_then(|n| n.get("Leaf"))
                .is_none()
        {
            return Err(bad());
        }
    }
    if seen.len() != 3 {
        return Err(bad());
    }
    if let Some(focus) = value.get("focused_surface").and_then(Value::as_u64)
        && surfaces
            .get(focus as usize)
            .is_none_or(|s| s.as_str() == Some("Empty"))
    {
        return Err(bad());
    }
    Ok(serde_json::from_value(value)?)
}
