use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::bus::MainBus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::joypad::{Button, JoyPort};
use crate::reverb::Reverb;
use crate::timer::Timer;

const SECS_PER_FRAME: f64 = 1.0 / 60.0;

static WINDOW_TITLE: &str = "NesOxyde";
pub const WIDTH: u32 = 256;
pub const HEIGHT: u32 = 240;

mod trace;

#[derive(Debug)]
pub enum Mode {
    VideoSync,
    AudioSync,
}

pub fn run<KeyMap>(cartridge: Cartridge, map_key: KeyMap, mode: Mode)
where
    KeyMap: Fn(Keycode, JoyPort) -> Option<Button>,
{
    // SDL2 init ----------------->
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let window = video_subsystem
        .window(WINDOW_TITLE, WIDTH * 2, HEIGHT * 2)
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

    let sample_size = 512;
    let sample_rate = 41000;
    let spec = AudioSpecDesired {
        freq: Some(sample_rate as i32),
        channels: Some(1),
        samples: Some(sample_size),
    };
    let queue = audio_subsystem.open_queue::<f32, _>(None, &spec).unwrap();
    queue.resume();

    let mut samples = vec![0.0; 1024];

    let mut reverbs = [
        Reverb::new(330, sample_rate, 0.15),
        Reverb::new(150, sample_rate, 0.1),
        Reverb::new(285, sample_rate, 0.05),
    ];

    println!("Audio driver: {}", audio_subsystem.current_audio_driver());
    println!("Emulation mode: {:?}", &mode);
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

    let mut timer = Timer::new();
    'nes: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'nes,
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => cpu.reset(),
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
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
            Mode::VideoSync => {
                let frame_count = cpu.frame_count();
                while cpu.frame_count() == frame_count {
                    cpu.execute();
                }
                timer.wait(Duration::from_secs_f64(SECS_PER_FRAME));
                timer.reset();
            }
            Mode::AudioSync => {
                while queue.size() > sample_size as u32 * 6 {
                    timer.reset();
                    timer.wait(Duration::from_micros(100));
                }

                while cpu.sample_count() < sample_size as usize {
                    cpu.clock();
                }
            }
        }

        samples.append(&mut cpu.samples());

        for r in reverbs.iter_mut() {
            r.apply(&mut samples);
        }

        queue.queue(&samples);
        samples.clear();
    }
}
