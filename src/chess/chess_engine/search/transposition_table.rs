use super::{
    NodeType,
    super::super::{
        Move, Score,
    }
};

#[derive(Clone)]
struct TTEntry {
    key: u16,
    best_move: Move, // best move for PV-nodes, refutation move for cut-nodes, 0 for other types
    depth: u8,
    score: Score,
    node_type: NodeType,
}

impl TTEntry {
    fn new(key: u16, best_move: Move, depth: u8, score: Score, node_type: NodeType) -> Self {
        Self { key, best_move, depth, score, node_type }
    }
    
    fn empty() -> Self {
        Self { key: 0, best_move: Move::empty(), depth: 0, score: 0, node_type: NodeType::LeafNode }
    }
}

pub struct TranspositionTable (
    Vec<TTEntry>
);

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        Self(vec![TTEntry::empty(); size])
    }

    #[inline(always)]
    pub fn probe(&self, index: usize, alpha: Score, beta: Score, depth: u8, key: u64) -> Option<Score> {
        let entry = &self.0[index];
        if entry.key == key as u16 && entry.depth >= depth {
            match entry.node_type {
                NodeType::PVNode | NodeType::LeafNode => return Some(entry.score),
                NodeType::AllNode => if entry.score <= alpha {return Some(entry.score);},
                NodeType::CutNode => if entry.score >= beta {return Some(entry.score);}
            }
        }
        None
    }

    #[inline(always)]
    pub fn record(&mut self, index: usize, key: u64, best_move: Move, depth: u8, score: Score, node_type: NodeType) {
        let entry = TTEntry::new(key as u16, best_move, depth, score, node_type);
        self.0[index] = entry;
    }
}