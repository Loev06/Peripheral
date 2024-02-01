
use std::{error::Error, fmt::{Debug, Display}};

use super::{
    Color, Color::*,
    PieceType, PieceType::*,
    precomputed, Bitboard, Square, util, CastlingFlags,
};

pub mod zobrist;
mod make_move;
mod undo_move;
use self::{history::GSHistory, zobrist::*};
mod parse_fen;
mod history;
use history::KeyHistory;

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
    pub is_in_check: bool,

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
            is_in_check: false,

            castling_rights: CastlingFlags::empty(),
            en_passant_mask: precomputed::EMPTY,
        }
    }

    fn switch_sides(&mut self) {
        (self.player_to_move, self.opponent_color) = (self.opponent_color, self.player_to_move);
        self.pt_offset = match self.player_to_move {
            White => WPawn,
            Black => BPawn
        };
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!(
            "curr player: {}\nking square: {}\ncastling: {}\nep: {}",
            self.player_to_move,
            precomputed::SQUARE_NAMES[self.playing_king_square as usize],
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
    pub bbs: [u64; 19],
    pub piece_list: [Option<PieceType>; 64],
    pub gs: GameState,
    pub key: u64,

    gs_history: GSHistory,
    key_history: KeyHistory
}

impl Board {
    pub fn empty() -> Self {
        Self {
            bbs: [0; 12 + 7],
            piece_list: [None; 64],
            gs: GameState::empty(),
            key: 0,

            gs_history: GSHistory::new(),
            key_history: KeyHistory::new(0)
        }
    }

    pub fn switch_sides(&mut self) {
        self.gs.switch_sides();
        self.key ^= ZOBRIST_BLACK_TO_MOVE;
    }

    pub fn place_piece(&mut self, pt: PieceType, sq: Square) {
        debug_assert!(self.piece_list[sq as usize] == None);
        debug_assert!(self.bbs[pt as usize] & util::bitboard_from_square(sq) == precomputed::EMPTY);

        self.piece_list[sq as usize] = Some(pt);
        self.bbs[pt as usize] ^= util::bitboard_from_square(sq);
        self.key ^= ZOBRIST_PIECE_SQUARE[pt as usize][sq as usize];
    }

    pub fn remove_piece(&mut self, pt: PieceType, sq: Square) {
        debug_assert!(self.piece_list[sq as usize] == Some(pt));
        debug_assert!(self.bbs[pt as usize] & util::bitboard_from_square(sq) != precomputed::EMPTY);

        self.piece_list[sq as usize] = None;
        self.bbs[pt as usize] ^= util::bitboard_from_square(sq);
        self.key ^= ZOBRIST_PIECE_SQUARE[pt as usize][sq as usize];
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
    }

    pub fn make_key(&self) -> u64 {
        let mut key = 0;

        for pt in 0..12 {
            let mut bb = self.bbs[pt];
            while bb != precomputed::EMPTY {
                let sq = util::pop_ls1b(&mut bb);
                key ^= ZOBRIST_PIECE_SQUARE[pt][sq as usize];
            }
        }

        if self.gs.en_passant_mask != precomputed::EMPTY {
            key ^= ZOBRIST_EP_SQUARE[util::get_square_x(util::ls1b_from_bitboard(self.gs.en_passant_mask)) as usize];
        }

        if self.gs.player_to_move == Black {
            key ^= ZOBRIST_BLACK_TO_MOVE;
        }

        key ^= ZOBRIST_CASTLING[self.gs.castling_rights.bits() as usize];

        key
    }

    pub fn get_fen(&self) -> String {
        format!("{} {} {} {} {} {}",
            (0..8).rev().map(|y| {
                let mut empty_count = 0;
                format!("{}{}",
                    (0..8).map(|x| {
                        match self.piece_list[util::square_from_coord(x, y) as usize] {
                            Some(pt) => {
                                let old_count = empty_count;
                                empty_count = 0;
                                format!("{}{}",
                                    if old_count > 0 {old_count.to_string()} else {String::new()},
                                    pt.to_string()
                                )
                            },
                            None => {
                                empty_count += 1;
                                String::from("")
                            }
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("")
                    .to_string(),
                    if empty_count > 0 {empty_count.to_string()} else {String::new()}
                )
            })
            .collect::<Vec<String>>()
            .join("/"),
            self.gs.player_to_move,
            self.gs.castling_rights,
            if self.gs.en_passant_mask == precomputed::EMPTY {
                "-"
            } else {
                precomputed::SQUARE_NAMES[util::ls1b_from_bitboard(self.gs.en_passant_mask) as usize]
            },
            0,
            0
        )
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!(
            "\n{}\nkey: {:X}\n{}\n",
            (0..8).rev().map(|y| {
                (0..8).map(|x| {
                    match self.piece_list[util::square_from_coord(x, y) as usize] {
                        Some(pt) => pt.to_string(),
                        None => String::from(" "),
                    }
                })
                .fold(String::from("| "), |a, b| a + &b + " | ")
                .trim_end()
                .to_owned()
            })
            .fold(String::from("+---+---+---+---+---+---+---+---+\n"), |a, b| a + &b + "\n+---+---+---+---+---+---+---+---+\n")
            .trim_end(),
            self.key,
            self.get_fen()
        ).as_str())
    }
}