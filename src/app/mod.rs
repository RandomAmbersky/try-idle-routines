use std::io;

use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use crate::{
    core::Game,
    input::Action,
    tui::Tui,
    ui::{self, DetailMouseTarget, MapTarget, Selection, SquadId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    Running,
    Paused,
}

pub struct App {
    game: Game,
    selection: Selection,
}

impl App {
    pub fn new() -> Self {
        Self {
            game: Game::new(),
            selection: Selection::None,
        }
    }

    pub fn run(mut self) -> io::Result<()> {
        let mut tui = Tui::enter()?;
        let backend = CrosstermBackend::new(tui.stdout());
        let mut terminal = Terminal::new(backend)?;
        let mut mode = RunMode::Running;

        loop {
            let mode_label = match mode {
                RunMode::Running => "running",
                RunMode::Paused => "paused",
            };
            let size = terminal.size()?;
            let area = Rect::new(0, 0, size.width, size.height);
            let layout = ui::compute_layout(area);
            terminal.draw(|f| ui::render(f, &layout, &self.game, mode_label, self.selection))?;

            let action = match mode {
                RunMode::Running => crate::input::read_action_tick_aware(1000)?,
                RunMode::Paused => crate::input::read_action_blocking()?,
            };

            match action {
                Action::Quit => break,
                Action::TogglePause => {
                    mode = match mode {
                        RunMode::Running => RunMode::Paused,
                        RunMode::Paused => RunMode::Running,
                    };
                }
                Action::Tick => {
                    self.game.tick(1000);
                }
                Action::Step => {
                    if mode == RunMode::Paused {
                        self.game.tick(1000);
                    }
                }
                Action::ClearSelection => {
                    self.selection = Selection::None;
                }
                Action::MousePress { column, row } => {
                    let inside_close = column >= layout.close_x_rect.x
                        && column
                            < layout
                                .close_x_rect
                                .x
                                .saturating_add(layout.close_x_rect.width)
                        && row >= layout.close_x_rect.y
                        && row
                            < layout
                                .close_x_rect
                                .y
                                .saturating_add(layout.close_x_rect.height);
                    if inside_close {
                        self.selection = Selection::None;
                        continue;
                    }

                    match ui::detail_mouse_target(&layout, &self.game, self.selection, column, row)
                    {
                        DetailMouseTarget::Close => {
                            self.selection = Selection::None;
                        }
                        DetailMouseTarget::BaseSquadRow { squad_index }
                        | DetailMouseTarget::MissionOnSiteRow { squad_index } => {
                            self.selection = Selection::Squad(SquadId(squad_index));
                        }
                        DetailMouseTarget::None => {
                            if let Some((cell_col, cell_row)) =
                                ui::terminal_xy_to_cell(layout.map_inner, column, row)
                            {
                                self.selection = match ui::map_target_at_cell(
                                    layout.map_inner,
                                    cell_col,
                                    cell_row,
                                ) {
                                    MapTarget::Base => Selection::Base,
                                    MapTarget::Mission => Selection::Mission,
                                    MapTarget::Empty => Selection::None,
                                };
                            }
                        }
                    }
                }
                Action::None => {}
            }
        }

        Ok(())
    }
}
