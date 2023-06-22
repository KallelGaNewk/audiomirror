use cpal::traits::StreamTrait;

mod loopback;
mod utils;

fn main() {
    let host = cpal::default_host();

    let (outputstream, inputstream) = match loopback::run(host) {
        Some(value) => value,
        None => return,
    };

    outputstream.play().unwrap();
    inputstream.play().unwrap();

    utils::handle_ctrlc();
}
