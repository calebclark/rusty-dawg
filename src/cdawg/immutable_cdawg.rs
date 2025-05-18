// An immutable version of CDawg

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use crate::cdawg::Cdawg;
use crate::cdawg::cdawg_edge_weight::CdawgEdgeWeight;
use crate::cdawg::cdawg_state::CdawgState;
use crate::cdawg::comparator::CdawgComparator;
use crate::cdawg::metadata::CdawgMetadata;
use crate::cdawg::token_backing::TokenBacking;
use crate::graph::avl_graph::AvlGraph;
use crate::graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};
use crate::graph::{EdgeRef, NodeRef};
use crate::memory_backing::{CacheConfig, DiskBacking, MemoryBacking, RamBacking};
use crate::weight::{DefaultWeight, Weight};


pub struct ImmutableCdawg<W = DefaultWeight, Ix = DefaultIx, Mb = RamBacking<W, CdawgEdgeWeight<Ix>, Ix>>
where
    Ix: IndexType,
    W: Weight + Clone,
    Mb: MemoryBacking<W, CdawgEdgeWeight<Ix>, Ix>,
{
    tokens: Rc<RefCell<dyn TokenBacking<u16>>>,
    graph: AvlGraph<W, CdawgEdgeWeight<Ix>, Ix, Mb>,
    source: NodeIndex<Ix>,
    sink: NodeIndex<Ix>,
    end_position: usize, // End position of current document.
}

impl<W, Ix> ImmutableCdawg<W, Ix>
where
    Ix: IndexType,
    W: Weight + Serialize + for<'de> Deserialize<'de> + Clone,
{
    pub fn new(mutable_cdawg : Cdawg<W, Ix>) -> Self {
        let mb: RamBacking<W, CdawgEdgeWeight<Ix>, Ix> = RamBacking::default();
        Self::new_mb(mutable_cdawg, mb)
    }
}

impl<W, Ix> ImmutableCdawg<W, Ix, DiskBacking<W, CdawgEdgeWeight<Ix>, Ix>>
where
    Ix: IndexType + Serialize + for<'de> serde::Deserialize<'de>,
    W: Weight + Copy + Serialize + for<'de> Deserialize<'de> + Clone + Default,
    CdawgEdgeWeight<Ix>: Serialize + for<'de> Deserialize<'de>,
{
    pub fn load<P: AsRef<Path> + Clone + std::fmt::Debug>(
        tokens: Rc<RefCell<dyn TokenBacking<u16>>>,
        path: P,
        cache_config: CacheConfig,
    ) -> Result<Self> {
        // Load source/sink from config file if it exists.
        let path2 = path.clone();
        let graph = AvlGraph::load(path, cache_config)?;

        let mut config_path = path2.as_ref().to_path_buf();
        config_path.push("metadata.json");
        if config_path.exists() {
            // FIXME(#98): This will fail silently if config file exists but is empty.
            let config = CdawgMetadata::load_json(config_path)?;
            Ok(Self {
                tokens,
                graph,
                source: NodeIndex::new(config.source),
                sink: NodeIndex::new(config.sink),
                end_position: config.end_position,
            })
        } else {
            Ok(Self {
                tokens,
                graph,
                source: NodeIndex::new(0),
                sink: NodeIndex::new(1),
                end_position: 0,
            })
        }
    }
}

impl<W, Ix, Mb> ImmutableCdawg<W, Ix, Mb>
where
    Ix: IndexType,
    W: Weight + Serialize + for<'de> Deserialize<'de> + Clone,
    Mb: MemoryBacking<W, CdawgEdgeWeight<Ix>, Ix>,
    Mb::EdgeRef: Copy,
{
    pub fn new_mb(mutable_cdawg : Cdawg<W, Ix>, mb: Mb, cache_config: CacheConfig) -> ImmutableCdawg<W, Ix, Mb> {
        let mut graph: AvlGraph<W, CdawgEdgeWeight<Ix>, Ix, Mb> = AvlGraph::new_mb(mb);
        let source = graph.add_node(W::new(0, None, 0));
        // FIXME: Hacky type conversion for sink failure.
        let sink = graph.add_node(W::new(0, Some(NodeIndex::new(source.index())), 1));
        Self {
            tokens,
            graph,
            source,
            sink,
            end_position: 0,
        }
    }

    // Get start, end, target associated with an edge.
    // This is 1-indexed for legacy reasons!
    pub fn get_start_end_target(&self, edge_idx: EdgeIndex<Ix>) -> (usize, usize, NodeIndex<Ix>) {
        let edge_ref = self.graph.get_edge(edge_idx);
        let target = edge_ref.get_target();
        let span = self.get_span(edge_ref.get_weight(), target);
        // Shift to 1-indexed and retrieve value of end pointer.
        (span.0, span.1, target)
    }

    // Convenience methods.

    pub fn get_graph(&self) -> &ArrayGraph<W, CdawgEdgeWeight<Ix>, Ix, Mb> {
        &self.graph
    }

    pub fn get_source(&self) -> NodeIndex<Ix> {
        self.source
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }


    // Only well-defined when token is not end-of-text.
    pub fn get_edge_by_token(&self, state: NodeIndex<Ix>, token: u16) -> Option<EdgeIndex<Ix>> {
        if token != u16::MAX {
            let weight = CdawgEdgeWeight::new(0, 0); // Doesn't matter.
            let cmp = CdawgComparator::new_with_token(self.tokens.clone(), token);
            self.graph
                .get_edge_by_weight_cmp(state, weight, Box::new(cmp))
        } else {
            None
        }
    }

    // Handle end-of-text tokens correctly.
    pub fn get_edge_by_token_index(
        &self,
        state: NodeIndex<Ix>,
        token_idx: usize,
    ) -> Option<EdgeIndex<Ix>> {
        let weight = CdawgEdgeWeight::new(token_idx, token_idx + 1);
        let token = self.tokens.borrow().get(token_idx);
        let cmp = CdawgComparator::new_with_token(self.tokens.clone(), token);
        self.graph
            .get_edge_by_weight_cmp(state, weight, Box::new(cmp))
    }

    pub fn add_balanced_edge(
        &mut self,
        state: NodeIndex<Ix>,
        target: NodeIndex<Ix>,
        gamma: (usize, usize),
    ) {
        // We should have gamma.0 <= gamma.1
        let weight = self._new_edge_weight(gamma.0, gamma.1);
        let token = self.tokens.borrow().get(gamma.0 - 1); // Map to 0-indexed
        let cmp = CdawgComparator::new_with_token(self.tokens.clone(), token);
        self.graph
            .add_balanced_edge_cmp(state, target, weight, Box::new(cmp))
    }

    // Methods for inference with the CDAWG.

    // Get the source state and initial values for transition quantities.
    pub fn get_initial(&self) -> CdawgState<Ix> {
        CdawgState {
            state: self.source,
            edge_start: 0,
            start: 0,
            end: 0,
            target: Some(self.source),
            length: 0,
        }
    }

    // TODO(#100): Refactor these into an Infinigram class that wraps a Cdawg

    /// Get the count of the suffix matched by a CdawgState.
    pub fn get_suffix_count(&self, cs: CdawgState<Ix>) -> usize {
        self.get_count(cs.target.unwrap())
    }

    /// Get the entropy of a CDAWG state in bits.
    pub fn get_entropy(&self, cs: CdawgState<Ix>) -> f64 {
        let (state, gamma) = cs.get_state_and_gamma();
        if gamma.0 != gamma.1 {
            return 0.;
        }

        let q = state.unwrap();
        let denom = self.get_count(q);
        let mut sum = 0.;
        for next_state in self.get_graph().neighbors(q) {
            let prob = (self.get_count(next_state) as f64) / (denom as f64);
            sum -= prob * f64::log2(prob);
        }
        sum
    }

    pub fn get_next_tokens(&self, cs: CdawgState<Ix>) -> Vec<(u16, f64)> {
        let (state, gamma) = cs.get_state_and_gamma();
        if gamma.0 != gamma.1 {
            let token = self.tokens.borrow().get(gamma.1);
            return vec![(token, 1.)];
        }

        let q = state.unwrap();
        let denom = self.get_count(q);
        let mut tokens = Vec::new();
        for edge in self.get_graph().edges(q) {
            // let edge_ref = self.graph.get_edge(edge_idx);
            let next_state = edge.get_target();
            let span = self.get_span(edge.get_weight(), next_state);
            let token = self.tokens.borrow().get(span.0 - 1); // Shift to 0 indexing.
            let prob = (self.get_count(next_state) as f64) / (denom as f64);
            tokens.push((token, prob));
        }
        tokens
    }
}
