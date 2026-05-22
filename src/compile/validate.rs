use std::collections::{BTreeMap, BTreeSet};

use crate::error::StamperError;
use crate::spec::TemplateSpec;
use crate::spec::account::Acc;
use crate::spec::data::DataPiece;

pub fn validate_spec(spec: &TemplateSpec) -> Result<(), StamperError> {
    let mut names: BTreeSet<&str> = BTreeSet::new();
    let mut computed: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut per_provider: BTreeSet<&str> = BTreeSet::new();

    for ix in &spec.ixs {
        for acc in &ix.accounts {
            match acc {
                Acc::Fixed(_, _) | Acc::Payer(_) => {}
                Acc::Slot { name, per_provider: pp, .. } => {
                    if !names.insert(name) {
                        return Err(StamperError::DuplicateSlotName { name: (*name).to_string() });
                    }
                    if *pp {
                        per_provider.insert(name);
                    }
                }
                Acc::Derived { name, deps, .. } => {
                    if !names.insert(name) {
                        return Err(StamperError::DuplicateSlotName { name: (*name).to_string() });
                    }
                    computed.insert(name, deps.iter().copied().collect());
                }
                Acc::AdditionalSigner { name, .. } => {
                    if !names.insert(name) {
                        return Err(StamperError::DuplicateSlotName { name: (*name).to_string() });
                    }
                }
            }
        }
        for piece in &ix.data.0 {
            let slot = match piece {
                DataPiece::U8Slot(n) | DataPiece::U16Slot(n) | DataPiece::U32Slot(n)
                | DataPiece::U64Slot(n) | DataPiece::PubkeySlot(n) => Some(*n),
                DataPiece::Bytes(_) => None,
            };
            if let Some(name) = slot {
                if !names.insert(name) {
                    return Err(StamperError::DuplicateSlotName { name: name.to_string() });
                }
            }
        }
    }

    for (computed_name, deps) in &computed {
        for dep in deps {
            if !names.contains(dep) {
                return Err(StamperError::MissingDependency {
                    computed: (*computed_name).to_string(),
                    missing: (*dep).to_string(),
                });
            }
            if per_provider.contains(dep) {
                return Err(StamperError::ComputedDependsOnPerProvider {
                    computed: (*computed_name).to_string(),
                    dep: (*dep).to_string(),
                });
            }
        }
    }

    detect_cycles(&computed)?;
    Ok(())
}

fn detect_cycles(graph: &BTreeMap<&str, Vec<&str>>) -> Result<(), StamperError> {
    #[derive(Clone, Copy)]
    enum Color { White, Gray, Black }
    let mut color: BTreeMap<&str, Color> = graph.keys().map(|k| (*k, Color::White)).collect();
    let mut path: Vec<&str> = Vec::new();

    fn dfs<'a>(
        node: &'a str,
        graph: &'a BTreeMap<&'a str, Vec<&'a str>>,
        color: &mut BTreeMap<&'a str, Color>,
        path: &mut Vec<&'a str>,
    ) -> Result<(), Vec<String>> {
        color.insert(node, Color::Gray);
        path.push(node);
        if let Some(deps) = graph.get(node) {
            for dep in deps {
                match color.get(dep) {
                    Some(Color::Gray) => {
                        let mut cycle: Vec<String> = path.iter().map(|s| (*s).to_string()).collect();
                        cycle.push((*dep).to_string());
                        return Err(cycle);
                    }
                    Some(Color::White) => {
                        dfs(dep, graph, color, path)?;
                    }
                    _ => {}
                }
            }
        }
        color.insert(node, Color::Black);
        path.pop();
        Ok(())
    }

    let node_keys: Vec<&str> = graph.keys().copied().collect();
    for node in node_keys {
        if matches!(color.get(node), Some(Color::White)) {
            if let Err(cycle) = dfs(node, graph, &mut color, &mut path) {
                return Err(StamperError::CyclicComputed { cycle });
            }
        }
    }
    Ok(())
}
