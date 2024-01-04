use super::{
    ChessEngine, TT_SIZE, TT_INDEX_SHIFT,
    super::{
        Move, Score, MIN_SCORE, CHECKMATE_SCORE, Eval, MoveList, grade
    }
};

mod transposition_table;
pub use transposition_table::TranspositionTable;

#[derive(Clone)]
pub enum NodeType {
    PVNode,
    AllNode,
    CutNode,
    LeafNode
}

impl ChessEngine {
    pub fn search(&mut self, depth: u8) -> (Move, Score) {
        self.root_negamax(depth)
    }

    fn root_negamax(&mut self, depth: u8) -> (Move, Score) {
        assert!(depth > 0, "Cannot run search on depth <= 0.");

        let mut moves = MoveList::new();
        self.mg.generate_legal_moves(&mut self.board, &mut moves, false);

        assert!(*moves.get_count() > 0, "Cannot run search on ended game.");

        let mut best_score = MIN_SCORE;
        let mut best_move = Move::empty();

        for mv in moves.sort_with_function(grade, &self.board) {
            self.board.make_move(&mv);
            let score = -self.negamax(MIN_SCORE, -best_score, depth - 1);
            self.board.undo_move(&mv);

            if score > best_score {
                best_score = score;
                best_move = mv;
            }
        }

        (best_move, best_score)
    }

    fn negamax(&mut self, mut alpha: Score, beta: Score, depth: u8) -> Score {
        let tt_index = (self.board.key as usize >> TT_INDEX_SHIFT) & (TT_SIZE - 1);
        if let Some(score) = self.tt.probe(tt_index, alpha, beta, depth, self.board.key) {
            return score;
        }

        if depth == 0 {
            let score = self.quiescence(alpha, beta);
            self.tt.record(tt_index, self.board.key, Move::empty(), depth, score, NodeType::LeafNode);
            return score;
        }

        let mut node_type = NodeType::AllNode;

        let mut moves = MoveList::new();
        self.mg.generate_legal_moves(&mut self.board, &mut moves, false);

        if *moves.get_count() == 0 {
            return if self.board.gs.is_in_check {-CHECKMATE_SCORE - depth as Score} else {0};
        }

        let mut best_score = MIN_SCORE;
        let mut best_move = Move::empty();

        for mv in moves.sort_with_function(grade, &self.board) {
            self.board.make_move(&mv);
            let score = -self.negamax(-beta, -alpha, depth - 1);
            self.board.undo_move(&mv);

            if score >= beta {
                self.tt.record(tt_index, self.board.key, mv, depth, score, NodeType::CutNode);
                return score; // fail-soft beta-cutoff - lower bound
            }

            if score > best_score {
                best_score = score;
                if score > alpha {
                    best_move = mv;
                    node_type = NodeType::PVNode;
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

        for mv in moves.sort_with_function(grade, &self.board) {
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