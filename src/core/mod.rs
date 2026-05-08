#[derive(Debug, Default)]
pub struct World;

#[derive(Debug, Default)]
pub struct Base;

#[derive(Debug, Default)]
pub struct Units;

#[derive(Debug)]
pub struct Game {
    pub world: World,
    pub base: Base,
    pub units: Units,
}

impl Game {
    pub fn new() -> Self {
        Self {
            world: World::default(),
            base: Base::default(),
            units: Units::default(),
        }
    }
}

