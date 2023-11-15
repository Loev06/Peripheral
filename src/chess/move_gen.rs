use super::{Board, MoveList, precomputed, util, PieceType::{*, self}, Color::*, Bitboard, Square, Move, CastlingFlags};

mod magics;

enum SpecialBitsContainer {
    NormalMove,
    MayPromote(Move),
    ExactMoveBits(Move)
}

pub struct MoveGenerator<'a> {
    // 2^12*4 + 2^11*24 + 2^10*36 = 102400 => 102400 * 8 / 1024 = 800kB
    rook_lookups: Vec<Bitboard>,
    // 2^9*4 + 2^7*12 + 2^6*4 + 2^5*44 = 5248 => 5248 * 8 / 1024 = 41kB
    bishop_lookups: Vec<Bitboard>,

    b: &'a Board
}

impl<'a> MoveGenerator<'a> {
    pub fn new(b: &'a Board) -> Self {
        let mut mg = Self {
            rook_lookups: Vec::new(),
            bishop_lookups: Vec::new(),
            b
        };
        mg.precompute_lookup_tables();
        mg
    }

    pub fn generate_legal_moves(&self, moves: &mut MoveList) {
        let b = self.b;
        moves.reset_count();

        let (check_mask, king_ban) = self.generate_check_mask_and_king_ban();
        let opponent_or_empty = !b.bbs[Any(b.player_to_move)];
        
        if check_mask != precomputed::EMPTY {
            let movable = opponent_or_empty & check_mask;

            let opponent_hv_sliders = b.bbs[HVslider(b.opponent_color)];
            let opponent_d_sliders = b.bbs[Dslider(b.opponent_color)];
            let pin_mask_hv = self.generate_pinmask(opponent_hv_sliders & precomputed::ROOK_MOVES[b.playing_king_square as usize]);
            let pin_mask_d = self.generate_pinmask(opponent_d_sliders & precomputed::BISHOP_MOVES[b.playing_king_square as usize]);
            
            self.add_pawn_moves(moves, movable, pin_mask_hv, pin_mask_d);
            self.add_moves_of_piece_type(moves, Knight(b.player_to_move), movable, pin_mask_hv | pin_mask_d, precomputed::EMPTY);
            self.add_moves_of_piece_type(moves, Bishop(b.player_to_move), movable, pin_mask_hv , pin_mask_d);
            self.add_moves_of_piece_type(moves, Rook(b.player_to_move), movable, pin_mask_d , pin_mask_hv);
            self.add_moves_of_piece_type(moves, Queen(b.player_to_move), movable, pin_mask_hv , pin_mask_d);

            let mut hv_pinned_queens = b.bbs[Queen(b.player_to_move)] & pin_mask_hv;
            self.add_moves_with_function(
                moves, &mut hv_pinned_queens,
                |sq: Square| self.get_rook_attacks(sq) & movable & pin_mask_hv,
                SpecialBitsContainer::NormalMove
            );
        }
        let king_move_mask = opponent_or_empty & !king_ban;
        let mut relevant_king_squares = precomputed::KING_MOVES[b.playing_king_square as usize] & king_move_mask;
        let mut legal_king_moves = self.eliminate_king_moves(&mut relevant_king_squares);
        self.add_castling_moves(moves, &legal_king_moves, &king_move_mask);
        self.add_moves(moves, b.playing_king_square, &mut legal_king_moves, &SpecialBitsContainer::NormalMove);
    }

    fn add_castling_moves(&self, moves: &mut MoveList, legal_king_moves: &Bitboard, king_move_mask: &Bitboard) {
        let b = self.b;
        if b.player_to_move == White {
            if b.castling_rights.contains(CastlingFlags::WK) &&
               legal_king_moves & precomputed::F1BB != 0 &&
               king_move_mask & precomputed::G1BB != 0 &&
               !self.square_attacked_non_pawn(precomputed::G1) {
                moves.add_move(Move::new(precomputed::E1, precomputed::G1, &Move::KING_CASTLE));
            }
            if b.castling_rights.contains(CastlingFlags::WQ) &&
               legal_king_moves & precomputed::D1BB != 0 &&
               king_move_mask & precomputed::C1BB != 0 &&
               !self.square_attacked_non_pawn(precomputed::C1) {
                moves.add_move(Move::new(precomputed::E1, precomputed::C1, &Move::QUEEN_CASTLE));
            }
        } else {
            if b.castling_rights.contains(CastlingFlags::BK) &&
               legal_king_moves & precomputed::F8BB != 0 &&
               king_move_mask & precomputed::G8BB != 0 &&
               !self.square_attacked_non_pawn(precomputed::G8) {
                moves.add_move(Move::new(precomputed::E8, precomputed::G8, &Move::KING_CASTLE));
            }
            if b.castling_rights.contains(CastlingFlags::BQ) &&
               legal_king_moves & precomputed::D8BB != 0 &&
               king_move_mask & precomputed::C8BB != 0 &&
               !self.square_attacked_non_pawn(precomputed::C8) {
                moves.add_move(Move::new(precomputed::E8, precomputed::C8, &Move::QUEEN_CASTLE));
            }
        }
    }

    fn eliminate_king_moves(&self, relevant: &mut Bitboard) -> Bitboard {
        let mut legal_moves = *relevant;
        while *relevant != precomputed::EMPTY {
            let candidate: Square = util::pop_ls1b(relevant);
            if self.square_attacked_non_pawn(candidate) {
                legal_moves ^= util::bitboard_from_square(candidate);
            }
        }
        legal_moves
    }

    fn square_attacked_non_pawn(&self, sq: Square) -> bool {
           precomputed::KNIGHT_MOVES[sq as usize] & self.b.bbs[Knight(self.b.opponent_color)] != 0
        || precomputed::KING_MOVES[sq as usize]   & self.b.bbs[King(self.b.opponent_color)] != 0
        || self.get_rook_attacks(sq)              & self.b.bbs[HVslider(self.b.opponent_color)] != 0
        || self.get_bishop_attacks(sq)            & self.b.bbs[Dslider(self.b.opponent_color)] != 0
    }

    fn add_pawn_moves(&self, moves: &mut MoveList, movable: Bitboard, pin_mask_hv: Bitboard, pin_mask_d: Bitboard) {
        let b = self.b;

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

            self.add_moves_with_function(moves, &mut can_push_single, |sq| util::bitboard_from_square(sq + 8), SpecialBitsContainer::MayPromote(Move::empty()));
            self.add_moves_with_function(moves, &mut can_push_double, |sq| util::bitboard_from_square(sq + 16), SpecialBitsContainer::ExactMoveBits(Move::DOUBLE_PAWN_PUSH));
            self.add_moves_with_function(moves, &mut can_take_d1, |sq| util::bitboard_from_square(sq + 7), SpecialBitsContainer::MayPromote(Move::CAPTURE));
            self.add_moves_with_function(moves, &mut can_take_d2, |sq| util::bitboard_from_square(sq + 9), SpecialBitsContainer::MayPromote(Move::CAPTURE));
        } else {
            let not_hor_or_d_pinned = not_pinned | not_d_pinned & (pin_mask_hv << 8);
            let forward_empty = not_hor_or_d_pinned & (!b.bbs[Any(Neutral)] << 8);

            let not_pinned_d1 = precomputed::NOT_H_FILE & ((only_d_pinned & (pin_mask_d << 7)) | not_pinned) & (movable) << 7;
            let not_pinned_d2 = precomputed::NOT_A_FILE & ((only_d_pinned & (pin_mask_d << 9)) | not_pinned) & (movable) << 9;
            
            can_push_single = forward_empty & (movable << 8);
            can_push_double = forward_empty & precomputed::SEVENTH_ROW & ((!b.bbs[Any(Neutral)] & movable) << 16);
            
            can_take_d1 = not_pinned_d1 & (b.bbs[Any(White)] << 7);
            can_take_d2 = not_pinned_d2 & (b.bbs[Any(White)] << 9);

            self.add_moves_with_function(moves, &mut can_push_single, |sq| util::bitboard_from_square(sq - 8), SpecialBitsContainer::MayPromote(Move::empty()));
            self.add_moves_with_function(moves, &mut can_push_double, |sq| util::bitboard_from_square(sq - 16), SpecialBitsContainer::ExactMoveBits(Move::DOUBLE_PAWN_PUSH));
            self.add_moves_with_function(moves, &mut can_take_d1, |sq| util::bitboard_from_square(sq - 7), SpecialBitsContainer::MayPromote(Move::CAPTURE));
            self.add_moves_with_function(moves, &mut can_take_d2, |sq| util::bitboard_from_square(sq - 9), SpecialBitsContainer::MayPromote(Move::CAPTURE));
        }

    }

    fn add_moves_of_piece_type(&self, moves: &mut MoveList, pt: PieceType, movable: Bitboard, blockading_pin: Bitboard, restricting_pin: Bitboard) {
        let mut moving_pieces = self.b.bbs[pt] & !blockading_pin;
        let mut pinned_pieces = moving_pieces & restricting_pin;
        moving_pieces ^= pinned_pieces;

        let move_gen = |sq: Square| match pt {
            Knight(_) => precomputed::KNIGHT_MOVES[sq as usize] & movable,
            Bishop(_) => self.get_bishop_attacks(sq) & movable,
            Rook(_) => self.get_rook_attacks(sq) & movable,
            Queen(_) => (self.get_bishop_attacks(sq)
                       | self.get_rook_attacks(sq)) & movable,
            _ => precomputed::EMPTY
        };

        let pinned_move_gen = |sq: Square| match pt {
            // Pinned knights may not move
            Knight(_) => precomputed::EMPTY,
            // This method only covers diagonally pinned queens. HV pinned queens get added later.
            Bishop(_) | Queen(_) => self.get_bishop_attacks(sq) & movable & restricting_pin,
            Rook(_) => self.get_rook_attacks(sq) & movable & restricting_pin,
            _ => precomputed::EMPTY
        };

        self.add_moves_with_function(moves, &mut moving_pieces, move_gen, SpecialBitsContainer::NormalMove);
        self.add_moves_with_function(moves, &mut pinned_pieces, pinned_move_gen, SpecialBitsContainer::NormalMove);
    }

    fn add_moves_with_function<F>(&self, moves: &mut MoveList, moving_pieces: &mut Bitboard, move_gen: F, special_bits: SpecialBitsContainer)
    where
        F: Fn(Square) -> Bitboard
    {
        while *moving_pieces != precomputed::EMPTY {
            let sq = util::pop_ls1b(moving_pieces);
            self.add_moves(moves, sq, &mut move_gen(sq), &special_bits);
        }
    }

    #[inline(always)] // Compiler did not inline this method by default..
    fn add_moves(&self, moves: &mut MoveList, sq: Square, to_squares: &mut Bitboard, special_bits: &SpecialBitsContainer) {
        while *to_squares != precomputed::EMPTY {
            let to_sq = util::pop_ls1b(to_squares);

            let special_bits = match special_bits {
                SpecialBitsContainer::NormalMove => if self.b.piece_list[to_sq as usize] == None {Move::empty()} else {Move::CAPTURE},
                SpecialBitsContainer::ExactMoveBits(bits) => *bits,
                SpecialBitsContainer::MayPromote(capture_flag) => {
                    if to_sq < 8 || to_sq >= 56 {
                        self.add_promotion_moves(moves, sq, to_sq, capture_flag);
                        continue;
                    }
                    *capture_flag
                }
            };

            let mv = Move::new(sq, to_sq, &special_bits);
            moves.add_move(mv);
        }
    }

    fn add_promotion_moves(&self, moves: &mut MoveList, sq: Square, to_sq: Square, capture_flag: &Move) {
        let mut flags = Move::QUEEN_PROMOTION.union(*capture_flag);
        moves.add_move(Move::new(sq, to_sq, &flags));

        flags.toggle(Move::QUEEN_TO_KNIGHT);
        moves.add_move(Move::new(sq, to_sq, &flags));

        flags.toggle(Move::KNIGHT_TO_ROOK);
        moves.add_move(Move::new(sq, to_sq, &flags));

        flags.toggle(Move::ROOK_TO_BISHOP);
        moves.add_move(Move::new(sq, to_sq, &flags));
    }
    
    fn generate_check_mask_and_king_ban(&self) -> (Bitboard, Bitboard) {
        let b = self.b;
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
    
        check_mask |= precomputed::KNIGHT_MOVES[king_square as usize] & b.bbs[Knight(b.opponent_color)];
        
        let mut already_in_check = check_mask != precomputed::EMPTY;
        
        let rook_attacks: Bitboard = self.get_rook_attacks(king_square);
        for ray in precomputed::ROOK_RAYS[king_square as usize] {

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

        let bishop_attacks: Bitboard = self.get_bishop_attacks(king_square);
        for ray in precomputed::BISHOP_RAYS[king_square as usize] {

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

    fn generate_pinmask(&self, snipers: Bitboard) -> Bitboard {
        let mut pin_mask = precomputed::EMPTY;
        let mut snipers = snipers;
        while snipers != precomputed::EMPTY {
            let sniper_square: Square = util::pop_ls1b(&mut snipers);
            let ray = precomputed::BETWEEN_BITBOARDS[self.b.playing_king_square as usize][sniper_square as usize];

            if (ray & self.b.bbs[Any(Neutral)]).is_power_of_two() {
                pin_mask |= ray | util::bitboard_from_square(sniper_square);
            }
        }
        pin_mask
    }
}
