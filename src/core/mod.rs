/// Milliseconds per one simulated second (`Game::tick` boundary and runtime poll interval).
pub const SIMULATED_SECOND_MS: u64 = 1000;

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
    /// One grid cell per simulated second along `Game::route_to_mission`.
    MovingToMission,
    Gathering { seconds_left: u32 },
    /// One grid cell per second back along the same route (index toward 0).
    MovingToBase,
}

#[derive(Debug)]
pub struct Squad {
    pub state: SquadState,
    /// Index into `Game::route_to_mission` while moving or on mission cell while gathering.
    pub path_index: usize,
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
                path_index: 0,
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
    /// Grid cells from first step off-base toward mission (last cell is mission), map-inner size `(route_map_w, route_map_h)`.
    pub route_to_mission: Vec<(u16, u16)>,
    pub route_map_w: u16,
    pub route_map_h: u16,
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
            route_to_mission: Vec::new(),
            route_map_w: 0,
            route_map_h: 0,
        }
    }

    pub fn tick(&mut self, ms: u64) {
        self.accum_ms += ms;
        while self.accum_ms >= SIMULATED_SECOND_MS {
            self.accum_ms -= SIMULATED_SECOND_MS;
            self.ticks += 1;
            self.simulate_second();
        }
    }

    fn simulate_second(&mut self) {
        let route_len = self.route_to_mission.len();
        let squad = &mut self.units.squads[0];
        match squad.state {
            SquadState::IdleAtBase => {
                if self.world.available_gather_missions > 0 {
                    self.world.available_gather_missions -= 1;
                    squad.state = SquadState::MovingToMission;
                    squad.path_index = 0;
                }
            }
            SquadState::MovingToMission => {
                if route_len == 0 {
                    squad.state = SquadState::Gathering {
                        seconds_left: GATHER_DURATION_SECS,
                    };
                    squad.path_index = 0;
                    return;
                }
                let last = route_len - 1;
                if squad.path_index < last {
                    squad.path_index += 1;
                } else {
                    squad.state = SquadState::Gathering {
                        seconds_left: GATHER_DURATION_SECS,
                    };
                }
            }
            SquadState::Gathering { seconds_left } => match seconds_left {
                1 => {
                    self.base.silver = self.base.silver.saturating_add(SILVER_PER_GATHER);
                    squad.state = SquadState::MovingToBase;
                    if route_len > 0 {
                        squad.path_index = route_len - 1;
                    }
                }
                n => {
                    squad.state = SquadState::Gathering {
                        seconds_left: n - 1,
                    };
                }
            },
            SquadState::MovingToBase => {
                if route_len == 0 {
                    squad.state = SquadState::IdleAtBase;
                    squad.path_index = 0;
                    self.world.available_gather_missions = self
                        .world
                        .available_gather_missions
                        .saturating_add(1);
                    return;
                }
                if squad.path_index > 0 {
                    squad.path_index -= 1;
                } else {
                    squad.state = SquadState::IdleAtBase;
                    squad.path_index = 0;
                    self.world.available_gather_missions = self
                        .world
                        .available_gather_missions
                        .saturating_add(1);
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sync_route_like_app(g: &mut Game) {
        use crate::ui::{route_outbound_cells, MAP_HEIGHT, MAP_WIDTH};

        if g.route_map_w == MAP_WIDTH && g.route_map_h == MAP_HEIGHT {
            return;
        }
        g.route_to_mission = route_outbound_cells();
        g.route_map_w = MAP_WIDTH;
        g.route_map_h = MAP_HEIGHT;
        if !g.route_to_mission.is_empty() {
            let max_i = g.route_to_mission.len() - 1;
            g.units.squads[0].path_index = g.units.squads[0].path_index.min(max_i);
        }
    }

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
        g.tick(SIMULATED_SECOND_MS);
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
        g.tick(2 * SIMULATED_SECOND_MS + 500);
        assert_eq!(g.ticks, 2);
        assert_eq!(g.accum_ms, 500);
    }

    #[test]
    fn autonomous_gather_loop_adds_silver_every_gather_cycle() {
        let mut g = Game::new();
        sync_route_like_app(&mut g);
        assert!(
            !g.route_to_mission.is_empty(),
            "route needed for travel simulation"
        );

        assert_eq!(g.world.available_gather_missions, 1);
        assert_eq!(g.base.silver, 0);

        let wait_silver = |g: &mut Game, target: u64| {
            for _ in 0..800 {
                if g.base.silver >= target {
                    return;
                }
                g.tick(SIMULATED_SECOND_MS);
            }
            panic!("timeout waiting for silver {target}");
        };
        let wait_home = |g: &mut Game| {
            for _ in 0..800 {
                if g.world.available_gather_missions > 0
                    && matches!(g.units.squads[0].state, SquadState::IdleAtBase)
                {
                    return;
                }
                g.tick(SIMULATED_SECOND_MS);
            }
            panic!("timeout waiting to return home");
        };

        wait_silver(&mut g, SILVER_PER_GATHER);
        assert_eq!(g.base.silver, SILVER_PER_GATHER);
        wait_home(&mut g);
        assert_eq!(g.world.available_gather_missions, 1);
        assert_eq!(g.units.squads[0].state, SquadState::IdleAtBase);

        wait_silver(&mut g, 2 * SILVER_PER_GATHER);
        assert_eq!(g.base.silver, 2 * SILVER_PER_GATHER);
        wait_home(&mut g);
        assert_eq!(g.world.available_gather_missions, 1);
        assert_eq!(g.units.squads[0].state, SquadState::IdleAtBase);
    }

    #[test]
    fn moving_to_mission_advances_one_route_index_per_tick() {
        let mut g = Game::new();
        sync_route_like_app(&mut g);
        let last = g.route_to_mission.len().saturating_sub(1);
        if last == 0 {
            return;
        }

        g.tick(SIMULATED_SECOND_MS);
        assert_eq!(g.units.squads[0].state, SquadState::MovingToMission);
        assert_eq!(g.units.squads[0].path_index, 0);

        let mut prev = g.route_to_mission[0];
        for _ in 0..last {
            g.tick(SIMULATED_SECOND_MS);
            assert_eq!(g.units.squads[0].state, SquadState::MovingToMission);
            let cur = g.route_to_mission[g.units.squads[0].path_index];
            let dc = prev.0.abs_diff(cur.0);
            let dr = prev.1.abs_diff(cur.1);
            assert!(dc <= 1 && dr <= 1 && (dc + dr > 0));
            prev = cur;
        }
    }
}
