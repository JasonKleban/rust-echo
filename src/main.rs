use std::thread;
use std::time::Duration;
use std::sync::mpsc::{ channel, sync_channel };

use burn::prelude::*;
use burn::{ backend::NdArray };

use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

// Shared buffer for audio data
const DURATION_SECONDS: u32 = 3;

fn main() {
    let device = Default::default();
    type B = burn::backend::NdArray;

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

    let (tx_done, rx_done) = sync_channel::<Result<(), ()>>(1);
    let tx_errored = tx_done.clone();
    let (tx, rx) = channel::<Tensor::<B, 1>>();
    let mut samples_remaining: usize = (sample_rate.0 * DURATION_SECONDS) as usize;

    println!("Recording for {:?} seconds...", DURATION_SECONDS);

    let input_stream = input_device.build_input_stream(
        &config.clone().into(),
        move |data: &[f32], _| {            
            tx.send(Tensor::<B, 1>::from_floats(data, &device)).unwrap();

            let bytes_read = data.len();
            if bytes_read <= samples_remaining {
                samples_remaining -= bytes_read;
            } else {
                tx_done.send(Ok(())).unwrap();
            }
        },
        move |err| {
            println!("Input stream error: {:?}", err);
            tx_errored.send(Err(())).unwrap();
        },
        None,
    ).unwrap();
    
    input_stream.play().unwrap();

    rx_done.recv()
        .expect("Failed to receive done signal")
        .expect("Recording failed");

    drop(input_stream); // Stop recording

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