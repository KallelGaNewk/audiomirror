use std::{
    io::{self, Write},
    sync::mpsc::channel,
};

use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, Host,
};

pub fn handle_ctrlc() {
    let (tx, rx) = channel();

    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    println!("Writing until Ctrl-C is pressed...");
    rx.recv().expect("Could not receive from channel.");
    println!("Got it!");
}

pub fn prompt_speaker(host: &Host) -> Device {
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

pub fn prompt(question: &str) -> String {
    print!("{}", question);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    input.trim().to_string()
}
