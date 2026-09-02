// Category-unlock screen: shown once after defeating a boss to pick a new
// scoring category from two random options.

use crossterm::event::KeyCode;
use rand::seq::IndexedRandom;
use ratatui::widgets::Paragraph;

use crate::{game::GamePhase, scoring::ScoreCategory};

use super::{App, Phase};

pub(super) struct CategoryUnlockPhase;

impl Phase for CategoryUnlockPhase {
    fn render(&self, app: &App, frame: &mut ratatui::Frame) {
        let text = match &app.unlock_options {
            Some(options) => format!(
                "BOSS DEFEATED! Choose a new scoring category:\n\n[1] {}\n[2] {}",
                options[0], options[1]
            ),
            None => "All categories unlocked! Press any key to continue.".to_string(),
        };
        frame.render_widget(Paragraph::new(text), frame.area());
    }

    fn handle_key(&self, app: &mut App, code: KeyCode) -> bool {
        if app.unlock_options.is_none() {
            // All categories already unlocked: any key descends.
            app.state.descend(&mut app.rng);
            app.state.phase = GamePhase::ChoosingRoom { cursor: 0 };
            return true;
        }

        let chosen = match (code, &app.unlock_options) {
            (KeyCode::Char('1'), Some(opts)) => Some(opts[0].clone()),
            (KeyCode::Char('2'), Some(opts)) => Some(opts[1].clone()),
            _ => return true,
        };

        if let Some(cat) = chosen {
            app.state.unlock_category(cat);
            app.unlock_options = None;
            app.state.descend(&mut app.rng);
            app.state.phase = GamePhase::ChoosingRoom { cursor: 0 };
        }

        true
    }
}

impl App {
    // Pick two unique categories from those not yet unlocked. Returns None when
    // fewer than two remain (all categories have been unlocked).
    pub(super) fn pick_unlock_options(&mut self) -> Option<[ScoreCategory; 2]> {
        const ALL_UNLOCKABLE: &[ScoreCategory] = &[
            ScoreCategory::Ones,
            ScoreCategory::Twos,
            ScoreCategory::Threes,
            ScoreCategory::Fours,
            ScoreCategory::Fives,
            ScoreCategory::Sixes,
            ScoreCategory::FullHouse,
            ScoreCategory::SmallStraight,
            ScoreCategory::LargeStraight,
            ScoreCategory::Yahtzee,
        ];

        let available: Vec<ScoreCategory> = ALL_UNLOCKABLE
            .iter()
            .filter(|c| !self.state.unlocked.contains(c))
            .cloned()
            .collect();

        if available.len() < 2 {
            return None;
        }

        let chosen: Vec<&ScoreCategory> = available.choose_multiple(&mut self.rng, 2).collect();
        Some([chosen[0].clone(), chosen[1].clone()])
    }
}
