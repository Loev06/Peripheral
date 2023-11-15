
use std::error::Error;

use super::{
    Color, Color::*,
    PieceType, PieceType::*,
    precomputed, Bitboard, Square, util, CastlingFlags,
};
use enum_map::{self, EnumMap};

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

pub struct Board {
    pub bbs: EnumMap<PieceType, Bitboard>,
    pub piece_list: [Option<PieceType>; 64],

    pub player_to_move: Color,
    pub opponent_color: Color,

    pub playing_king_square: Square,
    pub opponent_king_square: Square,

    pub castling_rights: CastlingFlags,
    pub en_passant_mask: Bitboard
}

impl Board {
    pub fn new() -> Self {
        let mut b = Self {
            bbs: Self::init_piece_bitboards(),
            piece_list: [None; 64],
            player_to_move: Color::White,
            opponent_color: Color::Black,
            playing_king_square: 64,
            opponent_king_square: 64,
            castling_rights: CastlingFlags::ALL,
            en_passant_mask: precomputed::EMPTY
        };
        b.piece_list[precomputed::E1 as usize] = Some(King(White));
        b.piece_list[precomputed::E8 as usize] = Some(King(Black));
        b.playing_king_square = util::ls1b_from_bitboard(b.bbs[King(b.player_to_move)]) as Square;
        b.playing_king_square = util::ls1b_from_bitboard(b.bbs[King(b.opponent_color)]) as Square;
        b
    }

    pub fn empty() -> Self {
        Self {
            bbs: enum_map::enum_map! {_ => precomputed::EMPTY},
            piece_list: [None; 64],
            player_to_move: Color::White,
            opponent_color: Color::Black,
            playing_king_square: 64,
            opponent_king_square: 64,
            castling_rights: CastlingFlags::empty(),
            en_passant_mask: precomputed::EMPTY
        }
    }

    pub fn try_from_fen(fen: &str) -> Result<Self, Box<dyn Error>> {
        let fen_data = FENdata::try_parse(fen)?;
        let mut b = Self::empty();
        
        for (row, row_str) in fen_data.rows.iter().enumerate() {
            let mut col: usize = 0;

            for pt in row_str.chars() {
                if pt.is_ascii_digit() {
                    col += pt.to_digit(10).expect("Not a digit") as usize;
                    continue;
                }

                let color = if pt.is_uppercase() {White} else {Black};

                let pt = match pt.to_ascii_uppercase() {
                    'P' => Pawn(color),
                    'N' => Knight(color),
                    'B' => Bishop(color),
                    'R' => Rook(color),
                    'Q' => Queen(color),
                    'K' => King(color),
                    p => return Err(format!("Not a valid piece: {p}").into())
                };

                b.add_piece(pt, util::square_from_coord(col, 7 - row));

                col += 1;
            }
        }

        b.player_to_move = if fen_data.color == 'w' {White} else {Black};
        b.opponent_color = -b.player_to_move;

        for c in fen_data.castling.chars() {
            b.castling_rights.insert(match c {
                'K' => CastlingFlags::WK,
                'Q' => CastlingFlags::WQ,
                'k' => CastlingFlags::BK,
                'q' => CastlingFlags::BQ,
                '-' => CastlingFlags::empty(),
                c => return Err(format!("Not a valid castling state: {}", c).into())
            });
        }

        if let Some(idx) = precomputed::SQUARE_NAMES.iter().position(|&s| s == fen_data.en_passant) {
            b.en_passant_mask = util::bitboard_from_square(idx as Square);
        }

        // TODO: Add parsing for rest of the FEN data
        
        Self::update_bitboards(&mut b.bbs);
        b.playing_king_square = util::ls1b_from_bitboard(b.bbs[King(b.player_to_move)]) as Square;
        b.opponent_king_square = util::ls1b_from_bitboard(b.bbs[King(b.opponent_color)]) as Square;

        Ok(b)
    }

    fn add_piece(&mut self, pt: PieceType, sq: Square) {
        self.piece_list[sq as usize] = Some(pt);
        self.bbs[pt] |= util::bitboard_from_square(sq);
    }

    fn init_piece_bitboards() -> EnumMap<PieceType, u64> {
        let mut em: EnumMap<PieceType, u64> = enum_map::enum_map! {
            King(White) => precomputed::E1BB,
            King(Black) => precomputed::E8BB,
            Knight(White) => precomputed::BORDER,
            _ => precomputed::EMPTY
        };
        Board::update_bitboards(&mut em);
        em
    }

    fn update_bitboards(em: &mut EnumMap<PieceType, Bitboard>) {
        em[Any(White)] = em[Pawn(White)] | em[Knight(White)] | em[Bishop(White)] | em[Rook(White)] | em[Queen(White)] | em[King(White)];
        em[Any(Black)] = em[Pawn(Black)] | em[Knight(Black)] | em[Bishop(Black)] | em[Rook(Black)] | em[Queen(Black)] | em[King(Black)];
        em[Any(Neutral)] = em[Any(Black)] | em[Any(White)];

        em[HVslider(White)] = em[Rook(White)] | em[Queen(White)];
        em[HVslider(Black)] = em[Rook(Black)] | em[Queen(Black)];
        em[Dslider(White)] = em[Bishop(White)] | em[Queen(White)];
        em[Dslider(Black)] = em[Bishop(Black)] | em[Queen(Black)];
    }
}