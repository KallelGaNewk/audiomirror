use cpal::traits::StreamTrait;

mod loopback;
mod utils;

fn main() {
    let host = cpal::default_host();

    let (anchorstream, mirrorstream) = match loopback::run(host) {
        Some(value) => value,
        None => return,
    };

    anchorstream.play().unwrap();
    mirrorstream.play().unwrap();

    utils::handle_ctrlc();
}
