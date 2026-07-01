// TUI application: crossterm setup, event loop, and screen dispatch.
//
// App owns the terminal handle and GameState. Each tick it:
//   1. Renders the current screen based on GameState::phase.
//   2. Polls for a crossterm event.
//   3. Dispatches the event to the appropriate input handler.
//
// All rendering is done through ratatui widgets defined in the sibling modules.
// Input handling is done in App methods; they mutate GameState and let the
// next render reflect the new state.

pub mod dice_view;
pub mod dungeon_view;
pub mod score_view;

use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::game::{GamePhase, GameState};

// ─── App ──────────────────────────────────────────────────────────────────────

pub struct App {
    state: GameState,
}

impl App {
    pub fn new(state: GameState) -> Self {
        Self { state }
    }

    // Enter alternate screen, enable raw mode, run the event loop, then clean up.
    pub fn run(&mut self) -> std::io::Result<()> {
        // enable_raw_mode()
        // execute!(stdout, EnterAlternateScreen)
        // let mut terminal = Terminal::new(CrosstermBackend::new(stdout))
        // loop:
        //   terminal.draw(|f| self.render(f))
        //   if event::poll(Duration::from_millis(16)):
        //     if let Event::Key(key) = event::read():
        //       if self.handle_key(key.code) == false: break
        // disable_raw_mode()
        // execute!(stdout, LeaveAlternateScreen)
        todo!()
    }

    // ── Rendering ─────────────────────────────────────────────────────────────

    // Dispatch to the correct screen renderer based on current phase.
    fn render(&self, frame: &mut ratatui::Frame) {
        // match &self.state.phase:
        //   GamePhase::Rolling | GamePhase::Scored { .. } => self.render_game(frame)
        //   GamePhase::Shop  => self.render_shop(frame)
        //   GamePhase::Rest  => self.render_rest(frame)
        //   GamePhase::Boss  => self.render_boss(frame)
        //   GamePhase::CategoryUnlock => self.render_unlock(frame)
        //   GamePhase::GameOver => self.render_game_over(frame)
        todo!()
    }

    // Main game screen: header + dice row + score panel + keybind hints.
    fn render_game(&self, frame: &mut ratatui::Frame) {
        // split area into: header (3 lines), dice row (5 lines), score panel (remainder)
        // frame.render_widget(DungeonView::new(&self.state), header_area)
        // frame.render_widget(DiceView::new(&self.state.dice_pool), dice_area)
        // frame.render_widget(ScoreView::new(&self.state.dice_pool, ...), score_area)
        // render keybind row at bottom: "[1-5] Hold  [R] Roll  [Q] Quit"
        todo!()
    }

    fn render_shop(&self, _frame: &mut ratatui::Frame) {
        // list shop items with prices; highlight selected; show gold
        todo!()
    }

    fn render_rest(&self, _frame: &mut ratatui::Frame) {
        // two choices: "Heal 15 HP" / "Upgrade a die"
        // if upgrade chosen: show die list, then upgrade list
        todo!()
    }

    fn render_boss(&self, _frame: &mut ratatui::Frame) {
        // same as render_game but with boss name, weakness, and debuff displayed
        todo!()
    }

    fn render_unlock(&self, _frame: &mut ratatui::Frame) {
        // show two category choices; player picks one to unlock
        todo!()
    }

    fn render_game_over(&self, _frame: &mut ratatui::Frame) {
        // floor reached, final score, prompt to restart or quit
        todo!()
    }

    // ── Input ─────────────────────────────────────────────────────────────────

    // Handle a key press. Returns false to signal the event loop to exit.
    fn handle_key(&mut self, code: KeyCode) -> bool {
        match &self.state.phase {
            GamePhase::Rolling => self.handle_rolling(code),
            GamePhase::Scored { .. } => self.handle_scored(code),
            GamePhase::Shop => self.handle_shop(code),
            GamePhase::Rest => self.handle_rest(code),
            GamePhase::Boss => self.handle_rolling(code), // same controls as rolling
            GamePhase::CategoryUnlock => self.handle_unlock(code),
            GamePhase::GameOver => self.handle_game_over(code),
        }
    }

    fn handle_rolling(&mut self, code: KeyCode) -> bool {
        // KeyCode::Char('1'..='5') => state.dice_pool.toggle_hold(index)
        // KeyCode::Char('r') | KeyCode::Char('R') => state.roll()
        // KeyCode::Char('s') | KeyCode::Enter => state.score()  (use best category)
        // KeyCode::Char('q') | KeyCode::Char('Q') => return false
        todo!()
    }

    fn handle_scored(&mut self, _code: KeyCode) -> bool {
        // any key: advance to next room or transition to shop/rest/boss/unlock
        todo!()
    }

    fn handle_shop(&mut self, _code: KeyCode) -> bool {
        // arrow keys to navigate items, Enter to buy, Q to leave
        todo!()
    }

    fn handle_rest(&mut self, _code: KeyCode) -> bool {
        // H to heal, U to upgrade a die; if upgrading, pick die then upgrade
        todo!()
    }

    fn handle_unlock(&mut self, _code: KeyCode) -> bool {
        // 1 or 2 to pick one of the two offered categories
        todo!()
    }

    fn handle_game_over(&mut self, code: KeyCode) -> bool {
        // Q or Enter to quit; R to restart (would reinitialise GameState)
        todo!()
    }
}
