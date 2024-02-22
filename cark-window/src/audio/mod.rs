pub mod audio_buffer;
pub mod audio_system;
pub mod item;

pub use audio_buffer::AudioBuffer;
pub use audio_system::AudioSystem;
pub use item::AudioItem;

use self::audio_buffer::AudioBufferRef;

pub fn render_to_buffer(sample_rate: f32, events: Vec<ezmid::Event>) -> Vec<[f32; 2]> {
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
            buffer.push([sample * scale; 2]);
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

pub fn generate_pop() -> AudioBufferRef {
    let mut buffer = Vec::new();
    let mut time = 0.0;
    let mut scale = 1.0;

    for _ in 0..(44100.0 * 0.1) as usize {
        let mut sample = 0.0;
        sample += (time * 880.0 / 44100.0 * std::f32::consts::TAU).sin();
        buffer.push([sample * scale; 2]);
        scale *= 0.999;
        time += 1.0;
    }

    AudioBufferRef::new(AudioBuffer::new(44100.0, buffer))
}
