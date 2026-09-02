// Scored result screen: shown after the player scores, before advancing to
// the next room.

use crossterm::event::KeyCode;
use ratatui::{
    text::Line,
    widgets::{Block, Paragraph},
};

use crate::{game::GamePhase, shop};

use super::{App, Phase};

pub(super) struct ScoredPhase;

impl Phase for ScoredPhase {
    fn render(&self, app: &App, frame: &mut ratatui::Frame) {
        let success = match &app.state.phase {
            GamePhase::Scored { success, .. } => *success,
            _ => return,
        };
        let main_area = app.vertical_layout(frame, "Press any key to continue...");

        // Central loot area or HP loss
        let is_boss = app.state.dungeon.current_floor().boss_next();
        let consequence = if success {
            if is_boss {
                "\nBoss defeated! Choose a new category.".to_string()
            } else {
                let gold = app
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

    fn handle_key(&self, app: &mut App, _code: KeyCode) -> bool {
        let success = match &app.state.phase {
            GamePhase::Scored { success, .. } => *success,
            _ => return true,
        };

        let is_boss = app.state.dungeon.current_floor().boss_next();

        if is_boss {
            if success {
                let options = app.pick_unlock_options();
                app.unlock_options = options;
                app.state.defeat_boss();
            } else {
                app.state.take_damage(20);
                if !matches!(app.state.phase, GamePhase::GameOver) {
                    app.state.begin_room(false);
                    app.state.phase = GamePhase::Boss;
                }
            }
            return true;
        }

        // Regular room: extract reward before mutating.
        let reward_gold: u32 = if success {
            app.state
                .dungeon
                .current_floor()
                .current_room()
                .map(|r| r.reward_gold())
                .unwrap_or(0)
        } else {
            0
        };

        if success {
            app.state.earn_gold(reward_gold);
            app.state.dice_pool.reset_for_room();
        } else {
            app.state.take_damage(10);
            if !matches!(app.state.phase, GamePhase::GameOver) {
                app.state.begin_room(false);
                app.state.phase = GamePhase::Rolling;
            }
        }

        if matches!(app.state.phase, GamePhase::GameOver) {
            return true;
        }

        if success {
            let items = shop::generate_shop_items(&app.state, &mut app.rng);
            app.state.phase = GamePhase::Shop { items, cursor: 0 };
        }
        true
    }
}
