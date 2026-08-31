// Game over screen: rendered when HP reaches zero.

use crossterm::event::KeyCode;
use ratatui::widgets::Paragraph;

use super::App;

impl App {
    pub(super) fn render_game_over(&self, frame: &mut ratatui::Frame) {
        let floor = self.state.dungeon.current_floor();
        frame.render_widget(
            Paragraph::new(format!(
                "GAME OVER\n\nFloor {}\nHP: {}/{}\n\n[Q] or [Enter] to quit",
                floor.floor_num, self.state.hp, self.state.max_hp
            )),
            frame.area(),
        );
    }

    pub(super) fn handle_game_over(&mut self, code: KeyCode) -> bool {
        matches!(
            code,
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Enter
        )
        .then(|| false)
        .unwrap_or(true)
    }
}
