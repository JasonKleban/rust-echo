use std::thread;
use std::time::Duration;
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    // Set up CPAL host and default input/output devices
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("No input device available");
    let output_device = host.default_output_device().expect("No output device available");

    println!("Recording from {:?}", input_device.name());
    println!("Outputting to {:?}", output_device.name());

    // Use the default input/output configs
    let config = input_device.default_input_config().unwrap();
    let sample_rate = config.sample_rate();

    // We'll use f32 samples for simplicity
    let sample_format = config.sample_format();

    if sample_format != cpal::SampleFormat::F32 {
        panic!("Only f32 sample format is supported in this example.");
    }

    // Shared buffer for audio data
    let duration_seconds = 3;
    let ring = HeapRb::<f32>::new((sample_rate.0 * duration_seconds * 2).try_into().unwrap());
    let (mut producer, mut consumer) = ring.split();

    println!("Recording for {:?} seconds...", duration_seconds);

    let input_stream = input_device.build_input_stream(
        &config.clone().into(),
        move |data: &[f32], _| {
            for &sample in data {
                if producer.try_push(sample).is_err() {

                }
            }
        },
        move |err| {
            eprintln!("Input stream error: {:?}", err);
        },
        None,
    ).unwrap();
    
    input_stream.play().unwrap();

    // Record for 3 seconds
    thread::sleep(Duration::from_secs(duration_seconds.into()));

    drop(input_stream); // Stop recording

    println!("Playback...");

    // Build output stream
    let output_stream = output_device.build_output_stream(
        &config.clone().into(),
        move |data: &mut [f32], _| {
            for sample in data {
                *sample = match consumer.try_pop() {
                    Some(s) => s,
                    None => { 0.0 }
                };
            }
        },
        move |err| {
            eprintln!("Output stream error: {:?}", err);
        },
        None,
    ).unwrap();

    // Playback for 3 seconds
    output_stream.play().unwrap();
    
    thread::sleep(Duration::from_secs(duration_seconds.into()));

    println!("Done.");
}