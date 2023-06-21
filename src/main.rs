use std::{
    io::{self, Write},
    sync::mpsc::channel,
    time::Duration,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host,
};

fn main() {
    let host = cpal::default_host();

    let outputspeaker = prompt_speaker(&host);
    let inputspeaker = prompt_speaker(&host);

    print!("\x1B[2J\x1B[1;1H");

    if outputspeaker.name().unwrap() == inputspeaker.name().unwrap() {
        println!("Output and input speakers cannot be the same.");
        return;
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
        return;
    }

    let audiochannel = channel();
    let error_callback = move |err| {
        println!("\x1b[31m{}\x1b[0m", err);
        std::process::exit(1);
    };

    let stream = outputspeaker
        .build_input_stream(
            &outputspeaker_config,
            move |data: &[f32], _: &_| {
                let mut output_data = Vec::new();

                if outputspeaker_config.channels != inputspeaker_config.channels {
                    for chunk in data.chunks(outputspeaker_config.channels as usize) {
                        for i in 0..(inputspeaker_config.channels) {
                            if let Some(&first_value) = chunk.get(i as usize) {
                                output_data.push(first_value);
                            }
                        }
                    }
                } else {
                    output_data.extend_from_slice(data);
                }

                for &sample in &output_data {
                    audiochannel.0.send(sample).unwrap();
                }
            },
            error_callback,
            Some(Duration::from_secs(10)),
        )
        .unwrap();

    stream.play().unwrap();

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

    inputstream.play().unwrap();

    handle_ctrlc();
}

fn handle_ctrlc() {
    let (tx, rx) = channel();

    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    println!("Writing until Ctrl-C is pressed...");
    rx.recv().expect("Could not receive from channel.");
    println!("Got it!");
}

fn prompt_speaker(host: &Host) -> Device {
    let all_speakers = host
        .output_devices()
        .expect("no output device available")
        .collect::<Vec<_>>();

    let default_speaker = host
        .default_output_device()
        .expect("no output device available");

    let default_speaker_index = all_speakers
        .iter()
        .position(|speaker| speaker.name().unwrap() == default_speaker.name().unwrap())
        .unwrap();

    for (speaker_index, speaker) in all_speakers.iter().enumerate() {
        println!("{}: {}", speaker_index, speaker.name().unwrap());
    }

    let input = prompt(&format!(
        "\x1b[36mChoose a speaker\x1b[0m [{}]: ",
        default_speaker_index
    ));

    println!("");

    let speaker_index = input
        .trim()
        .parse::<usize>()
        .unwrap_or(default_speaker_index);

    host.output_devices()
        .unwrap()
        .nth(speaker_index)
        .unwrap_or_else(|| {
            println!("Invalid speaker index, using default speaker");
            default_speaker
        })
}

fn prompt(question: &str) -> String {
    print!("{}", question);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    input.trim().to_string()
}
