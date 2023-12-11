
use std::{error::Error, fmt::{Debug, Display}};

use super::{
    Color, Color::*,
    PieceType, PieceType::*,
    precomputed, Bitboard, Square, util, CastlingFlags,
};

pub mod zobrist;
use self::zobrist::Zobrist;
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

#[derive(Clone, Copy, Debug)]
pub struct GameState {
    pub player_to_move: Color,
    pub opponent_color: Color,
    pub pt_offset: PieceType,

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
            pt_offset: WPawn,

            playing_king_square: 64,
            opponent_king_square: 64,

            castling_rights: CastlingFlags::empty(),
            en_passant_mask: precomputed::EMPTY,
        }
    }

    fn switch_sides(&mut self) {
        (self.player_to_move, self.opponent_color) = (self.opponent_color, self.player_to_move);
        self.pt_offset = match self.pt_offset {
            WPawn => BPawn,
            BPawn => WPawn,
            pt => panic!("Not a valid pt_offset: {pt}")
        }
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!(
            "curr player: {}\nking squares: {}, {}\ncastling: {}\nep: {}",
            if self.player_to_move == White {'w'} else {'b'},
            precomputed::SQUARE_NAMES[self.playing_king_square as usize],
            precomputed::SQUARE_NAMES[self.opponent_king_square as usize],
            self.castling_rights,
            if self.en_passant_mask == precomputed::EMPTY {
                "-"
            } else {
                precomputed::SQUARE_NAMES[util::ls1b_from_bitboard(self.en_passant_mask) as usize]
            }
        ).as_str())
    }
}

pub struct Board {
    pub bbs: [u64; 12 + 7],
    pub piece_list: [Option<PieceType>; 64],
    pub gs: GameState,
    pub gs_history: GSHistory,
    pub hash: u64,

    zobrists: Zobrist
}

impl Board {
    pub fn empty() -> Self {
        Self {
            bbs: [0; 12 + 7],
            piece_list: [None; 64],
            gs: GameState::empty(),
            gs_history: GSHistory::new(),
            hash: 0,
            zobrists: Zobrist::new().expect("Failed to generate zobrist values.")
        }
    }

    pub fn place_piece(&mut self, pt: PieceType, sq: Square) {
        debug_assert!(self.piece_list[sq as usize] == None);
        debug_assert!(self.bbs[pt as usize] & util::bitboard_from_square(sq) == precomputed::EMPTY);

        self.piece_list[sq as usize] = Some(pt);
        self.bbs[pt as usize] ^= util::bitboard_from_square(sq);
    }

    pub fn remove_piece(&mut self, pt: PieceType, sq: Square) {
        debug_assert!(self.piece_list[sq as usize] == Some(pt));
        debug_assert!(self.bbs[pt as usize] & util::bitboard_from_square(sq) != precomputed::EMPTY);

        self.piece_list[sq as usize] = None;
        self.bbs[pt as usize] ^= util::bitboard_from_square(sq);
    }

    pub fn move_piece(&mut self, pt: PieceType, from: Square, to: Square) {
        self.remove_piece(pt, from);
        self.place_piece(pt, to);
    }

    pub fn update_board_data(&mut self) {
        self.bbs[AnyWhite as usize] = self.bbs[WPawn as usize] | self.bbs[WKnight as usize] | self.bbs[WBishop as usize] | self.bbs[WRook as usize] | self.bbs[WQueen as usize] | self.bbs[WKing as usize];
        self.bbs[AnyBlack as usize] = self.bbs[BPawn as usize] | self.bbs[BKnight as usize] | self.bbs[BBishop as usize] | self.bbs[BRook as usize] | self.bbs[BQueen as usize] | self.bbs[BKing as usize];
        self.bbs[AnyPiece as usize] = self.bbs[AnyBlack as usize] | self.bbs[AnyWhite as usize];

        self.bbs[WHVSlider as usize] = self.bbs[WRook as usize] | self.bbs[WQueen as usize];
        self.bbs[BHVSlider as usize] = self.bbs[BRook as usize] | self.bbs[BQueen as usize];
        self.bbs[WDSlider as usize] = self.bbs[WBishop as usize] | self.bbs[WQueen as usize];
        self.bbs[BDSlider as usize] = self.bbs[BBishop as usize] | self.bbs[BQueen as usize];

        self.gs.playing_king_square = util::ls1b_from_bitboard(self.bbs[WKing + self.gs.pt_offset]);
        self.gs.opponent_king_square = util::ls1b_from_bitboard(self.bbs[WKing + self.gs.pt_offset]);
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!(
            "\n{}\nhash: {}\n{}\n",
            (0..8).rev().map(|y| {
                (0..8).map(|x| {
                    match self.piece_list[util::square_from_coord(x, y) as usize] {
                        Some(pt) => pt.to_string(),
                        None => String::from(" "),
                    }
                })
                .fold(String::new(), |a, b| a + &b + " ")
                .trim_end()
                .to_owned()
            })
            .fold(String::new(), |a, b| a + &b + "\n")
            .trim_end(),
            self.hash,
            self.gs
        ).as_str())
    }
}