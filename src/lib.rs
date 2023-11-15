use std::error::Error;

pub use chess::{MoveGenerator, Board, MoveList};

#[allow(dead_code)]
mod chess;

const FEN: &str = "rnbqkbnr/8/8/8/8/4K3/8/RNBQ1BNR w HAkq - 0 1";

pub fn run_bot() -> Result<(), Box<dyn Error>> {
    println!("Main bot function");
    
    let mut moves = MoveList::new();
    let board = &Board::try_from_fen(FEN)?;
    MoveGenerator::new(&board).generate_legal_moves(&mut moves);
    dbg!(moves.count);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*; 

    #[test]
    fn it_works() {
        run_bot().expect("run_bot returned an error.");
    }
}
