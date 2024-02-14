pub const NAME: &str = "Peripheral";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = "Loev06";
pub const DATE: &str = "2024-02-15";

use std::error::Error;
use serde::{Deserialize, Serialize};

pub use chess::{MoveGenerator, Board, MoveList, Perft, ChessEngine, SearchParams, util, PieceType::*, Color::*, Move, Eval, grade};

#[allow(dead_code)]
mod chess;

const FEN: &str = "k3r3/8/8/3nQr2/4b3/3K4/8/8 w - - 0 1";

pub fn run_bot() -> Result<(), Box<dyn Error>> {
    println!("Main bot function");

    let mg = MoveGenerator::new();
    let mut board = Board::try_from_fen(FEN)?;
    let mut moves = MoveList::new();
    mg.generate_legal_moves(&mut board, &mut moves, true);
    for mv in moves.sort_with_grading_function(grade, Move::new(19, 27, &Move::empty()), &board) {
        println!("{} | {}", mv, grade(mv, Move::new(19, 27, &Move::empty()), &board));
    }
    
    // let mut chess_engine = ChessEngine::new(FEN);
    // let res = chess_engine.search(6);
    // println!("bestmove {} ({})", res.0, res.1);


    // let mut board = Board::try_from_fen(FEN)?;
    // println!("{}", board);
    // let mut perft = Perft::new(board);

    // perft.verb_perft(6, true, false);

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct TestPosition {
    depth: u8,
    nodes: u64,
    fen: String
}


/*
cargo test --release -- --nocapture
*/

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn perft() {
        let json_str = fs::read_to_string("./test_positions.json").expect("Error loading json file.");
        let test_positions: Vec<TestPosition> = serde_json::from_str(&json_str).unwrap();
        for i in 0..test_positions.len() {
            let test_position = &test_positions[i];
            let board = Board::try_from_fen(&test_position.fen).expect("Error loading board from fen.");
            let mut perft = Perft::new(board);

            if perft.hash_perft(test_position.depth, false, false) != test_position.nodes {
                println!("Incorrect perft result:");
                perft.verb_perft(test_position.depth, true, false);
            } else {
                println!("Finished {}/{}", i + 1, test_positions.len());
            }
        }
    }
}
