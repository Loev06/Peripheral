mod board;
pub use self::board::Board;

mod chess_move;
pub use self::chess_move::Move;

mod piece_type;
pub use self::piece_type::PieceType;
pub use self::piece_type::Color;

mod move_gen;
pub use self::move_gen::MoveGenerator;

mod move_list;
pub use self::move_list::MoveList;

mod precomputed;
pub mod util;

pub const MAX_MOVE_COUNT: usize = 218;

pub type Square = usize;
pub type Bitboard = u64;