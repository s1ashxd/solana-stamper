use std::collections::{BTreeMap, BTreeSet};

pub fn topo_sort(graph: &BTreeMap<&str, Vec<&str>>) -> Vec<String> {
    let mut visited: BTreeSet<&str> = BTreeSet::new();
    let mut out: Vec<String> = Vec::new();

    fn visit<'a>(
        node: &'a str,
        graph: &'a BTreeMap<&'a str, Vec<&'a str>>,
        visited: &mut BTreeSet<&'a str>,
        out: &mut Vec<String>,
    ) {
        if !visited.insert(node) {
            return;
        }
        if let Some(deps) = graph.get(node) {
            for dep in deps {
                visit(dep, graph, visited, out);
            }
        }
        out.push(node.to_string());
    }

    let keys: Vec<&str> = graph.keys().copied().collect();
    for node in keys {
        visit(node, graph, &mut visited, &mut out);
    }
    out
}
