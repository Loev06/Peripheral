use std::error::Error;

pub use chess::{MoveGenerator, Board, MoveList, Perft, util, PieceType::*, Color::*};

#[allow(dead_code)]
mod chess;

const FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn run_bot() -> Result<(), Box<dyn Error>> {
    println!("Main bot function");
    
    let board = Board::try_from_fen(FEN)?;
    let mut perft = Perft::new(board)?;

    println!("{}", perft.board);
    util::print_bb(perft.board.bbs[Knight(White)]);
    dbg!(perft.perft(6, true, false));

    perft.board.update_board_data();

    println!("{}", perft.board);
    util::print_bb(perft.board.bbs[Knight(White)]);
    dbg!(perft.perft(6, true, false));

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
