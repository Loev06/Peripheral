/*
cargo bench --bench bench
*/
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bot::{self, MoveGenerator, Board, MoveList};

fn move_gen(mg: &MoveGenerator, moves: &mut MoveList) {
    mg.generate_legal_moves(moves);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut moves = MoveList::new();

    let board_start = Board::try_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").expect("Incorrect fen");
    let board_pawnless = Board::try_from_fen("rnbqkbnr/8/8/8/8/4K3/8/RNBQ1BNR w kq - 0 1").expect("Incorrect fen");
    let board_random = Board::try_from_fen("2r3k1/5pb1/p3p1pp/q1Pp3n/br1P4/5P2/P1RQNBPP/4RBK1 w - - 5 24").expect("Incorrect fen");
    let board_castling = Board::try_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").expect("Incorrect fen");

    let mg_start = MoveGenerator::new(&board_start);
    let mg_pawnless = MoveGenerator::new(&board_pawnless);
    let mg_random = MoveGenerator::new(&board_random);
    let mg_castling = MoveGenerator::new(&board_castling);
    
    c.bench_function("startpos_move_gen", |b| b.iter(|| move_gen(black_box(&mg_start), black_box(&mut moves))));
    c.bench_function("pawnless_move_gen", |b| b.iter(|| move_gen(black_box(&mg_pawnless), black_box(&mut moves))));
    c.bench_function("random_board_move_gen", |b| b.iter(|| move_gen(black_box(&mg_random), black_box(&mut moves))));
    c.bench_function("castling_move_gen", |b| b.iter(|| move_gen(black_box(&mg_castling), black_box(&mut moves))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);