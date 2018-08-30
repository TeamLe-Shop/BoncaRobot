extern crate aarena;
extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use aarena::Game;
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
                let turn = game.turn;
                let response = game.interpret(&line, turn);
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
