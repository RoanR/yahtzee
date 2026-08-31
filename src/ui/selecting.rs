// Category selection screen: shown once all rolls are used or the player
// chooses to score early.

use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout};

use crate::{game::GamePhase, scoring::ScoreCategory};

use super::App;

impl App {
    pub(super) fn render_selecting(&self, frame: &mut ratatui::Frame, cursor: usize) {
        let main_area = self.vertical_layout(
            frame,
            "[Up/Down] Select Category  [S/Enter] Confirm [R/Esc] To Roll [Q] Quit",
        );

        let [left_area, right_area] =
            Layout::horizontal([Constraint::Fill(2), Constraint::Fill(3)]).areas(main_area);

        frame.render_widget(self.dice_widget(), left_area);
        frame.render_widget(self.score_view_widget().with_cursor(cursor), right_area);
    }

    pub(super) fn handle_selecting(&mut self, code: KeyCode, cursor: usize) -> bool {
        let from_boss = match self.state.phase {
            GamePhase::SelectingCategory { from_boss, .. } => from_boss,
            _ => false,
        };

        let available: Vec<ScoreCategory> = self
            .state
            .unlocked
            .iter()
            .filter(|c| !self.state.used_this_room.contains(c))
            .cloned()
            .collect();

        if available.is_empty() {
            return true;
        }

        let cursor = cursor.min(available.len() - 1);

        match code {
            KeyCode::Up => {
                let new_cursor = if cursor == 0 {
                    available.len() - 1
                } else {
                    cursor - 1
                };
                self.state.phase = GamePhase::SelectingCategory {
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
                self.state.phase = GamePhase::SelectingCategory {
                    cursor: new_cursor,
                    from_boss,
                };
                true
            }
            KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter => {
                let chosen = available[cursor].clone();
                self.state.score_category(chosen);
                true
            }
            KeyCode::Esc | KeyCode::Char('r') | KeyCode::Char('R') => {
                self.state.back_room();
                true
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => false,
            _ => true,
        }
    }
}
