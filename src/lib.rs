pub const NAME: &str = "Rust Chess Engine";
pub const VERSION: &str = "0.1.1";
pub const AUTHOR: &str = "Loev06";
pub const DATE: &str = "2023-12-14";

use std::error::Error;
use serde::{Deserialize, Serialize};

pub use chess::{MoveGenerator, Board, MoveList, Perft, ChessEngine, util, PieceType::*, Color::*, Move};

#[allow(dead_code)]
mod chess;

const FEN: &str = "4r3/2P3R1/R1N2k1P/5Np1/K1p1p3/1pr5/3P4/Bn3Q2 w - - 0 0";

pub fn run_bot() -> Result<(), Box<dyn Error>> {
    println!("Main bot function");
    
    let board = Board::try_from_fen(FEN)?;
    println!("{}", board);

    let mut chess_engine = ChessEngine::new();
    let res = chess_engine.search(board, 6);
    println!("bestmove {} ({})", res.0, res.1);

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
