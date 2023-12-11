use std::error::Error;

use crate::chess::PieceType;

use super::{Board, super::{ Color::*, PieceType::*, precomputed, Square, util, CastlingFlags}, GameState};

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

impl Board {

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
    
                let pt = PieceType::from_color(match pt.to_ascii_uppercase() {
                    'P' => WPawn,
                    'N' => WKnight,
                    'B' => WBishop,
                    'R' => WRook,
                    'Q' => WQueen,
                    'K' => WKing,
                    p => return Err(format!("Not a valid piece: {p}").into())
                }, color);
    
                b.place_piece(pt, util::square_from_coord(col, 7 - row));
    
                col += 1;
            }
        }

        let mut gs = GameState::empty();
    
        gs.player_to_move = if fen_data.color == 'w' {White} else {Black};
        gs.opponent_color = -gs.player_to_move;
        gs.pt_offset = if gs.player_to_move == White {WPawn} else {BPawn};
    
        for c in fen_data.castling.chars() {
            gs.castling_rights.insert(match c {
                'K' => CastlingFlags::WK,
                'Q' => CastlingFlags::WQ,
                'k' => CastlingFlags::BK,
                'q' => CastlingFlags::BQ,
                '-' => CastlingFlags::empty(),
                c => return Err(format!("Not a valid castling state: {}", c).into())
            });
        }
    
        if let Some(idx) = precomputed::SQUARE_NAMES.iter().position(|&s| s == fen_data.en_passant) {
            gs.en_passant_mask = util::bitboard_from_square(idx as Square);
        }
    
        // TODO: Add parsing for rest of the FEN data
        
        b.update_board_data();
        gs.playing_king_square = util::ls1b_from_bitboard(b.bbs[WKing + gs.pt_offset]) as Square;
    
        b.gs = gs;
        b.key = b.make_key();

        Ok(b)
    }
}
