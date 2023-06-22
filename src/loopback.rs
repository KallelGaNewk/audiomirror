use std::{sync::mpsc::channel, time::Duration};

use cpal::{traits::DeviceTrait, Stream};

use crate::utils;

pub fn run(host: cpal::Host) -> Option<(cpal::Stream, cpal::Stream)> {
    let anchorspeaker = utils::prompt_speaker(&host);
    let mirrorspeaker = utils::prompt_speaker(&host);

    print!("\x1B[2J\x1B[1;1H");

    if anchorspeaker.name().unwrap() == mirrorspeaker.name().unwrap() {
        println!("Output and input speakers cannot be the same.");
        return None;
    }

    let anchorspeaker_config = anchorspeaker.default_output_config().unwrap().config();
    let mirrorspeaker_config = mirrorspeaker.default_output_config().unwrap().config();

    println!(
        "Anchor speaker: \"\x1b[32m{}\x1b[0m\" ({} channels, {} Hz)",
        anchorspeaker.name().unwrap(),
        anchorspeaker_config.channels,
        anchorspeaker_config.sample_rate.0
    );

    println!(
        "Mirror speaker: \"\x1b[33m{}\x1b[0m\" ({} channels, {} Hz)",
        mirrorspeaker.name().unwrap(),
        mirrorspeaker_config.channels,
        mirrorspeaker_config.sample_rate.0
    );

    if anchorspeaker_config.sample_rate != mirrorspeaker_config.sample_rate {
        println!("\x1b[31mSample rate of output and input speakers must be the same.\x1b[0m");
        println!(
            "\x1b[31mBoth speakers must be {} Hz or {} Hz.\x1b[0m",
            anchorspeaker_config.sample_rate.0, mirrorspeaker_config.sample_rate.0
        );
        return None;
    }

    Some(start_loopback(anchorspeaker, mirrorspeaker))
}

fn start_loopback(anchorspeaker: cpal::Device, mirrorspeaker: cpal::Device) -> (Stream, Stream) {
    let audiochannel = channel();
    let anchorspeaker_config = anchorspeaker.default_output_config().unwrap().config();
    let mirrorspeaker_config = mirrorspeaker.default_output_config().unwrap().config();
    let error_callback = move |err| {
        println!("\x1b[31m{}\x1b[0m", err);
        std::process::exit(1);
    };

    let data_callback = move |data: &[f32], _: &_| {
        let mut output_data = Vec::new();

        if anchorspeaker_config.channels != mirrorspeaker_config.channels {
            for chunk in data.chunks(anchorspeaker_config.channels as usize) {
                for i in 0..(mirrorspeaker_config.channels) {
                    if let Some(&first_value) = chunk.get(i as usize) {
                        output_data.push(first_value);
                    } else {
                        output_data.push(0.0);
                    }
                }
            }
        } else {
            output_data.extend_from_slice(data);
        }

        for &sample in &output_data {
            audiochannel.0.send(sample).unwrap();
        }
    };

    let anchorstream = anchorspeaker
        .build_input_stream(
            &anchorspeaker_config,
            data_callback,
            error_callback,
            Some(Duration::from_secs(10)),
        )
        .unwrap();

    let mirrorstream = mirrorspeaker
        .build_output_stream(
            &mirrorspeaker_config,
            move |data: &mut [f32], _: &_| {
                for sample in data.iter_mut() {
                    *sample = audiochannel.1.recv().unwrap();
                }
            },
            error_callback,
            Some(Duration::from_secs(10)),
        )
        .unwrap();

    (anchorstream, mirrorstream)
}
