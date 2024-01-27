use std::time::Instant;
use super::{
    ChessEngine, SearchParams, TT_SIZE, TT_INDEX_SHIFT, MAX_DEPTH,
    super::{
        Move, Score, MIN_SCORE, MAX_SCORE, CHECKMATE_SCORE, Eval, MoveList, Color, grade
    }
};

mod transposition_table;
pub use transposition_table::TranspositionTable;
use transposition_table::{TTProbeResult, TTEntry, NodeType};

impl ChessEngine {
    pub fn search(&mut self, search_params: SearchParams) -> (Move, Score) {
        self.timer = Instant::now();
        self.search_time = search_params.move_time.unwrap_or(
            match self.board.gs.player_to_move {
                Color::White => search_params.wtime,
                Color::Black => search_params.btime,
            } / 20
        );

        let mut pv = Vec::new();

        let tt_index = (self.board.key as usize >> TT_INDEX_SHIFT) & (TT_SIZE - 1);
        let mut last_search = TTEntry::empty();

        self.current_position = self.board.key;

        for current_depth in 1..=std::cmp::min(search_params.depth, MAX_DEPTH as u8) {
            self.tt.next_generation();
            self.nodes = 0;
            self.search_canceled = false;

            self.negamax(MIN_SCORE, MAX_SCORE, current_depth, 0);

            let option_last_search = self.tt.get_entry(tt_index, self.board.key);
            if option_last_search.is_none() {
                dbg!("TT did not include root", self.tt.tt[tt_index]);
            }
            last_search = option_last_search.expect("TT should include root");

            pv = self.tt.get_pv(&mut self.board, current_depth);
            println!(
                "info depth {} score cp {} nodes {} time {} pv{}",
                current_depth,
                last_search.score,
                self.nodes,
                self.timer.elapsed().as_millis(),
                pv.iter().fold(String::new(), |acc, x| {format!("{acc} {x}")})
            );

            if self.timer.elapsed().as_millis() >= self.search_time {
                break;
            }
        }

        let ponder = pv.get(1);
        if let Some(mv) = ponder {
            println!("bestmove {} ponder {}", pv[0], mv);
        } else {
            println!("bestmove {}", pv[0]);
        }
        (last_search.best_move, last_search.score)
    }

    fn negamax(&mut self, mut alpha: Score, beta: Score, depth: u8, ply: u8) -> Score {
        if depth == 0 {
            // Check for timeout on leaf node
            if self.nodes & 2047 == 0 && self.timer.elapsed().as_millis() >= self.search_time {
                self.search_canceled = true;
                return 0;
            }

            self.nodes += 1;

            let score = self.quiescence(alpha, beta);
            
            // self.tt.record(tt_index, self.board.key, Move::empty(), depth, score, NodeType::Exact);

            return score;
        }

        let tt_index = (self.board.key as usize >> TT_INDEX_SHIFT) & (TT_SIZE - 1);
        let mut best_move = match self.tt.probe(tt_index, alpha, beta, depth, self.board.key) {
            TTProbeResult::Score(score) => return score,
            TTProbeResult::BestMove(mv) => mv,
            TTProbeResult::None => Move::empty()
        };

        let mut node_type = NodeType::All;

        let mut moves = MoveList::new();
        self.mg.generate_legal_moves(&mut self.board, &mut moves, false);

        if *moves.get_count() == 0 {
            return if self.board.gs.is_in_check {-CHECKMATE_SCORE + ply as Score} else {0};
        }

        let mut best_score = MIN_SCORE;

        for mv in moves.sort_with_function(grade, best_move, &self.board) {
            self.board.make_move(&mv);
            let score = -self.negamax(-beta, -alpha, depth - 1, ply + 1);
            self.board.undo_move(&mv);

            if self.search_canceled {
                return 0;
            }

            if score >= beta {
                self.tt.record(tt_index, self.board.key, mv, depth, score, NodeType::Cut);
                return score; // fail-soft beta-cutoff - lower bound
            }

            if score > best_score {
                best_score = score;
                if score > alpha {
                    best_move = mv;
                    node_type = NodeType::Exact;
                    alpha = score;
                }
            }
        }

        self.tt.record(tt_index, self.board.key, best_move, depth, best_score, node_type);
        best_score
    }

    fn quiescence(&mut self, mut alpha: Score, beta: Score) -> Score {
        let mut best_score = Eval::eval(&self.board) * self.board.gs.player_to_move as Score;

        let mut moves = MoveList::new();
        self.mg.generate_legal_moves(&mut self.board, &mut moves, true);

        if self.board.gs.is_in_check {
            alpha = std::cmp::max(alpha, -CHECKMATE_SCORE)
        } else {
            if best_score >= beta {
                return best_score;
            }
            if best_score > alpha {
                alpha = best_score;
            }
        }

        for mv in moves.sort_with_function(grade, Move::empty(), &self.board) {
            self.board.make_move(&mv);
            let score = -self.quiescence(-beta, -alpha);
            self.board.undo_move(&mv);

            if score >= beta {
                return score;
            }
            if score > best_score {
                best_score = score;
                if score > alpha {
                    alpha = score;
                }
            }
        }
        best_score
    }
}