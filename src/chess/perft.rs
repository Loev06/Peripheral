use super::{MoveGenerator, Board, MoveList};
use std::time;

const PERFT_TT_SIZE: usize = (1 << 16) << 8;
const PERFT_TT_MASK: u64 = (PERFT_TT_SIZE - 1) as u64;

#[derive(Clone, Copy)]
struct PerftTTEntry {
    key: u64,
    move_count: u64
}

impl PerftTTEntry {
    fn new(key: u64, move_count: u64) -> PerftTTEntry {
        PerftTTEntry {
            key, move_count
        }
    }
}

pub struct Perft {
    board: Board,
    mg: MoveGenerator,
    
    tt: Vec<PerftTTEntry>,
    hits: u64,
    collisions: u64
}

impl Perft {
    pub fn new(b: Board) -> Self {
        Self {
            board: b,
            mg: MoveGenerator::new(),

            tt: vec![PerftTTEntry::new(0, 0); PERFT_TT_SIZE],
            hits: 0,
            collisions: 0
        }
    }

    pub fn verb_perft(&mut self, depth: u8, hash: bool, debug: bool) -> u64 {
        self.tt = vec![PerftTTEntry::new(0, 0); PERFT_TT_SIZE]; // not necessary, but reset table to properly count collisions / filled percentage
        self.hits = 0;
        self.collisions = 0;
        
        let start = time::Instant::now();

        let sum = if hash {
            self.hash_perft(depth, true, debug)
        } else {
            self.perft(depth, true, debug)
        };

        let elapsed = start.elapsed();
        let filled_count = self.tt.iter().filter(|&t| (*t).key != 0).count();

        println!("Total nodes: {}", sum);
        println!("Time taken: {:?} | NPS: {}M", elapsed, sum / std::cmp::max(elapsed.as_micros() as u64, 1));
        println!("TT hits: {} | collisions: {} | {:.2}% filled", self.hits, self.collisions, filled_count as f64 / PERFT_TT_SIZE as f64 * 100f64);

        sum
    }
    
    pub fn perft(&mut self, depth: u8, root: bool, debug: bool) -> u64 {
        debug_assert!(self.board.key == self.board.make_key());

        let mut moves = MoveList::new();
        self.mg.generate_legal_moves(&mut self.board, &mut moves);
        
        if debug {
            println!("{}", self.board);
            dbg!(&moves);
        }

        if depth <= 1 {
            *moves.get_count() as u64
        } else {
            moves.into_iter().map(|mv| {
                self.board.make_move(&mv);
                let count = self.perft(depth - 1, false, debug);
                self.board.undo_move(&mv);

                if root {
                    println!("{mv}: {count}");
                }

                count
            }).sum()
        }
    }

    pub fn hash_perft(&mut self, depth: u8, root: bool, debug: bool) -> u64 {
        debug_assert!(self.board.key == self.board.make_key());
        let idx = (self.board.key & PERFT_TT_MASK) as usize ^ depth as usize;

        if self.tt[idx].key == self.board.key {
            self.hits += 1;
            return self.tt[idx].move_count;
        }

        if self.tt[idx].key != 0 {
            self.collisions += 1;
        }

        let mut moves = MoveList::new();
        self.mg.generate_legal_moves(&mut self.board, &mut moves);
        
        if debug {
            println!("{}", self.board);
            dbg!(&moves);
        }

        let sum = if depth <= 1 {
            *moves.get_count() as u64
        } else {
            moves.into_iter().map(|mv| {
                self.board.make_move(&mv);
                let count = self.hash_perft(depth - 1, false, debug);
                self.board.undo_move(&mv);

                if root {
                    println!("{mv}: {count}");
                }

                count
            }).sum()
        };

        self.tt[idx] = PerftTTEntry::new(self.board.key, sum);
        sum
    }
}
