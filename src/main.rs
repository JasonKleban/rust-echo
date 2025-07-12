use std::sync::mpsc::{ channel, sync_channel };
use realfft::RealFftPlanner;

use burn::prelude::*;
//use burn::{ backend::NdArray };

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

// Shared buffer for audio data
const DURATION_SECONDS: u32 = 60;

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

    println!("Recording from {:?} @ {:?} sample_rate", input_device.name(), sample_rate);
    println!("Outputting to {:?}", output_device.name());

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

    let mut planner = RealFftPlanner::new();
    let fft = planner.plan_fft_forward(4096);
    // make a vector for storing the spectrum
    let mut spectrum = fft.make_output_vec();

    let glyphs = vec![
        "⠀⢀⢠⢰⢸".chars().collect::<Vec<char>>(),
        "⡀⣀⣠⣰⣸".chars().collect(),
        "⡄⣄⣤⣴⣼".chars().collect(),
        "⡆⣆⣦⣶⣾".chars().collect(),
        "⡇⣇⣧⣷⣿".chars().collect()];

    loop {
        let mut buffer = rx.iter().take(4096).collect::<Vec<f32>>();

        if buffer.len() < 4096 { break; }

        fft.process(&mut buffer, &mut spectrum).unwrap();

        spectrum.iter_mut().for_each(|x| {
            *x = *x / 64.; // Normalize the spectrum
        });

        let spectrum_amplitude = spectrum.iter()
            .map(|x| { (x.re * x.re + x.im * x.im).sqrt() })
            .collect::<Vec<f32>>();

        let display = 
            spectrum_amplitude[..2048]
            .chunks(16)
            .map(|chunk| {
                (5. * (chunk.iter().sum::<f32>() / 64.)).floor().clamp(0., 4.) as usize
            })
            .collect::<Vec<_>>();

        let pairs = display
            .chunks(2)
            .collect::<Vec<_>>();

        for (i, row) in pairs.iter().enumerate() {
            let [first, second] = *row else { panic!("bad size {:?}", row) };
            
            print!("{}", glyphs[*first][*second]);
        }
        
        println!();
    }
    
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