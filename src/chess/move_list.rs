use std::{mem, fmt};

use super::{Move, MAX_MOVE_COUNT};

pub struct MoveList {
    pub moves: [Move; MAX_MOVE_COUNT],
    pub count: usize
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: unsafe {
                let block: mem::MaybeUninit<[Move; MAX_MOVE_COUNT]> = mem::MaybeUninit::uninit();
                block.assume_init()
            },
            count: 0
        }
    }

    pub fn add_move(&mut self, mv: Move) {
        self.moves[self.count] = mv;
        self.count += 1;
    }

    pub fn reset_count(&mut self) {
        self.count = 0;
    }
}

impl fmt::Debug for MoveList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set()
         .entries(&self.moves[..self.count])
         .finish()
    }
}