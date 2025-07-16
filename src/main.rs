use std::sync::mpsc::{ channel, sync_channel };
use mel_spec::{mel, prelude::*};

use burn::prelude::*;
//use burn::{ backend::NdArray };

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

// Shared buffer for audio data
const DURATION_SECONDS: u32 = 10;

fn main() {
    //let device = Default::default();
    //type B = burn::backend::NdArray;

    // Set up CPAL host and default input/output devices
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("No input device available");
    let output_device = host.default_output_device().expect("No output device available");

    // Use the default input/output configs
    let config = input_device.default_input_config().unwrap();
    let sample_rate = config.sample_rate();

    println!("Recording from {:?} @ {:?} sample_rate", input_device.name().unwrap(), sample_rate);
    println!("Outputting to {:?}", output_device.name().unwrap());

    let sample_format = config.sample_format();

    if sample_format != cpal::SampleFormat::F32 {
        panic!("Only f32 sample format is supported in this example.");
    }

    let (tx, rx) = channel::<f32>();
    let mut tx = Some(tx);
    let mut samples_remaining: usize = (sample_rate.0 * DURATION_SECONDS) as usize;

    println!("Recording for {:?} seconds...", DURATION_SECONDS);

    let input_stream = input_device.build_input_stream(
        &config.clone().into(),
        move |data: &[f32], _| {
            let bytes_read = data.len();

            if bytes_read <= samples_remaining {
                samples_remaining -= bytes_read;

                for sample in data {
                    match tx {
                        Some(ref mut tx) => {
                            tx.send(*sample).unwrap();
                        },
                        None => {}
                    }
                    
                }
            } else {
                tx = None;
            }
        },
        move |err| {
            println!("Input stream error: {:?}", err);
        },
        None,
    ).unwrap();
    
    input_stream.play().unwrap();

    let glyphs = vec![
        "⠀⢀⢠⢰⢸".chars().collect::<Vec<char>>(),
        "⡀⣀⣠⣰⣸".chars().collect(),
        "⡄⣄⣤⣴⣼".chars().collect(),
        "⡆⣆⣦⣶⣾".chars().collect(),
        "⡇⣇⣧⣷⣿".chars().collect()]; 

    let mut spectrogram = Spectrogram::new(4096 * 4, 4096);
    let mut mel = MelSpectrogram::new(16384, 41000., 64);

    let mut max: f64 = 0.0;
    let mut min: f64 = 0.0;

    loop {
        let buffer = rx.iter().take(4096).collect::<Vec<f32>>();

        if buffer.len() < 4096 { break; }

        if let Some (fft_frame) = spectrogram.add(&buffer) {
            let mel_spec = mel.add(&fft_frame);

            let pairs = 
                mel_spec
                .exact_chunks((2, 1));


            for pair in pairs {
                // + 1.1) / 3.3) is to map the decibels(?) to the approximate range I've seen so far
                let first = (5. * ((pair[[0, 0]] + 1.1) / 3.3)).clamp(0.,4.).floor() as usize;
                let second = (5. * ((pair[[1, 0]] + 1.1) / 3.3)).clamp(0.,4.).floor() as usize;

                max = max.max(pair[[0, 0]]);
                max = max.max(pair[[1, 0]]);
                
                min = min.min(pair[[0, 0]]);
                min = min.min(pair[[1, 0]]);
                
                print!("{}", glyphs[first][second]);
            }
            
            println!();
        }
    }
                
    print!("{:?}..{:?}", min, max);
    
    drop(input_stream);

    // println!("Playback...");

    // // Build output stream
    // let output_stream = output_device.build_output_stream(
    //     &config.clone().into(),
    //     move |data: &mut [f32], _| {
    //         for sample in data {
    //             *sample = match rx. {
    //                 Some(s) => s,
    //                 None => { 0.0 }
    //             };
    //         }
    //     },
    //     move |err| {
    //         eprintln!("Output stream error: {:?}", err);
    //     },
    //     None,
    // ).unwrap();

    // // Playback for 3 seconds
    // output_stream.play().unwrap();
    
    // thread::sleep(Duration::from_secs(DURATION_SECONDS.into()));

    println!("Done.");
}