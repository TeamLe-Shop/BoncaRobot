extern crate rustyline;
extern crate scaproust;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use scaproust::proto::pair::Pair;
use scaproust::{Ipc, SessionBuilder};

fn main() {
    let command_str = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let mut session = SessionBuilder::new().with("ipc", Ipc).build().unwrap();
    let mut socket = session.create_socket::<Pair>().unwrap();
    let tmpdir = std::env::temp_dir();
    socket
        .connect(&format!(
            "ipc://{}/boncarobot.sock",
            tmpdir.to_str().unwrap()
        )).unwrap();
    if command_str.is_empty() {
        let mut editor = Editor::<()>::new();
        loop {
            match editor.readline("> ") {
                Ok(line) => {
                    socket.send(line.clone().into_bytes()).unwrap();
                    let reply = String::from_utf8(socket.recv().unwrap()).unwrap();
                    println!("{}", reply);
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
    } else {
        socket.send(command_str.into_bytes()).unwrap();
        let reply = String::from_utf8(socket.recv().unwrap()).unwrap();
        println!("{}", reply);
    }
}
