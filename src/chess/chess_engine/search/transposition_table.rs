use std::{fmt::Display, mem::size_of};

use super::super::super::{
    Move, Score, Board, MoveGenerator
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

impl Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            match self {
                Self::Exact => "exact",
                Self::All   => "all",
                Self::Cut   => "cut",
                Self::PV    => "PV"
            }
        )
    }
}

// GenBound contains (bit 1 being lsb):
// bits 3-8: generation (u8)
// bits 1-2: NodeType (enum)
#[derive(Clone, Copy, Debug)]
pub struct GenBound(u8);

impl GenBound {
    const GENERATION_STEPSIZE: u8 = 0b100;

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
        if self.key != key || !best_move.is_empty() {
            self.best_move = best_move;
        }
        
        self.gen_bound = gen_bound;
        self.key = key;
        self.depth = depth;
        self.score = score;
    }
}

impl Display for TTEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "key:\t{:X}\nmove:\t{}\ndepth:\t{}\nscore:\t{}\ntype:\t{}\ngen:\t{}",
            self.key,
            self.best_move,
            self.depth,
            self.score,
            self.gen_bound.node_type(),
            self.gen_bound.generation() / GenBound::GENERATION_STEPSIZE
        ))
    }
}

pub enum TTProbeResult {
    Score(Score),
    BestMove(Move),
    None
}

pub struct TranspositionTable {
    pub tt: Vec<TTEntry>,
    size: usize,
    mask: u64,
    shift: u32,
    generation: u8
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let size = Self::tt_size_from_mb(size_mb);
        Self{
            tt: vec![TTEntry::empty(); size],
            size,
            mask: size as u64 - 1,
            shift: usize::BITS - size.trailing_zeros(),
            generation: 0
        }
    }
    
    pub const fn tt_size_from_mb(mb: usize) -> usize {
        let preferred_size = mb * 1024 * 1024 / size_of::<TTEntry>();
        1 << preferred_size.ilog2() // round down
    }

    #[inline(always)]
    pub fn entry_is_position(entry: TTEntry, key: u64) -> bool {
        entry.key == key as TTKey
    }

    #[inline(always)]
    pub const fn calc_index(&self, key: u64) -> usize {
        ((key >> self.shift) & self.mask) as usize
    }

    pub fn next_generation(&mut self) {
        self.generation = self.generation.wrapping_add(GenBound::GENERATION_STEPSIZE)
    }

    pub fn get_pv(&mut self, board: &mut Board, depth: u8) -> Vec<Move> {
        let mut pv = Vec::new();

        let tt_index = self.calc_index(board.key);
        let entry = &mut self.tt[tt_index];
        let best_move = entry.best_move;

        if depth > 0 && entry.key == board.key as TTKey && best_move != Move::empty() {
            pv.push(best_move);

            // make sure the PV persist during the next generation
            entry.gen_bound = GenBound::new(self.generation.wrapping_add(GenBound::GENERATION_STEPSIZE), NodeType::PV);
            
            board.make_move(&best_move);
            pv.append(self.get_pv(board, depth - 1).as_mut());
            board.undo_move(&best_move);
        }

        pv
    }

    #[inline(always)]
    pub fn probe(&self, index: usize, alpha: Score, beta: Score, depth: u8, key: u64, b: &Board, mg: &MoveGenerator) -> TTProbeResult {
        let entry = self.tt[index];
        if entry.key == key as TTKey {
            if entry.depth >= depth {
                match entry.gen_bound.node_type() {
                    NodeType::PV => return TTProbeResult::BestMove(entry.best_move), // no TT-cutoff on PV nodes, no legality check: move gets discarded during sort
                    NodeType::Exact => return if mg.is_pseudo_legal_move(b, entry.best_move) {
                            TTProbeResult::Score(entry.score)
                        } else {
                            TTProbeResult::None
                        },
                    NodeType::All => if entry.score <= alpha {return TTProbeResult::Score(entry.score);},
                    NodeType::Cut => if entry.score >= beta {return TTProbeResult::Score(entry.score);}
                }
            } else {
                return TTProbeResult::BestMove(entry.best_move) // no legality check: move gets discarded during sort
            }
        }
        TTProbeResult::None
    }

    #[inline(always)]
    pub fn get_entry(&self, index: usize, key: u64) -> Option<TTEntry> {
        let entry = self.get_entry_at_index(index);
        if Self::entry_is_position(entry, key) {
            Some(entry)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_entry_at_index(&self, index: usize) -> TTEntry {
        self.tt[index]
    }

    pub fn get_gen(&self) -> u8 {
        self.generation / GenBound::GENERATION_STEPSIZE
    }
    
    #[inline(always)]
    pub fn record(&mut self, index: usize, key: u64, best_move: Move, depth: u8, score: Score, node_type: NodeType) {
        self.tt[index].save(key as TTKey, best_move, depth, score, GenBound::new(self.generation, node_type));
    }

    pub fn hash_full(&self) -> usize {
        self.tt.iter()
            .take(1000)
            .filter(|x| x.gen_bound.generation() == self.generation)
            .count()
    }
}