use std::error::Error;

use super::{MoveGenerator, Board, MoveList};


pub struct Perft {
    board: Board,
    mg: MoveGenerator
}

impl Perft {
    pub fn new(b: Board) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            board: b,
            mg: MoveGenerator::new()
        })
    }

    pub fn perft(&mut self, depth: u8, is_root: bool, debug: bool) -> u64 {
        let mut moves = MoveList::new();
        self.mg.generate_legal_moves(&self.board, &mut moves);

        if debug {
            dbg!(&moves);
        }

        if depth <= 1 {
            return moves.count as u64;
        }

        moves.into_iter().map(|mv| {
            self.board.make_move(&mv);
            let count = self.perft(depth - 1, false, debug);
            self.board.undo_move(&mv);

            if is_root || debug {
                println!("{mv}: {count}")
            }

            count
        }).sum()
    }
}