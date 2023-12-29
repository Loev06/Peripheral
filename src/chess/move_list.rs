use std::{mem::{self, MaybeUninit}, fmt::{self, Debug}};
use partial_sort::PartialSort;

use super::{Move, MAX_MOVE_COUNT, Board, Grade};

pub struct MoveListEntry {
    mv: Move,
    grade: MaybeUninit<Grade>
}


impl MoveListEntry {
    fn new(mv: Move) -> Self {
        Self {
            mv: mv,
            grade: mem::MaybeUninit::uninit()
        }
    }

    fn set_score(&mut self, score: Grade) {
        self.grade.write(score);
    }
}

impl Debug for MoveListEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MoveListEntry").field("mv", &self.mv).field("score", &self.grade).finish()
    }
}

pub struct MoveList {
    moves: [MaybeUninit<MoveListEntry>; MAX_MOVE_COUNT],
    count: usize
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            // assume_init() on MaybeUninit type is not undefined behaviour
            moves: unsafe { mem::MaybeUninit::uninit().assume_init() },
            count: 0
        }
    }

    pub fn get_count(&self) -> &usize {
        &self.count
    }

    pub fn add_move(&mut self, mv: Move) {
        self.moves[self.count].write(MoveListEntry::new(mv));
        self.count += 1;
    }

    fn grade_moves_with_function<F>(&mut self, grading_function: F, board: &Board)
    where 
        F: Fn(Move, &Board) -> Grade
    {
        // The array is initialized until at least self.count - 1 and self.count < MAX_MOVE_COUNT,
        // so assume_init_mut and get_unchecked_mut are not undefined behaviour
        for i in 0..self.count {
            let mle = unsafe { self.moves.get_unchecked_mut(i).assume_init_mut() };
            mle.set_score(grading_function(mle.mv, board));
        }
    }

    pub fn sort_with_function<F>(mut self, grading_function: F, board: &Board) -> SortingMoveList
    where 
        F: Fn(Move, &Board) -> Grade
    {
        self.grade_moves_with_function(grading_function, board);
        SortingMoveList::new(self)
    }
}

impl Iterator for MoveList {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None;
        }
        self.count -= 1;
        // The array is initialized until at least self.count and self.count < MAX_MOVE_COUNT,
        // so assume_init_ref and get_unchecked are not undefined behaviour
        Some(unsafe { self.moves.get_unchecked(self.count).assume_init_ref().mv })
    }
}

impl fmt::Debug for MoveList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set()
         .entries(&self.moves[..self.count])
         .finish()
    }
}

pub struct SortingMoveList {
    list: MoveList,
    current: usize,
    sort_end: usize,
    sort_size: usize
}

impl SortingMoveList {
    fn new(list: MoveList) -> Self {
        Self {
            list,
            current: 0,
            sort_end: 0,
            sort_size: 4 // TODO: test different values when branching factor is lower.
        }
    }
}

impl Iterator for SortingMoveList {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.list.count {
            return None;
        }

        // Implementation inspired by http://larshagencpp.github.io/blog/2016/04/23/fast-incremental-sort - partial sort
        if self.current == self.sort_end {
            self.sort_end = std::cmp::min(self.sort_end + self.sort_size, self.list.count);

            self.list.moves[self.current..self.list.count].partial_sort(self.sort_end - self.current, |a, b| {
                unsafe {
                    a.assume_init_ref().grade.assume_init().cmp(&(b.assume_init_ref().grade.assume_init())).reverse()
                }
            });

            self.sort_size <<= 1; // TODO: benchmark not increasing sort_size (possibly leads to less redundant partial sorts)
        }

        self.current += 1;

        Some(unsafe { self.list.moves.get_unchecked(self.current - 1).assume_init_ref().mv })
    }
}