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
        self.mg.generate_legal_moves(&mut self.board, &mut moves);

        assert!(*moves.get_count() > 0, "Cannot run search on ended game.");

        let mut best_score = MIN_SCORE;
        let mut best_move = Move::empty();

        moves.grade_moves_with_function(grade, &self.board);

        for mv in moves.sort() {
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
            return Eval::eval(&self.board, false) * self.board.gs.player_to_move as Score;
        }

        let mut moves = MoveList::new();
        self.mg.generate_legal_moves(&mut self.board, &mut moves);

        if *moves.get_count() == 0 {
            return if self.board.gs.is_in_check {-CHECKMATE_SCORE - depth as Score} else {0};
        }
        
        moves.grade_moves_with_function(grade, &self.board);

        for mv in moves.sort() {
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
}