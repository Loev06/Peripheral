use std::error::Error;
use serde::{Deserialize, Serialize};

pub use chess::{MoveGenerator, Board, MoveList, Perft, util, PieceType::*, Color::*};

#[allow(dead_code)]
mod chess;

const FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn run_bot() -> Result<(), Box<dyn Error>> {
    println!("Main bot function");
    
    let board = Board::try_from_fen(FEN)?;
    let mut perft = Perft::new(board);

    println!("{}", perft.board);
    perft.verb_perft(7, false, false);

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
