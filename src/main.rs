use std::{fs::File, io::Write};

use clap::command;
use cpal::{traits::StreamTrait, Stream};
use loopback_udp::StreamType;
use serde::{Deserialize, Serialize};
use utils::handle_result;

mod loopback;
mod loopback_udp;
mod utils;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    lan: LanConfig,
    local: LocalConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct LanConfig {
    device: String,
    role: String,
    port: u16,
    ip: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct LocalConfig {
    anchor_device: String,
    mirror_device: String,
}

const DEFAULT_CONFIG: &str = r#"Config(
    lan: LanConfig(
        // List devices using `audiomirror -l`
        device: "Speakers",

        // `client` will receive the audio,
        // and `server` will send.
        role: "client",

        // Specify server to connect or bind the server.
        ip: "127.0.0.1",
        port: 9727,
    ),
    local: LocalConfig (
        // List devices using `audiomirror -l`
        anchor_device: "Headphones",
        mirror_device: "Speakers"
    ),
)"#;

fn main() {
    let cmd = clap::Command::new("AudioMirror")
        .bin_name("audiomirror")
        .subcommand_required(true)
        .subcommands([command!("lan"), command!("local")])
        .arg(
            clap::arg!("list")
                .short('l')
                .long("list")
                .action(clap::ArgAction::SetTrue)
                .help("List available devices"),
        );

    let config_raw = match File::open("audiomirror.ron") {
        Ok(value) => value,
        Err(_) => {
            let mut file = File::create("audiomirror.ron").unwrap();
            file.write_all(DEFAULT_CONFIG.as_bytes()).unwrap();
            println!("audiomirror.ron created, please configure it");
            return;
        }
    };
    let config: Config = ron::de::from_reader(config_raw).unwrap();

    let host = cpal::default_host();

    let args = cmd.get_matches();
    let (stream1, stream2): (Stream, Option<Stream>) = match args.subcommand() {
        Some(("lan", _)) => {
            let stream = loopback_udp::run(
                host,
                config.lan.ip,
                config.lan.port,
                match config.lan.role.as_str() {
                    "client" => StreamType::Client,
                    "server" => StreamType::Server,
                    _ => {
                        println!("Invalid role on config file");
                        return;
                    }
                },
            );

            (handle_result(stream), None)
        }
        Some(("local", _)) => {
            let streams = loopback::run(host);

            match streams {
                Some(value) => (value.0, Some(value.1)),
                None => return,
            }
        }
        _ => {
            println!("Invalid command");
            return;
        }
    };

    stream1.play().unwrap();
    match stream2 {
        Some(value) => value.play().unwrap(),
        None => (),
    }

    // let stream = match streamtype.as_str() {
    //     "client" => loopback_udp::run(host, ip, port, StreamType::Client),
    //     "server" => loopback_udp::run(host, ip, port, StreamType::Server),
    //     _ => {
    //         println!("Invalid stream type");
    //         return;
    //     },
    // };

    // match handle_result(stream).play() {
    //     Ok(_) => (),
    //     Err(err) => handle_result(Err(err.into())),
    // };

    // let (anchorstream, mirrorstream) = match loopback::run(host) {
    //     Some(value) => value,
    //     None => return,
    // };

    // anchorstream.play().unwrap();
    // mirrorstream.play().unwrap();

    utils::handle_ctrlc();
}
