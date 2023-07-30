use mki::{bind_key, Action, InhibitEvent, Keyboard, Sequence};
use screenshots::Screen;
use simplelog::*;
use std::fs;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::sync::Arc;
use std::time::{Duration, Instant};
use wasapi::*;
use notify_rust::Notification;
type Res<T> = Result<T, Box<dyn std::error::Error>>;

struct CircularBuffer {
    buffer: Vec<Vec<u8>>,
    buffer_size: usize,
    write_position: usize,
    frame_rate: usize,
    duration_secs: usize,
}

impl CircularBuffer {
    fn new(frame_rate: usize, duration_secs: usize, frame_size: usize) -> CircularBuffer {
        let buffer_size = duration_secs * frame_rate;
        debug!("My max capacity is: {}", buffer_size);
        CircularBuffer {
            buffer: vec![vec![0; frame_size]],
            buffer_size,
            write_position: 0,
            frame_rate,
            duration_secs,
        }
    }

    fn write(&mut self, data: Vec<u8>) {
        if self.buffer.len() < self.buffer_size {
            self.buffer.push(data);
        } else {
            self.buffer[self.write_position] = data;
            self.write_position = (self.write_position + 1) % self.buffer_size;
        }
    }

    fn read_all(&self) -> Vec<Vec<u8>> {
        // Filter out any empty frames from the buffer before returning
        self.buffer
            .iter()
            .filter(|frame| !frame.is_empty())
            .cloned()
            .collect()
    }
}

fn calculate_frame_size(screen: &Screen) -> usize {
    let width = screen.display_info.width as usize;
    let height = screen.display_info.height as usize;
    let bytes_per_pixel = 4;
    width * height * bytes_per_pixel
}

fn transform_frames_to_video(fps: usize) {
    let fpstring = fps.to_string();
    info!("Frames will be {}", fpstring);
    let output = std::process::Command::new("cmd")
        .arg("/C")
        .arg("python")
        .arg("clips/convert.py")
        .arg(fpstring)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to convert");

    if let Some(mut child_stdout) = output.stdout {
        let mut output_data = String::new();
        child_stdout
            .read_to_string(&mut output_data)
            .expect("Failed to read child process");
        trace!("{}", output_data);
    }
    info!("Clipped successfully");
    Notification::new()
    .summary("Clipped succesfully")
    .show().unwrap();
}

pub fn execute_video_capture(wavwritten:Arc<AtomicBool>) -> Res<()> {
    let _ = SimpleLogger::init(
        LevelFilter::Info,
        ConfigBuilder::new()
            .set_time_format_rfc3339()
            .set_time_offset_to_local()
            .unwrap()
            .build(),
    );
    let screens = Screen::all().unwrap();
    let s = screens[0];
    let framerate = 24;
    let mut videobuffer = CircularBuffer::new(framerate, 30, calculate_frame_size(&s));
    initialize_mta()?;

    loop {
        if mki::are_pressed(&[Keyboard::LeftAlt, Keyboard::X]) {
            info!("Should start clipping");
            let buffered_frames = videobuffer.read_all();
            let mut count = 0;
            for frame in buffered_frames {
                fs::write(format!("clips/{}.png", count), frame).unwrap();
                count += 1;
            }
            while !wavwritten.load(Ordering::Relaxed){
                info!("Waiting");
                continue
            }
            thread::spawn(move || transform_frames_to_video(framerate));
        }
        videobuffer.write(capture(&s));
        //println!("Buffer size: {}", videobuffer.read_all().len()); // Add this line to check buffer size
    }
}

fn capture(screen: &Screen) -> Vec<u8> {
    let image = screen.capture().unwrap();
    let buffer = image.buffer();
    buffer.to_owned()
}
