
use std::error::Error;

use super::{
    Color, Color::*,
    PieceType, PieceType::*,
    precomputed, Bitboard, Square, util, CastlingFlags,
};
use enum_map::{self, EnumMap};

mod make_move;
mod undo_move;
use self::undo_move::GSHistory;
mod parse_fen;

struct FENdata<'a> {
    rows: Vec<&'a str>,
    color: char,
    castling: &'a str,
    en_passant: &'a str,
    half_moves: usize,
    full_moves: usize
}

impl<'a> FENdata<'a> {
    fn try_parse(fen: &'a str) -> Result<Self, Box<dyn Error>> {
        let mut parts = fen.split_ascii_whitespace();
        Ok(Self {
            rows: parts.next().ok_or("No row data")?.split('/').collect(),
            color: parts.next().ok_or("No color data")?.chars().next().ok_or("No color char")?,
            castling: parts.next().ok_or("No castling data")?,
            en_passant: parts.next().ok_or("No en-passant data")?,
            half_moves: parts.next().ok_or("No half move data")?.to_string().parse()?,
            full_moves: parts.next().ok_or("No full move data")?.to_string().parse()?
        })
    }
}

#[derive(Clone, Copy)]
pub struct GameState {
    pub player_to_move: Color,
    pub opponent_color: Color,

    pub playing_king_square: Square,
    pub opponent_king_square: Square,

    pub castling_rights: CastlingFlags,
    pub en_passant_mask: Bitboard,
}

impl GameState {
    fn empty() -> Self {
        Self {
            player_to_move: Color::White,
            opponent_color: Color::Black,

            playing_king_square: 64,
            opponent_king_square: 64,

            castling_rights: CastlingFlags::empty(),
            en_passant_mask: precomputed::EMPTY,
        }
    }

    fn switch_sides(&mut self) {
        (self.player_to_move, self.opponent_color) = (self.opponent_color, self.player_to_move);
    }
}

pub struct Board {
    pub bbs: EnumMap<PieceType, Bitboard>,
    pub piece_list: [Option<PieceType>; 64],
    pub gs: GameState,
    pub gs_history: GSHistory
}

impl Board {
    pub fn empty() -> Self {
        Self {
            bbs: enum_map::enum_map! {_ => precomputed::EMPTY},
            piece_list: [None; 64],
            gs: GameState::empty(),
            gs_history: GSHistory::new()
        }
    }

    pub fn place_piece(&mut self, pt: PieceType, sq: Square) {
        debug_assert!(self.piece_list[sq as usize] == None);
        debug_assert!(self.bbs[pt] & util::bitboard_from_square(sq) == precomputed::EMPTY);

        self.piece_list[sq as usize] = Some(pt);
        self.bbs[pt] ^= util::bitboard_from_square(sq);
    }

    pub fn remove_piece(&mut self, pt: PieceType, sq: Square) {
        debug_assert!(self.piece_list[sq as usize] == Some(pt));
        debug_assert!(self.bbs[pt] & util::bitboard_from_square(sq) != precomputed::EMPTY);

        self.piece_list[sq as usize] = None;
        self.bbs[pt] ^= util::bitboard_from_square(sq);
    }

    pub fn move_piece(&mut self, pt: PieceType, from: Square, to: Square) {
        self.remove_piece(pt, from);
        self.place_piece(pt, to);
    }

    pub fn update_board_data(&mut self) {
        self.bbs[Any(White)] = self.bbs[Pawn(White)] | self.bbs[Knight(White)] | self.bbs[Bishop(White)] | self.bbs[Rook(White)] | self.bbs[Queen(White)] | self.bbs[King(White)];
        self.bbs[Any(Black)] = self.bbs[Pawn(Black)] | self.bbs[Knight(Black)] | self.bbs[Bishop(Black)] | self.bbs[Rook(Black)] | self.bbs[Queen(Black)] | self.bbs[King(Black)];
        self.bbs[Any(Neutral)] = self.bbs[Any(Black)] | self.bbs[Any(White)];

        self.bbs[HVslider(White)] = self.bbs[Rook(White)] | self.bbs[Queen(White)];
        self.bbs[HVslider(Black)] = self.bbs[Rook(Black)] | self.bbs[Queen(Black)];
        self.bbs[Dslider(White)] = self.bbs[Bishop(White)] | self.bbs[Queen(White)];
        self.bbs[Dslider(Black)] = self.bbs[Bishop(Black)] | self.bbs[Queen(Black)];

        self.gs.playing_king_square = util::ls1b_from_bitboard(self.bbs[King(self.gs.player_to_move)]);
        self.gs.opponent_king_square = util::ls1b_from_bitboard(self.bbs[King(self.gs.opponent_color)]);
    }
}