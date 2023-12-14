use super::{Board, Score};

const PAWN_VALUE: Score = 100;
const KNIGHT_VALUE: Score = 300;
const BISHOP_VALUE: Score = 330;
const ROOK_VALUE: Score = 500;
const QUEEN_VALUE: Score = 900;
const PIECE_VALUES: [Score; 6] = [PAWN_VALUE, KNIGHT_VALUE, BISHOP_VALUE, ROOK_VALUE, QUEEN_VALUE, 0];
pub struct Eval;

impl Eval {
    pub fn eval(board: &Board) -> Score {
        let mut eval = 0;

        for pt in 0..5 {
            let wpieces = board.bbs[pt];
            let bpieces = board.bbs[pt + 6];
            eval += (wpieces.count_ones() as i16 - bpieces.count_ones() as i16) * PIECE_VALUES[pt];
        }

        eval
    }
}