//! SemanticProcessor — graph operations for cognitive fragments.
//!
//! Handles extraction, grafting, conflict detection, and merging
//! of semantic fragments between communicating agents.

use crate::types::*;
use std::collections::{HashMap, HashSet};

/// Result of grafting a semantic fragment onto a graph.
#[derive(Debug, Clone)]
pub struct GraftResult {
    /// Number of nodes added
    pub nodes_added: usize,
    /// Number of edges added
    pub edges_added: usize,
    /// Graft points that were used
    pub graft_points_used: Vec<String>,
    /// Conflicts detected during graft
    pub conflicts: Vec<SemanticConflict>,
    /// Whether the graft was successful
    pub success: bool,
}

/// A semantic conflict between two fragments.
#[derive(Debug, Clone)]
pub struct ConflictDetail {
    /// Node ID where conflict occurs
    pub node_id: String,
    /// Value in the existing graph
    pub existing_value: String,
    /// Value in the incoming fragment
    pub incoming_value: String,
    /// Conflict severity: "low", "medium", "high"
    pub severity: String,
}

/// Processes semantic fragments for extraction, grafting, and conflict detection.
#[derive(Debug, Default)]
pub struct SemanticProcessor {
    /// The local cognitive graph (node_id -> CognitiveNode)
    pub graph: HashMap<String, CognitiveNode>,
    /// Edges in the local graph
    pub edges: Vec<CognitiveEdge>,
    /// History of graft operations
    pub graft_history: Vec<GraftResult>,
}

impl SemanticProcessor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the local cognitive graph.
    pub fn add_node(&mut self, node: CognitiveNode) {
        self.graph.insert(node.id.clone(), node);
    }

    /// Add an edge to the local cognitive graph.
    pub fn add_edge(&mut self, edge: CognitiveEdge) {
        self.edges.push(edge);
    }

    /// Extract a semantic fragment from the local graph around a set of focus nodes.
    ///
    /// Traverses from each focus node up to `context_depth` hops,
    /// collecting all reachable nodes and edges.
    pub fn extract_fragment(
        &self,
        focus_nodes: &[String],
        context_depth: usize,
        perspective: &str,
    ) -> SemanticFragment {
        let mut collected_nodes = HashSet::new();
        let mut collected_edges = Vec::new();
        let mut seen_edges: HashSet<(String, String)> = HashSet::new();

        // BFS from each focus node
        for focus in focus_nodes {
            let mut frontier = vec![focus.clone()];
            let mut visited = HashSet::new();

            for _depth in 0..context_depth {
                let mut next_frontier = Vec::new();
                for node_id in &frontier {
                    if visited.contains(node_id) {
                        continue;
                    }
                    visited.insert(node_id.clone());
                    collected_nodes.insert(node_id.clone());

                    // Find connected nodes via edges
                    for edge in &self.edges {
                        if &edge.from == node_id && !visited.contains(&edge.to) {
                            next_frontier.push(edge.to.clone());
                            let key = (edge.from.clone(), edge.to.clone());
                            if !seen_edges.contains(&key) {
                                seen_edges.insert(key);
                                collected_edges.push(edge.clone());
                            }
                        }
                        if &edge.to == node_id && !visited.contains(&edge.from) {
                            next_frontier.push(edge.from.clone());
                            let key = (edge.from.clone(), edge.to.clone());
                            if !seen_edges.contains(&key) {
                                seen_edges.insert(key);
                                collected_edges.push(edge.clone());
                            }
                        }
                    }
                }
                frontier = next_frontier;
            }
        }

        // Build fragment
        let nodes: Vec<CognitiveNode> = collected_nodes
            .iter()
            .filter_map(|id| self.graph.get(id).cloned())
            .collect();

        // Graft points are the focus nodes themselves
        let graft_points: Vec<String> = focus_nodes.to_vec();

        SemanticFragment {
            content: format!(
                "Fragment from {} focus nodes, depth {}",
                focus_nodes.len(),
                context_depth
            ),
            role: String::new(),
            confidence: 1.0,
            nodes,
            edges: collected_edges,
            graft_points,
            context: format!("depth={}, focus={}", context_depth, focus_nodes.join(",")),
            perspective: perspective.to_string(),
        }
    }

    /// Graft a received semantic fragment onto the local cognitive graph.
    ///
    /// Merges nodes and edges, detecting conflicts where existing nodes
    /// have different label than incoming ones.
    pub fn graft_fragment(&mut self, fragment: &SemanticFragment) -> GraftResult {
        let mut nodes_added = 0;
        let mut edges_added = 0;
        let mut conflicts = Vec::new();
        let mut graft_points_used = Vec::new();

        // Check graft points
        for gp in &fragment.graft_points {
            if self.graph.contains_key(gp) {
                graft_points_used.push(gp.clone());
            }
        }

        // Merge nodes
        for node in &fragment.nodes {
            if let Some(existing) = self.graph.get(&node.id) {
                // Conflict detection: same ID, different label
                if existing.label != node.label {
                    conflicts.push(SemanticConflict {
                        id: conflicts.len() as u64,
                        description: format!(
                            "Node '{}': existing='{}' vs incoming='{}'",
                            node.id, existing.label, node.label
                        ),
                        severity: "medium".to_string(),
                        channel_id: None,
                    });
                }
                // Don't overwrite existing nodes on conflict
            } else {
                self.graph.insert(node.id.clone(), node.clone());
                nodes_added += 1;
            }
        }

        // Merge edges (add new edges, skip duplicates)
        let existing_edges: HashSet<(String, String)> = self
            .edges
            .iter()
            .map(|e| (e.from.clone(), e.to.clone()))
            .collect();

        for edge in &fragment.edges {
            let key = (edge.from.clone(), edge.to.clone());
            if !existing_edges.contains(&key) {
                self.edges.push(edge.clone());
                edges_added += 1;
            }
        }

        let result = GraftResult {
            nodes_added,
            edges_added,
            graft_points_used,
            conflicts: conflicts.clone(),
            success: conflicts.is_empty(),
        };

        self.graft_history.push(result.clone());
        result
    }

    /// Detect conflicts between local graph and an incoming fragment
    /// without actually grafting.
    pub fn detect_conflicts(&self, fragment: &SemanticFragment) -> Vec<ConflictDetail> {
        let mut conflicts = Vec::new();

        for node in &fragment.nodes {
            if let Some(existing) = self.graph.get(&node.id) {
                if existing.label != node.label {
                    let severity =
                        if existing.node_type != node.node_type { "high" } else { "medium" };
                    conflicts.push(ConflictDetail {
                        node_id: node.id.clone(),
                        existing_value: existing.label.clone(),
                        incoming_value: node.label.clone(),
                        severity: severity.to_string(),
                    });
                }
            }
        }

        conflicts
    }

    /// Get the number of nodes in the local graph.
    pub fn node_count(&self) -> usize {
        self.graph.len()
    }

    /// Get the number of edges in the local graph.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: &str) -> Option<&CognitiveNode> {
        self.graph.get(id)
    }

    /// Get all conflict history from grafts.
    pub fn conflict_history(&self) -> Vec<&[SemanticConflict]> {
        self.graft_history
            .iter()
            .filter(|r| !r.conflicts.is_empty())
            .map(|r| r.conflicts.as_slice())
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a CognitiveNode with defaults.
    fn make_node(id: &str, label: &str) -> CognitiveNode {
        CognitiveNode {
            id: id.to_string(),
            label: label.to_string(),
            node_type: CognitiveNodeType::default(),
            confidence: 0.0,
            source: None,
            content: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Helper: create a CognitiveEdge with defaults.
    fn make_edge(from: &str, to: &str) -> CognitiveEdge {
        CognitiveEdge {
            from: from.to_string(),
            to: to.to_string(),
            edge_type: CognitiveEdgeType::default(),
            weight: 1.0,
        }
    }

    /// Build a small test graph:
    ///   A -> B -> C
    ///        B -> D
    fn build_test_processor() -> SemanticProcessor {
        let mut sp = SemanticProcessor::new();
        sp.add_node(make_node("A", "Node A"));
        sp.add_node(make_node("B", "Node B"));
        sp.add_node(make_node("C", "Node C"));
        sp.add_node(make_node("D", "Node D"));
        sp.add_edge(make_edge("A", "B"));
        sp.add_edge(make_edge("B", "C"));
        sp.add_edge(make_edge("B", "D"));
        sp
    }

    #[test]
    fn test_extract_fragment_basic() {
        let sp = build_test_processor();

        // Extract around node B with depth 2 — should reach A, B, C, D
        let fragment = sp.extract_fragment(&["B".to_string()], 2, "test-agent");

        assert!(!fragment.nodes.is_empty(), "Should collect at least some nodes");
        let node_ids: HashSet<String> = fragment.nodes.iter().map(|n| n.id.clone()).collect();
        // Depth 1: B itself, then neighbors A, C, D.  Depth 2: nothing new.
        assert!(node_ids.contains("B"), "Focus node B must be present");
        assert!(node_ids.contains("A"), "Neighbor A must be present");
        assert!(node_ids.contains("C"), "Neighbor C must be present");
        assert!(node_ids.contains("D"), "Neighbor D must be present");

        assert!(!fragment.edges.is_empty(), "Should collect edges");
        assert_eq!(fragment.perspective, "test-agent");
        assert_eq!(fragment.graft_points, vec!["B".to_string()]);
    }

    #[test]
    fn test_graft_fragment_no_conflicts() {
        let mut sp = SemanticProcessor::new();
        sp.add_node(make_node("X", "Node X"));

        // Build a non-overlapping fragment
        let fragment = SemanticFragment {
            content: "test fragment".to_string(),
            role: String::new(),
            confidence: 1.0,
            nodes: vec![make_node("Y", "Node Y"), make_node("Z", "Node Z")],
            edges: vec![make_edge("Y", "Z")],
            graft_points: vec!["X".to_string()],
            context: String::new(),
            perspective: "remote-agent".to_string(),
        };

        let result = sp.graft_fragment(&fragment);

        assert!(result.success, "Graft should succeed without conflicts");
        assert_eq!(result.nodes_added, 2, "Should add Y and Z");
        assert_eq!(result.edges_added, 1, "Should add Y->Z edge");
        assert_eq!(result.graft_points_used, vec!["X".to_string()]);
        assert!(result.conflicts.is_empty());

        // Verify nodes are in the graph
        assert_eq!(sp.node_count(), 3); // X + Y + Z
        assert_eq!(sp.edge_count(), 1);
        assert!(sp.get_node("Y").is_some());
        assert!(sp.get_node("Z").is_some());
    }

    #[test]
    fn test_graft_fragment_with_conflicts() {
        let mut sp = SemanticProcessor::new();
        sp.add_node(make_node("A", "Original A"));
        sp.add_node(make_node("B", "Original B"));

        // Fragment with a conflicting node A (different label)
        let fragment = SemanticFragment {
            content: "conflicting fragment".to_string(),
            role: String::new(),
            confidence: 1.0,
            nodes: vec![
                make_node("A", "Modified A"), // conflict
                make_node("C", "New C"),       // no conflict
            ],
            edges: vec![make_edge("A", "C")],
            graft_points: vec!["A".to_string()],
            context: String::new(),
            perspective: "remote-agent".to_string(),
        };

        let result = sp.graft_fragment(&fragment);

        assert!(!result.success, "Graft should report conflicts");
        assert_eq!(result.conflicts.len(), 1, "Should have exactly one conflict");
        assert!(
            result.conflicts[0].description.contains("Original A"),
            "Conflict should mention existing value"
        );
        assert!(
            result.conflicts[0].description.contains("Modified A"),
            "Conflict should mention incoming value"
        );

        // Existing node should NOT be overwritten
        assert_eq!(sp.get_node("A").unwrap().label, "Original A");
        // New node should be added
        assert!(sp.get_node("C").is_some());
        assert_eq!(result.nodes_added, 1); // only C
    }

    #[test]
    fn test_detect_conflicts_without_grafting() {
        let mut sp = SemanticProcessor::new();
        sp.add_node(make_node("A", "Original A"));
        sp.add_node(make_node("B", "Original B"));

        let fragment = SemanticFragment {
            content: "probe fragment".to_string(),
            role: String::new(),
            confidence: 1.0,
            nodes: vec![
                make_node("A", "Modified A"), // conflict
                make_node("B", "Original B"), // no conflict (same label)
                make_node("C", "Brand new"),  // no conflict (new node)
            ],
            edges: vec![],
            graft_points: vec![],
            context: String::new(),
            perspective: "checker".to_string(),
        };

        let conflicts = sp.detect_conflicts(&fragment);

        assert_eq!(conflicts.len(), 1, "Only node A should conflict");
        assert_eq!(conflicts[0].node_id, "A");
        assert_eq!(conflicts[0].existing_value, "Original A");
        assert_eq!(conflicts[0].incoming_value, "Modified A");
        assert_eq!(conflicts[0].severity, "medium");

        // Graph should be unchanged
        assert_eq!(sp.node_count(), 2, "No nodes should be added");
        assert_eq!(sp.edge_count(), 0, "No edges should be added");
    }

    #[test]
    fn test_extract_respects_depth() {
        // Graph: A -> B -> C -> D -> E (chain)
        let mut sp = SemanticProcessor::new();
        sp.add_node(make_node("A", "Node A"));
        sp.add_node(make_node("B", "Node B"));
        sp.add_node(make_node("C", "Node C"));
        sp.add_node(make_node("D", "Node D"));
        sp.add_node(make_node("E", "Node E"));
        sp.add_edge(make_edge("A", "B"));
        sp.add_edge(make_edge("B", "C"));
        sp.add_edge(make_edge("C", "D"));
        sp.add_edge(make_edge("D", "E"));

        // Extract from A with depth=1: should get only A (visited at depth 0)
        // and B pushed onto frontier but visited at depth 1
        let frag_1 = sp.extract_fragment(&["A".to_string()], 1, "test");
        let ids_1: HashSet<String> = frag_1.nodes.iter().map(|n| n.id.clone()).collect();
        assert!(ids_1.contains("A"), "Depth 1: A should be present");
        // At depth 0 iteration: A is visited, B is pushed to next_frontier
        // Then the loop ends (only 1 iteration). B is NOT visited.
        assert!(!ids_1.contains("B"), "Depth 1: B should NOT be reached (it is in the frontier but not visited)");

        // Extract from A with depth=2: should reach A, B
        let frag_2 = sp.extract_fragment(&["A".to_string()], 2, "test");
        let ids_2: HashSet<String> = frag_2.nodes.iter().map(|n| n.id.clone()).collect();
        assert!(ids_2.contains("A"));
        assert!(ids_2.contains("B"));
        // C is pushed to frontier at depth 1, not visited yet
        assert!(!ids_2.contains("C"), "Depth 2: C should NOT be reached");

        // Extract from A with depth=3: should reach A, B, C
        let frag_3 = sp.extract_fragment(&["A".to_string()], 3, "test");
        let ids_3: HashSet<String> = frag_3.nodes.iter().map(|n| n.id.clone()).collect();
        assert!(ids_3.contains("A"));
        assert!(ids_3.contains("B"));
        assert!(ids_3.contains("C"));
        assert!(!ids_3.contains("D"), "Depth 3: D should NOT be reached");

        // Depth=5 from A: should reach everything
        let frag_5 = sp.extract_fragment(&["A".to_string()], 5, "test");
        let ids_5: HashSet<String> = frag_5.nodes.iter().map(|n| n.id.clone()).collect();
        assert_eq!(ids_5.len(), 5, "Depth 5 from A in chain of 5 should get all nodes");
    }

    #[test]
    fn test_empty_graph_extract() {
        let sp = SemanticProcessor::new();

        let fragment = sp.extract_fragment(&["nonexistent".to_string()], 3, "empty");

        assert!(fragment.nodes.is_empty(), "Empty graph should yield no nodes");
        assert!(fragment.edges.is_empty(), "Empty graph should yield no edges");
        assert_eq!(fragment.graft_points, vec!["nonexistent".to_string()]);
        assert_eq!(fragment.perspective, "empty");
    }
}
