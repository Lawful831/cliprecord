use std::fs;
use std::time::{Duration, Instant};
use device_query::{DeviceEvents, DeviceState, DeviceQuery};
use device_query::keymap::Keycode;
use screenshots::Screen;
use ffmpeg_next::{
    codec::encoder::video::Video,
    dictionary::Dictionary,
    format::context::output::{Audio, Output},
    media::Type,
    Rational,
};
struct CircularBuffer {
    buffer: Vec<Vec<u8>>,
    buffer_size: usize,
    write_position: usize,
    frame_rate: usize,
    duration_secs: usize,
}

impl CircularBuffer {
    fn new(frame_rate: usize, duration_secs: usize, frame_size: usize) -> CircularBuffer {
        let buffer_size = frame_size * frame_rate;
        CircularBuffer {
            buffer: vec![vec![0;frame_size]],
            buffer_size,
            write_position: 0,
            frame_rate,
            duration_secs,
        }
    }

    fn write(&mut self, data: Vec<u8>) {
        if self.buffer.len() >= self.buffer_size {
            //println!("I reached my limit!");
            self.buffer[self.write_position] = data;
            self.write_position = (self.write_position + 1) % self.buffer_size;
        } else {
            self.buffer.push(data);
        }

        // Trim the buffer to the desired duration
        let max_frames = self.frame_rate * self.duration_secs;
        if self.buffer.len() > max_frames {
            println!("Reached max content");
            let trim_frames = self.buffer.len() - max_frames;
            self.buffer.drain(0..trim_frames);
            self.write_position = (self.write_position + trim_frames) % self.buffer_size;
        }
    }

    fn read_all(&self) -> Vec<Vec<u8>> {
        self.buffer.clone()
    }

}

fn calculate_frame_size(screen: &Screen) -> usize {
    let width = screen.display_info.width as usize;
    let height = screen.display_info.height as usize;
    let bytes_per_pixel = 4;

    width * height
}

fn main() {
    let screens = Screen::all().unwrap();
    let s = screens[0];

    let mut videobuffer = CircularBuffer::new(18, 10, calculate_frame_size(&s));
    let device_state = DeviceState::new();
    println!("Query? {:#?}", device_state.query_keymap());

    let target_fps = 30;
    let frame_duration = Duration::from_secs(1) / target_fps;
    let mut last_frame_time = Instant::now();

    loop {
        let keys: Vec<Keycode> = device_state.get_keys();
        if keys.contains(&Keycode::Q) {
            break;
        }
    
        if keys.contains(&Keycode::X) {
            println!("Should start clipping");
            let buffered_frames = videobuffer.read_all();
            let mut count = 0;
            for frame in buffered_frames {
                fs::write(format!("clips/{}.png", count), frame).unwrap();
                count += 1;
            }
            println!("Clipped successfully");
        }
    
        videobuffer.write(capture(&s));
    
        let elapsed = last_frame_time.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    
        last_frame_time += frame_duration; // Add frame_duration to last_frame_time
    }
}

fn capture(screen: &Screen) -> Vec<u8> {
    let image = screen.capture().unwrap();
    let buffer = image.buffer();
    buffer.to_owned()
}
