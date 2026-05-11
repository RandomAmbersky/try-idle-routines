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

    fn start_route_to_base_from(&mut self, from_cell: (u16, u16)) {
        let steps =
            crate::map_geometry::route_outbound_cells_from(from_cell, self.world.base_cell);
        let mut home = Vec::with_capacity(steps.len().saturating_add(1));
        home.push(from_cell);
        home.extend(steps);
        self.route_to_mission = home;
    }

    fn simulate_second(&mut self) {
        let route_len = self.route_to_mission.len();
        let state = self.units.squads[0].state;
        match state {
            SquadState::IdleAtBase => {
                if !self
                    .world
                    .active_missions
                    .iter()
                    .any(|m| m.silver_remaining > 0)
                {
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
                let squad = &mut self.units.squads[0];
                squad.state = SquadState::MovingToMission;
                squad.path_index = 0;
            }
            SquadState::MovingToMission => {
                if route_len == 0 {
                    let squad = &mut self.units.squads[0];
                    squad.state = SquadState::Gathering {
                        seconds_left: GATHER_DURATION_SECS,
                    };
                    squad.path_index = 0;
                    return;
                }
                let last = route_len - 1;
                let squad = &mut self.units.squads[0];
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
                    self.gathering_just_completed = true;
                    let mission_cell = *self
                        .route_to_mission
                        .last()
                        .expect("gathering requires mission cell in route_to_mission");

                    let mission_idx = self
                        .world
                        .active_missions
                        .iter()
                        .position(|m| m.cell == mission_cell)
                        .expect("gathering mission must exist in active_missions");

                    let (hold_full, take) = {
                        let squad = &mut self.units.squads[0];
                        let room = squad
                            .cargo_capacity
                            .saturating_sub(squad.cargo_silver);
                        let pool = self.world.active_missions[mission_idx].silver_remaining;
                        let take = SILVER_PER_GATHER.min(pool).min(room);

                        squad.cargo_silver = squad.cargo_silver.saturating_add(take);
                        let hold_full = squad.cargo_silver >= squad.cargo_capacity;
                        (hold_full, take)
                    };

                    self.world.active_missions[mission_idx].silver_remaining = self.world
                        .active_missions[mission_idx]
                        .silver_remaining
                        .saturating_sub(take);

                    if self.world.active_missions[mission_idx].silver_remaining == 0 {
                        self.world.active_missions.remove(mission_idx);
                    }

                    let site_empty = !self
                        .world
                        .active_missions
                        .iter()
                        .any(|m| m.cell == mission_cell);

                    if hold_full {
                        self.start_route_to_base_from(mission_cell);
                        let squad = &mut self.units.squads[0];
                        squad.state = SquadState::MovingToBase;
                        squad.path_index = 0;
                    } else if site_empty {
                        let has_other_with_silver = self
                            .world
                            .active_missions
                            .iter()
                            .any(|m| m.silver_remaining > 0);

                        if has_other_with_silver {
                            let mission_i = pick_closest_gather_mission_index(
                                mission_cell,
                                &self.world.active_missions,
                            )
                            .expect("non-empty mission list has a closest mission");
                            let next_cell = self.world.active_missions[mission_i].cell;
                            self.route_to_mission =
                                crate::map_geometry::route_outbound_cells_from(
                                    mission_cell,
                                    next_cell,
                                );
                            let squad = &mut self.units.squads[0];
                            squad.state = SquadState::MovingToMission;
                            squad.path_index = 0;
                        } else {
                            self.start_route_to_base_from(mission_cell);
                            let squad = &mut self.units.squads[0];
                            squad.state = SquadState::MovingToBase;
                            squad.path_index = 0;
                        }
                    } else {
                        let squad = &mut self.units.squads[0];
                        squad.state = SquadState::Gathering {
                            seconds_left: GATHER_DURATION_SECS,
                        };
                        squad.path_index = 0;
                    }
                }
                n => {
                    let squad = &mut self.units.squads[0];
                    squad.state = SquadState::Gathering {
                        seconds_left: n - 1,
                    };
                }
            },
            SquadState::MovingToBase => {
                if route_len == 0 {
                    let squad = &mut self.units.squads[0];
                    let unload = squad.cargo_silver;
                    if unload > 0 {
                        self.base.silver = self.base.silver.saturating_add(unload);
                        squad.cargo_silver = 0;
                    }
                    squad.state = SquadState::IdleAtBase;
                    squad.path_index = 0;
                    return;
                }
                let last = route_len - 1;
                let squad = &mut self.units.squads[0];
                if squad.path_index < last {
                    squad.path_index += 1;
                } else {
                    let unload = squad.cargo_silver;
                    if unload > 0 {
                        self.base.silver = self.base.silver.saturating_add(unload);
                        squad.cargo_silver = 0;
                    }
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
    fn gather_completion_keeps_mission_until_pool_empty() {
        let mission = (14u16, 50u16);
        let mut g = Game::new_from_layout_for_test(
            (10, 50),
            vec![
                GatherMission::new(mission, SILVER_PER_GATHER + 5),
                GatherMission::new((30, 50), 10),
            ],
        );

        g.route_to_mission =
            crate::map_geometry::route_outbound_cells_from(g.world.base_cell, mission);
        assert_eq!(g.route_to_mission.last(), Some(&mission));
        g.units.squads[0].state = SquadState::Gathering { seconds_left: 1 };
        g.units.squads[0].cargo_capacity = 100;

        g.tick(SIMULATED_SECOND_MS);

        assert_eq!(g.base.silver, 0, "silver only increases on base unload");
        assert_eq!(g.units.squads[0].cargo_silver, SILVER_PER_GATHER);
        assert!(
            g.world
                .active_missions
                .iter()
                .any(|m| m.cell == mission && m.silver_remaining == 5),
            "mission must remain active until pool is fully depleted"
        );
        assert!(matches!(g.units.squads[0].state, SquadState::Gathering { .. }));
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

    #[test]
    fn chain_to_second_mission_without_base_silver_when_first_depletes() {
        let a = (12u16, 48u16);
        let b = (20u16, 50u16);
        let mut g = Game::new_from_layout_for_test(
            (10, 50),
            vec![GatherMission::new(a, 20), GatherMission::new(b, 100)],
        );

        g.units.squads[0].cargo_capacity = 30;
        g.route_to_mission = crate::map_geometry::route_outbound_cells_from(g.world.base_cell, a);
        g.units.squads[0].state = SquadState::Gathering { seconds_left: 1 };

        // First gather completion takes 10; second completion (after countdown) takes remaining 10.
        // Wait until mission `a` is removed and squad starts moving to another site (not to base).
        for _ in 0..800 {
            g.tick(SIMULATED_SECOND_MS);
            let a_present = g.world.active_missions.iter().any(|m| m.cell == a);
            if !a_present {
                break;
            }
        }

        assert_eq!(g.base.silver, 0, "must not unload at base when chaining sites");
        assert!(
            !g.world.active_missions.iter().any(|m| m.cell == a),
            "first mission should be removed only once its pool hits zero"
        );
        assert!(
            g.units.squads[0].cargo_silver > 0 && g.units.squads[0].cargo_silver < 30,
            "should carry partial cargo while chaining"
        );
        assert!(matches!(g.units.squads[0].state, SquadState::MovingToMission));
        assert_eq!(g.route_to_mission.last(), Some(&b));
    }

    #[test]
    fn hold_full_returns_while_mission_has_silver_left() {
        let mission = (14u16, 50u16);
        let mut g = Game::new_from_layout_for_test(
            (10, 50),
            vec![GatherMission::new(mission, 100)],
        );

        g.units.squads[0].cargo_capacity = 25;
        g.route_to_mission =
            crate::map_geometry::route_outbound_cells_from(g.world.base_cell, mission);
        g.units.squads[0].state = SquadState::Gathering { seconds_left: 1 };

        // Each gather completion transfers up to SILVER_PER_GATHER=10.
        // With a 3-second gather cycle, we need 3 completions (10+10+5) to fill to 25.
        for _ in 0..800 {
            if matches!(g.units.squads[0].state, SquadState::MovingToBase) {
                break;
            }
            g.tick(SIMULATED_SECOND_MS);
        }

        assert_eq!(g.base.silver, 0, "no base credit until unload");
        assert_eq!(g.units.squads[0].cargo_silver, 25);
        assert!(
            g.world
                .active_missions
                .iter()
                .any(|m| m.cell == mission && m.silver_remaining == 75),
            "mission remains active with remaining pool"
        );
        assert!(matches!(g.units.squads[0].state, SquadState::MovingToBase));
        assert_eq!(g.route_to_mission.first(), Some(&mission));
        assert_eq!(
            g.route_to_mission.last(),
            Some(&g.world.base_cell),
            "return route must end on current base"
        );
    }

    #[test]
    fn last_mission_empty_partial_hold_returns_to_base_and_unloads() {
        let mission = (14u16, 50u16);
        let mut g = Game::new_from_layout_for_test(
            (10, 50),
            vec![GatherMission::new(mission, 15)],
        );

        g.units.squads[0].cargo_capacity = 40;
        g.route_to_mission =
            crate::map_geometry::route_outbound_cells_from(g.world.base_cell, mission);
        g.units.squads[0].state = SquadState::Gathering { seconds_left: 1 };

        // Two gather completions (10 then 5) should fully deplete the mission.
        for _ in 0..800 {
            g.tick(SIMULATED_SECOND_MS);
            if g.world.active_missions.is_empty() {
                break;
            }
        }

        assert!(g.world.active_missions.is_empty(), "mission removed at pool==0");
        assert_eq!(g.units.squads[0].cargo_silver, 15);
        assert!(matches!(g.units.squads[0].state, SquadState::MovingToBase));
        assert_eq!(g.base.silver, 0);

        // Wait until idle at base (unload happens on arrival).
        for _ in 0..800 {
            if matches!(g.units.squads[0].state, SquadState::IdleAtBase) {
                break;
            }
            g.tick(SIMULATED_SECOND_MS);
        }
        assert!(matches!(g.units.squads[0].state, SquadState::IdleAtBase));
        assert_eq!(g.units.squads[0].cargo_silver, 0);
        assert_eq!(g.base.silver, 15);
    }

    #[test]
    fn chaining_picks_closest_next_mission_from_current_cell() {
        let base = (10u16, 50u16);
        let empty_site = (12u16, 48u16);
        let closer = (13u16, 48u16);
        let farther = (30u16, 20u16);

        let mut g = Game::new_from_layout_for_test(
            base,
            vec![
                // This site will be depleted first.
                GatherMission::new(empty_site, 10),
                // Two remaining candidates: must pick the closer from `empty_site`.
                GatherMission::new(closer, 100),
                GatherMission::new(farther, 100),
            ],
        );

        g.units.squads[0].cargo_capacity = 30;
        g.route_to_mission = crate::map_geometry::route_outbound_cells_from(base, empty_site);
        g.units.squads[0].state = SquadState::Gathering { seconds_left: 1 };

        // One gather completion fully depletes `empty_site`.
        g.tick(SIMULATED_SECOND_MS);

        assert!(
            !g.world.active_missions.iter().any(|m| m.cell == empty_site),
            "depleted site should be removed from active_missions"
        );
        assert!(matches!(g.units.squads[0].state, SquadState::MovingToMission));
        assert_eq!(
            g.route_to_mission.last(),
            Some(&closer),
            "must pick closest next mission from current cell"
        );
    }
}
