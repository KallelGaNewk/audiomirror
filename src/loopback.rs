use std::{sync::mpsc::channel, time::Duration};

use cpal::{traits::DeviceTrait, Stream};

use crate::utils;

pub fn run(host: cpal::Host) -> Option<(cpal::Stream, cpal::Stream)> {
    let outputspeaker = utils::prompt_speaker(&host);
    let inputspeaker = utils::prompt_speaker(&host);

    print!("\x1B[2J\x1B[1;1H");

    if outputspeaker.name().unwrap() == inputspeaker.name().unwrap() {
        println!("Output and input speakers cannot be the same.");
        return None;
    }

    let outputspeaker_config = outputspeaker.default_output_config().unwrap().config();
    let inputspeaker_config = inputspeaker.default_output_config().unwrap().config();

    println!(
        "Output speaker: \"\x1b[32m{}\x1b[0m\" ({} channels, {} Hz)",
        outputspeaker.name().unwrap(),
        outputspeaker_config.channels,
        outputspeaker_config.sample_rate.0
    );

    println!(
        "Input speaker: \"\x1b[33m{}\x1b[0m\" ({} channels, {} Hz)",
        inputspeaker.name().unwrap(),
        inputspeaker_config.channels,
        inputspeaker_config.sample_rate.0
    );

    if outputspeaker_config.sample_rate != inputspeaker_config.sample_rate {
        println!("\x1b[31mSample rate of output and input speakers must be the same.\x1b[0m");
        println!(
            "\x1b[31mBoth speakers must be {} Hz or {} Hz.\x1b[0m",
            outputspeaker_config.sample_rate.0, inputspeaker_config.sample_rate.0
        );
        return None;
    }

    Some(start_loopback(outputspeaker, inputspeaker))
}

fn start_loopback(outputspeaker: cpal::Device, inputspeaker: cpal::Device) -> (Stream, Stream) {
    let audiochannel = channel();
    let outputspeaker_config = outputspeaker.default_output_config().unwrap().config();
    let inputspeaker_config = inputspeaker.default_output_config().unwrap().config();
    let error_callback = move |err| {
        println!("\x1b[31m{}\x1b[0m", err);
        std::process::exit(1);
    };

    let data_callback = move |data: &[f32], _: &_| {
        let mut output_data = Vec::new();

        if outputspeaker_config.channels != inputspeaker_config.channels {
            for chunk in data.chunks(outputspeaker_config.channels as usize) {
                for i in 0..(inputspeaker_config.channels) {
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

    let outputstream = outputspeaker
        .build_input_stream(
            &outputspeaker_config,
            data_callback,
            error_callback,
            Some(Duration::from_secs(10)),
        )
        .unwrap();

    let inputstream = inputspeaker
        .build_output_stream(
            &inputspeaker_config,
            move |data: &mut [f32], _: &_| {
                for sample in data.iter_mut() {
                    *sample = audiochannel.1.recv().unwrap();
                }
            },
            error_callback,
            Some(Duration::from_secs(10)),
        )
        .unwrap();

    (outputstream, inputstream)
}
