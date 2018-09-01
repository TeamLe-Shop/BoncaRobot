extern crate aarena;
extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use aarena::Game;

fn main() {
    let mut game = Game::new("Pl1".into(), "Pl2".into());
    let mut editor = Editor::<()>::new();
    loop {
        match editor.readline(&format!("[{}]> ", game.current_player().name)) {
            Ok(line) => {
                let current_player_pid = game.current_player_pid();
                let response = game.interpret(&line, current_player_pid);
                for line in &response.lines {
                    println!("{}", line);
                }
                editor.add_history_entry(line);
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
