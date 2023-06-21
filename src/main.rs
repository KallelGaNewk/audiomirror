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
    // get all speakers device
    let host = cpal::default_host();
    let speaker = prompt_speaker(&host);
    let speaker_config = speaker.default_output_config().unwrap().config();

    println!(
        "Using speaker: {:?}, ({} channels, {} Hz)",
        speaker.name().unwrap(),
        speaker_config.channels,
        speaker_config.sample_rate.0
    );

    let mut writer = hound::WavWriter::create(
        "output.wav",
        hound::WavSpec {
            channels: speaker_config.channels as u16,
            sample_rate: speaker_config.sample_rate.0,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .unwrap();

    let audiochannel = channel();

    let stream = speaker
        .build_input_stream(
            &speaker_config,
            move |data: &[f32], _: &_| {
                for &sample in data {
                    writer
                        .write_sample((sample * i16::MAX as f32) as i16)
                        .unwrap();

                    audiochannel.0.send(sample).unwrap();
                }

                writer.flush().unwrap();
            },
            move |err| {
                println!("an error occurred on stream: {}", err);
            },
            Some(Duration::from_secs(10)),
        )
        .unwrap();

    stream.play().unwrap();

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

    // get the default output device
    let default_speaker = host
        .default_output_device()
        .expect("no output device available");

    // get index of default speaker, of the list of all speakers
    let default_speaker_index = all_speakers
        .iter()
        .position(|speaker| speaker.name().unwrap() == default_speaker.name().unwrap())
        .unwrap();

    // print all speakers device
    for (speaker_index, speaker) in all_speakers.iter().enumerate() {
        println!("{}: {}", speaker_index, speaker.name().unwrap());
    }

    println!("");

    // prompt user to choose a speaker
    let input = prompt(&format!(
        "Choose a speaker (default: {}): ",
        default_speaker_index
    ));

    // get the speaker index
    let speaker_index = input
        .trim()
        .parse::<usize>()
        .unwrap_or(default_speaker_index);

    // get the speaker
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
