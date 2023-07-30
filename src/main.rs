#[macro_use]
extern crate log;
use simplelog::*;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::sync::Arc;
mod lawfulvideo;
mod audio;

fn main() {
    let wavwritten = Arc::new(AtomicBool::new(false));  
    let ww = wavwritten.clone();
    let _ = SimpleLogger::init(
        LevelFilter::Info,
        ConfigBuilder::new()
            .set_time_format_rfc3339()
            .set_time_offset_to_local()
            .unwrap()
            .build(),
    );
    // Spawn two threads to execute the video and audio capture functions
    let audio_thread = thread::spawn(move || {
        audio::execute_audio_capture(wavwritten).unwrap();
    });
    let video_thread = thread::spawn(move || {
        lawfulvideo::execute_video_capture(ww).unwrap();
    });
    // Wait for the threads to finish capturing video and audio
    audio_thread.join().expect("Audio thread panicked!");
    video_thread.join().expect("Video thread panicked!");
}
