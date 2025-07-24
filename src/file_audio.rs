use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::default::{get_codecs, get_probe};
use std::sync::mpsc::{ Sender };
use std::thread;
use cpal::{SampleRate, StreamConfig };

pub fn stream_file_audio<T : MediaSource + 'static>(sender : Sender<f32>, file : T) -> StreamConfig {
    // Open the MP3 file.
    //let file = File::open("your_audio.mp3").expect("Failed to open file");
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // Probe the media source.
    let probed = get_probe()
        .format(
            &Default::default(),
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .expect("Unsupported format");

    let mut format = probed.format;

    // Get the default audio track.
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .expect("No supported audio tracks");

    let n_channels = track.codec_params.channels.unwrap().count();

    // Create a decoder for the track.
    let mut decoder = get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .expect("Unsupported codec");

    let config = StreamConfig {
        buffer_size: cpal::BufferSize::Default,
        channels: 1,
        sample_rate: SampleRate(track.codec_params.sample_rate.unwrap())
    };

    // mimic the realtime API for live microphone
    thread::spawn(move || {
        // Decode and iterate over PCM frames.
        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(_) => break,
            };

            // Decode the packet into audio samples.
            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    match audio_buf {
                        AudioBufferRef::F32(buf) => {
                            for frame_idx in 0..buf.frames() {
                                let avg = (0..n_channels)
                                    .map(|ch| buf.chan(ch)[frame_idx])
                                    .sum::<f32>() / n_channels as f32;

                                sender.send(avg).unwrap();
                            }
                        }
                        AudioBufferRef::U8(buf) => {
                            for frame_idx in 0..buf.frames() {
                                let avg = (0..n_channels)
                                    .map(|ch| buf.chan(ch)[frame_idx] as f32)
                                    .sum::<f32>() / n_channels as f32 / 255.0;

                                sender.send(avg).unwrap();
                            }
                        }
                        AudioBufferRef::S16(buf) => {
                            for frame_idx in 0..buf.frames() {
                                let avg = (0..n_channels)
                                    .map(|ch| buf.chan(ch)[frame_idx] as f32)
                                    .sum::<f32>() / n_channels as f32 / 32768.0;

                                sender.send(avg).unwrap();
                            }
                        }
                        AudioBufferRef::S24(buf) => {
                            for frame_idx in 0..buf.frames() {
                                let avg = (0..n_channels)
                                    .map(|ch| buf.chan(ch)[frame_idx].0 as f32)
                                    .sum::<f32>() / n_channels as f32 / 8_388_608.0;

                                sender.send(avg).unwrap();
                            }
                        }
                        AudioBufferRef::S32(buf) => {
                            for frame_idx in 0..buf.frames() {
                                let avg = (0..n_channels)
                                    .map(|ch| buf.chan(ch)[frame_idx] as f32)
                                    .sum::<f32>() / n_channels as f32 / 2_147_483_648.0;

                                sender.send(avg).unwrap();
                            }
                        }
                        _ => {}
                    }
                },
                Err(e) => {
                    eprintln!("Decode error: {:?}", e);
                }
            }
        }
    });

    config
}