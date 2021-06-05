use bitflags::bitflags;

bitflags! {
    /// State of the controller buttons
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

/// Controller port of the NES
pub enum JoyPort {
    Port1,
    Port2,
}

/// Buttons on the NES controller
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

/// NES controller
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

    /// Strobes the controller
    ///
    /// If bit 0 is set, the controller continuously latches the current state of the buttons.
    /// If bit 0 is clear, stops latching
    pub fn strobe(&mut self, v: u8) {
        if self.strobe {
            self.snapshot = self.state.bits();
        }
        self.strobe = v & 0x1 != 0;
    }

    /// Reads the controller input data
    ///
    /// If the controller is strobing, returns the state of A button. Otherwise, shifts out the state of the button to read.
    ///
    /// Buttons are always read in the order: A, B, Select, Start, Up, Down, Left, Right
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

    /// Updates the state of the buttons
    ///
    /// This function is used to update the buttons from SDL2 keyboard events
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
