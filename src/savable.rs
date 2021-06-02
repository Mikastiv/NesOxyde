use std::fs::File;

pub trait Savable {
    fn save(&self, _output: &File) -> bincode::Result<()> {
        Ok(())
    }
    fn load(&mut self, _input: &File) -> bincode::Result<()> {
        Ok(())
    }
}
