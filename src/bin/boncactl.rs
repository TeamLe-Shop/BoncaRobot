extern crate zmq;

fn main() {
    let mut zmq_ctx = zmq::Context::new();
    let mut sock = zmq_ctx.socket(zmq::SocketType::PUSH).unwrap();
    sock.connect("ipc:///tmp/boncarobot.sock").unwrap();
    sock.send_str("quit", zmq::DONTWAIT).unwrap();
}
