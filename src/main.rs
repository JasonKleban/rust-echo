use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    // Set up CPAL host and default input/output devices
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("No input device available");
    let output_device = host.default_output_device().expect("No output device available");

    println!("Recording from {:?}", input_device.name());
    println!("Outputting to {:?}", output_device.name());

    // Use the default input/output configs
    let input_config = input_device.default_input_config().unwrap();
    let output_config = output_device.default_output_config().unwrap();
    let output_sample_rate = output_config.sample_rate().0;

    // We'll use f32 samples for simplicity
    let sample_format = input_config.sample_format();
    let config = input_config.into();

    // Shared buffer for audio data
    let audio_data: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));

    // Clone for move into closure
    let audio_data_in = audio_data.clone();

    println!("Recording for 3 seconds...");

    // Build input stream
    let input_stream = match sample_format {
        cpal::SampleFormat::F32 => input_device.build_input_stream(
            &config,
            move |data: &[f32], _| {
                let mut buffer = audio_data_in.lock().unwrap();
                buffer.extend_from_slice(data);
            },
            move |err| {
                eprintln!("Input stream error: {:?}", err);
            },
            None,
        ),
        _ => panic!("Only f32 sample format is supported in this example."),
    }.unwrap();

    input_stream.play().unwrap();

    // Record for 3 seconds
    thread::sleep(Duration::from_secs(3));

    drop(input_stream); // Stop recording

    println!("Playback...");

    // Move the Vec<f32> out of the Mutex, dropping Arc/Mutex
    let audio_data_out = Arc::try_unwrap(audio_data)
        .expect("Multiple references to audio_data exist")
        .into_inner()
        .unwrap();
    let mut playback_pos = 0;

    let output_samples = audio_data_out.len() as f32;

    // Build output stream
    let output_stream = output_device.build_output_stream(
        &output_config.into(),
        move |output: &mut [f32], _| {
            for sample in output.iter_mut() {
                if playback_pos < audio_data_out.len() {
                    *sample = audio_data_out[playback_pos];
                    playback_pos += 1;
                } else {
                    *sample = 0.0;
                }
            }
        },
        move |err| {
            eprintln!("Output stream error: {:?}", err);
        },
        None,
    ).unwrap();

    output_stream.play().unwrap();

    // Wait for playback to finish
    let playback_duration = Duration::from_secs_f32(output_samples / output_sample_rate as f32);
    //let playback_duration = Duration::from_secs(3);
    thread::sleep(playback_duration);

    println!("Done.");
}