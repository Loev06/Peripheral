use bot::{ChessEngine, Move, Board, Perft, Eval, SearchParams};
use std::io;

const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub struct Uci {
    engine: ChessEngine,
    ready: bool
}

impl Uci {
    pub fn new() -> Self {
        Self {
            engine: ChessEngine::new(START_FEN),
            ready: true
        }
    }

    pub fn get_header() -> String {
        format!("{} v{} - {} ({})", bot::NAME, bot::VERSION, bot::AUTHOR, bot::DATE)
    }

    pub fn run(&mut self) {
        println!("{}", Self::get_header());

        loop {
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Could not read line.");

            let args: Vec<&str> = input.split_ascii_whitespace().collect();
            
            match args[0] {
                "help"       => self.help(),
                "uci"        => self.uci(),
                "isready"    => self.isready(),
                "setoption"  => self.setoption(args),
                "ucinewgame" => self.ucinewgame(),
                "position"   => self.position(args),
                "go"         => self.go(args),
                "d"          => self.d(),
                "eval"       => self.eval(),
                "run"        => self.run_bot(),
                "quit"       => break,
                other => println!("Unknown command: '{}'. Type 'help' for a list of commands.", other)
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
- quit       Quit.", 
            Self::get_header()
        );
    }

    fn uci(&self) {
        println!("id name {} v{}
id author {}

uciok",
            bot::NAME, bot::VERSION,
            bot::AUTHOR
        );
    }

    fn isready(&self) {
        while !self.ready {}
        println!("readyok");
    }

    fn setoption(&mut self, _args: Vec<&str>) {
        self.ready = false;

        self.ready = true;
    }

    fn ucinewgame(&mut self) {
        self.ready = false;
        self.engine.reset_table();
        self.ready = true;
    }

    fn position(&mut self, args: Vec<&str>) {
        self.ready = false;

        let moves_index = args.clone().into_iter().position(|s| s == "moves");
        
        let start_fen = if let Some(pos_type) = args.get(1) {
            match *pos_type {
                "startpos" => {
                    String::from(START_FEN)
                },
                "fen" => args[2..moves_index.unwrap_or(args.capacity())].join(" "),
                _ => return
            }
        } else {
            println!("No position given!");
            return;
        };

        let mut board = match Board::try_from_fen(&start_fen) {
            Ok(b) => b,
            Err(_) => {
                println!("Invalid position!");
                return;
            }
        };

        if let Some(moves_index) = moves_index {
            for mv_str in &args[moves_index + 1..] {
                let mv = match Move::try_from_str(mv_str, &board) {
                    Ok(mv) => mv,
                    Err(e) => {
                        println!("Error parsing move {}: {}", mv_str, e);
                        return;
                    }
                };
                board.make_move(&mv);
            }
        }

        self.engine.set_board(board.get_fen().as_str());
        self.ready = true;
    }

    fn go(&mut self, args: Vec<&str>) {
        let mut search_params = SearchParams::new();
        let mut args = args.iter();
        args.next();
        while let Some(a) = args.next() {
            match *a {
                "perft" => {
                    let board = Board::try_from_fen(self.engine.get_board_fen().as_str()).expect("Engine returned an incorrect fen");
                    let depth = args.next().expect("no depth given").parse::<u8>().expect("depth not a byte");
                    Perft::new(board).verb_perft(depth, true, false);
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
        println!("{}", self.engine.get_board_string());
    }

    fn eval(&self) {
        let board = Board::try_from_fen(self.engine.get_board_fen().as_str()).expect("Engine returned an incorrect fen");
        println!("{}", Eval::eval(&board));
    }

    fn run_bot(&self) {
        bot::run_bot().unwrap_or_else(|e| println!("Bot returned an error: {}", e));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        Uci::new().run()
    }
}
