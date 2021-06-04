use std::fs::File;

pub trait Savable {
    /// Save state
    fn save(&self, _output: &File) -> bincode::Result<()> {
        Ok(())
    }

    /// Load state
    fn load(&mut self, _input: &File) -> bincode::Result<()> {
        Ok(())
    }
}
