use super::{Board, MoveList, precomputed, util, PieceType::{*, self}, Color::*, Bitboard, Square, Move};

mod magics;

pub struct MoveGenerator {
    // 2^12*4 + 2^11*24 + 2^10*36 = 102400 => 102400 * 8 / 1024 = 800kB
    rook_lookups: Vec<Bitboard>,
    // 2^9*4 + 2^7*12 + 2^6*4 + 2^5*44 = 5248 => 5248 * 8 / 1024 = 41kB
    bishop_lookups: Vec<Bitboard>
}

impl MoveGenerator {
    pub fn new() -> Self {
        let mut mg = Self {
            rook_lookups: Vec::new(),
            bishop_lookups: Vec::new()
        };
        mg.precompute_lookup_tables();
        mg
    }

    pub fn generate_legal_moves(&self, b: &Board, moves: &mut MoveList) {
        moves.reset_count();

        let (check_mask, king_ban) = self.generate_check_mask_and_king_ban(b);
        let opponent_or_empty = !b.bbs[Any(b.player_to_move)];
        
        if check_mask != precomputed::EMPTY {
            let movable = opponent_or_empty & check_mask;

            let opponent_hv_sliders = b.bbs[HVslider(b.opponent_color)];
            let opponent_d_sliders = b.bbs[Dslider(b.opponent_color)];
            let pin_mask_hv = self.generate_pinmask(b, opponent_hv_sliders & precomputed::ROOK_MOVES[b.playing_king_square]);
            let pin_mask_d = self.generate_pinmask(b, opponent_d_sliders & precomputed::BISHOP_MOVES[b.playing_king_square]);
            
            self.add_pawn_moves(moves, b, movable, pin_mask_hv, pin_mask_d);
            self.add_moves_of_piece_type(moves, b, Knight(b.player_to_move), movable, pin_mask_hv | pin_mask_d, precomputed::EMPTY);
            self.add_moves_of_piece_type(moves, b, Bishop(b.player_to_move), movable, pin_mask_hv , pin_mask_d);
            self.add_moves_of_piece_type(moves, b, Rook(b.player_to_move), movable, pin_mask_d , pin_mask_hv);
            self.add_moves_of_piece_type(moves, b, Queen(b.player_to_move), movable, pin_mask_hv , pin_mask_d);

            let mut hv_pinned_queens = b.bbs[Queen(b.player_to_move)] & pin_mask_hv;
            self.add_moves_with_function(
                moves, b, Queen(b.player_to_move), &mut hv_pinned_queens,
                |sq: Square| self.get_rook_attacks(sq, b) & movable & pin_mask_hv
            );
        }
        let mut relevant_king_squares = precomputed::KING_MOVES[b.playing_king_square] & opponent_or_empty & !king_ban;
        let mut legal_king_moves = self.eliminate_king_moves(b, &mut relevant_king_squares);
        self.add_moves(moves, b, King(b.player_to_move), b.playing_king_square, &mut legal_king_moves);
    }

    fn eliminate_king_moves(&self, b: &Board, relevant: &mut Bitboard) -> Bitboard {
        let mut legal_moves = *relevant;
        while *relevant != precomputed::EMPTY {
            let candidate: Square = util::pop_ls1b(relevant);
            if self.square_attacked_non_pawn(b, candidate) {
                legal_moves ^= util::bitboard_from_square(candidate);
            }
        }
        legal_moves
    }

    fn square_attacked_non_pawn(&self, b: &Board, sq: Square) -> bool {
           precomputed::KNIGHT_MOVES[sq] & b.bbs[Knight(b.opponent_color)] != 0
        || precomputed::KING_MOVES[sq] & b.bbs[King(b.opponent_color)] != 0
        || self.get_rook_attacks(sq, b) & b.bbs[HVslider(b.opponent_color)] != 0
        || self.get_bishop_attacks(sq, b) & b.bbs[Dslider(b.opponent_color)] != 0
    }

    fn add_pawn_moves(&self, moves: &mut MoveList, b: &Board, movable: Bitboard, pin_mask_hv: Bitboard, pin_mask_d: Bitboard) {
        let pawns = b.bbs[Pawn(b.player_to_move)];
        let not_hv_pinned = pawns & !pin_mask_hv;
        let not_d_pinned = pawns & !pin_mask_d;
        let not_pinned = not_d_pinned & not_hv_pinned;
        let only_d_pinned = not_hv_pinned & pin_mask_d;

        let mut can_push_single: Bitboard;
        let mut can_push_double: Bitboard;
        let mut can_take_d1: Bitboard;
        let mut can_take_d2: Bitboard;

        // TODO: Less copying of code (inlining reverse shifting operation?)
        if b.player_to_move == White {
            let not_hor_or_d_pinned = not_pinned | not_d_pinned & (pin_mask_hv >> 8);
            let forward_empty = not_hor_or_d_pinned & (!b.bbs[Any(Neutral)] >> 8);

            let not_pinned_d1 = precomputed::NOT_A_FILE & ((only_d_pinned & (pin_mask_d >> 7)) | not_pinned) & (movable) >> 7;
            let not_pinned_d2 = precomputed::NOT_H_FILE & ((only_d_pinned & (pin_mask_d >> 9)) | not_pinned) & (movable) >> 9;
            
            can_push_single = forward_empty & (movable >> 8);
            can_push_double = forward_empty & precomputed::SECOND_ROW & ((!b.bbs[Any(Neutral)] & movable) >> 16);
            
            can_take_d1 = not_pinned_d1 & (b.bbs[Any(Black)] >> 7);
            can_take_d2 = not_pinned_d2 & (b.bbs[Any(Black)] >> 9);

            self.add_moves_with_function(moves, b, Pawn(White), &mut can_push_single, |sq| util::bitboard_from_square(sq + 8));
            self.add_moves_with_function(moves, b, Pawn(White), &mut can_push_double, |sq| util::bitboard_from_square(sq + 16));
            self.add_moves_with_function(moves, b, Pawn(White), &mut can_take_d1, |sq| util::bitboard_from_square(sq + 7));
            self.add_moves_with_function(moves, b, Pawn(White), &mut can_take_d2, |sq| util::bitboard_from_square(sq + 9));
        } else {
            let not_hor_or_d_pinned = not_pinned | not_d_pinned & (pin_mask_hv << 8);
            let forward_empty = not_hor_or_d_pinned & (!b.bbs[Any(Neutral)] << 8);

            let not_pinned_d1 = precomputed::NOT_H_FILE & ((only_d_pinned & (pin_mask_d << 7)) | not_pinned) & (movable) << 7;
            let not_pinned_d2 = precomputed::NOT_A_FILE & ((only_d_pinned & (pin_mask_d << 9)) | not_pinned) & (movable) << 9;
            
            can_push_single = forward_empty & (movable << 8);
            can_push_double = forward_empty & precomputed::SEVENTH_ROW & ((!b.bbs[Any(Neutral)] & movable) << 16);
            
            can_take_d1 = not_pinned_d1 & (b.bbs[Any(White)] << 7);
            can_take_d2 = not_pinned_d2 & (b.bbs[Any(White)] << 9);

            self.add_moves_with_function(moves, b, Pawn(Black), &mut can_push_single, |sq| util::bitboard_from_square(sq - 8));
            self.add_moves_with_function(moves, b, Pawn(Black), &mut can_push_double, |sq| util::bitboard_from_square(sq - 16));
            self.add_moves_with_function(moves, b, Pawn(Black), &mut can_take_d1, |sq| util::bitboard_from_square(sq - 7));
            self.add_moves_with_function(moves, b, Pawn(Black), &mut can_take_d2, |sq| util::bitboard_from_square(sq - 9));
        }

    }

    fn add_moves_of_piece_type(&self, moves: &mut MoveList, b: &Board, pt: PieceType, movable: Bitboard, blockading_pin: Bitboard, restricting_pin: Bitboard) {
        let mut moving_pieces = b.bbs[pt] & !blockading_pin;
        let mut pinned_pieces = moving_pieces & restricting_pin;
        moving_pieces ^= pinned_pieces;

        let move_gen = |sq: Square| match pt {
            Knight(_) => precomputed::KNIGHT_MOVES[sq] & movable,
            Bishop(_) => self.get_bishop_attacks(sq, b) & movable,
            Rook(_) => self.get_rook_attacks(sq, b) & movable,
            Queen(_) => (self.get_bishop_attacks(sq, b)
                        | self.get_rook_attacks(sq, b)) & movable,
            _ => precomputed::EMPTY
        };

        let pinned_move_gen = |sq: Square| match pt {
            // Pinned knights may not move
            Knight(_) => precomputed::EMPTY,
            // This method only covers diagonally pinned queens. HV pinned queens get added later.
            Bishop(_) | Queen(_) => self.get_bishop_attacks(sq, b) & movable & restricting_pin,
            Rook(_) => self.get_rook_attacks(sq, b) & movable & restricting_pin,
            _ => precomputed::EMPTY
        };

        self.add_moves_with_function(moves, b, pt, &mut moving_pieces, move_gen);
        self.add_moves_with_function(moves, b, pt, &mut pinned_pieces, pinned_move_gen)
    }

    fn add_moves_with_function<F>(&self, moves: &mut MoveList, b: &Board, pt: PieceType, moving_pieces: &mut Bitboard, move_gen: F)
    where
        F: Fn(Square) -> Bitboard
    {
        while *moving_pieces != precomputed::EMPTY {
            let sq = util::pop_ls1b(moving_pieces);
            self.add_moves(moves, b, pt, sq, &mut move_gen(sq));
        }
    }

    fn add_moves(&self, moves: &mut MoveList, b: &Board, pt: PieceType, sq: Square, to_squares: &mut Bitboard) {
        while *to_squares != precomputed::EMPTY {
            let to_sq = util::pop_ls1b(to_squares);

            if (to_sq >= 56 && pt == Pawn(White)) || (to_sq < 8 && pt == Pawn(Black)) {
                // add_castling_moves
                continue;
            }

            let mv = Move::new(pt, sq, to_sq, b.piece_list[to_sq]);
            moves.add_move(mv);
        }
    }
    
    fn generate_check_mask_and_king_ban(&self, b: &Board) -> (Bitboard, Bitboard) {
        let mut check_mask = precomputed::EMPTY;

        let king_mask = util::bitboard_from_square(b.playing_king_square);
        let king_square = b.playing_king_square;
    
        // Left shift by negative integer not allowed, consider inlining with function
        let mut king_ban = if b.player_to_move == White {
              ((b.bbs[Pawn(Black)] & precomputed::NOT_H_FILE) >> 7)
            | ((b.bbs[Pawn(Black)] & precomputed::NOT_A_FILE) >> 9)
        } else {
              ((b.bbs[Pawn(White)] & precomputed::NOT_A_FILE) << 7)
            | ((b.bbs[Pawn(White)] & precomputed::NOT_H_FILE) << 9)
        };

        check_mask |= king_ban & king_mask;
    
        check_mask |= precomputed::KNIGHT_MOVES[king_square] & b.bbs[Knight(b.opponent_color)];
        
        let mut already_in_check = check_mask != precomputed::EMPTY;
        
        let rook_attacks: Bitboard = self.get_rook_attacks(king_square, b);
        for ray in precomputed::ROOK_RAYS[king_square] {

            let attacker = rook_attacks & ray & b.bbs[HVslider(b.opponent_color)];
            if attacker != precomputed::EMPTY {
                // Consider precomputing kingban tables from king square, only XOR attacker needed instead of trailingzeros
                king_ban |= precomputed::ROOK_MOVES[util::ls1b_from_bitboard(attacker)];

                if already_in_check {
                    return (0, king_ban); // double check
                }
                already_in_check = true;

                check_mask |= rook_attacks & ray;
                // Do not break loop here, double attack by two rooks is possible when promoting:
                // (https://lichess.org/editor/3nk3/4P3/8/8/8/8/8/K3R3_w_-_-_0_1)
            }
        }

        let bishop_attacks: Bitboard = self.get_bishop_attacks(king_square, b);
        for ray in precomputed::BISHOP_RAYS[king_square] {

            let attacker = bishop_attacks & ray & b.bbs[Dslider(b.opponent_color)];
            if attacker != precomputed::EMPTY {
                king_ban |= precomputed::BISHOP_MOVES[util::ls1b_from_bitboard(attacker)];

                if already_in_check {
                    return (0, king_ban); // double check
                }

                return (bishop_attacks & ray, king_ban); // No more checks possible.
            }
        }
    
        return (
            if check_mask == precomputed::EMPTY {precomputed::FULL} else {check_mask},
            king_ban
        )
    }

    fn generate_pinmask(&self, board: &Board, snipers: Bitboard) -> Bitboard {
        let mut pin_mask = precomputed::EMPTY;
        let mut snipers = snipers;
        while snipers != precomputed::EMPTY {
            let sniper_square: Square = util::pop_ls1b(&mut snipers);
            let ray = precomputed::BETWEEN_BITBOARDS[board.playing_king_square][sniper_square];

            if (ray & board.bbs[Any(Neutral)]).is_power_of_two() {
                pin_mask |= ray | util::bitboard_from_square(sniper_square);
            }
        }
        pin_mask
    }
}
