// TUI application: crossterm setup, event loop, and screen dispatch.
//
// App owns the terminal handle and GameState. Each tick it:
//   1. Renders the current screen based on GameState::phase.
//   2. Polls for a crossterm event.
//   3. Dispatches the event to the appropriate input handler.
//
// Each GamePhase variant has a small Phase-implementing wrapper struct
// (defined here for now; each is being moved into its own sibling module
// alongside that phase's render_*/handle_* methods) so phase_view() is the
// only place that needs updating when a phase is added, instead of two
// separate matches. This file holds only what's genuinely shared: the App
// struct itself, the run loop, the Phase trait and phase_view() dispatch,
// roll-animation ticking, and small helpers (dice_widget, score_view_widget,
// vertical_layout, is_quit, render_rest_shop/handle_rest_shop,
// transition_after_advance) used by two or more phases.

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
    dice::Die,
    game::{GamePhase, GameState, UpgradeKind},
    scoring::ScoreCategory,
};

use roll_animation::RollAnimation;

// Shared by every phase's input handler to quit on 'q'/'Q'.
fn is_quit(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('q') | KeyCode::Char('Q'))
}

// A GamePhase, as a self-contained state that knows how to render itself and
// handle its own input. Built fresh from GameState::phase each time it's
// needed (see App::phase_view) rather than stored, so it owns whatever data
// it needs (cloning small Vecs/Options out of GamePhase where relevant) and
// can never go stale relative to the phase GameState is actually in.
trait Phase {
    fn render(&self, app: &App, frame: &mut ratatui::Frame);
    // Returns false to signal the event loop to exit.
    fn handle_key(&self, app: &mut App, code: KeyCode) -> bool;
}

// Thin wrappers delegating to each phase's existing render_X/handle_X App
// methods; each is being relocated into its own sibling module (with its
// logic inlined directly) one at a time.
//
// Only owns cursor: ShopItem isn't Clone (it can hold a Box<dyn Relic>), so
// items are read fresh from GameState::phase in render/handle_key rather
// than cloned into the phase struct.
struct ShopPhase {
    cursor: usize,
}
impl Phase for ShopPhase {
    fn render(&self, app: &App, frame: &mut ratatui::Frame) {
        let GamePhase::Shop { items, .. } = &app.state.phase else {
            return;
        };
        app.render_shop(frame, items, self.cursor);
    }
    fn handle_key(&self, app: &mut App, code: KeyCode) -> bool {
        let GamePhase::Shop { items, .. } = &app.state.phase else {
            return true;
        };
        let len = items.len();
        app.handle_rest_shop(code, self.cursor, len, App::handle_shop)
    }
}

struct RestPhase {
    cursor: usize,
}
impl Phase for RestPhase {
    fn render(&self, app: &App, frame: &mut ratatui::Frame) {
        app.render_rest(frame, self.cursor);
    }
    fn handle_key(&self, app: &mut App, code: KeyCode) -> bool {
        app.handle_rest_shop(code, self.cursor, 3, App::handle_rest)
    }
}

struct UpgradeSelectDiePhase {
    die_cursor: usize,
    kind: UpgradeKind,
    from_shop: bool,
    pending_die: Option<Die>,
}
impl Phase for UpgradeSelectDiePhase {
    fn render(&self, app: &App, frame: &mut ratatui::Frame) {
        app.render_upgrade_select_die(frame, self.die_cursor, self.kind, self.pending_die.as_ref());
    }
    fn handle_key(&self, app: &mut App, code: KeyCode) -> bool {
        app.handle_upgrade_select_die(
            code,
            self.die_cursor,
            self.kind,
            self.from_shop,
            self.pending_die.clone(),
        )
    }
}

struct UpgradeSelectFacePhase {
    die_index: usize,
    face_cursor: usize,
    kind: UpgradeKind,
    from_shop: bool,
}
impl Phase for UpgradeSelectFacePhase {
    fn render(&self, app: &App, frame: &mut ratatui::Frame) {
        app.render_upgrade_select_face(frame, self.die_index, self.face_cursor, self.kind);
    }
    fn handle_key(&self, app: &mut App, code: KeyCode) -> bool {
        app.handle_upgrade_select_face(
            code,
            self.die_index,
            self.face_cursor,
            self.kind,
            self.from_shop,
        )
    }
}

struct ChoosingRoomPhase {
    cursor: usize,
}
impl Phase for ChoosingRoomPhase {
    fn render(&self, app: &App, frame: &mut ratatui::Frame) {
        app.render_choosing_room(frame, self.cursor);
    }
    fn handle_key(&self, app: &mut App, code: KeyCode) -> bool {
        app.handle_choosing_room(code, self.cursor)
    }
}

struct CategoryUnlockPhase;
impl Phase for CategoryUnlockPhase {
    fn render(&self, app: &App, frame: &mut ratatui::Frame) {
        app.render_unlock(frame);
    }
    fn handle_key(&self, app: &mut App, code: KeyCode) -> bool {
        app.handle_unlock(code)
    }
}

struct GameOverPhase;
impl Phase for GameOverPhase {
    fn render(&self, app: &App, frame: &mut ratatui::Frame) {
        app.render_game_over(frame);
    }
    fn handle_key(&self, app: &mut App, code: KeyCode) -> bool {
        app.handle_game_over(code)
    }
}

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

    // ── Phase dispatch ───────────────────────────────────────────────────────

    // The single place that maps the current GamePhase to its Phase struct.
    fn phase_view(&self) -> Box<dyn Phase> {
        match &self.state.phase {
            GamePhase::Rolling | GamePhase::Boss => Box::new(rolling::RollingPhase),
            GamePhase::SelectingCategory { cursor, .. } => {
                Box::new(selecting::SelectingPhase { cursor: *cursor })
            }
            GamePhase::Scored { .. } => Box::new(scored::ScoredPhase),
            GamePhase::Shop { cursor, .. } => Box::new(ShopPhase { cursor: *cursor }),
            GamePhase::Rest { cursor } => Box::new(RestPhase { cursor: *cursor }),
            GamePhase::UpgradeSelectDie {
                die_cursor,
                kind,
                from_shop,
                pending_die,
            } => Box::new(UpgradeSelectDiePhase {
                die_cursor: *die_cursor,
                kind: *kind,
                from_shop: *from_shop,
                pending_die: pending_die.clone(),
            }),
            GamePhase::UpgradeSelectFace {
                die_index,
                face_cursor,
                kind,
                from_shop,
            } => Box::new(UpgradeSelectFacePhase {
                die_index: *die_index,
                face_cursor: *face_cursor,
                kind: *kind,
                from_shop: *from_shop,
            }),
            GamePhase::ChoosingRoom { cursor } => Box::new(ChoosingRoomPhase { cursor: *cursor }),
            GamePhase::CategoryUnlock => Box::new(CategoryUnlockPhase),
            GamePhase::GameOver => Box::new(GameOverPhase),
        }
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        self.phase_view().render(self, frame);
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
            return !is_quit(code);
        }
        self.phase_view().handle_key(self, code)
    }

    fn handle_rest_shop(
        &mut self,
        code: KeyCode,
        cursor: usize,
        len: usize,
        enter: fn(&mut App, usize),
    ) -> bool {
        if is_quit(code) {
            return false;
        }
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
