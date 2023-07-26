use std::fs;
use std::io::Read;
use std::time::{Duration, Instant};
use device_query::{DeviceState, DeviceQuery};
use mki::{bind_key, Action, InhibitEvent, Keyboard, Sequence};
use device_query::keymap::Keycode;
use screenshots::Screen;
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
        println!("My max capacity is: {}",buffer_size);
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
            println!("I reached my limit!");
            self.buffer[self.write_position] = data;
            self.write_position = (self.write_position + 1) % self.buffer_size;
        } else {
            self.buffer.push(data);
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
    width * height*bytes_per_pixel
}

fn transform_frames_to_video(fps:usize){
    let python_script_path = "clips/convert.py";
    let fpstring = fps.to_string();
    println!("Frames will be {}",fpstring);
    let output = std::process::Command::new("cmd").
                               arg("/C").
                               arg("python").
                               arg("clips/convert.py").
                               arg(fpstring)
                               .stdout(std::process::Stdio::piped())
                               .spawn()
                               .expect("Failed to convert");
                    
    if let Some(mut child_stdout) = output.stdout{
        let mut output_data = String::new();
        child_stdout
            .read_to_string(&mut output_data)
            .expect("Failed to read child process");
        println!("{}",output_data);
    }
}

fn main() {
    let screens = Screen::all().unwrap();
    let s = screens[0];
    let framerate = 18;
    let mut videobuffer = CircularBuffer::new(framerate, 60, calculate_frame_size(&s));
    let device_state = DeviceState::new();
    println!("Query? {:#?}", device_state.query_keymap());

    /*mki::register_hotkey(&[Keyboard::LeftAlt, Keyboard::X], move || {
        println!("Alt+X Pressed");
    });*/

    loop {
        
        let keys: Vec<Keycode> = device_state.get_keys();
        if keys.contains(&Keycode::Q) {
            break;
        }
    
        if mki::are_pressed(&[Keyboard::LeftAlt, Keyboard::X]) {
            println!("Should start clipping");
            let buffered_frames = videobuffer.read_all();
            let mut count = 0;
            for frame in buffered_frames {
                /*if count == 0 {
                    count += 1;
                    continue
                };*/
                fs::write(format!("clips/{}.png", count), frame).unwrap();
                count += 1;
                println!("Count{}",count);
            }
            transform_frames_to_video(framerate);
            println!("Clipped successfully");
        }
    
        videobuffer.write(capture(&s));
        println!("Buffer size: {}", videobuffer.read_all().len()); // Add this line to check buffer size
    }
}

fn capture(screen: &Screen) -> Vec<u8> {
    let image = screen.capture().unwrap();
    let buffer = image.buffer();
    buffer.to_owned()
}
