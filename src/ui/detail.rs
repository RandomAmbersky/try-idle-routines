use ratatui::text::{Line, Text};

use crate::core::{Game, SquadState};

use super::{MainLayout, Selection, SquadId};

pub const BASE_IDLE_ROSTER_START_REL_Y: u16 = 4;
pub const MISSION_ON_SITE_REL_Y: u16 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailMouseTarget {
    None,
    Close,
    BaseSquadRow { squad_index: usize },
    MissionOnSiteRow { squad_index: usize },
}

pub fn detail_mouse_target(
    layout: &MainLayout,
    game: &Game,
    selection: Selection,
    column: u16,
    row: u16,
) -> DetailMouseTarget {
    if contains(layout.close_x_rect, column, row) {
        return DetailMouseTarget::Close;
    }

    if !contains(layout.detail_inner, column, row) {
        return DetailMouseTarget::None;
    }

    let rel_y = row - layout.detail_inner.y;

    match selection {
        Selection::Base => {
            let mut idle_index = 0u16;
            for (squad_index, squad) in game.units.squads.iter().enumerate() {
                if squad.state == SquadState::IdleAtBase {
                    if rel_y == BASE_IDLE_ROSTER_START_REL_Y + idle_index {
                        return DetailMouseTarget::BaseSquadRow { squad_index };
                    }
                    idle_index += 1;
                }
            }
            DetailMouseTarget::None
        }
        Selection::Mission => {
            if mission_squad_row_active(game) && rel_y == MISSION_ON_SITE_REL_Y {
                DetailMouseTarget::MissionOnSiteRow { squad_index: 0 }
            } else {
                DetailMouseTarget::None
            }
        }
        Selection::None | Selection::Squad(_) => DetailMouseTarget::None,
    }
}

pub fn format_detail(game: &Game, selection: Selection) -> Text<'static> {
    let mut lines = Vec::new();

    match selection {
        Selection::None => {
            lines.push(Line::from("Select base or mission."));
        }
        Selection::Base => {
            lines.push(Line::from(" [X]"));
            lines.push(Line::from(format!("Silver: {}", game.base.silver)));
            lines.push(Line::from(""));
            lines.push(Line::from("Roster"));
            push_blank_until(&mut lines, BASE_IDLE_ROSTER_START_REL_Y);
            for (squad_index, squad) in game.units.squads.iter().enumerate() {
                if squad.state == SquadState::IdleAtBase {
                    lines.push(Line::from(format!(
                        "Squad {squad_index}: idle (cargo {}/{})",
                        squad.cargo_silver, squad.cargo_capacity
                    )));
                }
            }
        }
        Selection::Mission => {
            lines.push(Line::from(" [X]"));
            lines.push(Line::from("Mission"));
            lines.push(Line::from(format!(
                "Remaining sites: {}",
                game.world.active_missions.len()
            )));
            let total_site: u64 = game
                .world
                .active_missions
                .iter()
                .map(|m| m.silver_remaining)
                .sum();
            lines.push(Line::from(format!("Silver on sites (sum): {total_site}")));
            lines.push(Line::from(""));
            lines.push(Line::from("Status"));
            push_blank_until(&mut lines, MISSION_ON_SITE_REL_Y);
            if let Some(squad) = game.units.squads.first() {
                let line = match squad.state {
                    SquadState::MovingToMission => {
                        let n = game.route_to_mission.len();
                        format!(
                            "Squad 0: to site ({}/{})",
                            squad.path_index.saturating_add(1),
                            n
                        )
                    }
                    SquadState::Gathering { seconds_left } => {
                        format!("Squad on site: 0 ({seconds_left}s left)")
                    }
                    SquadState::MovingToBase => {
                        let n = game.route_to_mission.len();
                        format!(
                            "Squad 0: to base ({}/{})",
                            squad.path_index.saturating_add(1),
                            n
                        )
                    }
                    SquadState::IdleAtBase => String::new(),
                };
                if !line.is_empty() {
                    lines.push(Line::from(line));
                }
            }
        }
        Selection::Squad(SquadId(squad_index)) => {
            lines.push(Line::from(" [X]"));
            lines.push(Line::from(format!("Squad {squad_index}")));
            if let Some(squad) = game.units.squads.get(squad_index) {
                match squad.state {
                    SquadState::IdleAtBase => lines.push(Line::from("State: idle at base")),
                    SquadState::MovingToMission => lines.push(Line::from(format!(
                        "State: moving to site (step {} of {})",
                        squad.path_index.saturating_add(1),
                        game.route_to_mission.len().max(1)
                    ))),
                    SquadState::Gathering { seconds_left } => lines.push(Line::from(format!(
                        "State: gathering ({seconds_left}s left)"
                    ))),
                    SquadState::MovingToBase => lines.push(Line::from(format!(
                        "State: returning (step {} of {})",
                        squad.path_index.saturating_add(1),
                        game.route_to_mission.len().max(1)
                    ))),
                }
                lines.push(Line::from(format!(
                    "Cargo: {} / {}",
                    squad.cargo_silver, squad.cargo_capacity
                )));
            } else {
                lines.push(Line::from("State: unknown squad"));
            }
        }
    }

    Text::from(lines)
}

fn mission_squad_row_active(game: &Game) -> bool {
    matches!(
        game.units.squads.first().map(|squad| squad.state),
        Some(SquadState::Gathering { .. } | SquadState::MovingToMission | SquadState::MovingToBase)
    )
}

fn contains(rect: ratatui::layout::Rect, column: u16, row: u16) -> bool {
    column >= rect.x
        && column < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

fn push_blank_until(lines: &mut Vec<Line<'static>>, target_rel_y: u16) {
    while lines.len() < usize::from(target_rel_y) {
        lines.push(Line::from(""));
    }
}

#[cfg(test)]
mod detail_tests {
    use ratatui::layout::Rect;

    use crate::core::Game;

    use super::*;
    use crate::ui::compute_layout;

    #[test]
    fn detects_close_hit() {
        let layout = compute_layout(Rect::new(0, 0, 80, 24));
        let column = layout.close_x_rect.x;
        let row = layout.close_x_rect.y;
        let game = Game::new();

        assert_eq!(
            detail_mouse_target(&layout, &game, Selection::Base, column, row),
            DetailMouseTarget::Close
        );
    }

    #[test]
    fn detects_base_idle_squad_row() {
        let layout = compute_layout(Rect::new(0, 0, 80, 24));
        let column = layout.detail_inner.x;
        let row = layout.detail_inner.y + BASE_IDLE_ROSTER_START_REL_Y;
        let game = Game::new();

        assert_eq!(
            detail_mouse_target(&layout, &game, Selection::Base, column, row),
            DetailMouseTarget::BaseSquadRow { squad_index: 0 }
        );
    }
}
