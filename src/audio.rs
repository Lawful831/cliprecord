use mki::{bind_key, Action, InhibitEvent, Keyboard, Sequence};
use std::collections::VecDeque;
use std::error;
use std::fs::File;
use std::io::prelude::*;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;
use std::sync::Arc;
use hound::WavSpec;
use hound::WavWriter;
use simplelog::*;
use std::time::Duration;
use std::time::Instant;
use wasapi::*;
use notify_rust::Notification;
type Res<T> = Result<T, Box<dyn error::Error>>;

fn capture_loop(tx_capt: std::sync::mpsc::SyncSender<Vec<u8>>, chunksize: usize) -> Res<()> {
    // Use `Direction::Capture` for normal capture,
    // or `Direction::Render` for loopback mode (for capturing from a playback device).
    let device = get_default_device(&Direction::Render)?;

    let mut audio_client = device.get_iaudioclient()?;

    let desired_format = WaveFormat::new(32, 32, &SampleType::Float, 44100, 2, None);

    let blockalign = desired_format.get_blockalign();
    debug!("Desired capture format: {:?}", desired_format);

    let (def_time, min_time) = audio_client.get_periods()?;
    debug!("default period {}, min period {}", def_time, min_time);

    audio_client.initialize_client(
        &desired_format,
        min_time as i64,
        &Direction::Capture,
        &ShareMode::Shared,
        true,
    )?;
    debug!("initialized capture");

    let h_event = audio_client.set_get_eventhandle()?;

    let buffer_frame_count = audio_client.get_bufferframecount()?;

    let render_client = audio_client.get_audiocaptureclient()?;
    let mut sample_queue: VecDeque<u8> = VecDeque::with_capacity(
        100 * blockalign as usize * (1024 + 2 * buffer_frame_count as usize),
    );
    let session_control = audio_client.get_audiosessioncontrol()?;

    debug!("state before start: {:?}", session_control.get_state());
    audio_client.start_stream()?;
    debug!("state after start: {:?}", session_control.get_state());
    loop {
        while sample_queue.len() > (blockalign as usize * chunksize as usize) {
            debug!("pushing samples");
            let mut chunk = vec![0u8; blockalign as usize * chunksize as usize];
            for element in chunk.iter_mut() {
                *element = sample_queue.pop_front().unwrap();
            }
            tx_capt.send(chunk)?;
        }
        trace!("capturing");
        render_client.read_from_device_to_deque(blockalign as usize, &mut sample_queue)?;

        if h_event.wait_for_event(3000).is_err() {
            error!("timeout error, capture fail???");
        }
    }
    Ok(())
}

// Main loop
pub fn execute_audio_capture(wavwritten:Arc<AtomicBool>) -> Res<()> {
    let _ = SimpleLogger::init(
        LevelFilter::Info,
        ConfigBuilder::new()
            .set_time_format_rfc3339()
            .set_time_offset_to_local()
            .unwrap()
            .build(),
    );

    initialize_mta()?;

    let (tx_capt, rx_capt): (
        std::sync::mpsc::SyncSender<Vec<u8>>,
        std::sync::mpsc::Receiver<Vec<u8>>,
    ) = mpsc::sync_channel(2);
    let chunksize = 4096;
    // Capture
    let _handle = thread::Builder::new()
        .name("Capture".to_string())
        .spawn(move || {
            let result = capture_loop(tx_capt, chunksize);
            if let Err(err) = result {
                error!("Capture failed with error {}", err);
            }
        });

    let sample_rate = 44100; // The sample rate you are using (modify as needed)
    let bits_per_sample = 32; // The number of bits per sample (modify as needed)
    let num_channels = 2; // The number of audio channels (modify as needed)

    // Define circular buffer size for 3 seconds of audio (adjust based on the sample rate)
    let sduration = 7;
    let circular_buffer_size: usize = (sample_rate * bits_per_sample / 8 * sduration) as usize;

    // Initialize circular buffer
    let mut circular_buffer: VecDeque<f32> = VecDeque::with_capacity(circular_buffer_size);

    loop {
        match rx_capt.recv() {
            Ok(chunk) => {
                // Convert the raw bytes (u8) to f32 samples
                let chunk_as_f32: Vec<f32> = chunk
                    .chunks_exact(4)
                    .map(|bytes| {
                        let value = i32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                        f32::from_bits(value as u32) // Corrected conversion here
                    })
                    .collect();

                // Update the circular buffer with the new chunk
                for sample in &chunk_as_f32 {
                    circular_buffer.push_back(*sample);
                }

                // Enforce the 3-second buffer limit
                while circular_buffer.len() > circular_buffer_size {
                    circular_buffer.pop_front();
                }

                // Check if the Alt + X combination is pressed
                if mki::are_pressed(&[Keyboard::LeftAlt, Keyboard::X]) {
                    // Write the contents of the circular buffer to the WAV file
                    Notification::new()
                    .summary("Clipping last 14 seconds")
                    .show()?;
                    info!("Save to wav");
                    let spec = WavSpec {
                        channels: num_channels as u16,
                        sample_rate: sample_rate,
                        bits_per_sample: bits_per_sample as u16,
                        sample_format: hound::SampleFormat::Float,
                    };

                    let mut writer = WavWriter::create("clips/recorded.wav", spec)?;
                    for &sample in &circular_buffer {
                        writer.write_sample(sample)?;
                    }
                    writer.finalize()?;
                    wavwritten.store(true,Ordering::Relaxed);
                }
            }
            Err(err) => {
                // Handle errors and finalize the WAV file
                // ...
            }
        }
    }
}
