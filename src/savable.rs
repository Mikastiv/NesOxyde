use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

pub trait Savable {
    /// Save state
    fn save(&self, _output: &mut BufWriter<File>) -> bincode::Result<()> {
        Ok(())
    }

    /// Load state
    fn load(&mut self, _input: &mut BufReader<File>) -> bincode::Result<()> {
        Ok(())
    }
}
