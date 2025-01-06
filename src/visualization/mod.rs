use petgraph::graph::{Graph, NodeIndex};
use petgraph::dot::{Dot, Config};
use std::collections::HashMap;
use serde_json::Value;
use anyhow::Result;

pub struct NetworkGraph {
    graph: Graph<String, String>,
    node_indices: HashMap<String, NodeIndex>,
    node_capacities: HashMap<String, u64>,
    node_states: HashMap<String, String>,
}

impl NetworkGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_indices: HashMap::new(),
            node_capacities: HashMap::new(),
            node_states: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node_id: &str, alias: &str) -> NodeIndex {
        if let Some(&idx) = self.node_indices.get(node_id) {
            return idx;
        }
        let capacity = self.node_capacities.get(node_id).unwrap_or(&0);
        let unknown = "unknown".to_string();
        let state = self.node_states.get(node_id).unwrap_or(&unknown);
        let node_label = format!(
            "{}\n({})\nCapacité: {} sats\nÉtat: {}",
            alias,
            node_id.chars().take(8).collect::<String>(),
            capacity,
            state
        );
        let idx = self.graph.add_node(node_label);
        self.node_indices.insert(node_id.to_string(), idx);
        idx
    }

    pub fn add_channel(&mut self, from_id: &str, to_id: &str, capacity: &str) {
        if let (Some(&from_idx), Some(&to_idx)) = (
            self.node_indices.get(from_id),
            self.node_indices.get(to_id)
        ) {
            let edge_label = format!(
                "Capacité: {}\nÉtat: actif",
                capacity
            );
            self.graph.add_edge(from_idx, to_idx, edge_label);
        }
    }

    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph {\n");
        dot.push_str("    rankdir=LR;\n");
        dot.push_str("    splines=curved;\n");
        dot.push_str("    bgcolor=\"#ffffff\";\n");
        dot.push_str("    node [\n");
        dot.push_str("        style=\"filled\",\n");
        dot.push_str("        gradientangle=270,\n");
        dot.push_str("        fillcolor=\"#88c0d0:#5e81ac\",\n");
        dot.push_str("        shape=\"box\",\n");
        dot.push_str("        rounded=true,\n");
        dot.push_str("        fontname=\"Arial\",\n");
        dot.push_str("        fontsize=12\n");
        dot.push_str("    ];\n");
        dot.push_str("    edge [\n");
        dot.push_str("        color=\"#a3be8c\",\n");
        dot.push_str("        penwidth=2.0,\n");
        dot.push_str("        arrowsize=0.8\n");
        dot.push_str("    ];\n");
        
        dot.push_str(&format!("{:?}", Dot::with_config(&self.graph, &[
            Config::NodeIndexLabel,
            Config::EdgeNoLabel,
            Config::GraphContentOnly,
        ])));
        
        dot.push_str("\n    subgraph cluster_legend {\n");
        dot.push_str("        label=\"Légende\";\n");
        dot.push_str("        node [shape=none];\n");
        dot.push_str("        legend [label=<\n");
        dot.push_str("            <table border=\"0\">\n");
        dot.push_str("                <tr><td>Nœud actif</td></tr>\n");
        dot.push_str("                <tr><td>Canal ouvert</td></tr>\n");
        dot.push_str("            </table>\n");
        dot.push_str("        >];\n");
        dot.push_str("    }\n");
        dot.push_str("}\n");
        
        dot
    }

    pub fn update_from_node_info(&mut self, node_info: &Value, channels: &Value) -> Result<()> {
        let node_id = node_info["result"]["id"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Node ID not found"))?;
        let alias = node_info["result"]["alias"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Node alias not found"))?;

        if let Some(channel_list) = channels["result"]["channels"].as_array() {
            let total_capacity: u64 = channel_list.iter()
                .filter_map(|c| c["amount_msat"].as_str())
                .filter_map(|amount| amount.parse::<u64>().ok())
                .sum();
            self.node_capacities.insert(node_id.to_string(), total_capacity);
            self.node_states.insert(node_id.to_string(), "actif".to_string());
        }

        self.add_node(node_id, alias);

        if let Some(channel_list) = channels["result"]["channels"].as_array() {
            for channel in channel_list {
                if let (Some(peer_id), Some(capacity)) = (
                    channel["peer_id"].as_str(),
                    channel["amount_msat"].as_str()
                ) {
                    if !self.node_indices.contains_key(peer_id) {
                        self.add_node(peer_id, "Unknown");
                    }
                    self.add_channel(node_id, peer_id, capacity);
                }
            }
        }

        Ok(())
    }
} 