use std::sync::mpsc::{channel, Receiver};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamConfig,
};

use dasp::{interpolate::floor::Floor, signal::from_iter, Signal};

pub fn out_audio(
    receiver: Receiver<f32>,
    stream_config: StreamConfig,
) -> (std::sync::mpsc::Receiver<()>, cpal::Stream) {
    let host = cpal::default_host();
    let output_device = host
        .default_output_device()
        .expect("No output device available");
    let config = output_device.default_output_config().unwrap();

    let (exhausted_send, exhausted) = channel();

    println!("Outputting to {:?}", output_device.name().unwrap());

    let mut resampled = from_iter(receiver).from_hz_to_hz(
        Floor::new(0.0),
        stream_config.sample_rate.0 as f64,
        config.sample_rate().0 as f64,
    );

    // Build output stream
    let output_stream = output_device
        .build_output_stream(
            &config.clone().into(),
            move |data: &mut [f32], _| {
                for chunk in data.chunks_exact_mut(2) {
                    let pair: &mut [f32; 2] = chunk.try_into().unwrap();
                    *pair = if resampled.is_exhausted() {
                        exhausted_send.send(()).unwrap();
                        [0.0, 0.0]
                    } else {
                        let mono = resampled.next();
                        [mono, mono]
                    };
                }
            },
            move |err| {
                eprintln!("Output stream error: {:?}", err);
            },
            None,
        )
        .unwrap();

    output_stream.play().unwrap();

    (exhausted, output_stream)
}
