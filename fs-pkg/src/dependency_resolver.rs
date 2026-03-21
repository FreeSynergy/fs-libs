// dependency_resolver.rs — Service dependency graph for FreeSynergy packages.
//
// Each package declares which other packages it requires. This module builds
// a directed dependency graph and resolves the correct install order using
// topological sorting (Kahn's algorithm).
//
// Design:
//   DepGraph             — directed graph: package_id → [dependencies]
//   DependencyResolver   — builds install order, detects cycles
//   ResolutionError      — cycle or missing dependency
//
// Pattern: Builder (DepGraph), Algorithm encapsulation (DependencyResolver).

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

// ── PackageDep ────────────────────────────────────────────────────────────────

/// One package with its declared dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDep {
    /// Unique package identifier, e.g. `"iam/kanidm"`.
    pub id: String,

    /// IDs of packages that must be installed before this one.
    #[serde(default)]
    pub requires: Vec<String>,
}

// ── ResolutionError ───────────────────────────────────────────────────────────

/// Why dependency resolution failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionError {
    /// A cyclic dependency was detected (the vector lists the cycle).
    Cycle(Vec<String>),
    /// A required package is not registered in the graph.
    MissingPackage { required_by: String, missing: String },
}

impl std::fmt::Display for ResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cycle(cycle) => write!(f, "dependency cycle detected: {}", cycle.join(" → ")),
            Self::MissingPackage { required_by, missing } => {
                write!(f, "'{required_by}' requires '{missing}', which is not registered")
            }
        }
    }
}

// ── DepGraph ──────────────────────────────────────────────────────────────────

/// Directed dependency graph.
///
/// Nodes are package IDs. Edges point from a package to its dependency
/// (i.e. `"bookstack" → "kanidm"` means bookstack requires kanidm).
///
/// # Example
///
/// ```rust
/// use fs_pkg::dependency_resolver::{DepGraph, PackageDep};
///
/// let mut graph = DepGraph::new();
/// graph.add(PackageDep { id: "database/postgres".into(), requires: vec![] });
/// graph.add(PackageDep { id: "iam/kanidm".into(), requires: vec!["database/postgres".into()] });
/// graph.add(PackageDep { id: "wiki/outline".into(), requires: vec!["iam/kanidm".into()] });
///
/// let order = graph.install_order().unwrap();
/// let db_pos = order.iter().position(|s| s == "database/postgres").unwrap();
/// let iam_pos = order.iter().position(|s| s == "iam/kanidm").unwrap();
/// assert!(db_pos < iam_pos);
/// ```
#[derive(Debug, Clone, Default)]
pub struct DepGraph {
    nodes: HashMap<String, PackageDep>,
}

impl DepGraph {
    /// Create an empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a package. Re-registering replaces the existing entry.
    pub fn add(&mut self, pkg: PackageDep) {
        self.nodes.insert(pkg.id.clone(), pkg);
    }

    /// Remove a package by ID.
    pub fn remove(&mut self, id: &str) {
        self.nodes.remove(id);
    }

    /// Returns all direct dependencies of a package.
    pub fn deps_of(&self, id: &str) -> &[String] {
        self.nodes.get(id).map(|p| p.requires.as_slice()).unwrap_or(&[])
    }

    /// Returns all packages that directly depend on `id`.
    pub fn dependents_of(&self, id: &str) -> Vec<&str> {
        self.nodes
            .values()
            .filter(|p| p.requires.iter().any(|r| r == id))
            .map(|p| p.id.as_str())
            .collect()
    }

    /// Compute a topological install order (Kahn's algorithm).
    ///
    /// Returns a list of package IDs in the order they should be installed —
    /// dependencies always precede their dependents.
    pub fn install_order(&self) -> Result<Vec<String>, ResolutionError> {
        DependencyResolver::new(self).resolve()
    }

    /// All registered package IDs.
    pub fn package_ids(&self) -> Vec<&str> {
        self.nodes.keys().map(String::as_str).collect()
    }

    /// Check whether all declared dependencies are registered.
    pub fn validate(&self) -> Result<(), ResolutionError> {
        for pkg in self.nodes.values() {
            for req in &pkg.requires {
                if !self.nodes.contains_key(req.as_str()) {
                    return Err(ResolutionError::MissingPackage {
                        required_by: pkg.id.clone(),
                        missing:     req.clone(),
                    });
                }
            }
        }
        Ok(())
    }
}

// ── DependencyResolver ────────────────────────────────────────────────────────

/// Topological sort over a `DepGraph`.
struct DependencyResolver<'a> {
    graph: &'a DepGraph,
}

impl<'a> DependencyResolver<'a> {
    fn new(graph: &'a DepGraph) -> Self {
        Self { graph }
    }

    /// Kahn's algorithm — O(V + E).
    fn resolve(&self) -> Result<Vec<String>, ResolutionError> {
        // Validate first: catch missing deps early.
        self.graph.validate()?;

        // Build in-degree map.
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for id in self.graph.nodes.keys() {
            in_degree.entry(id.as_str()).or_insert(0);
        }
        for pkg in self.graph.nodes.values() {
            for dep in &pkg.requires {
                *in_degree.entry(dep.as_str()).or_insert(0) += 0; // ensure present
                // increment the requirer's consumer count — actually we need
                // reverse: dep is a prerequisite of pkg, so pkg has in-degree
                // incremented for each of its dependencies.
            }
        }

        // Re-compute correctly: in_degree[pkg] = number of its requirements.
        let mut in_degree: HashMap<String, usize> = self
            .graph
            .nodes
            .values()
            .map(|p| (p.id.clone(), p.requires.len()))
            .collect();

        // Reverse adjacency: dep → [packages that depend on dep]
        let mut rev_adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for pkg in self.graph.nodes.values() {
            for dep in &pkg.requires {
                rev_adj.entry(dep.as_str()).or_default().push(pkg.id.as_str());
            }
        }

        // Start with nodes that have no dependencies.
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(id, _)| id.clone())
            .collect();

        // Sort for deterministic output.
        let mut sorted_init: Vec<String> = queue.drain(..).collect();
        sorted_init.sort();
        queue.extend(sorted_init);

        let mut result = Vec::with_capacity(self.graph.nodes.len());

        while let Some(id) = queue.pop_front() {
            result.push(id.clone());

            if let Some(dependents) = rev_adj.get(id.as_str()) {
                let mut next_ready: Vec<String> = Vec::new();
                for &dep in dependents {
                    let count = in_degree.get_mut(dep).unwrap();
                    *count -= 1;
                    if *count == 0 {
                        next_ready.push(dep.to_string());
                    }
                }
                next_ready.sort();
                queue.extend(next_ready);
            }
        }

        if result.len() != self.graph.nodes.len() {
            // Not all nodes were processed → cycle exists.
            let in_cycle: Vec<String> = in_degree
                .iter()
                .filter(|(_, &d)| d > 0)
                .map(|(id, _)| id.clone())
                .collect();
            Err(ResolutionError::Cycle(in_cycle))
        } else {
            Ok(result)
        }
    }
}

// ── SubgraphBuilder ───────────────────────────────────────────────────────────

/// Utility: build the transitive dependency set for a list of package IDs.
pub fn transitive_deps(graph: &DepGraph, roots: &[&str]) -> HashSet<String> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut stack: Vec<&str> = roots.to_vec();
    while let Some(id) = stack.pop() {
        if !visited.insert(id.to_string()) {
            continue;
        }
        for dep in graph.deps_of(id) {
            stack.push(dep.as_str());
        }
    }
    visited
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_graph() -> DepGraph {
        let mut g = DepGraph::new();
        g.add(PackageDep { id: "database/postgres".into(), requires: vec![] });
        g.add(PackageDep {
            id:       "iam/kanidm".into(),
            requires: vec!["database/postgres".into()],
        });
        g.add(PackageDep {
            id:       "wiki/outline".into(),
            requires: vec!["iam/kanidm".into(), "database/postgres".into()],
        });
        g.add(PackageDep {
            id:       "proxy/zentinel".into(),
            requires: vec![],
        });
        g
    }

    #[test]
    fn install_order_respects_deps() {
        let g = simple_graph();
        let order = g.install_order().unwrap();

        let pos = |id: &str| order.iter().position(|s| s == id).unwrap();
        assert!(pos("database/postgres") < pos("iam/kanidm"));
        assert!(pos("iam/kanidm")        < pos("wiki/outline"));
        assert!(pos("database/postgres") < pos("wiki/outline"));
    }

    #[test]
    fn install_order_contains_all() {
        let g = simple_graph();
        let order = g.install_order().unwrap();
        assert_eq!(order.len(), 4);
    }

    #[test]
    fn cycle_detection() {
        let mut g = DepGraph::new();
        g.add(PackageDep { id: "a".into(), requires: vec!["b".into()] });
        g.add(PackageDep { id: "b".into(), requires: vec!["c".into()] });
        g.add(PackageDep { id: "c".into(), requires: vec!["a".into()] });

        let err = g.install_order().unwrap_err();
        assert!(matches!(err, ResolutionError::Cycle(_)));
    }

    #[test]
    fn missing_dependency_error() {
        let mut g = DepGraph::new();
        g.add(PackageDep {
            id:       "iam/kanidm".into(),
            requires: vec!["database/postgres".into()], // not registered
        });

        let err = g.validate().unwrap_err();
        assert!(matches!(err, ResolutionError::MissingPackage { .. }));
    }

    #[test]
    fn dependents_of() {
        let g = simple_graph();
        let mut dependents = g.dependents_of("database/postgres");
        dependents.sort();
        assert!(dependents.contains(&"iam/kanidm"));
        assert!(dependents.contains(&"wiki/outline"));
    }

    #[test]
    fn transitive_deps_includes_all() {
        let g = simple_graph();
        let deps = transitive_deps(&g, &["wiki/outline"]);
        assert!(deps.contains("iam/kanidm"));
        assert!(deps.contains("database/postgres"));
        assert!(deps.contains("wiki/outline")); // self is included
    }

    #[test]
    fn no_dependencies_any_order() {
        let mut g = DepGraph::new();
        g.add(PackageDep { id: "a".into(), requires: vec![] });
        g.add(PackageDep { id: "b".into(), requires: vec![] });
        g.add(PackageDep { id: "c".into(), requires: vec![] });
        let order = g.install_order().unwrap();
        assert_eq!(order.len(), 3);
    }
}
