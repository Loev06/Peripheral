use super::{Board, Score, precomputed, util};

mod piece_square;
use piece_square::{MG_PIECE_SQUARE_TABLES, EG_PIECE_SQUARE_TABLES, GAME_PHASE_INCREMENT, MAX_GAME_PHASE};

pub struct Eval;

impl Eval {
    // #[inline(always)]
    pub fn eval(board: &Board) -> Score {
        let mut mg = 0;
        let mut eg = 0;
        let mut mg_game_phase = 0;

        for pt in 0..12 {
            let mut pieces = board.bbs[pt];
            while pieces != precomputed::EMPTY {
                let sq = util::pop_ls1b(&mut pieces);
                // if verbose {
                //     println!("{} on {}", util::piece_name_from_usize(pt), precomputed::SQUARE_NAMES[sq as usize]);
                //     println!("mg: {}", MG_PIECE_SQUARE_TABLES[pt][sq as usize]);
                //     println!("eg: {}", EG_PIECE_SQUARE_TABLES[pt][sq as usize]);
                //     println!("phase: {}", GAME_PHASE_INCREMENT[pt]);
                // }
                mg += MG_PIECE_SQUARE_TABLES[pt][sq as usize];
                eg += EG_PIECE_SQUARE_TABLES[pt][sq as usize];
                mg_game_phase += GAME_PHASE_INCREMENT[pt];
            }
        }

        if mg_game_phase > MAX_GAME_PHASE {
            mg_game_phase = MAX_GAME_PHASE
        }
        let eg_game_phase = MAX_GAME_PHASE - mg_game_phase;
        // if verbose {
        //     println!("MG: {} | EG: {} | phase: {}/{}", mg, eg, mg_game_phase, MAX_GAME_PHASE);
        // }
        (mg_game_phase * mg + eg_game_phase * eg) / MAX_GAME_PHASE
    }
}
