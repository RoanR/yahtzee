// Scored result screen: shown after the player scores, before advancing to
// the next room.

use crossterm::event::KeyCode;
use ratatui::{
    text::Line,
    widgets::{Block, Paragraph},
};

use crate::{game::GamePhase, shop};

use super::App;

impl App {
    pub(super) fn render_scored(&self, frame: &mut ratatui::Frame) {
        let (_score, _target, success) = match &self.state.phase {
            GamePhase::Scored {
                score,
                target,
                success,
            } => (*score, *target, *success),
            _ => return,
        };
        let main_area = self.vertical_layout(frame, "Press any key to continue...");

        // Central loot area or HP loss
        let is_boss = self.state.dungeon.current_floor().boss_next();
        let consequence = if success {
            if is_boss {
                "\nBoss defeated! Choose a new category.".to_string()
            } else {
                let gold = self
                    .state
                    .dungeon
                    .current_floor()
                    .current_room()
                    .map(|r| r.reward_gold())
                    .unwrap_or(0);
                format!("\n+{}g earned", gold)
            }
        } else {
            let hp_loss = if is_boss { 20 } else { 10 };
            format!("\n-{} HP", hp_loss)
        };
        frame.render_widget(
            Paragraph::new(Line::from(consequence).centered())
                .block(Block::bordered().border_type(ratatui::widgets::BorderType::Rounded)),
            main_area,
        );
    }

    pub(super) fn handle_scored(&mut self, _code: KeyCode) -> bool {
        let success = match &self.state.phase {
            GamePhase::Scored { success, .. } => *success,
            _ => return true,
        };

        let is_boss = self.state.dungeon.current_floor().boss_next();

        if is_boss {
            if success {
                let options = self.pick_unlock_options();
                self.unlock_options = options;
                self.state.defeat_boss();
            } else {
                self.state.take_damage(20);
                if !matches!(self.state.phase, GamePhase::GameOver) {
                    self.state.begin_room(false);
                    self.state.phase = GamePhase::Boss;
                }
            }
            return true;
        }

        // Regular room: extract reward before mutating.
        let reward_gold: u32 = if success {
            self.state
                .dungeon
                .current_floor()
                .current_room()
                .map(|r| r.reward_gold())
                .unwrap_or(0)
        } else {
            0
        };

        if success {
            self.state.earn_gold(reward_gold);
            self.state.dice_pool.reset_for_room();
        } else {
            self.state.take_damage(10);
            if !matches!(self.state.phase, GamePhase::GameOver) {
                self.state.begin_room(false);
                self.state.phase = GamePhase::Rolling;
            }
        }

        if matches!(self.state.phase, GamePhase::GameOver) {
            return true;
        }

        if success {
            let items = shop::generate_shop_items(&self.state, &mut self.rng);
            self.state.phase = GamePhase::Shop { items, cursor: 0 };
        }
        true
    }
}
