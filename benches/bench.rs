/*
cargo bench --bench bench
*/
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bot::{self, MoveGenerator, Board, MoveList};

fn move_gen(mg: &MoveGenerator, b: &Board, moves: &mut MoveList) {
    mg.generate_legal_moves(b, moves);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mg = MoveGenerator::new();
    let mut moves = MoveList::new();
    let board_start = Board::try_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").expect("Incorrect fen");
    let board_pawnless = Board::try_from_fen("rnbqkbnr/8/8/8/8/4K3/8/RNBQ1BNR w HAkq - 0 1").expect("Incorrect fen");
    c.bench_function("startpos_move_gen", |b| b.iter(|| move_gen(black_box(&mg), black_box(&board_start), black_box(&mut moves))));
    c.bench_function("pawnless_move_gen", |b| b.iter(|| move_gen(black_box(&mg), black_box(&board_pawnless), black_box(&mut moves))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);