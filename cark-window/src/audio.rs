use std::sync::{Arc, Mutex};

use cpal::traits::*;

pub fn start_audio() -> Result<AudioResult, String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "No output device available".to_string())?;
    let config = device.default_output_config().unwrap();
    let sample_format = config.sample_format();
    let mut config = config.config();

    // max channels: 2
    config.channels = config.channels.min(2);

    let proc = Arc::new(Mutex::new(Box::new(move |len| vec![0.0f32; len])
        as Box<dyn FnMut(usize) -> Vec<f32> + Send + Sync + 'static>));

    match sample_format {
        cpal::SampleFormat::F32 => run::<f32>(device, config.clone(), proc.clone()),
        cpal::SampleFormat::I16 => run::<i16>(device, config.clone(), proc.clone()),
        cpal::SampleFormat::U16 => run::<u16>(device, config.clone(), proc.clone()),
        _ => todo!(),
    };

    Ok(AudioResult {
        sample_rate: config.sample_rate.0 as usize,
        channels: config.channels as usize,
        callback: proc,
    })
}

type ProcFn = Arc<Mutex<Box<dyn FnMut(usize) -> Vec<f32> + Send + Sync + 'static>>>;

pub struct AudioResult {
    pub sample_rate: usize,
    pub channels: usize,
    pub callback: ProcFn,
}

fn run<T>(device: cpal::Device, config: cpal::StreamConfig, proc: ProcFn)
where
    T: cpal::SizedSample + cpal::FromSample<f32> + 'static,
{
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                let buf = proc.lock().unwrap()(data.len());
                for i in 0..data.len() {
                    data[i] = T::from_sample(buf[i]);
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| e.to_string())
        .unwrap();
    stream.play().map_err(|e| e.to_string()).unwrap();
    std::mem::forget(stream);
}

pub fn render_to_buffer(sample_rate: f32, events: Vec<ezmid::Event>) -> Vec<f32> {
    struct Note {
        channel: u8,
        note: u8,
        amp: f32,
        frequency: f32,
        start_time: f32,
    }

    let scale = 1.0 / 32.0;

    let mut buffer = Vec::new();
    let mut notes: Vec<Note> = vec![];
    let mut time = 0.0;

    for event in ezmid::Dispatcher::new(events) {
        for _ in 0..(event.dtime as f32 * sample_rate) as usize {
            let mut sample = 0.0;
            for note in &notes {
                sample += note.amp
                    * ((time - note.start_time) * note.frequency / sample_rate
                        * std::f32::consts::TAU)
                        .sin();
            }
            buffer.push(sample * scale);
            time += 1.0;
        }

        let channel = event.event.channel;
        match event.event.body {
            ezmid::EventBody::NoteOn {
                notenum, velocity, ..
            } => {
                let frequency = 440.0 * 2.0f32.powf((notenum as f32 - 69.0) / 12.0);
                notes.push(Note {
                    channel,
                    note: notenum,
                    amp: (velocity as f32).powf(1.0),
                    frequency,
                    start_time: time,
                });
            }
            ezmid::EventBody::NoteOff { notenum, .. } => {
                notes.retain(|n| !(n.channel == channel && n.note == notenum));
            }
            _ => {}
        }
    }

    buffer
}
