/// Milliseconds per one simulated second (`Game::tick` boundary and runtime poll interval).
pub const SIMULATED_SECOND_MS: u64 = 100;

mod world_gen;
pub use world_gen::{generate_base_and_three_missions, pick_closest_gather_mission_index};

use rand::thread_rng;

const GATHER_DURATION_SECS: u32 = 3;
const SILVER_PER_GATHER: u64 = 10;

/// Initial and remaining silver on a gather site (spec: mission stays until `silver_remaining == 0`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GatherMission {
    pub cell: (u16, u16),
    pub silver_initial: u64,
    pub silver_remaining: u64,
}

impl GatherMission {
    pub fn new(cell: (u16, u16), silver: u64) -> Self {
        Self {
            cell,
            silver_initial: silver,
            silver_remaining: silver,
        }
    }
}

pub const DEFAULT_MISSION_SILVER_POOL: u64 = 100;
pub const DEFAULT_SQUAD_CARGO_CAPACITY: u64 = 30;

#[derive(Debug, Default)]
pub struct World {
    pub base_cell: (u16, u16),
    pub active_missions: Vec<GatherMission>,
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
    /// One grid cell per second along the **return** route (mission→base), recomputed when return starts.
    MovingToBase,
}

#[derive(Debug)]
pub struct Squad {
    pub state: SquadState,
    /// Index into `Game::route_to_mission` (outbound: forward to mission; return: forward to base).
    pub path_index: usize,
    pub cargo_silver: u64,
    pub cargo_capacity: u64,
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
                cargo_silver: 0,
                cargo_capacity: DEFAULT_SQUAD_CARGO_CAPACITY,
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
    /// Outbound: first step off base → mission (last = mission). Return: mission cell, then steps toward base (last = base).
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
        active_missions: Vec<GatherMission>,
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
                let mission_i = pick_closest_gather_mission_index(
                    self.world.base_cell,
                    &self.world.active_missions,
                )
                .expect("non-empty mission list has a closest mission");
                let mission = self.world.active_missions[mission_i].cell;
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
                    let mission_cell = self.route_to_mission.last().copied();
                    if let Some(mission) = mission_cell {
                        if let Some(i) = self
                            .world
                            .active_missions
                            .iter()
                            .position(|m| m.cell == mission)
                        {
                            self.world.active_missions.remove(i);
                        }
                    }
                    // New path to base (not reverse of outbound) so future map changes stay valid.
                    self.route_to_mission = match mission_cell {
                        Some(mission) => {
                            let steps = crate::map_geometry::route_outbound_cells_from(
                                mission,
                                self.world.base_cell,
                            );
                            let mut home = Vec::with_capacity(steps.len().saturating_add(1));
                            home.push(mission);
                            home.extend(steps);
                            home
                        }
                        None => Vec::new(),
                    };
                    squad.state = SquadState::MovingToBase;
                    squad.path_index = 0;
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
                let last = route_len - 1;
                if squad.path_index < last {
                    squad.path_index += 1;
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
        let mut g = Game::new_from_layout_for_test(
            (10, 50),
            vec![
                GatherMission::new((12, 48), DEFAULT_MISSION_SILVER_POOL),
                GatherMission::new((20, 50), DEFAULT_MISSION_SILVER_POOL),
            ],
        );

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
        let mut g = Game::new_from_layout_for_test(
            (10, 50),
            vec![GatherMission::new((16, 48), DEFAULT_MISSION_SILVER_POOL)],
        );
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

    /// Spec: mission is removed from the active list when gathering completes (before base is reached).
    #[test]
    fn gather_completion_drops_mission_from_active_list_while_returning() {
        let mission = (14u16, 50u16);
        let mut g = Game::new_from_layout_for_test(
            (10, 50),
            vec![
                GatherMission::new(mission, DEFAULT_MISSION_SILVER_POOL),
                GatherMission::new((30, 50), DEFAULT_MISSION_SILVER_POOL),
            ],
        );
        g.route_to_mission =
            crate::map_geometry::route_outbound_cells_from(g.world.base_cell, mission);
        assert!(
            g.route_to_mission.last() == Some(&mission),
            "route should end on mission cell"
        );
        g.units.squads[0].state = SquadState::Gathering { seconds_left: 1 };

        assert_eq!(g.world.active_missions.len(), 2);
        g.tick(SIMULATED_SECOND_MS);

        assert!(
            !g.world.active_missions.iter().any(|m| m.cell == mission),
            "marker must not correspond to an active mission during return"
        );
        assert_eq!(g.base.silver, SILVER_PER_GATHER);
        assert!(matches!(g.units.squads[0].state, SquadState::MovingToBase));
        assert!(g.gathering_just_completed);
        assert_eq!(g.route_to_mission.first(), Some(&mission));
        assert_eq!(
            g.route_to_mission.last(),
            Some(&g.world.base_cell),
            "return route must end on current base"
        );
    }
}
