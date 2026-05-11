const GATHER_DURATION_SECS: u32 = 3;
const SILVER_PER_GATHER: u64 = 10;

#[derive(Debug, Default)]
pub struct World {
    pub available_gather_missions: u32,
}

#[derive(Debug, Default)]
pub struct Base {
    pub silver: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SquadState {
    IdleAtBase,
    /// Moving from base toward the mission site (UI shows on map).
    TravelingToMission { seconds_left: u32 },
    Gathering { seconds_left: u32 },
    /// Heading home after completing work on site.
    ReturningToBase { seconds_left: u32 },
}

#[derive(Debug)]
pub struct Squad {
    pub state: SquadState,
}

#[derive(Debug)]
pub struct Units {
    pub squads: Vec<Squad>,
}

impl Default for Units {
    fn default() -> Self {
        Self {
            squads: vec![Squad {
                state: SquadState::IdleAtBase,
            }],
        }
    }
}

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
            world: World {
                available_gather_missions: 1,
            },
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
            self.simulate_second();
        }
    }

    fn simulate_second(&mut self) {
        let squad = &mut self.units.squads[0];
        match squad.state {
            SquadState::IdleAtBase => {
                if self.world.available_gather_missions > 0 {
                    self.world.available_gather_missions -= 1;
                    squad.state = SquadState::TravelingToMission { seconds_left: 1 };
                }
            }
            SquadState::TravelingToMission { seconds_left } => match seconds_left {
                1 => {
                    squad.state = SquadState::Gathering {
                        seconds_left: GATHER_DURATION_SECS,
                    };
                }
                n => {
                    squad.state = SquadState::TravelingToMission {
                        seconds_left: n - 1,
                    };
                }
            },
            SquadState::Gathering { seconds_left } => match seconds_left {
                1 => {
                    self.base.silver = self.base.silver.saturating_add(SILVER_PER_GATHER);
                    squad.state = SquadState::ReturningToBase { seconds_left: 1 };
                }
                n => {
                    squad.state = SquadState::Gathering {
                        seconds_left: n - 1,
                    };
                }
            },
            SquadState::ReturningToBase { seconds_left } => match seconds_left {
                1 => {
                    squad.state = SquadState::IdleAtBase;
                    self.world.available_gather_missions = self
                        .world
                        .available_gather_missions
                        .saturating_add(1);
                }
                n => {
                    squad.state = SquadState::ReturningToBase {
                        seconds_left: n - 1,
                    };
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_base_has_zero_silver() {
        let g = Game::new();
        assert_eq!(g.base.silver, 0);
    }

    #[test]
    fn new_game_has_one_gather_mission_available() {
        let g = Game::new();
        assert_eq!(g.world.available_gather_missions, 1);
    }

    #[test]
    fn new_game_single_squad_idle_at_base() {
        let g = Game::new();
        assert_eq!(g.units.squads.len(), 1);
        assert_eq!(g.units.squads[0].state, SquadState::IdleAtBase);
    }

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

    #[test]
    fn autonomous_gather_loop_adds_silver_every_gather_cycle() {
        let mut g = Game::new();
        assert_eq!(g.world.available_gather_missions, 1);
        assert_eq!(g.base.silver, 0);

        g.tick(1000); // 1: leave base, travel to mission
        assert_eq!(g.world.available_gather_missions, 0);
        assert_eq!(
            g.units.squads[0].state,
            SquadState::TravelingToMission { seconds_left: 1 }
        );

        g.tick(1000); // 2: arrive on site, start gathering
        assert_eq!(
            g.units.squads[0].state,
            SquadState::Gathering {
                seconds_left: GATHER_DURATION_SECS,
            }
        );

        g.tick(1000);
        assert_eq!(
            g.units.squads[0].state,
            SquadState::Gathering {
                seconds_left: GATHER_DURATION_SECS - 1,
            }
        );

        g.tick(1000);
        assert_eq!(
            g.units.squads[0].state,
            SquadState::Gathering { seconds_left: 1 }
        );

        g.tick(1000); // 5: finish work, silver, return leg
        assert_eq!(
            g.units.squads[0].state,
            SquadState::ReturningToBase { seconds_left: 1 }
        );
        assert_eq!(g.base.silver, SILVER_PER_GATHER);

        g.tick(1000); // 6: home, mission slot back
        assert_eq!(g.units.squads[0].state, SquadState::IdleAtBase);
        assert_eq!(g.world.available_gather_missions, 1);

        // Second full gather cycle
        g.tick(1000);
        assert_eq!(g.world.available_gather_missions, 0);
        assert_eq!(
            g.units.squads[0].state,
            SquadState::TravelingToMission { seconds_left: 1 }
        );

        g.tick(1000);
        assert_eq!(
            g.units.squads[0].state,
            SquadState::Gathering {
                seconds_left: GATHER_DURATION_SECS,
            }
        );

        g.tick(1000);
        assert_eq!(
            g.units.squads[0].state,
            SquadState::Gathering {
                seconds_left: GATHER_DURATION_SECS - 1,
            }
        );

        g.tick(1000);
        assert_eq!(
            g.units.squads[0].state,
            SquadState::Gathering { seconds_left: 1 }
        );

        g.tick(1000);
        assert_eq!(
            g.units.squads[0].state,
            SquadState::ReturningToBase { seconds_left: 1 }
        );
        assert_eq!(g.base.silver, 2 * SILVER_PER_GATHER);

        g.tick(1000);
        assert_eq!(g.units.squads[0].state, SquadState::IdleAtBase);
        assert_eq!(g.world.available_gather_missions, 1);
    }
}
