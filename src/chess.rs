mod chess_engine;
pub use chess_engine::ChessEngine;

mod board;
pub use board::Board;
pub use board::zobrist;

mod chess_move;
pub use chess_move::Move;

mod grade;
pub use grade::Grade;
pub use grade::grade;

mod castling_flags;
pub use castling_flags::CastlingFlags;

mod piece_type;
pub use piece_type::PieceType;
pub use piece_type::Color;

mod move_gen;
pub use move_gen::MoveGenerator;

mod move_list;
pub use move_list::MoveList;

mod eval;
pub use eval::Eval;

mod perft;
pub use perft::Perft;

mod precomputed;
pub mod util;

pub const MAX_MOVE_COUNT: usize = 218;
pub const MAX_SCORE: Score = Score::MAX;
pub const MIN_SCORE: Score = -MAX_SCORE;
pub const CHECKMATE_SCORE: Score = (MAX_SCORE - 100) / 100 * 100;

pub type Square = u8;
pub type Bitboard = u64;
pub type Score = i16;