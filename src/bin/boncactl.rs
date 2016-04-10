extern crate zmq;
use std::io::prelude::*;

fn main() {
    let mut zmq_ctx = zmq::Context::new();
    let mut sock = zmq_ctx.socket(zmq::SocketType::PUSH).unwrap();
    let command_str = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    sock.connect("ipc:///tmp/boncarobot.sock").unwrap();
    if command_str.is_empty() {
        let stdin = std::io::stdin();
        let mut lines = stdin.lock().lines();
        loop {
            print!("> ");
            std::io::stdout().flush().unwrap();
            if let Some(l) = lines.next() {
                sock.send_str(&l.unwrap(), zmq::DONTWAIT).unwrap();
            } else {
                println!("Goodbye!");
                return;
            }
        }
    } else {
        sock.send_str(&command_str, zmq::DONTWAIT).unwrap();
    }
}
