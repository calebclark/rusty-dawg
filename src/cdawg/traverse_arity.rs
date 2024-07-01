use anyhow::Result;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::cdawg::cdawg_edge_weight::CdawgEdgeWeight;
use crate::cdawg::inenaga::Cdawg;
use crate::cdawg::stack::Stack;
use crate::graph::indexing::{IndexType, NodeIndex};
use crate::memory_backing::{DiskVec, MemoryBacking};
use crate::weight::Weight;

/// Based on Topological Counter.
/// TODO: Could standardize names and potentially generalize.
pub struct TraverseArity<Sb> {
    stack: Sb,
    visited: Vec<bool>,  // Only support RAM.
}

impl<Ix> TraverseArity<Vec<Ix>>
where
    Ix: IndexType + Serialize + for<'de> Deserialize<'de>,
{
    pub fn new_ram(capacity: usize) -> Self {
        Self { stack: Vec::new(), visited: vec![false; capacity] }
    }
}

impl<Ix> TraverseArity<DiskVec<Ix>>
where
    Ix: IndexType + Serialize + for<'de> Deserialize<'de>,
{
    pub fn new_disk<P: AsRef<Path> + std::fmt::Debug>(path: P, capacity: usize) -> Result<Self> {
        let stack = DiskVec::new(path, capacity)?;
        Ok(Self { stack, visited: vec![false; capacity] })
    }
}

impl<Sb> TraverseArity<Sb> {
    /// DFS implementation of graph traversal.
    pub fn traverse_arity<Ix, W, Mb>(&mut self, cdawg: &mut Cdawg<W, Ix, Mb>) -> Vec<usize>
    where
        Ix: IndexType + Serialize + for<'de> Deserialize<'de>,
        W: Weight + Serialize + for<'de> Deserialize<'de> + Clone,
        Mb: MemoryBacking<W, CdawgEdgeWeight<Ix>, Ix>,
        Sb: Stack<usize>,
    {
        let mut arities = Vec::new();
        self.stack.push(cdawg.get_source().index());
        while let Some(state) = self.stack.pop() {
            if self.visited[state.index()] {
                continue;
            }

            let idx = NodeIndex::new(state);
            let next_states: Vec<NodeIndex<Ix>> = cdawg.get_graph().neighbors(idx).collect();
            arities.push(next_states.len());
            for next_state in next_states {
                if self.visited[next_state.index()] {
                    continue;
                }
                self.stack.push(next_state.index());
            self.visited[state] = true;
            }
        }
        arities
    }
}

#[cfg(test)]
#[allow(unused_variables)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_traverse_arities_cocoa() {
        let (c, o, a) = (0, 1, 2);
        let mut cdawg: Cdawg = Cdawg::new(Rc::new(RefCell::new(vec![c, o, c, o, a, u16::MAX])));
        cdawg.build();
        let mut ta = TraverseArity::new_ram(20);
        let arities = ta.traverse_arity(&mut cdawg);
        assert_eq!(arities, vec![4, 2, 1]);  // 4 at source, 1 at sink (self loop), 2 at internal
    }
}
