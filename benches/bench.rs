/*
cargo bench --bench bench
*/
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bot::{self, MoveGenerator, Board, MoveList, Perft, ChessEngine, SearchParams};

fn move_gen(b: &mut Board, mg: &MoveGenerator) {
    mg.generate_legal_moves(b, &mut MoveList::new(), false);
}

fn run_perft(perft: &mut Perft, depth: u8) {
    perft.perft(depth, false, false);
}

fn run_search(engine: &mut ChessEngine, depth: u8) {
    engine.reset_table();
    let mut search_params = SearchParams::new();
    search_params.depth = depth;
    engine.search(search_params, false);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut engine = ChessEngine::new("6k1/1N2q1pp/2pn1p2/p2n4/P2P4/1Q3N2/1P3PPP/6K1 b - - 0 30");

    let board_perft = Board::try_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").expect("Incorrect fen");
    let mut perft = Perft::new(board_perft);

    let mg = MoveGenerator::new();
    let mut board_start = Board::try_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").expect("Incorrect fen");
    let mut board_pawnless = Board::try_from_fen("rnbqkbnr/8/8/8/8/4K3/8/RNBQ1BNR w kq - 0 1").expect("Incorrect fen");
    let mut board_random = Board::try_from_fen("2r3k1/5pb1/p3p1pp/q1Pp3n/br1P4/5P2/P1RQNBPP/4RBK1 w - - 5 24").expect("Incorrect fen");
    let mut board_castling = Board::try_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").expect("Incorrect fen");
    
    
    c.bench_function("tt_init_overhead", |b| b.iter_with_large_drop(|| engine.reset_table()));
    c.bench_function("Search depth 6", |b| b.iter_with_large_drop(|| run_search(black_box(&mut engine), black_box(6))));
    // c.bench_function("Search depth 7", |b| b.iter(|| run_search(black_box(&mut engine), black_box(7))));
    // c.bench_function("Search depth 8", |b| b.iter(|| run_search(black_box(&mut engine), black_box(8))));
    
    c.bench_function("Perft 6 startpos", |b| b.iter(|| run_perft(black_box(&mut perft), black_box(6))));

    c.bench_function("startpos_move_gen", |b| b.iter(|| move_gen(black_box(&mut board_start), black_box(&mg))));
    c.bench_function("pawnless_move_gen", |b| b.iter(|| move_gen(black_box(&mut board_pawnless), black_box(&mg))));
    c.bench_function("random_board_move_gen", |b| b.iter(|| move_gen(black_box(&mut board_random), black_box(&mg))));
    c.bench_function("castling_move_gen", |b| b.iter(|| move_gen(black_box(&mut board_castling), black_box(&mg))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);