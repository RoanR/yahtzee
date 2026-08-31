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
pub mod game_over;
pub mod rest;
pub mod roll_animation;
pub mod rolling;
pub mod room_select;
pub mod score_view;
pub mod scored;
pub mod selecting;
pub mod shop;
pub mod unlock;
pub mod upgrade;

use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::rngs::ThreadRng;
use ratatui::{
    Terminal, Viewport,
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, Widget},
};

use crate::{
    game::{GamePhase, GameState},
    scoring::ScoreCategory,
};

use roll_animation::RollAnimation;

// ─── App ──────────────────────────────────────────────────────────────────────

const MAX_WIDTH: u16 = 80;
const MAX_HEIGHT: u16 = 24;

pub struct App {
    state: GameState,
    rng: ThreadRng,
    // The two categories offered after a boss defeat; cleared after unlock.
    unlock_options: Option<[ScoreCategory; 2]>,
    roll_animation: Option<RollAnimation>,
    // Shop items held aside while the player picks a die/face for an upgrade purchased in shop.
    stashed_shop: Option<(Vec<crate::shop::ShopItem>, usize)>,
}

impl App {
    pub fn new(state: GameState) -> Self {
        Self {
            state,
            rng: rand::rng(),
            unlock_options: None,
            roll_animation: None,
            stashed_shop: None,
        }
    }

    // Enter alternate screen, enable raw mode, run the event loop, then clean up.
    pub fn run(&mut self) -> std::io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let mut terminal = Terminal::with_options(
            CrosstermBackend::new(stdout),
            ratatui::TerminalOptions {
                viewport: Viewport::Fixed(Rect::new(0, 0, MAX_WIDTH, MAX_HEIGHT)),
            },
        )?;

        loop {
            self.tick_animation();
            terminal.draw(|f| self.render(f))?;
            if event::poll(Duration::from_millis(16))?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && !self.handle_key(key.code)
            {
                break;
            }
        }
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        Ok(())
    }

    // ── Rendering ─────────────────────────────────────────────────────────────

    // Dispatch to the correct screen renderer based on current phase.
    fn render(&self, frame: &mut ratatui::Frame) {
        match &self.state.phase {
            GamePhase::Rolling => self.render_game(frame),
            GamePhase::SelectingCategory { cursor, .. } => self.render_selecting(frame, *cursor),
            GamePhase::Scored { .. } => self.render_scored(frame),
            GamePhase::Boss => self.render_game(frame),
            GamePhase::Shop { items, cursor } => self.render_shop(frame, items, *cursor),
            GamePhase::Rest { cursor } => self.render_rest(frame, *cursor),
            GamePhase::UpgradeSelectDie {
                die_cursor,
                kind,
                pending_die,
                ..
            } => self.render_upgrade_select_die(frame, *die_cursor, *kind, pending_die.as_ref()),
            GamePhase::UpgradeSelectFace {
                die_index,
                face_cursor,
                kind,
                ..
            } => self.render_upgrade_select_face(frame, *die_index, *face_cursor, *kind),
            GamePhase::ChoosingRoom { cursor } => self.render_choosing_room(frame, *cursor),
            GamePhase::CategoryUnlock => self.render_unlock(frame),
            GamePhase::GameOver => self.render_game_over(frame),
        }
    }

    // Used for shop and rest sites
    fn render_rest_shop(&self, frame: &mut ratatui::Frame, widget: impl Widget) {
        let main_area = self.vertical_layout(
            frame,
            "[Up/Down] Select  [Enter] Confirm  [L] Leave [Q] Quit",
        );
        frame.render_widget(widget, main_area);
    }

    // ── Input ─────────────────────────────────────────────────────────────────

    // Handle a key press. Returns false to signal the event loop to exit.
    fn handle_key(&mut self, code: KeyCode) -> bool {
        // During a roll animation only Q is processed; everything else is ignored.
        if self.roll_animation.is_some() {
            return !matches!(code, KeyCode::Char('q') | KeyCode::Char('Q'));
        }
        match &self.state.phase {
            GamePhase::Rolling => self.handle_rolling(code),
            GamePhase::SelectingCategory { cursor, .. } => {
                let cursor = *cursor;
                self.handle_selecting(code, cursor)
            }
            GamePhase::Scored { .. } => self.handle_scored(code),
            GamePhase::Boss => self.handle_rolling(code),
            GamePhase::Shop { cursor, items } => {
                self.handle_rest_shop(code, *cursor, items.len(), self::App::handle_shop)
            }
            GamePhase::Rest { cursor } => {
                self.handle_rest_shop(code, *cursor, 3, self::App::handle_rest)
            }
            GamePhase::UpgradeSelectDie {
                die_cursor,
                kind,
                from_shop,
                pending_die,
            } => {
                let (c, k, fs) = (*die_cursor, *kind, *from_shop);
                let pd = pending_die.clone();
                self.handle_upgrade_select_die(code, c, k, fs, pd)
            }
            GamePhase::UpgradeSelectFace {
                die_index,
                face_cursor,
                kind,
                from_shop,
            } => {
                let (di, fc, k, fs) = (*die_index, *face_cursor, *kind, *from_shop);
                self.handle_upgrade_select_face(code, di, fc, k, fs)
            }
            GamePhase::ChoosingRoom { cursor } => {
                let cursor = *cursor;
                self.handle_choosing_room(code, cursor)
            }
            GamePhase::CategoryUnlock => self.handle_unlock(code),
            GamePhase::GameOver => self.handle_game_over(code),
        }
    }

    fn handle_rest_shop(
        &mut self,
        code: KeyCode,
        cursor: usize,
        len: usize,
        enter: fn(&mut App, usize),
    ) -> bool {
        match code {
            KeyCode::Up if len > 0 => {
                self.state.phase.set_cursor((cursor + len - 1) % len);
                true
            }
            KeyCode::Down if len > 0 => {
                self.state.phase.set_cursor((cursor + 1) % len);
                true
            }
            KeyCode::Char('l') | KeyCode::Char('L') | KeyCode::Esc => {
                self.state.dungeon.current_floor_mut().advance();
                self.transition_after_advance();
                true
            }
            KeyCode::Enter => {
                enter(self, cursor);
                true
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => false,
            _ => true,
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    // Shared by rolling.rs (Rolling/Boss) and selecting.rs (SelectingCategory),
    // which both show the dice pool but pick a different score-view mode.
    fn dice_widget(&self) -> dice_view::DiceView<'_> {
        match &self.roll_animation {
            Some(anim) => dice_view::DiceView::animated(&self.state.dice_pool, &anim.display),
            None => dice_view::DiceView::new(&self.state.dice_pool),
        }
    }

    fn score_view_widget(&self) -> score_view::ScoreView<'_> {
        score_view::ScoreView::new(
            &self.state.dice_pool,
            &self.state.unlocked,
            &self.state.used_this_room,
        )
    }

    // Advance the roll animation by one tick (called once per render loop iteration).
    // When the frame count reaches zero, clears the animation so real values render.
    fn tick_animation(&mut self) {
        let still_active = match self.roll_animation.as_mut() {
            None => return,
            Some(anim) => anim.tick(&self.state.dice_pool, &mut self.rng),
        };

        if !still_active {
            self.roll_animation = None;
        }
    }

    // Set the correct phase after the floor's step index has been advanced.
    fn transition_after_advance(&mut self) {
        if self.state.dungeon.current_floor().boss_next() {
            self.state.begin_room(true);
            self.state.phase = GamePhase::Boss;
        } else {
            self.state.phase = GamePhase::ChoosingRoom { cursor: 0 };
        }
    }

    fn vertical_layout(&self, frame: &mut ratatui::Frame, hints: &str) -> Rect {
        let [header_area, body_area, hints_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        frame.render_widget(dungeon_view::DungeonView::new(&self.state), header_area);
        frame.render_widget(Paragraph::new(hints), hints_area);

        body_area
    }
}
