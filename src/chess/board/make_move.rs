use super::{
    Board,
    history::GSHistoryEntry, 
    super::{
        Move, PieceType, Bitboard, PieceType::*, util, precomputed, zobrist::*
    }
};

impl Board {
    #[inline(always)]
    pub fn make_move(&mut self, mv: &Move) {
        let from = mv.get_from();
        let to = mv.get_to();
        let moving_piece = self.piece_list[from as usize].expect("Moving piece should exist");
        let capturing_piece = self.piece_list[to as usize];
        let mut revertable: bool;

        let new_piece_type = if mv.is_promotion() {
            revertable = false;
            mv.get_promotion_piece(self.gs.player_to_move)
        } else {
            revertable = true;
            moving_piece
        };

        self.gs_history.push(GSHistoryEntry {
            gs: self.gs,
            captured_piece: capturing_piece
        });

        self.key ^= ZOBRIST_EP_SQUARE[util::ls1b_from_bitboard(self.gs.en_passant_mask) as usize];
        self.gs.en_passant_mask = precomputed::EMPTY; // Double pawn push check is later

        self.key ^= ZOBRIST_CASTLING[self.gs.castling_rights.bits() as usize];
        self.gs.castling_rights.update(from, to);
        self.key ^= ZOBRIST_CASTLING[self.gs.castling_rights.bits() as usize];
        
        if let Some(pt) = capturing_piece {
            self.remove_piece(pt, to);
            revertable = false;
        } else {
            match new_piece_type {
                WPawn | BPawn => {
                    revertable = false;
                    if mv.is_ep() {
                        self.remove_piece(PieceType::from_color(WPawn, self.gs.opponent_color), to ^ 8); // En-passant
                    } else if mv.intersects(Move::DOUBLE_PAWN_PUSH) {
                        self.gs.en_passant_mask = util::bitboard_from_square(to ^ 8); // Double pawn push
                        self.key ^= ZOBRIST_EP_SQUARE[to as usize];
                    }
                },
                WKing | BKing => {
                    let rook_type = PieceType::from_color(WRook, self.gs.player_to_move);
                    if mv.contains(Move::QUEEN_CASTLE) {
                        revertable = false;
                        self.remove_piece(rook_type, from - 4); // Queen castle
                        self.place_piece(rook_type, from - 1);
                    } else if mv.contains(Move::KING_CASTLE) {
                        revertable = false;
                        self.remove_piece(rook_type, from + 3); // King castle
                        self.place_piece(rook_type, from + 1);
                    }
                },
                _ => ()
            }
        }
        
        self.remove_piece(moving_piece, from);
        self.place_piece(new_piece_type, to);
        
        self.switch_sides();
        self.update_board_data();
        self.key_history.push_key(self.key, revertable)
    }

    #[inline(always)]
    pub fn make_null_move(&mut self) -> Bitboard {
        self.switch_sides();
        self.gs.playing_king_square = util::ls1b_from_bitboard(self.bbs[WKing + self.gs.pt_offset]);
        
        let ep_mask = self.gs.en_passant_mask;
        self.gs.en_passant_mask = precomputed::EMPTY;
        self.key ^= ZOBRIST_EP_SQUARE[util::ls1b_from_bitboard(ep_mask) as usize];
        ep_mask
    }
}