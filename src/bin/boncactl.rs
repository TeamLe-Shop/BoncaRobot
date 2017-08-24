extern crate rustyline;
extern crate zmq;

use rustyline::Editor;
use rustyline::error::ReadlineError;

fn main() {
    let zmq_ctx = zmq::Context::new();
    let sock = zmq_ctx.socket(zmq::SocketType::REQ).unwrap();
    let command_str = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    sock.connect("ipc:///tmp/boncarobot.sock").unwrap();
    if command_str.is_empty() {
        let mut editor = Editor::<()>::new();
        loop {
            match editor.readline("> ") {
                Ok(line) => {
                    sock.send(line.as_bytes(), 0).unwrap();
                    let reply = sock.recv_string(0).unwrap().unwrap();
                    println!("{}", reply);
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
    } else {
        sock.send(command_str.as_bytes(), 0).unwrap();
        let reply = sock.recv_string(0).unwrap().unwrap();
        println!("{}", reply);
    }
}
