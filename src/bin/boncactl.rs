extern crate zmq;
extern crate rustyline;

use rustyline::Editor;
use rustyline::error::ReadlineError;

fn main() {
    let mut zmq_ctx = zmq::Context::new();
    let mut sock = zmq_ctx.socket(zmq::SocketType::PUSH).unwrap();
    let command_str = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    sock.connect("ipc:///tmp/boncarobot.sock").unwrap();
    if command_str.is_empty() {
        let mut editor = Editor::new();
        loop {
            match editor.readline("> ") {
                Ok(line) => {
                    sock.send_str(&line, zmq::DONTWAIT).unwrap();
                    editor.add_history_entry(&line);
                }
                Err(e) => {
                    match e {
                        ReadlineError::Eof |
                        ReadlineError::Interrupted => {}
                        _ => panic!("error: {}", e),
                    }
                    return;
                }
            }
        }
    } else {
        sock.send_str(&command_str, zmq::DONTWAIT).unwrap();
    }
}
