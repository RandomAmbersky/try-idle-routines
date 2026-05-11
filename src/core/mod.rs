/// Milliseconds per one simulated second (`Game::tick` boundary and runtime poll interval).
pub const SIMULATED_SECOND_MS: u64 = 100;

mod world_gen;
pub use world_gen::{generate_base_and_three_missions, pick_closest_mission_index};

use rand::thread_rng;

const GATHER_DURATION_SECS: u32 = 3;
const SILVER_PER_GATHER: u64 = 10;

#[derive(Debug, Default)]
pub struct World {
    pub base_cell: (u16, u16),
    pub active_missions: Vec<(u16, u16)>,
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
    Gathering {
        seconds_left: u32,
    },
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
    pub gathering_just_completed: bool,
}

impl Game {
    pub fn new() -> Self {
        let (base_cell, active_missions) = generate_base_and_three_missions(&mut thread_rng());
        Self::new_from_layout_for_test(base_cell, active_missions)
    }

    pub fn new_from_layout_for_test(
        base_cell: (u16, u16),
        active_missions: Vec<(u16, u16)>,
    ) -> Self {
        Self {
            world: World {
                base_cell,
                active_missions,
            },
            base: Base::default(),
            units: Units::default(),
            ticks: 0,
            accum_ms: 0,
            route_to_mission: Vec::new(),
            route_map_w: crate::constants::MAP_WIDTH,
            route_map_h: crate::constants::MAP_HEIGHT,
            gathering_just_completed: false,
        }
    }

    pub fn tick(&mut self, ms: u64) {
        self.gathering_just_completed = false;
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
                if self.world.active_missions.is_empty() {
                    return;
                }
                let mission_i =
                    pick_closest_mission_index(self.world.base_cell, &self.world.active_missions)
                        .expect("non-empty mission list has a closest mission");
                let mission = self.world.active_missions[mission_i];
                self.route_to_mission =
                    crate::map_geometry::route_outbound_cells_from(self.world.base_cell, mission);
                squad.state = SquadState::MovingToMission;
                squad.path_index = 0;
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
                    self.gathering_just_completed = true;
                    if let Some(&mission) = self.route_to_mission.last() {
                        if let Some(i) = self
                            .world
                            .active_missions
                            .iter()
                            .position(|&cell| cell == mission)
                        {
                            self.world.active_missions.remove(i);
                        }
                    }
                    squad.state = SquadState::MovingToBase;
                    if route_len > 0 {
                        squad.path_index = route_len - 1;
                    } else {
                        squad.path_index = 0;
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
                    return;
                }
                if squad.path_index > 0 {
                    squad.path_index -= 1;
                } else {
                    squad.state = SquadState::IdleAtBase;
                    squad.path_index = 0;
                }
            }
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
    fn new_game_has_three_active_gather_missions() {
        let g = Game::new();
        assert_eq!(g.world.active_missions.len(), 3);
    }

    #[test]
    fn new_game_single_squad_idle_at_base() {
        let g = Game::new();
        assert_eq!(g.units.squads.len(), 1);
        assert_eq!(g.units.squads[0].state, SquadState::IdleAtBase);
    }

    #[test]
    fn tick_one_simulated_second_increments_once() {
        let mut g = Game::new();
        g.tick(SIMULATED_SECOND_MS);
        assert_eq!(g.ticks, 1);
        assert_eq!(g.accum_ms, 0);
    }

    #[test]
    fn tick_accumulates_partial_ms() {
        let mut g = Game::new();
        g.tick(40);
        assert_eq!(g.ticks, 0);
        assert_eq!(g.accum_ms, 40);

        g.tick(60);
        assert_eq!(g.ticks, 1);
        assert_eq!(g.accum_ms, 0);
    }

    #[test]
    fn tick_can_roll_over_multiple_seconds() {
        let mut g = Game::new();
        g.tick(2 * SIMULATED_SECOND_MS + 50);
        assert_eq!(g.ticks, 2);
        assert_eq!(g.accum_ms, 50);
    }

    #[test]
    fn autonomous_gather_loop_adds_silver_every_gather_cycle() {
        let mut g = Game::new_from_layout_for_test((10, 50), vec![(12, 48), (20, 50)]);

        assert_eq!(g.world.active_missions.len(), 2);
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
                if matches!(g.units.squads[0].state, SquadState::IdleAtBase) {
                    return;
                }
                g.tick(SIMULATED_SECOND_MS);
            }
            panic!("timeout waiting to return home");
        };

        wait_silver(&mut g, SILVER_PER_GATHER);
        assert_eq!(g.base.silver, SILVER_PER_GATHER);
        assert_eq!(g.world.active_missions.len(), 1);
        wait_home(&mut g);
        assert_eq!(g.units.squads[0].state, SquadState::IdleAtBase);

        wait_silver(&mut g, 2 * SILVER_PER_GATHER);
        assert_eq!(g.base.silver, 2 * SILVER_PER_GATHER);
        assert!(g.world.active_missions.is_empty());
        wait_home(&mut g);
        assert_eq!(g.units.squads[0].state, SquadState::IdleAtBase);
    }

    #[test]
    fn moving_to_mission_advances_one_route_index_per_tick() {
        let mut g = Game::new_from_layout_for_test((10, 50), vec![(16, 48)]);
        g.tick(SIMULATED_SECOND_MS);
        let last = g.route_to_mission.len().saturating_sub(1);
        if last == 0 {
            return;
        }

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

    #[test]
    fn no_dispatch_when_missions_empty() {
        let mut g = Game::new_from_layout_for_test((5, 5), vec![]);
        g.units.squads[0].state = SquadState::IdleAtBase;

        for _ in 0..20 {
            g.tick(SIMULATED_SECOND_MS);
        }

        assert_eq!(g.units.squads[0].state, SquadState::IdleAtBase);
        assert!(g.route_to_mission.is_empty());
    }
}
