// Game over screen: rendered when HP reaches zero.

use crossterm::event::KeyCode;
use ratatui::widgets::Paragraph;

use super::{App, Phase};

pub(super) struct GameOverPhase;

impl Phase for GameOverPhase {
    fn render(&self, app: &App, frame: &mut ratatui::Frame) {
        let floor = app.state.dungeon.current_floor();
        frame.render_widget(
            Paragraph::new(format!(
                "GAME OVER\n\nFloor {}\nHP: {}/{}\n\n[Q] or [Enter] to quit",
                floor.floor_num, app.state.hp, app.state.max_hp
            )),
            frame.area(),
        );
    }

    fn handle_key(&self, _app: &mut App, code: KeyCode) -> bool {
        !matches!(
            code,
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Enter
        )
    }
}
