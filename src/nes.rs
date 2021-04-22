use sdl2::audio::{AudioCallback, AudioSpecDesired};
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
use crate::timer::Timer;

const SECS_PER_FRAME: f64 = 1.0 / 60.0;

static WINDOW_TITLE: &str = "NesOxyde";
pub const WIDTH: u32 = 256;
pub const HEIGHT: u32 = 240;

mod trace;

pub fn run<KeyMap>(cartridge: Cartridge, map_key: KeyMap)
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

    let spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: Some(512),
    };
    let device = audio_subsystem
        .open_playback(None, &spec, |_spec| NesAudio(0))
        .unwrap();
    // >----------------- SDL2 init

    let bus = MainBus::new(Rc::new(RefCell::new(cartridge)), move |frame| {
        texture.update(None, frame, (WIDTH * 3) as usize).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
    });

    let mut cpu = Cpu::new(bus);
    cpu.reset();

    let mut timer = Timer::new();
    'nes: loop {
        let frame_count = cpu.frame_count();
        while cpu.frame_count() == frame_count {
            cpu.execute();
        }

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

        timer.wait(Duration::from_secs_f64(SECS_PER_FRAME));
        timer.reset();
    }
}

struct NesAudio(u16);
impl AudioCallback for NesAudio {
    type Channel = u16;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for sample in out.iter_mut() {
            *sample = self.0;
        }
    }
}
