mod file_audio;
mod live_audio;
mod out_audio;

use std::sync::mpsc::{ channel };
use mel_spec::{mel, prelude::*};

//use burn::prelude::*;
//use burn::{ backend::NdArray };

fn main() {

    //let device = Default::default();
    //type B = burn::backend::NdArray;

    let (tx, rx) = channel::<f32>();

    let config = file_audio::stream_file_audio(tx, std::fs::File::open("C:/Users/Jason/Downloads/clips/common_voice_en_43427406.mp3").expect("Failed to open file"));

    //let (_input_stream, config) = live_audio::live_audio(tx, 5);

    let glyphs = vec![
        "⠀⢀⢠⢰⢸".chars().collect::<Vec<char>>(),
        "⡀⣀⣠⣰⣸".chars().collect(),
        "⡄⣄⣤⣴⣼".chars().collect(),
        "⡆⣆⣦⣶⣾".chars().collect(),
        "⡇⣇⣧⣷⣿".chars().collect()]; 

    let mut spectrogram = Spectrogram::new(4096 * 4, 4096);
    let mut mel = MelSpectrogram::new(16384, 41000., 64);

    loop {
        let buffer = rx.iter().take(4096).collect::<Vec<f32>>();

        if buffer.len() < 4096 { break; }

        if let Some (fft_frame) = spectrogram.add(&buffer) {
            let mel_spec = mel.add(&fft_frame);

            let pairs = 
                mel_spec
                .exact_chunks((2, 1));

            print!("|");

            for pair in pairs {
                // + 1.1) / 3.3) is to map the decibels(?) to the approximate range I've seen so far
                let first = (5. * ((pair[[0, 0]] + 1.1) / 3.3)).clamp(0.,4.).floor() as usize;
                let second = (5. * ((pair[[1, 0]] + 1.1) / 3.3)).clamp(0.,4.).floor() as usize;
                
                print!("{}", glyphs[first][second]);
            }
            
            println!("|");
        }
    }
    
    //let (exhausted, _output_stream) = out_audio::out_audio(rx, config);
    //exhausted.recv().unwrap();

    println!("Done.");
}