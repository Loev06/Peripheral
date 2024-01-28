use super::{
    TT_INDEX_SHIFT, TT_SIZE,
    super::super::{
        Move, Score, Board
    }
};

// Key as stored in a TTEntry. u16 should be enough for no collisions to be occuring (1 in (2^16 * TT_SIZE) for each node)
type TTKey = u16;

#[derive(Clone, Copy, PartialEq)]
pub enum NodeType {
    Exact, // contains best_move
    All,
    Cut, // contains cut_move
    PV // contains best_move
}

// GenBound contains (bit 1 being lsb):
// bits 3-8: generation (u8)
// bits 1-2: NodeType (enum)
#[derive(Clone, Copy, Debug)]
pub struct GenBound(u8);

impl GenBound {
    #[inline(always)]
    pub fn new(generation: u8, node_type: NodeType) -> Self {
        Self(generation
            | node_type as u8)
    }

    pub fn generation(&self) -> u8 {
        self.0 & 0b11111100
    }

    pub fn node_type(&self) -> NodeType {
        match self.0 & 0b11 {
            0b00 => NodeType::Exact,
            0b01 => NodeType::All,
            0b10 => NodeType::Cut,
            0b11 => NodeType::PV,
            _ => panic!("invalid node type")
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    pub key: TTKey,
    pub best_move: Move, // best move for PV-nodes, refutation move for cut-nodes, 0 for other types
    pub depth: u8,
    pub score: Score,
    gen_bound: GenBound,
}

impl TTEntry {
    pub fn new(key: TTKey, best_move: Move, depth: u8, score: Score, gen_bound: GenBound) -> Self {
        Self { key, best_move, depth, score, gen_bound }
    }

    pub fn empty() -> Self {
        Self { key: 0, best_move: Move::empty(), depth: 0, score: 0, gen_bound: GenBound(0) }
    }


    /*
        Replacement scheme:
        if old.gen != new.gen:
            replace
        else if old.is_pv:
            if old == new:
                if new.depth > old.depth:
                    replace
        else if new.depth > old.depth:
            replace
        
        inversion:
        if old.gen == new.gen
            && (
                old.depth >= new.depth
                || (old.is_pv && old != new)
            )
     */

    pub fn save(&mut self, key: TTKey, best_move: Move, depth: u8, score: Score, gen_bound: GenBound) {
        if self.gen_bound.generation() == gen_bound.generation()
            && (
                self.depth >= depth // cheapest check first
                || (self.gen_bound.node_type() == NodeType::PV && self.key != key)
            )
        {
            return;
        }

        // inspired by https://github.com/official-stockfish/Stockfish/blob/8b4583bce76da7d27aaa565e6302d2e540cd496a/src/tt.cpp#L38
        if !best_move.is_empty() || self.key != key {
            self.best_move = best_move;
        }
        
        self.gen_bound = gen_bound;
        self.key = key;
        self.depth = depth;
        self.score = score;
    }
}

pub enum TTProbeResult {
    Score(Score),
    BestMove(Move),
    None
}

pub struct TranspositionTable {
    pub tt: Vec<TTEntry>,
    generation: u8
}

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        Self{
            tt: vec![TTEntry::empty(); size],
            generation: 0
        }
    }

    pub fn next_generation(&mut self) {
        self.generation = self.generation.wrapping_add(4)
    }

    pub fn get_pv(&mut self, board: &mut Board, depth: u8) -> Vec<Move> {
        let mut pv = Vec::new();

        let tt_index = (board.key as usize >> TT_INDEX_SHIFT) & (TT_SIZE - 1);
        let entry = &mut self.tt[tt_index];
        let best_move = entry.best_move;

        if depth > 0 && entry.key == board.key as TTKey && best_move != Move::empty() {
            pv.push(best_move);

            // make sure the PV persist during the next generation
            entry.gen_bound = GenBound::new(self.generation.wrapping_add(4), NodeType::PV);
            
            board.make_move(&best_move);
            pv.append(self.get_pv(board, depth - 1).as_mut());
            board.undo_move(&best_move);
        }

        pv
    }

    #[inline(always)]
    pub fn probe(&self, index: usize, alpha: Score, beta: Score, depth: u8, key: u64) -> TTProbeResult {
        let entry = self.tt[index];
        if entry.key == key as TTKey {
            if entry.depth >= depth {
                match entry.gen_bound.node_type() {
                    NodeType::Exact | NodeType::PV => return TTProbeResult::Score(entry.score),
                    NodeType::All => if entry.score <= alpha {return TTProbeResult::Score(entry.score);},
                    NodeType::Cut => if entry.score >= beta {return TTProbeResult::Score(entry.score);}
                }
            } else {
                return TTProbeResult::BestMove(entry.best_move)
            }
        }
        TTProbeResult::None
    }

    #[inline(always)]
    pub fn get_entry(&self, index: usize, key: u64) -> Option<TTEntry> {
        let entry = self.tt[index];
        if entry.key == key as TTKey {
            Some(entry)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn record(&mut self, index: usize, key: u64, best_move: Move, depth: u8, score: Score, node_type: NodeType) {
        self.tt[index].save(key as TTKey, best_move, depth, score, GenBound::new(self.generation, node_type));
    }
}