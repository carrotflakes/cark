use std::sync::{Arc, Mutex};

use cpal::traits::*;

use super::AudioItem;

type ProcFn = Arc<Mutex<Box<dyn FnMut(usize) -> Vec<f32> + Send + Sync + 'static>>>;

pub struct AudioSystem {
    pub sample_rate: f32,
    pub channels: usize,
    pub callback: ProcFn,
    pub items: Arc<Mutex<Vec<AudioItem>>>,
}

impl AudioSystem {
    pub fn start() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "No output device available".to_string())?;
        let config = device.default_output_config().unwrap();
        let sample_format = config.sample_format();
        let mut config = config.config();

        // max channels: 2
        config.channels = config.channels.min(2);

        let items = Arc::new(Mutex::new(Vec::<AudioItem>::new()));
        let proc = Arc::new(Mutex::new(Box::new({
            let items = items.clone();
            let sample_rate = config.sample_rate.0 as f32;
            move |len| {
                let mut buf = vec![0.0f32; len];

                let mut items = items.lock().unwrap();
                for item in items.iter_mut() {
                    item.add_to_buffer(sample_rate, &mut buf);
                }
                items.retain(|item| !item.is_end());

                buf
            }
        })
            as Box<dyn FnMut(usize) -> Vec<f32> + Send + Sync + 'static>));

        match sample_format {
            cpal::SampleFormat::F32 => run::<f32>(device, config.clone(), proc.clone()),
            cpal::SampleFormat::I16 => run::<i16>(device, config.clone(), proc.clone()),
            cpal::SampleFormat::U16 => run::<u16>(device, config.clone(), proc.clone()),
            _ => todo!(),
        };

        Ok(AudioSystem {
            sample_rate: config.sample_rate.0 as f32,
            channels: config.channels as usize,
            callback: proc,
            items,
        })
    }
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
