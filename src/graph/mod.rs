pub mod avl_graph;
#[allow(dead_code)]
pub mod indexing;
pub mod comparator;

use ::comparator::Comparator;
use crate::graph::avl_graph::{Edges, Neighbors};
use crate::graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};
use crate::memory_backing::{CacheConfig, MemoryBacking, RamBacking};
pub use self::avl_graph::{EdgeRef, NodeRef};

pub trait Graph<N, E, Ix = DefaultIx, Mb = RamBacking<N, E, Ix>>
where
    Mb: MemoryBacking<N, E, Ix>, Ix: IndexType
{
    fn new_mb(mb: Mb) -> Self;
    fn with_capacity_mb(
        mb: Mb,
        n_nodes: usize,
        n_edges: usize,
        cache_config: CacheConfig,
    ) -> Self;
    fn get_edge_by_weight(
        &self,
        a: NodeIndex<Ix>,
        weight: E,
        cmp: Box<dyn Comparator<E>>
    ) -> Option<EdgeIndex<Ix>>;
    fn n_edges(&self, a: NodeIndex<Ix>) -> usize;
    fn node_count(&self) -> usize;
    fn edge_count(&self) -> usize;
    fn neighbors(&self, node: NodeIndex<Ix>) -> Neighbors<N, E, Ix, Mb>;
    fn edges(&self, edges: NodeIndex<Ix>) -> Edges<N, E, Ix, Mb>;
    fn get_node(&self, node: NodeIndex<Ix>) -> Mb::NodeRef;
    fn get_edge(&self, edge: EdgeIndex<Ix>) -> Mb::EdgeRef;
    fn edge_target(&self,
                   a: NodeIndex<Ix>,
                   weight: E,
                   cmp: Box<dyn Comparator<E>>) -> Option<NodeIndex<Ix>>;
}

pub trait MutableGraph<N, E, Ix = DefaultIx, Mb = RamBacking<N, E, Ix>>: Graph<N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix>, Ix: IndexType
{
    fn add_node(&mut self, weight: N) -> NodeIndex<Ix>;
    fn clone_edges(&mut self, old: NodeIndex<Ix>, new: NodeIndex<Ix>);
    fn add_edge(
        &mut self,
        a: NodeIndex<Ix>,
        b: NodeIndex<Ix>,
        weight: E,
        cmp: Box<dyn Comparator<E>>,
    );
    fn get_node_mut(&mut self, node: NodeIndex<Ix>) -> Mb::NodeMutRef;
    fn get_edge_mut(&mut self, edge: EdgeIndex<Ix>) -> Mb::EdgeMutRef;
    fn reroute_edge(&mut self,
                    a: NodeIndex<Ix>,
                    b: NodeIndex<Ix>,
                    weight: E,
                    cmp: Box<dyn Comparator<E>>) -> bool;
}
pub trait TreeGraph<N, E, Ix = DefaultIx, Mb = RamBacking<N, E, Ix>>: MutableGraph<N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix>, Ix: IndexType
{
    fn balance_ratio(&self, node: NodeIndex<Ix>) -> f64;
}
