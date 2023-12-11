mod board;
pub use board::Board;
pub use board::zobrist;

mod chess_move;
pub use chess_move::Move;

mod castling_flags;
pub use castling_flags::CastlingFlags;

mod piece_type;
pub use piece_type::PieceType;
pub use piece_type::Color;

mod move_gen;
pub use move_gen::MoveGenerator;

mod move_list;
pub use move_list::MoveList;

mod perft;
pub use perft::Perft;

mod precomputed;
pub mod util;

pub const MAX_MOVE_COUNT: usize = 218;

pub type Square = u8;
pub type Bitboard = u64;