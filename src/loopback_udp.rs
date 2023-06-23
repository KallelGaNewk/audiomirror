use std::{io::ErrorKind, net::UdpSocket, process, time::Duration};

use anyhow::Result;
use cpal::{traits::DeviceTrait, Device, Stream};
use serde::{Deserialize, Serialize};

use crate::utils::{self, handle_result};

pub enum StreamType {
    Client,
    Server,
}

#[derive(Serialize, Deserialize)]
struct Data {
    channels: u16,
    sample_rate: u32,
    data: Vec<f32>,
}

pub fn run(host: cpal::Host, ip: String, port: u16, streamtype: StreamType) -> Result<Stream> {
    let localspeaker = utils::prompt_speaker(&host);

    print!("\x1B[2J\x1B[1;1H");

    let localspeaker_config = localspeaker.default_output_config()?.config();

    println!(
        "Local speaker: \"\x1b[32m{}\x1b[0m\" ({} channels, {} Hz)",
        localspeaker.name().unwrap(),
        localspeaker_config.channels,
        localspeaker_config.sample_rate.0
    );

    let stream = match streamtype {
        StreamType::Client => loopback_client(localspeaker, ip, port),
        StreamType::Server => loopback_server(localspeaker, ip, port),
    };

    stream
}

fn loopback_client(localspeaker: Device, ip: String, port: u16) -> Result<Stream> {
    let socket = UdpSocket::bind(format!("{}:{}", ip, port))?;
    socket.set_nonblocking(true)?;

    println!("Server opened in {}", socket.local_addr()?.to_string());

    let localspeaker_config = localspeaker.default_output_config()?.config();
    let data_callback = move |data: &mut [f32], _: &_| {
        let mut buffer = [0u8; 65535];
        let bytes = match socket.recv(&mut buffer[..]) {
            Ok(v) => v,
            Err(err) => {
                if err.kind() == ErrorKind::WouldBlock {
                    return;
                } else {
                    handle_result(Err(err.into()))
                }
            }
        };

        let remotespeaker: Data = match bincode::deserialize(&buffer[..bytes]) {
            Ok(v) => v,
            Err(err) => handle_result(Err(err.into())),
        };

        if remotespeaker.sample_rate != localspeaker_config.sample_rate.0 {
            println!(
                "\x1b[31mRemote sample rate ({}) does not match local sample rate ({}), ignoring.\x1b[0m",
                remotespeaker.sample_rate, localspeaker_config.sample_rate.0
            );
            return;
        }

        let mut output_data = Vec::new();
        let coming_data = remotespeaker.data.as_slice();

        for chunk in coming_data.chunks(remotespeaker.channels as usize) {
            for i in 0..(localspeaker_config.channels) {
                if let Some(&first_value) = chunk.get(i as usize) {
                    output_data.push(first_value);
                } else {
                    output_data.push(0.0);
                }
            }
        }

        for (i, sample) in output_data.iter().enumerate() {
            match data.get_mut(i) {
                Some(v) => *v = *sample,
                None => continue,
            }
        }
    };

    let localstream = localspeaker.build_output_stream(
        &localspeaker_config,
        data_callback,
        move |err| {
            println!("\x1b[31m{}\x1b[0m", err);
            process::exit(1);
        },
        Some(Duration::from_secs(10)),
    )?;

    Ok(localstream)
}

fn loopback_server(localspeaker: Device, ip: String, port: u16) -> Result<Stream> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.set_nonblocking(true)?;

    println!("Connecting to {}:{}", ip, port);
    socket.connect(format!("{}:{}", ip, port))?;

    let localspeaker_config = localspeaker.default_output_config()?.config();
    let data_callback = move |raw_data: &[f32], _: &_| {
        let data = raw_data.to_vec();

        let data = Data {
            channels: localspeaker_config.channels,
            sample_rate: localspeaker_config.sample_rate.0,
            data,
        };

        let data = match bincode::serialize(&data) {
            Ok(v) => v,
            Err(err) => handle_result(Err(err.into())),
        };

        match socket.send(&data) {
            Ok(_) => {}
            Err(err) => match err.kind() {
                ErrorKind::WouldBlock => return,
                ErrorKind::NotConnected => return,
                _ => handle_result(Err(err.into())),
            },
        }
    };

    let localstream = localspeaker.build_input_stream(
        &localspeaker_config,
        data_callback,
        move |err| handle_result(Err(err.into())),
        Some(Duration::from_secs(10)),
    )?;

    Ok(localstream)
}
