#[derive(Clone, Copy)]
pub struct Tile {
    pub lo: u8,
    pub hi: u8,
    pub attr: u8,
    pub id: u8,
}

impl Tile {
    pub fn new() -> Self {
        Self {
            lo: 0,
            hi: 0,
            attr: 0,
            id: 0,
        }
    }
}
