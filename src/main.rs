#[macro_use]
extern crate log;
use simplelog::*;
use std::thread;
mod lawfulvideo;
mod audio;
fn main() {

    // Initialize the logger here if needed

    // Spawn two threads to execute the video and audio capture functions
    let audio_thread = thread::spawn(|| {
        audio::execute_audio_capture().unwrap();
    });
    let video_thread = thread::spawn(|| {
        lawfulvideo::execute_video_capture().unwrap();
    });
    // Wait for the threads to finish capturing video and audio
    //video_thread.join().expect("Video thread panicked!");
    audio_thread.join().expect("Audio thread panicked!");
}
