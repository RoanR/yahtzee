// Category selection screen: shown once all rolls are used or the player
// chooses to score early.

use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout};

use crate::{game::GamePhase, scoring::ScoreCategory};

use super::{App, Phase, is_quit};

pub(super) struct SelectingPhase {
    pub cursor: usize,
}

impl Phase for SelectingPhase {
    fn render(&self, app: &App, frame: &mut ratatui::Frame) {
        let main_area = app.vertical_layout(
            frame,
            "[Up/Down] Select Category  [S/Enter] Confirm [R/Esc] To Roll [Q] Quit",
        );

        let [left_area, right_area] =
            Layout::horizontal([Constraint::Fill(2), Constraint::Fill(3)]).areas(main_area);

        frame.render_widget(app.dice_widget(), left_area);
        frame.render_widget(app.score_view_widget().with_cursor(self.cursor), right_area);
    }

    fn handle_key(&self, app: &mut App, code: KeyCode) -> bool {
        let from_boss = match app.state.phase {
            GamePhase::SelectingCategory { from_boss, .. } => from_boss,
            _ => false,
        };

        let available: Vec<ScoreCategory> = app
            .state
            .unlocked
            .iter()
            .filter(|c| !app.state.used_this_room.contains(c))
            .cloned()
            .collect();

        if available.is_empty() {
            return true;
        }

        if is_quit(code) {
            return false;
        }

        let cursor = self.cursor.min(available.len() - 1);

        match code {
            KeyCode::Up => {
                let new_cursor = if cursor == 0 {
                    available.len() - 1
                } else {
                    cursor - 1
                };
                app.state.phase = GamePhase::SelectingCategory {
                    cursor: new_cursor,
                    from_boss,
                };
                true
            }
            KeyCode::Down => {
                let new_cursor = if cursor + 1 >= available.len() {
                    0
                } else {
                    cursor + 1
                };
                app.state.phase = GamePhase::SelectingCategory {
                    cursor: new_cursor,
                    from_boss,
                };
                true
            }
            KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter => {
                let chosen = available[cursor].clone();
                app.state.score_category(chosen);
                true
            }
            KeyCode::Esc | KeyCode::Char('r') | KeyCode::Char('R') => {
                app.state.back_room();
                true
            }
            _ => true,
        }
    }
}
