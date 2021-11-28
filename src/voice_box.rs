use std::thread::sleep;
use std::time::Duration;
use windows::Media::SpeechSynthesis::SpeechSynthesizer;
use windows::Media::Playback::MediaPlayer;
use windows::Media::Core::MediaSource;

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

impl VoiceBox {
    pub fn say(&mut self, text: &str) {
        let op = self.synth.SynthesizeTextToStreamAsync(text).unwrap();
        let stream = op.get().unwrap();
        let source = MediaSource::CreateFromStream(&stream, stream.ContentType().unwrap()).unwrap();
        self.player.SetSource(&source).unwrap();
        self.player.Play().unwrap();
        sleep(Duration::from_millis(2000));
    }
}
