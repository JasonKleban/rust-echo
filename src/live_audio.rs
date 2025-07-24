use std::sync::mpsc::{ Sender };

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, StreamConfig};

pub fn live_audio(sender : Sender<f32>, duration_seconds: u32) -> (cpal::Stream, cpal::StreamConfig) {
    let mut sender = Some(sender);

    // Set up CPAL host and default input/output devices
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("No input device available");

    // Use the default input/output configs
    let config = input_device.default_input_config().unwrap();
    let sample_rate = config.sample_rate();

    let sample_format = config.sample_format();

    if sample_format != cpal::SampleFormat::F32 {
        panic!("Only f32 sample format is supported in this example.");
    }

    let mut frames_remaining: usize = (sample_rate.0 * duration_seconds) as usize;

    println!("Recording {:?} frames from {:?} ", frames_remaining, input_device.name().unwrap());

    let out_config = StreamConfig {
        buffer_size: cpal::BufferSize::Default,
        channels: 1,
        sample_rate
    };

    let input_stream = input_device.build_input_stream(
        &config.clone().into(),
        move |data: &[f32], _| {
            let bytes_read = data.len();

            if bytes_read % config.channels() as usize != 0 {
                panic!("Badly shaped data");
            }

            let frames_read = bytes_read / config.channels() as usize;

            if frames_read <= frames_remaining {
                frames_remaining -= frames_read;

                for sample in data.chunks(config.channels() as usize) {
                    match &sender {
                        Some(sender) => {
                            sender.send((*sample).iter().sum::<f32>() / config.channels() as f32).unwrap();
                        },
                        None => { }
                    }
                    
                }
            } else {
                sender = None;
            }
        },
        move |err| {
            println!("Input stream error: {:?}", err);
        },
        None,
    ).unwrap();
    
    input_stream.play().unwrap();

    (input_stream, out_config)
}