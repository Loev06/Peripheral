/*
cargo bench --bench bench
*/
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bot::{self, MoveGenerator, Board, MoveList, Perft};

fn move_gen(b: &mut Board, mg: &MoveGenerator) {
    mg.generate_legal_moves(b, &mut MoveList::new());
}

fn run_perft(perft: &mut Perft, depth: u8) {
    perft.perft(depth, false, false);
}

fn criterion_benchmark(c: &mut Criterion) {

    let board_perft = Board::try_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").expect("Incorrect fen");
    let mut board_start = Board::try_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").expect("Incorrect fen");
    let mut board_pawnless = Board::try_from_fen("rnbqkbnr/8/8/8/8/4K3/8/RNBQ1BNR w kq - 0 1").expect("Incorrect fen");
    let mut board_random = Board::try_from_fen("2r3k1/5pb1/p3p1pp/q1Pp3n/br1P4/5P2/P1RQNBPP/4RBK1 w - - 5 24").expect("Incorrect fen");
    let mut board_castling = Board::try_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").expect("Incorrect fen");
    
    let mg = MoveGenerator::new();
    let mut perft = Perft::new(board_perft);
    
    c.bench_function("Perft 6 startpos", |b| b.iter(|| run_perft(black_box(&mut perft), black_box(6))));

    c.bench_function("startpos_move_gen", |b| b.iter(|| move_gen(black_box(&mut board_start), black_box(&mg))));
    c.bench_function("pawnless_move_gen", |b| b.iter(|| move_gen(black_box(&mut board_pawnless), black_box(&mg))));
    c.bench_function("random_board_move_gen", |b| b.iter(|| move_gen(black_box(&mut board_random), black_box(&mg))));
    c.bench_function("castling_move_gen", |b| b.iter(|| move_gen(black_box(&mut board_castling), black_box(&mg))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);