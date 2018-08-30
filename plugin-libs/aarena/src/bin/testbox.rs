extern crate aarena;
extern crate rustyline;
extern crate split_whitespace_rest;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use aarena::{Game, Pid};
use split_whitespace_rest::SplitWhitespace;
use std::env;

fn main() {
    let mut args = env::args().skip(1);
    let p1name = args.next().expect("p1name");
    let p2name = args.next().expect("p2name");
    let mut game = Game::new(p1name, p2name);
    let mut editor = Editor::<()>::new();
    loop {
        match editor.readline(&format!("[{}]> ", game.current_player().name)) {
            Ok(line) => {
                let mut sw = SplitWhitespace::new(&line);
                let pstring = sw.next().expect("player");
                let p = match pstring.trim() {
                    "p1" => Pid::P1,
                    "p2" => Pid::P2,
                    _ => panic!("Wrong player"),
                };
                let response = game.interpret(sw.rest_as_slice(), p);
                for line in &response.lines {
                    println!("{}", line);
                }
                editor.add_history_entry(&line);
            }
            Err(e) => {
                match e {
                    ReadlineError::Eof | ReadlineError::Interrupted => {}
                    _ => panic!("error: {}", e),
                }
                return;
            }
        }
    }
}
