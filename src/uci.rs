use chess_engine::{ChessEngine, Board, Perft, Eval, SearchParams};
use std::{io, str::SplitAsciiWhitespace};

const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

const DEFAULT_TABLE_SIZE: usize = 16;
const MIN_TABLE_SIZE: usize = 1;
const MAX_TABLE_SIZE: usize = 1024;

pub struct Uci {
    engine: ChessEngine,
    ready: bool,
    table_size: usize
}

impl Uci {
    pub fn new() -> Self {
        Self {
            engine: ChessEngine::new(START_FEN, DEFAULT_TABLE_SIZE),
            ready: true,
            table_size: DEFAULT_TABLE_SIZE
        }
    }

    pub fn get_header() -> String {
        format!("{} v{} - {} ({})", chess_engine::NAME, chess_engine::VERSION, chess_engine::AUTHOR, chess_engine::DATE)
    }

    pub fn run(&mut self) {
        println!("{}", Self::get_header());

        loop {
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Could not read line.");

            let mut args: SplitAsciiWhitespace = input.split_ascii_whitespace();
            
            if let Some(command) = args.next() {
                match command {
                    "help"       => self.help(),
                    "uci"        => self.uci(),
                    "isready"    => self.isready(),
                    "setoption"  => self.setoption(&mut args),
                    "ucinewgame" => self.ucinewgame(),
                    "position"   => self.position(&mut args),
                    "go"         => self.go(&mut args),
                    "d"          => self.d(),
                    "eval"       => self.eval(),
                    "run"        => self.run_bot(),
                    "make"       => self.make(&mut args),
                    "undo"       => self.undo(),
                    "probe"      => self.probe(),
                    "gen"        => self.gen(),
                    "quit"       => break,
                    other => println!("Unknown command: '{}'. Type 'help' for a list of commands.", other)
                }
            }
        }
    }

    fn help(&self) {
        println!("{}
List of known commands:
- help       Show this message
- uci        Switch to uci mode
- isready    Check whether engine is ready
- setoption  Change an engine option
- ucinewgame Start a new game
- position   Set a position
- go         Start thinking
- d          Print current board
- eval       Static eval of position
- run        Run main function of the bot
- make       Make move
- undo       Undo last move made
- probe      Probe current position in the transposition table
- gen        Get the TT generation of the last search
- quit       Quit.", 
            Self::get_header()
        );
    }

    fn uci(&self) {
        println!("id name {} v{}
id author {}

option name Hash type spin default {} min {} max {}
uciok",
            chess_engine::NAME, chess_engine::VERSION,
            chess_engine::AUTHOR,

            DEFAULT_TABLE_SIZE, MIN_TABLE_SIZE, MAX_TABLE_SIZE
        );
    }

    fn isready(&self) {
        while !self.ready {}
        println!("readyok");
    }

    fn setoption(&mut self, args: &mut SplitAsciiWhitespace) {
        self.ready = false;

        args.next(); // "name"
        let name = args.next();
        args.next(); // "value"
        let Some(value) = args.next() else {
            println!("No value given");
            return;
        };
        let Ok(value) = value.parse::<usize>() else {
            println!("Invalid value");
            return;
        };

        if let Some(name) = name {
            match name.to_ascii_lowercase().as_str() {
                "hash" => {
                        if value < MIN_TABLE_SIZE || value > MAX_TABLE_SIZE {
                            println!("Hash size not within required bounds");
                            return;
                        }
                        self.table_size = value;
                        println!("Set hash size to {}mb", self.table_size);
                    },
                _ => ()
            }
        }

        self.ready = true;
    }

    fn ucinewgame(&mut self) {
        self.ready = false;
        self.engine.reset_table(self.table_size);
        self.ready = true;
    }

    fn position(&mut self, args: &mut SplitAsciiWhitespace) {
        self.ready = false;

        let start_fen = if let Some(pos_type) = args.next() {
            match pos_type {
                "startpos" => {
                    String::from(START_FEN)
                },
                "fen" => args.clone().take_while(|x| !(*x).eq("moves"))
                    .fold(String::new(), |acc, x| format!("{acc}{x} ")),
                _ => return
            }
        } else {
            println!("No position given!");
            return;
        };

        match self.engine.set_board(&start_fen) {
            Ok(_) => (),
            Err(err) => {
                println!("Invalid fen: {}", err);
                return;
            }
        };

        let moves = args.skip_while(|x| !(*x).eq("moves")).skip(1); // skip "moves" string

        for mv_str in moves {
            match self.engine.make_uci_move(mv_str) {
                Ok(mv) => mv,
                Err(e) => {
                    println!("Error parsing move {}: {}", mv_str, e);
                    return;
                }
            };
        }

        self.ready = true;
    }

    fn go(&mut self, args: &mut SplitAsciiWhitespace) {
        let mut search_params = SearchParams::new();
        while let Some(a) = args.next() {
            match a {
                "perft" => {
                    let board = Board::try_from_fen(self.engine.get_board().get_fen().as_str()).expect("Engine returned an incorrect fen");
                    let depth = args.next().expect("no depth given").parse::<u8>().expect("depth not a byte");
                    Perft::new(board).verb_perft(depth, false, false);
                    return;
                },
                "movetime" => search_params.move_time = Some(args.next().expect("no movetime given").parse::<u128>().expect("movetime not an integer")),
                "wtime" => search_params.wtime = args.next().expect("no wtime given").parse::<u128>().expect("wtime not an integer"),
                "btime" => search_params.btime = args.next().expect("no btime given").parse::<u128>().expect("btime not an integer"),
                "winc"  => search_params.winc  = args.next().expect("no winc given").parse::<u128>().expect("winc not an integer"),
                "binc"  => search_params.binc  = args.next().expect("no binc given").parse::<u128>().expect("binc not an integer"),
                "depth" => search_params.depth = args.next().expect("no depth given").parse::<u8>().expect("depth not a byte"),
                _ => ()
            }
        }
        self.engine.search(search_params, true);
    }

    fn d(&self) {
        println!("{}", self.engine.get_board().to_string());
    }

    fn eval(&self) {
        let board = Board::try_from_fen(self.engine.get_board().get_fen().as_str()).expect("Engine returned an incorrect fen");
        println!("{}", Eval::eval(&board));
    }

    fn run_bot(&self) {
        chess_engine::run_bot().unwrap_or_else(|e| println!("Bot returned an error: {}", e));
    }

    fn make(&mut self, args: &mut SplitAsciiWhitespace) {
        let Some(mv) = args.next() else {
            println!("No move given");
            return;
        };
        match self.engine.make_uci_move(mv) {
            Ok(mv) => mv,
            Err(e) => {
                println!("Error parsing move {}: {}", mv, e);
                return;
            }
        };
    }

    fn undo(&mut self) {
        self.engine.undo_move().unwrap_or_else(|err| println!("{err}"));
    }

    fn probe(&self) {
        println!("{}", self.engine.probe_tt());
    }

    fn gen(&self) {
        println!("{}", self.engine.get_gen())
    }
}
