use cpal::traits::StreamTrait;
use loopback_udp::StreamType;

mod loopback_udp;
mod loopback;
mod utils;

fn main() {
    let host = cpal::default_host();

    let streamtype = utils::prompt("Client or server?").to_lowercase();
    let ip = utils::prompt("IP address");
    let port = utils::prompt("Port").parse::<u16>().unwrap();
    // let port: u16 = 12345;

    let stream = match streamtype.as_str() {
        "client" => loopback_udp::run(host, ip, port, StreamType::Client),
        "server" => loopback_udp::run(host, ip, port, StreamType::Server),
        _ => {
            println!("Invalid stream type");
            return;
        },
    };

    stream.play().unwrap();

    // let (anchorstream, mirrorstream) = match loopback::run(host) {
    //     Some(value) => value,
    //     None => return,
    // };

    // anchorstream.play().unwrap();
    // mirrorstream.play().unwrap();

    utils::handle_ctrlc();
}
