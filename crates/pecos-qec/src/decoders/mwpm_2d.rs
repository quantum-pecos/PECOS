use pecos_core::QubitId;
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use std::collections::HashMap;

// Recreate pecos/decoders/mwpm2d/{mwpm2d/py, precomputing.py}

#[derive(Debug)]
pub struct PrecomputedData {
    x_graph: Graph<(), f64>,
    z_graph: Graph<(), f64>,
    virtual_edges: HashMap<NodeIndex, VirtualEdgeData>,
    edge_to_data: HashMap<(NodeIndex, NodeIndex), QubitId>,
}

#[derive(Debug)]
pub struct VirtualEdgeData {
    pub virtual_node: NodeIndex,
    pub weight: f64,
    pub syndrome_path: Vec<NodeIndex>,
    pub data_path: Vec<QubitId>,
}

struct Syndrome {}
struct Recovery {}
struct SurfaceCode {}

// #[derive(Debug)]
pub struct MWPM2DDecoder {
    precomputed: PrecomputedData,
    cached_recoveries: HashMap<Vec<Syndrome>, Recovery>,
}

impl MWPM2DDecoder {
    pub fn new(code: &SurfaceCode) -> Self {
        let precomputed = Self::precompute(code);
        Self {
            precomputed,
            cached_recoveries: HashMap::new(),
        }
    }

    fn precompute(code: &SurfaceCode) -> PrecomputedData {
        // Build distance graphs and lookup tables
        // Similar to precomputing.py
        PrecomputedData {
            x_graph: Graph::new(),
            z_graph: Graph::new(),
            virtual_edges: HashMap::new(),
            edge_to_data: HashMap::new(),
        }
    }

    pub fn decode(&mut self, syndromes: &[Syndrome]) -> Recovery {
        // 1. Check cache
        // 2. Build matching graph
        // 3. Add virtual nodes
        // 4. Find maximum weight perfect matching
        // 5. Convert matching to recovery operations
        Recovery {}
    }
}
