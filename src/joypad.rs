use bitflags::bitflags;

bitflags! {
    struct State: u8 {
        const A      = 0b00000001;
        const B      = 0b00000010;
        const SELECT = 0b00000100;
        const START  = 0b00001000;
        const UP     = 0b00010000;
        const DOWN   = 0b00100000;
        const LEFT   = 0b01000000;
        const RIGHT  = 0b10000000;
    }
}

pub enum JoyPort {
    Port1,
    Port2,
}

#[derive(Debug)]
pub enum Button {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy)]
pub struct JoyPad {
    strobe: bool,
    state: State,
    snapshot: u8,
}

impl JoyPad {
    pub fn new() -> Self {
        Self {
            strobe: false,
            state: State::from_bits_truncate(0),
            snapshot: 0,
        }
    }

    pub fn strobe(&mut self, v: u8) {
        if self.strobe {
            self.snapshot = self.state.bits();
        }
        self.strobe = v & 0x1 != 0;
    }

    pub fn read(&mut self) -> u8 {
        if self.strobe {
            self.state.contains(State::A) as u8
        } else {
            let output = self.snapshot & 0x1;
            self.snapshot >>= 1;
            self.snapshot |= 0x80;
            output
        }
    }

    pub fn update(&mut self, button: Button, pressed: bool) {
        match button {
            Button::A => self.state.set(State::A, pressed),
            Button::B => self.state.set(State::B, pressed),
            Button::Select => self.state.set(State::SELECT, pressed),
            Button::Start => self.state.set(State::START, pressed),
            Button::Up => self.state.set(State::UP, pressed),
            Button::Down => self.state.set(State::DOWN, pressed),
            Button::Left => self.state.set(State::LEFT, pressed),
            Button::Right => self.state.set(State::RIGHT, pressed),
        }
    }
}
