use super::{super::{precomputed, Bitboard, Square, util}, MoveGenerator};

// Thanks to GunshipPenguin for these magic numbers
// https://github.com/GunshipPenguin/shallow-blue/blob/c6d7e9615514a86533a9e0ffddfc96e058fc9cfd/src/attacks.h
const ROOK_MAGIC_NUMBERS: [u64; 64] = [0xa8002c000108020u64,0x6c00049b0002001u64,0x100200010090040u64,0x2480041000800801u64,0x280028004000800u64,0x900410008040022u64,0x280020001001080u64,0x2880002041000080u64,0xa000800080400034u64,0x4808020004000u64,0x2290802004801000u64,0x411000d00100020u64,0x402800800040080u64,0xb000401004208u64,0x2409000100040200u64,0x1002100004082u64,0x22878001e24000u64,0x1090810021004010u64,0x801030040200012u64,0x500808008001000u64,0xa08018014000880u64,0x8000808004000200u64,0x201008080010200u64,0x801020000441091u64,0x800080204005u64,0x1040200040100048u64,0x120200402082u64,0xd14880480100080u64,0x12040280080080u64,0x100040080020080u64,0x9020010080800200u64,0x813241200148449u64,0x491604001800080u64,0x100401000402001u64,0x4820010021001040u64,0x400402202000812u64,0x209009005000802u64,0x810800601800400u64,0x4301083214000150u64,0x204026458e001401u64,0x40204000808000u64,0x8001008040010020u64,0x8410820820420010u64,0x1003001000090020u64,0x804040008008080u64,0x12000810020004u64,0x1000100200040208u64,0x430000a044020001u64,0x280009023410300u64,0xe0100040002240u64,0x200100401700u64,0x2244100408008080u64,0x8000400801980u64,0x2000810040200u64,0x8010100228810400u64,0x2000009044210200u64,0x4080008040102101u64,0x40002080411d01u64,0x2005524060000901u64,0x502001008400422u64,0x489a000810200402u64,0x1004400080a13u64,0x4000011008020084u64,0x26002114058042u64];
const BISHOP_MAGIC_NUMBERS: [u64; 64] = [0x89a1121896040240u64,0x2004844802002010u64,0x2068080051921000u64,0x62880a0220200808u64,0x4042004000000u64,0x100822020200011u64,0xc00444222012000au64,0x28808801216001u64,0x400492088408100u64,0x201c401040c0084u64,0x840800910a0010u64,0x82080240060u64,0x2000840504006000u64,0x30010c4108405004u64,0x1008005410080802u64,0x8144042209100900u64,0x208081020014400u64,0x4800201208ca00u64,0xf18140408012008u64,0x1004002802102001u64,0x841000820080811u64,0x40200200a42008u64,0x800054042000u64,0x88010400410c9000u64,0x520040470104290u64,0x1004040051500081u64,0x2002081833080021u64,0x400c00c010142u64,0x941408200c002000u64,0x658810000806011u64,0x188071040440a00u64,0x4800404002011c00u64,0x104442040404200u64,0x511080202091021u64,0x4022401120400u64,0x80c0040400080120u64,0x8040010040820802u64,0x480810700020090u64,0x102008e00040242u64,0x809005202050100u64,0x8002024220104080u64,0x431008804142000u64,0x19001802081400u64,0x200014208040080u64,0x3308082008200100u64,0x41010500040c020u64,0x4012020c04210308u64,0x208220a202004080u64,0x111040120082000u64,0x6803040141280a00u64,0x2101004202410000u64,0x8200000041108022u64,0x21082088000u64,0x2410204010040u64,0x40100400809000u64,0x822088220820214u64,0x40808090012004u64,0x910224040218c9u64,0x402814422015008u64,0x90014004842410u64,0x1000042304105u64,0x10008830412a00u64,0x2520081090008908u64,0x40102000a0a60140u64];

const ROOK_SHIFTS: [usize; 64] = [
    52, 53, 53, 53, 53, 53, 53, 52,
    53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53,
    52, 53, 53, 53, 53, 53, 53, 52
];
const BISHOP_SHIFTS: [usize; 64] = [
    58, 59, 59, 59, 59, 59, 59, 58,
    59, 59, 59, 59, 59, 59, 59, 59,
    59, 59, 57, 57, 57, 57, 59, 59,
    59, 59, 57, 55, 55, 57, 59, 59,
    59, 59, 57, 55, 55, 57, 59, 59,
    59, 59, 57, 57, 57, 57, 59, 59,
    59, 59, 59, 59, 59, 59, 59, 59,
    58, 59, 59, 59, 59, 59, 59, 58
];

const ROOK_MAGICS: [Magic; 64] = precompute_magics_array(ROOK_MAGIC_NUMBERS, ROOK_SHIFTS, precomputed::ROOK_MOVES_NO_BORDER);
const BISHOP_MAGICS: [Magic; 64] = precompute_magics_array(BISHOP_MAGIC_NUMBERS, BISHOP_SHIFTS, precomputed::BISHOP_MOVES_NO_BORDER);

impl MoveGenerator {
    pub fn precompute_lookup_tables(&mut self) {
        self.rook_lookups = precompute_lookups(precomputed::ROOK_DIRS, ROOK_MAGICS);
        self.bishop_lookups = precompute_lookups(precomputed::BISHOP_DIRS, BISHOP_MAGICS);
    }

    pub fn get_rook_attacks(&self, occ: Bitboard, sq: Square) -> Bitboard {
        self.rook_lookups[ROOK_MAGICS[sq as usize].calculate_index(occ)]
    }
    pub fn get_bishop_attacks(&self, occ: Bitboard, sq: Square) -> Bitboard {
        self.bishop_lookups[BISHOP_MAGICS[sq as usize].calculate_index(occ)]
    }
}

#[derive(Clone, Copy)]
struct Magic {
    magic_nr: u64,
    shift: usize,
    mask: Bitboard,
    offset: usize
}

impl Magic {
    const fn empty() -> Self {
        Magic { magic_nr: 0, shift: 0, mask: 0, offset: 0 }
    }

    const fn calculate_index(&self, occ: Bitboard) -> usize {
        (
            ((occ & self.mask).wrapping_mul(self.magic_nr) >> self.shift)
        ) as usize + self.offset
    }
}

const fn precompute_magics_array(magic_nrs: [u64; 64], shifts: [usize; 64], masks: [Bitboard; 64]) -> [Magic; 64] {
    let mut magics = [Magic::empty(); 64];

    let mut offset = 0;
    let mut sq: usize = 0;
    while sq < 64 {
        magics[sq] = Magic {
            magic_nr: magic_nrs[sq],
            shift: shifts[sq],
            mask: masks[sq],
            offset
        };

        offset += 1 << (64 - shifts[sq]);
        sq += 1;
    }

    magics
}

fn precompute_lookups(dirs: [(isize, isize); 4], magics: [Magic; 64]) -> Vec<Bitboard> {
    let table_size = magics[63].offset + (1 << (64 - magics[63].shift));
    let mut lookup = vec![precomputed::EMPTY; table_size];

    let mut sq: usize = 0;
    while sq < 64 {
        let mut blockers: Bitboard = precomputed::EMPTY;
        loop {
            debug_assert!(lookup[magics[sq].calculate_index(blockers)] == precomputed::EMPTY);

            let mut attacking_squares = precomputed::EMPTY;
            let mut dir = 0;
            while dir < 4 {
                let mut x = util::get_square_x(sq as Square) as isize;
                let mut y = util::get_square_y(sq as Square) as isize;

                loop {
                    x += dirs[dir].0;
                    y += dirs[dir].1;

                    debug_assert!(!(dirs[dir].0 == 0 && dirs[dir].1 == 0));

                    if util::is_out_of_bounds(x, y) {
                        break;
                    }

                    let square_mask = util::bitboard_from_square(util::square_from_coord(x as usize, y as usize));

                    attacking_squares |= square_mask;
                    if blockers & square_mask != 0 {
                        break;
                    }
                }
                dir += 1;
            }

            lookup[magics[sq].calculate_index(blockers)] = attacking_squares;
            
            blockers = blockers.wrapping_sub(magics[sq].mask) & magics[sq].mask;
            if blockers == precomputed::EMPTY {
                break;
            }
        }

        sq += 1;
    }

    // Make sure the lookup function is surjective:
    let mut idx = 0;
    while idx < table_size {
        debug_assert!(lookup[idx] != precomputed::EMPTY);
        idx += 1;
    }

    lookup
}
