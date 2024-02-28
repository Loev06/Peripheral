use super::{
    MoveGenerator,
    super::{Board, Move, Square, precomputed, util, PieceType::{self, *}, Color::*, CastlingFlags}
};

impl MoveGenerator {
    // a move is pseudo-legal if:
    // 1. from-square is own piece
    // 2. empty or opponent piece on to-square
    // 3. match pieces:
    //      pawn:   normal move: to-square is empty and << 8
    //              capture: to-square is opponent and << 7 or << 9
    //              en-passant: to-square is empty and << 7 or << 9, AND en-passant square is opponent pawn
    //              double pawn push: to-square and in-between squares are empty (<< 8 and << 16)
    //      knight: to-square is in lookup
    //      sliding piece: lookup magic attack
    //      king:   normal move: to-square is in lookup
    //              castling move: check castling right and empty squares
    pub fn is_pseudo_legal_move(&self, b: &Board, mv: Move) -> bool {
        let from = mv.get_from();
        let to = mv.get_to();

        let Some(moving_piece) = b.piece_list[from as usize] else {
            return false; // from-square is empty
        };

        let opponent_pieces = b.bbs[PieceType::from_color(AnyWhite, b.gs.opponent_color) as usize];
        
        let from_bb = util::bitboard_from_square(from);
        if from_bb & opponent_pieces != precomputed::EMPTY {
            return false; // from-square is opponent
        }

        let own_pieces = b.bbs[PieceType::from_color(AnyWhite, b.gs.player_to_move) as usize];
        let to_bb = util::bitboard_from_square(to);
        if to_bb & own_pieces != precomputed::EMPTY {
            return false; // to-square is own piece
        }

        match moving_piece {
            WPawn | BPawn => {
                let to_empty = to_bb & opponent_pieces == precomputed::EMPTY;
                if to_empty {
                    (mv.is_ep() && to_bb & b.gs.en_passant_mask != precomputed::EMPTY) // en-passant
                        ||  to == from + ( 8 * b.gs.player_to_move as i8) as Square // normal pawn push
                        || (to == from + (16 * b.gs.player_to_move as i8) as Square
                            && util::shift_dir(from_bb, 8, b.gs.player_to_move) & b.bbs[AnyPiece as usize] == precomputed::EMPTY) // double pawn push
                } else {
                    to == from + (7 * b.gs.player_to_move as i8) as Square // captures
                        || to == from + (9 * b.gs.player_to_move as i8) as Square
                }
            },
            WKnight | BKnight => to_bb & precomputed::KNIGHT_MOVES[from as usize] != precomputed::EMPTY,
            WKing | BKing => {
                if mv.contains(Move::QUEEN_CASTLE) {
                    match b.gs.player_to_move {
                        White => b.gs.castling_rights.contains(CastlingFlags::WQ) && precomputed::WQ_CASTLE_SQUARES & b.bbs[AnyPiece as usize] == precomputed::EMPTY,
                        Black => b.gs.castling_rights.contains(CastlingFlags::BQ) && precomputed::BQ_CASTLE_SQUARES & b.bbs[AnyPiece as usize] == precomputed::EMPTY
                    }
                } else if mv.contains(Move::KING_CASTLE) {
                    match b.gs.player_to_move {
                        White => b.gs.castling_rights.contains(CastlingFlags::WK) && precomputed::WK_CASTLE_SQUARES & b.bbs[AnyPiece as usize] == precomputed::EMPTY,
                        Black => b.gs.castling_rights.contains(CastlingFlags::BK) && precomputed::BK_CASTLE_SQUARES & b.bbs[AnyPiece as usize] == precomputed::EMPTY
                    }
                } else {
                    to_bb & precomputed::KING_MOVES[from as usize] != precomputed::EMPTY
                }
            },
            WRook | BRook => self.get_rook_attacks(b.bbs[AnyPiece as usize], from) & to_bb != precomputed::EMPTY,
            WBishop | BBishop => self.get_bishop_attacks(b.bbs[AnyPiece as usize], from) & to_bb != precomputed::EMPTY,
            WQueen | BQueen => self.get_rook_attacks(b.bbs[AnyPiece as usize], from) & to_bb != precomputed::EMPTY
                            || self.get_bishop_attacks(b.bbs[AnyPiece as usize], from) & to_bb != precomputed::EMPTY,
            pt => panic!("Invalid piece type: {}", pt)
        }
    }
}