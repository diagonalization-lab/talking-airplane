fn main() {
    windows::build!(
        Windows::Media::SpeechSynthesis::SpeechSynthesizer,
        Windows::Media::SpeechSynthesis::SpeechSynthesisStream,
        Windows::Media::Playback::MediaPlayer,
        Windows::Media::Core::MediaSource,
    );
}
