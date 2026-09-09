use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Owns the audio device and a beep tone decoded once at startup, instead of
/// re-opening and re-decoding `beep.wav` from disk on every tick the sound
/// timer is active (as the original implementation did, up to 60x/sec).
pub struct Beeper {
    _stream: OutputStream,
    sink: Sink,
    samples: rodio::buffer::SamplesBuffer<i16>,
    playing: bool,
}

impl Beeper {
    pub fn new(beep_path: &Path) -> Self {
        let (_stream, stream_handle) =
            OutputStream::try_default().expect("Failed to open audio device");
        let sink = Sink::try_new(&stream_handle).expect("Failed to create audio sink");

        let file = BufReader::new(File::open(beep_path).expect("Failed to open beep.wav"));
        let decoded = Decoder::new(file).expect("Failed to decode beep.wav");
        let channels = decoded.channels();
        let sample_rate = decoded.sample_rate();
        let samples: Vec<i16> = decoded.convert_samples().collect();
        let samples = rodio::buffer::SamplesBuffer::new(channels, sample_rate, samples);

        Beeper {
            _stream,
            sink,
            samples,
            playing: false,
        }
    }

    /// Call once per frame with whether the CHIP-8 sound timer is active.
    pub fn set_playing(&mut self, should_play: bool) {
        if should_play && !self.playing {
            self.sink.append(self.samples.clone().repeat_infinite());
        } else if !should_play && self.playing {
            self.sink.stop();
        }
        self.playing = should_play;
    }
}
