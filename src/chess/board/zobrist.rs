use std::error::Error;

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

pub const NUM_ZOBRIST_VALUES: usize = 12 * 64 + 1 + 16 + 8;

pub struct Zobrist {
    pub piece_square: [[u64; 64]; 12],
    pub black_to_move: u64,
    pub castling: [u64; 16],
    pub ep_file: [u64; 8]
}

impl Zobrist {
    pub fn new() -> Result<Zobrist, Box<dyn Error>> {
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        let mut zobrist = Zobrist {
            piece_square: [[0; 64]; 12],
            black_to_move: 0,
            castling: [0; 16],
            ep_file: [0; 8]
        };
        
        for i in 0..12 {
            rng.try_fill(&mut zobrist.piece_square[i])?;
        }
        zobrist.black_to_move = rng.next_u64();
        rng.try_fill(&mut zobrist.castling)?;
        rng.try_fill(&mut zobrist.ep_file)?;

        Ok(zobrist)
    }
}