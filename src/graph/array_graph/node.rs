use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::marker::Copy;

use crate::graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};
use crate::weight::Weight;

#[derive(Deserialize, Serialize, Copy, Default)]
pub struct ArrayNode<N, Ix = DefaultIx> {
    #[serde(bound(
        serialize = "N: Serialize, Ix: Serialize",
        deserialize = "N: Deserialize<'de>, Ix: Deserialize<'de>",
    ))]
    pub weight: N,
    pub first_edge: EdgeIndex<Ix>,
    pub num_edges: usize,
}

impl<N, Ix> ArrayNode<N, Ix>
where
    Ix: IndexType + Copy,
{
    pub fn new(weight: N, first_edge: EdgeIndex<Ix>, num_edges: usize) -> Self {
        Self {
            weight,
            first_edge,
            num_edges,
        }
    }
}

pub trait ArrayNodeRef<N, Ix> {
    fn get_weight(self) -> N
    where
        N: Clone;
    fn get_length(self) -> u64;
    fn get_count(self) -> usize;
    fn get_first_edge(self) -> EdgeIndex<Ix>;
    fn get_num_edges(self) -> usize;
}

// We can use a Node object as a "reference" to data on disk.
impl<N, Ix> ArrayNodeRef<N, Ix> for ArrayNode<N, Ix>
where
    Ix: IndexType,
    N: Weight,
{
    fn get_weight(self) -> N
    where
        N: Clone,
    {
        self.weight.clone()
    }

    fn get_length(self) -> u64 {
        self.weight.get_length()
    }

    fn get_count(self) -> usize {
        // FIXME: The count is actually stored in u16.
        self.weight.get_count()
    }

    fn get_first_edge(self) -> EdgeIndex<Ix> {
        self.first_edge
    }

    fn get_num_edges(self) -> usize {
        self.num_edges
    }
}

// FIXME(#52): We probably should not be allowing these clippy warnings but works for now :/
impl<N, Ix> ArrayNodeRef<N, Ix> for *const ArrayNode<N, Ix>
where
    Ix: IndexType,
    N: Weight,
{
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn get_weight(self) -> N
    where
        N: Clone,
    {
        unsafe { (*self).weight.clone() }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn get_length(self) -> u64 {
        unsafe { (*self).weight.get_length() }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn get_count(self) -> usize {
        unsafe { (*self).weight.get_count() }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn get_first_edge(self) -> EdgeIndex<Ix> {
        unsafe { (*self).first_edge }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn get_num_edges(self) -> usize {
        unsafe { (*self).num_edges }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weight::DefaultWeight;
    use bincode;
    use bincode::{deserialize, serialize, Options};

    #[test]
    fn test_serialize_deserialize_node() {
        type NodeType = ArrayNode<DefaultWeight, DefaultIx>;
        let node: NodeType = ArrayNode::new(DefaultWeight::new(42, Some(NodeIndex::new(2)), 2));
        let bytes = serialize(&node).unwrap();
        let new_node: NodeType = deserialize(&bytes).unwrap();
        assert_eq!(node.get_length(), new_node.get_length());
        assert_eq!(node.get_count(), new_node.get_count());
    }

    #[test]
    fn test_serialize_deserialize_node_with_fixint() {
        type T = ArrayNode<DefaultWeight, DefaultIx>;
        let node: T = ArrayNode::new(DefaultWeight::new(42, Some(NodeIndex::new(2)), 2));
        let bytes = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .serialize(&node)
            .unwrap();
        let new_node = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .deserialize::<T>(&bytes)
            .unwrap();
        assert_eq!(node.get_length(), new_node.get_length());
        assert_eq!(node.get_count(), new_node.get_count());
    }
}
