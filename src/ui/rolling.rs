// Main game screen: rolling and holding dice (Rolling and Boss phases).
//
//   [main]    Min(0)     horizontal split:
//     [left]  Fill(2)    DiceView (die boxes + rolls indicator below)
//     [right] Fill(3)    ScoreView (title + category list)
// The Fill(2)/Fill(3) ratio gives the left panel ~40% and the right ~60% of
// the width. On an 80-col terminal the left gets ~32 cols (enough for 5 dice
// at 5 chars each) and the right gets ~48 cols (enough for long category names).

use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout};

use super::{App, roll_animation::RollAnimation};

impl App {
    pub(super) fn render_game(&self, frame: &mut ratatui::Frame) {
        let main_area = self.vertical_layout(frame, self.roll_hint());

        let [left_area, right_area] =
            Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
                .areas(main_area);

        frame.render_widget(self.dice_widget(), left_area);
        frame.render_widget(self.score_view_widget(), right_area);
    }

    pub(super) fn handle_rolling(&mut self, code: KeyCode) -> bool {
        match (
            self.state.dice_pool.max_rolls != self.state.dice_pool.rolls_remaining,
            code,
        ) {
            (true, KeyCode::Right) => {
                self.state.dice_pool.next_die();
                true
            }
            (true, KeyCode::Left) => {
                self.state.dice_pool.prev_die();
                true
            }
            (true, KeyCode::Char(' ')) => {
                self.state.dice_pool.toggle_selected();
                true
            }
            (_, KeyCode::Char('r') | KeyCode::Char('R')) => {
                if self.state.roll() {
                    // Roll committed; start display animation.
                    self.roll_animation =
                        Some(RollAnimation::new(&self.state.dice_pool, &mut self.rng));
                }
                true
            }
            (true, KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter) => {
                self.state.begin_scoring();
                true
            }
            (_, KeyCode::Char('q') | KeyCode::Char('Q')) => false,
            _ => true,
        }
    }

    fn roll_hint(&self) -> &'static str {
        if self.state.dice_pool.rolls_remaining == self.state.dice_pool.max_rolls {
            "[R] Roll  [Q] Quit"
        } else {
            "[<Arrow Keys>] Select Die  [<Space>] Hold  [R] Roll  [S] Score  [Q] Quit"
        }
    }
}
