use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use spin_sleep::SpinSleeper;
use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;
use std::time::Duration;

use crate::bus::MainBus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::joypad::{Button, JoyPort};
use crate::reverb::Reverb;
use crate::savable::Savable;
use crate::timer::Timer;

/// Time between each frame (at 60fps)
const SECS_PER_FRAME: f64 = 1.0 / 60.0;

static WINDOW_TITLE: &str = "NesOxyde";
/// NES screen width
pub const WIDTH: u32 = 256;
/// NES screen height
pub const HEIGHT: u32 = 240;

/// Step when adjusting volume
const VOLUME_STEP: f32 = 0.05;

mod trace;

/// Emulation sync mode
#[derive(Debug)]
pub enum Mode {
    VideoSync,
    AudioSync,
}

/// Runs the emulation
pub fn run<KeyMap>(cartridge: Cartridge, map_key: KeyMap, mode: Mode)
where
    KeyMap: Fn(Keycode, JoyPort) -> Option<Button>,
{
    // SDL2 init ----------------->
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let filename = cartridge.filename();
    let savestate_file = format!("{}.save", &filename);
    let formated_name = if filename.is_empty() {
        "".to_string()
    } else {
        format!(" - {}", &filename)
    };
    let window = video_subsystem
        .window(
            &format!("{}{}", WINDOW_TITLE, &formated_name),
            WIDTH * 2,
            HEIGHT * 2,
        )
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, WIDTH as u32, HEIGHT as u32)
        .unwrap();

    let buffer_size = 1024;
    let sample_rate = 44100;
    let spec = AudioSpecDesired {
        freq: Some(sample_rate as i32),
        channels: Some(1),
        samples: Some(buffer_size),
    };
    let queue = audio_subsystem.open_queue::<f32, _>(None, &spec).unwrap();
    queue.resume();

    let mut samples = vec![0.0; 1024];
    let mut volume = 0.5;

    let mut reverbs = [
        Reverb::new(330, sample_rate, 0.15),
        Reverb::new(150, sample_rate, 0.1),
        Reverb::new(285, sample_rate, 0.05),
    ];

    println!("Audio driver: {}", audio_subsystem.current_audio_driver());
    println!("Emulation mode: {:?}", &mode);
    println!("Vol: {:.0}", volume * 100.0);
    // >----------------- SDL2 init

    let bus = MainBus::new(
        Rc::new(RefCell::new(cartridge)),
        move |frame| {
            texture.update(None, frame, (WIDTH * 3) as usize).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        },
        sample_rate as f64,
    );

    let mut cpu = Cpu::new(bus);
    cpu.reset();

    let update_vol = |vol, step| {
        let old = (vol * 100.0) as u32;
        let new_vol = vol + step;
        let out = match step < 0.0 {
            true => f32::max(0.0, new_vol),
            false => f32::min(1.0, new_vol),
        };

        if old != (out * 100.0) as u32 {
            println!("Vol: {:.0}", out * 100.0);
        }
        out
    };

    let mut timer = Timer::new();
    let spin_sleeper = SpinSleeper::default();
    // Main loop
    'nes: loop {
        // Process all the SDL events
        for event in event_pump.poll_iter() {
            match event {
                // Quit
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'nes,
                // Volume down
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => volume = update_vol(volume, -VOLUME_STEP),
                // Volume up
                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => volume = update_vol(volume, VOLUME_STEP),
                // Reset
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => cpu.reset(),
                // Save state
                Event::KeyDown {
                    keycode: Some(Keycode::F1),
                    ..
                } => match File::create(&savestate_file) {
                    Ok(file) => match cpu.save(&file) {
                        Ok(_) => println!("State saved!"),
                        Err(e) => println!("Error while saving state: {}", e),
                    },
                    Err(e) => println!("Error while saving state: {} -> {}", e, &savestate_file),
                },
                // Load state
                Event::KeyDown {
                    keycode: Some(Keycode::F2),
                    ..
                } => match File::open(&savestate_file) {
                    Ok(file) => match cpu.load(&file) {
                        Ok(_) => {
                            println!("State loaded!");
                            samples.clear();
                            queue.clear();
                            reverbs.iter_mut().for_each(|r| r.clear());
                        }
                        Err(e) => println!("Error while loading state: {}", e),
                    },
                    Err(e) => println!("Error while loading state: {} -> {}", e, &savestate_file),
                },
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    // If a button is found from the mapping, update the proper controller state
                    if let Some(button) = map_key(key, JoyPort::Port1) {
                        cpu.update_joypad(button, true, JoyPort::Port1)
                    }
                    if let Some(button) = map_key(key, JoyPort::Port2) {
                        cpu.update_joypad(button, true, JoyPort::Port2)
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    // If a button is found from the mapping, update the proper controller state
                    if let Some(button) = map_key(key, JoyPort::Port1) {
                        cpu.update_joypad(button, false, JoyPort::Port1)
                    }
                    if let Some(button) = map_key(key, JoyPort::Port2) {
                        cpu.update_joypad(button, false, JoyPort::Port2)
                    }
                }
                _ => {}
            }
        }

        match mode {
            // Sync emulation at 60 fps
            Mode::VideoSync => {
                let frame_count = cpu.frame_count();
                // Clock until a new frame is rendered
                while cpu.frame_count() == frame_count {
                    cpu.clock();
                }
                // Wait if not enough time has passed
                timer.wait(Duration::from_secs_f64(SECS_PER_FRAME));
                timer.reset();
            }
            // Sync emulation with the audio sample rate
            Mode::AudioSync => {
                // While theres too many samples in the queue, wait a bit
                while queue.size() > buffer_size as u32 * 4 {
                    spin_sleeper.sleep(Duration::from_micros(256));
                }

                // Clock until enough samples are generated
                while cpu.sample_count() < buffer_size as usize {
                    cpu.clock();
                }
            }
        }

        // Add the samples to a buffer
        samples.append(&mut cpu.samples());

        // Apply reverb to the samples
        for r in reverbs.iter_mut() {
            r.apply(&mut samples);
        }

        // Adjust the volume
        samples.iter_mut().for_each(|s| *s *= volume);

        // Add the samples to the SDL audio queue
        queue.queue(&samples);
        // Empty the samples buffer
        samples.clear();
    }
}
