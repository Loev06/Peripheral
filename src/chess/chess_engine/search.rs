use std::time::Instant;
use super::{
    ChessEngine, SearchParams, MAX_DEPTH,
    super::{
        Move, Score, MIN_SCORE, MAX_SCORE, CHECKMATE_SCORE, Eval, MoveList, Color, grade
    }
};

mod transposition_table;
pub use transposition_table::TranspositionTable;
use transposition_table::{TTProbeResult, TTEntry, NodeType};

impl ChessEngine {
    pub fn search(&mut self, search_params: SearchParams, verbose: bool) -> (Move, Score) {
        self.timer = Instant::now();
        self.search_time = search_params.move_time.unwrap_or(
            match self.board.gs.player_to_move {
                Color::White => search_params.wtime / 20 + search_params.winc / 2,
                Color::Black => search_params.btime / 20 + search_params.binc / 2,
            }
        );

        let mut pv = Vec::new();

        let tt_index = self.tt.calc_index(self.board.key);
        let mut last_search = TTEntry::empty();

        for current_depth in 1..=std::cmp::min(search_params.depth, MAX_DEPTH as u8) {
            self.tt.next_generation();
            self.nodes = 0;
            self.search_canceled = false;

            self.negamax(MIN_SCORE, MAX_SCORE, current_depth as i8, 0, true);

            last_search = self.tt.get_entry(tt_index, self.board.key).expect("TT should include root");

            pv = self.tt.get_pv(&mut self.board, current_depth);

            if verbose {
                self.print_search_info(last_search, current_depth, &pv);
            }

            if self.timer.elapsed().as_millis() >= self.search_time {
                break;
            }
        }

        if verbose {
            let ponder = pv.get(1);
            if let Some(mv) = ponder {
                println!("bestmove {} ponder {}", pv[0], mv);
            } else {
                println!("bestmove {}", pv[0]);
            }
        }
        (last_search.best_move, last_search.score)
    }

    fn print_search_info(&self, last_search: TTEntry, depth: u8, pv: &Vec<Move>) {
        let elapsed = self.timer.elapsed().as_millis();

        let mate_score = {
            let unsigned_mate_in_plies = CHECKMATE_SCORE - last_search.score.abs();
            if unsigned_mate_in_plies < 100 {
                Some(unsigned_mate_in_plies / 2 * last_search.score.signum())
            } else {
                None
            }
        };

        println!(
            "info depth {} score {} nodes {} nps {} hashfull {} time {} pv{}",
            depth,
            if let Some(mate) = mate_score {
                format!("mate {}", mate)
            } else {
                format!("cp {}", last_search.score)
            },
            self.nodes,
            if elapsed == 0 { 0 } else { self.nodes as u128 * 1000 / elapsed },
            self.tt.hash_full(),
            elapsed,
            pv.iter().fold(String::new(), |acc, x| {format!("{acc} {x}")})
        );
    }

    fn negamax(&mut self, mut alpha: Score, beta: Score, mut depth: i8, ply: u8, null_allowed: bool) -> Score {
        let tt_index = self.tt.calc_index(self.board.key);

        if ply <= 1 {
            if self.board.key_history.contains_3fold() {
                // self.tt.record(tt_index, self.board.key, Move::empty(), depth, 0, NodeType::Exact);
                // self.board.key_history.print_history();
                return 0;
            }
        } else {
            if self.board.key_history.contains_2fold() {
                // self.tt.record(tt_index, self.board.key, Move::empty(), depth, 0, NodeType::Exact);
                return 0;
            }
        }

        if depth <= 0 {
            // Check for timeout on leaf node
            if self.nodes & 2047 == 0 && self.timer.elapsed().as_millis() >= self.search_time {
                self.search_canceled = true;
                return 0;
            }

            let score = self.quiescence(alpha, beta);
            
            // TODO: SPRT uncommented when branching factor is lower
            // self.tt.record(tt_index, self.board.key, Move::empty(), depth, score, NodeType::Exact);

            return score;
        }

        let mut best_move = match self.tt.probe(tt_index, alpha, beta, depth as u8, self.board.key) {
            TTProbeResult::Score(score) => return score,
            TTProbeResult::BestMove(mv) => mv,
            TTProbeResult::None => Move::empty()
        };

        let mut moves = MoveList::new();
        self.mg.generate_legal_moves(&mut self.board, &mut moves, false);

        if null_allowed && !self.board.gs.is_in_check {
            let r = if depth > 6 {4} else {3};

            let ep_mask = self.board.make_null_move();
            let score = -self.zero_window_search(1 - beta, depth - r - 1, ply);
            self.board.undo_null_move(ep_mask);

            if score >= beta {
                depth -= r;
                if depth <= 0 {
                    return self.quiescence(alpha, beta);
                }
            }
        }

        let mut node_type = NodeType::All;


        if *moves.get_count() == 0 {
            return if self.board.gs.is_in_check {-CHECKMATE_SCORE + ply as Score + 1} else {0};
        }

        let mut best_score = MIN_SCORE;

        for mv in moves.sort_with_grading_function(grade, best_move, &self.board) {
            self.board.make_move(&mv);
            self.nodes += 1;
            let score = -self.negamax(-beta, -alpha, depth - 1, ply + 1, null_allowed);
            self.board.undo_move(&mv);

            if self.search_canceled {
                return 0;
            }

            // println!("{} {}", mv, score);

            if score >= beta {
                self.tt.record(tt_index, self.board.key, mv, depth as u8, score, NodeType::Cut);
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

        self.tt.record(tt_index, self.board.key, best_move, depth as u8, best_score, node_type);
        best_score
    }

    fn zero_window_search(&mut self, beta: Score, depth: i8, ply: u8) -> Score {
        // alpha = beta - 1
        // This is either a cut- or all-node
        return self.negamax(beta - 1, beta, depth, ply, false);
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

        for mv in moves.sort_with_grading_function(grade, Move::empty(), &self.board) {
            self.board.make_move(&mv);
            self.nodes += 1;
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
