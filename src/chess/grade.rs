use super::{Move, Board, PieceType::{*, self}};

pub type Grade = u16;

const EP_GRADE: Grade = mvv_lva(WPawn, WPawn);

const fn mvv_lva(moving: PieceType, capturing: PieceType) -> Grade { // ranges from 2 (king captures pawn) to 39 (pawn captures queen)
    let moving = moving as Grade % 6;
    let capturing = capturing as Grade % 6;
    (capturing << 3) | (0b111 ^ moving)
}

pub fn grade(mv: Move, b: &Board) -> Grade {
    if mv.is_capture() {
        match b.piece_list[mv.get_to() as usize] {
            Some(capture_pt) => {
                let moving_pt = b.piece_list[mv.get_from() as usize].expect("Moving piece should exist");
                mvv_lva(moving_pt, capture_pt) // normal capture -> MVV-LVA
            },
            None => EP_GRADE // en-passant
        }
    } else {
        0
    }
}