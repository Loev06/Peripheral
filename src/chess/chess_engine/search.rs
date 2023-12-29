use super::{
    ChessEngine,
    super::{
        Move, Score, MIN_SCORE, CHECKMATE_SCORE, Eval, MoveList, grade
    }
};

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
        if depth == 0 {
            return self.quiescence(alpha, beta);
        }

        let mut moves = MoveList::new();
        self.mg.generate_legal_moves(&mut self.board, &mut moves, false);

        if *moves.get_count() == 0 {
            return if self.board.gs.is_in_check {-CHECKMATE_SCORE - depth as Score} else {0};
        }

        for mv in moves.sort_with_function(grade, &self.board) {
            self.board.make_move(&mv);
            let score = -self.negamax(-beta, -alpha, depth - 1);
            self.board.undo_move(&mv);

            if score >= beta {
                return beta;
            }

            if score > alpha {
                alpha = score;
            }
        }
        alpha
    }

    fn quiescence(&mut self, mut alpha: Score, beta: Score) -> Score {
        let standing_pat = Eval::eval(&self.board) * self.board.gs.player_to_move as Score;

        let moves = MoveList::new();
        if standing_pat >= beta {
            return beta;
        }
        if standing_pat > alpha {
            alpha = standing_pat;
        }

        for mv in moves.sort_with_function(grade, &self.board) {
            self.board.make_move(&mv);
            let score = -self.quiescence(-beta, -alpha);
            self.board.undo_move(&mv);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score
            }
        }
        alpha
    }
}