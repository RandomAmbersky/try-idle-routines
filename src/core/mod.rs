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
    pub ticks: u64,
    pub accum_ms: u64,
}

impl Game {
    pub fn new() -> Self {
        Self {
            world: World::default(),
            base: Base::default(),
            units: Units::default(),
            ticks: 0,
            accum_ms: 0,
        }
    }

    pub fn tick(&mut self, ms: u64) {
        self.accum_ms += ms;
        while self.accum_ms >= 1000 {
            self.accum_ms -= 1000;
            self.ticks += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_1000ms_increments_once() {
        let mut g = Game::new();
        g.tick(1000);
        assert_eq!(g.ticks, 1);
        assert_eq!(g.accum_ms, 0);
    }

    #[test]
    fn tick_accumulates_partial_ms() {
        let mut g = Game::new();
        g.tick(400);
        assert_eq!(g.ticks, 0);
        assert_eq!(g.accum_ms, 400);

        g.tick(600);
        assert_eq!(g.ticks, 1);
        assert_eq!(g.accum_ms, 0);
    }

    #[test]
    fn tick_can_roll_over_multiple_seconds() {
        let mut g = Game::new();
        g.tick(2500);
        assert_eq!(g.ticks, 2);
        assert_eq!(g.accum_ms, 500);
    }
}
