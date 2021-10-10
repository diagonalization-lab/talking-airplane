use bindings::Windows::Media::{
    Core::MediaSource, Playback::MediaPlayer, SpeechSynthesis::SpeechSynthesizer,
};

pub struct VoiceBox {
    synth: SpeechSynthesizer,
    player: MediaPlayer,
}

impl Default for VoiceBox {
    fn default() -> Self {
        let synth = SpeechSynthesizer::new().unwrap();
        let player = MediaPlayer::new().unwrap();
        Self { synth, player }
    }
}

use std::thread::sleep;
use std::time::Duration;

impl VoiceBox {
    pub fn say(&mut self, text: &str) {
        let op = SpeechSynthesizer::SynthesizeTextToStreamAsync(&self.synth, text).unwrap();
        let stream = op.get().unwrap();
        let source = MediaSource::CreateFromStream(&stream, stream.ContentType().unwrap()).unwrap();
        MediaPlayer::SetSource(&self.player, &source).unwrap();
        MediaPlayer::Play(&self.player).unwrap();
        sleep(Duration::from_millis(2000));
    }
}
