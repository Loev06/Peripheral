use std::mem;

use super::{
    GameState,
    super::PieceType
};

#[derive(Clone, Copy)]
pub struct GSHistoryEntry {
    pub gs: GameState,
    pub captured_piece: Option<PieceType>
}

pub struct GSHistory {
    history: [GSHistoryEntry; Self::MOVE_HISTORY_CAPACITY],
    count: usize
}

impl GSHistory {
    const MOVE_HISTORY_CAPACITY: usize = 512;

    pub fn new() -> Self {
        Self {
            history: unsafe {
                let block: mem::MaybeUninit<[GSHistoryEntry; Self::MOVE_HISTORY_CAPACITY]> = mem::MaybeUninit::uninit();
                block.assume_init()
            },
            count: 0
        }
    }

    pub fn push(&mut self, gs: GSHistoryEntry) {
        debug_assert!(self.count < Self::MOVE_HISTORY_CAPACITY);
        self.history[self.count] = gs;
        self.count += 1;
    }

    pub fn pop(&mut self) -> GSHistoryEntry {
        debug_assert!(self.count > 0);
        self.count -= 1;
        self.history[self.count]
    }
}

struct KeyHistoryEntry(u64);

impl KeyHistoryEntry {
    const COUNT_BITS: u8 = 7; // max 128 revertable moves (100 is the maximum due to 50 move rule)
    const COUNT_MASK: u64 = (1 << Self::COUNT_BITS) - 1;
    const KEY_MASK: u64 = !Self::COUNT_MASK;

    #[inline(always)]
    fn new(key: u64, count: u64) -> Self {
        Self(key & Self::KEY_MASK | count & Self::COUNT_MASK)
    }

    #[inline(always)]
    fn equal_key(&self, key: u64) -> bool {
        (self.0 ^ key) & Self::KEY_MASK == 0
    }

    #[inline(always)]
    fn get_count(&self) -> usize {
        (self.0 & Self::COUNT_MASK) as usize
    }
}

pub struct KeyHistory(Vec<KeyHistoryEntry>);

impl KeyHistory {
    pub fn new(start_pos_key: u64) -> Self {
        Self(vec![KeyHistoryEntry::new(start_pos_key, 1)])
    }

    pub fn contains_2fold(&self) -> bool {
        let last = self.0.last().expect("History should not be empty");
        let iters = last.get_count();
        let key = last.0;
        self.0
            .iter()
            .rev()
            .take(iters)
            .skip(4) // repetition is not possible within 4 plies
            .step_by(2)
            .any(|x| x.equal_key(key))
    }

    pub fn contains_3fold(&self) -> bool {
        let last = self.0.last().expect("History should not be empty");
        let iters = last.get_count();
        let key = last.0;
        self.0
            .iter()
            .rev()
            .take(iters)
            .skip(4) // repetition is not possible within 4 plies
            .step_by(2)
            .filter(|x| x.equal_key(key))
            .count() >= 2
    }

    pub fn push_key(&mut self, key: u64, last_move_revertable: bool) {
        self.0.push(KeyHistoryEntry::new(key, if last_move_revertable {
            self.0.last().expect("History should not be empty").0 + 1 // add one to last entry's count
        } else {
            1 // reset count
        }))
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }

    pub fn print_history(&self) {
        for entry in &self.0 {
            println!(
                "Key: {:X}\tCount: {}",
                entry.0 & KeyHistoryEntry::KEY_MASK,
                entry.0 & KeyHistoryEntry::COUNT_MASK,
            )
        }
    }
}