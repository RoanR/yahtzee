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
pub mod shop_view;

use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::{Rng, rngs::ThreadRng, seq::IndexedRandom};
use ratatui::{
    Terminal, Viewport,
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{Block, Paragraph},
};

use crate::{
    dungeon::room,
    game::{GamePhase, GameState},
    scoring::ScoreCategory,
    shop,
};

// ─── RollAnimation ────────────────────────────────────────────────────────────

// 18 ticks * 16ms ~= 288ms of visible cycling before snapping to the real result.
const ROLL_ANIM_FRAMES: u8 = 18;

struct RollAnimation {
    frames_remaining: u8,
    // Per-die value to display this frame. None = held die (show actual current_value).
    // Always 1-6; no WILD sentinel during animation.
    display: Vec<Option<u8>>,
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
}

impl App {
    pub fn new(state: GameState) -> Self {
        Self {
            state,
            rng: rand::rng(),
            unlock_options: None,
            roll_animation: None,
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
            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && !self.handle_key(key.code) {
                        break;
                    }
                }
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
            GamePhase::SelectingCategory { cursor, from_boss } => {
                self.render_selecting(frame, *cursor, *from_boss)
            }
            GamePhase::Scored { .. } => self.render_scored(frame),
            GamePhase::Boss => self.render_boss(frame),
            GamePhase::Shop { items, cursor } => self.render_shop(frame, items, *cursor),
            GamePhase::Rest => self.render_rest(frame),
            GamePhase::CategoryUnlock => self.render_unlock(frame),
            GamePhase::GameOver => self.render_game_over(frame),
        }
    }

    // Main game screen - Option A layout:
    //
    //   [header]  Length(2)  DungeonView (title + status row 0, target + HP bar row 1)
    //   [main]    Min(0)     horizontal split:
    //     [left]  Fill(2)    DiceView (die boxes + rolls indicator below)
    //     [right] Fill(3)    ScoreView (title + category list)
    //   [hints]   Length(1)  keybind line
    //
    // The Fill(2)/Fill(3) ratio gives the left panel ~40% and the right ~60% of
    // the width. On an 80-col terminal the left gets ~32 cols (enough for 5 dice
    // at 5 chars each) and the right gets ~48 cols (enough for long category names).
    fn render_game(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();

        // Outer vertical split.
        let [header_area, main_area, hints_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(area);

        // Inner horizontal split inside main_area.
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
                .areas(main_area);

        // Render DungeonView into header_area (2 rows).
        frame.render_widget(dungeon_view::DungeonView::new(&self.state), header_area);

        // Render DiceView (die boxes + rolls indicator) into left_area.
        let dice_widget = match &self.roll_animation {
            Some(anim) => dice_view::DiceView::animated(&self.state.dice_pool, &anim.display),
            None => dice_view::DiceView::new(&self.state.dice_pool),
        };
        frame.render_widget(dice_widget, left_area);

        // Render ScoreView (title + categories) into right_area.
        frame.render_widget(
            score_view::ScoreView::new(
                &self.state.dice_pool,
                &self.state.unlocked,
                &self.state.used_this_room,
            ),
            right_area,
        );

        // Render keybind hint into hints_area.
        frame.render_widget(Paragraph::new(self.roll_hint()), hints_area);
    }

    // Scored result screen: shown after player scores, before advancing to next room.
    // Loot screen - layout:
    //
    //   [header]  Length(2)  DungeonView (title + status row 0, target + HP bar row 1)
    //   [loot]    Central    loot box
    //   [hints]   Length(1)  keybind line
    fn render_scored(&self, frame: &mut ratatui::Frame) {
        let (_score, _target, success) = match &self.state.phase {
            GamePhase::Scored {
                score,
                target,
                success,
            } => (*score, *target, *success),
            _ => return,
        };

        // Vertical Division
        let [header_area, loot_area, hints_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        // Render DungeonView into header_area (2 rows)
        frame.render_widget(dungeon_view::DungeonView::new(&self.state), header_area);

        // Central loot area or HP loss
        let is_boss = self.state.dungeon.current_floor().boss_next();
        let consequence = if success {
            if is_boss {
                "\nBoss defeated! Choose a new category.".to_string()
            } else {
                let gold = match self.state.dungeon.current_floor().current_room() {
                    Some(room::Room::Challenge(t)) => t.reward_gold,
                    Some(room::Room::Elite(t)) => t.reward_gold,
                    _ => 0,
                };
                format!("\n+{}g earned", gold)
            }
        } else {
            let hp_loss = if is_boss { 20 } else { 10 };
            format!("\n-{} HP", hp_loss)
        };
        frame.render_widget(
            Paragraph::new(Line::from(consequence).centered())
                .block(Block::bordered().border_type(ratatui::widgets::BorderType::Rounded)),
            loot_area,
        );

        // Hints area
        frame.render_widget(Paragraph::new("Press any key to continue..."), hints_area);
    }

    // Boss screen layout:
    //
    //   [header]  Length(2)  BossHeaderView (boss name + gold row 0,
    //                                        weakness + target + HP bar row 1)
    //   [main]    Min(0)     horizontal split (same as render_game):
    //     [left]  Fill(2)    DiceView (die boxes + rolls indicator)
    //     [right] Fill(3)    ScoreView
    //   [hints]   Length(1)  keybind line
    //
    // BossHeaderView is a new widget defined in dungeon_view.rs. It takes the same
    // &GameState as DungeonView and pulls boss data from state.dungeon.current_floor().boss.
    fn render_boss(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();

        // Identical outer vertical + inner horizontal split as render_game.
        let [header_area, main_area, hints_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(area);

        let [left_area, right_area] =
            Layout::horizontal([Constraint::Fill(2), Constraint::Fill(3)]).areas(main_area);

        // Use BossHeaderView instead of DungeonView for the header.
        frame.render_widget(dungeon_view::BossHeaderView::new(&self.state), header_area);

        // Remaining widgets identical to render_game.
        let dice_widget = match &self.roll_animation {
            Some(anim) => dice_view::DiceView::animated(&self.state.dice_pool, &anim.display),
            None => dice_view::DiceView::new(&self.state.dice_pool),
        };
        frame.render_widget(dice_widget, left_area);
        frame.render_widget(
            score_view::ScoreView::new(
                &self.state.dice_pool,
                &self.state.unlocked,
                &self.state.used_this_room,
            ),
            right_area,
        );
        frame.render_widget(Paragraph::new(self.roll_hint()), hints_area);
    }

    fn render_selecting(&self, frame: &mut ratatui::Frame, cursor: usize, from_boss: bool) {
        let area = frame.area();

        let [header_area, main_area, hints_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(area);

        let [left_area, right_area] =
            Layout::horizontal([Constraint::Fill(2), Constraint::Fill(3)]).areas(main_area);

        if from_boss {
            frame.render_widget(dungeon_view::BossHeaderView::new(&self.state), header_area);
        } else {
            frame.render_widget(dungeon_view::DungeonView::new(&self.state), header_area);
        }

        let dice_widget = match &self.roll_animation {
            Some(anim) => dice_view::DiceView::animated(&self.state.dice_pool, &anim.display),
            None => dice_view::DiceView::new(&self.state.dice_pool),
        };
        frame.render_widget(dice_widget, left_area);

        frame.render_widget(
            score_view::ScoreView::new(
                &self.state.dice_pool,
                &self.state.unlocked,
                &self.state.used_this_room,
            )
            .with_cursor(cursor),
            right_area,
        );

        frame.render_widget(
            Paragraph::new("[Up/Down] Select Category  [S/Enter] Confirm  [Q] Quit"),
            hints_area,
        );
    }

    fn render_shop(&self, frame: &mut ratatui::Frame, items: &[shop::ShopItem], cursor: usize) {
        let area = frame.area();

        let [header_area, main_area, hints_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(area);

        frame.render_widget(dungeon_view::DungeonView::new(&self.state), header_area);

        frame.render_widget(
            shop_view::ShopView::new(items, self.state.gold, cursor),
            main_area,
        );

        frame.render_widget(
            Paragraph::new("[Up/Down] Select  [Enter] Buy  [L] Leave  [Q] Quit"),
            hints_area,
        );
    }

    fn render_rest(&self, frame: &mut ratatui::Frame) {
        frame.render_widget(
            Paragraph::new(format!(
                "REST\nHP: {}/{}\n\n[H] Heal 15 HP\n[Q] Quit",
                self.state.hp, self.state.max_hp
            )),
            frame.area(),
        );
    }

    fn render_unlock(&self, frame: &mut ratatui::Frame) {
        let text = match &self.unlock_options {
            Some(options) => format!(
                "BOSS DEFEATED! Choose a new scoring category:\n\n[1] {}\n[2] {}",
                options[0], options[1]
            ),
            None => "All categories unlocked! Press any key to continue.".to_string(),
        };
        frame.render_widget(Paragraph::new(text), frame.area());
    }

    fn render_game_over(&self, frame: &mut ratatui::Frame) {
        let floor = self.state.dungeon.current_floor();
        frame.render_widget(
            Paragraph::new(format!(
                "GAME OVER\n\nFloor {}\nHP: {}/{}\n\n[Q] or [Enter] to quit",
                floor.floor_num, self.state.hp, self.state.max_hp
            )),
            frame.area(),
        );
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
            GamePhase::Shop { cursor, .. } => {
                let cursor = *cursor;
                self.handle_shop(code, cursor)
            }
            GamePhase::Rest => self.handle_rest(code),
            GamePhase::CategoryUnlock => self.handle_unlock(code),
            GamePhase::GameOver => self.handle_game_over(code),
        }
    }

    fn handle_rolling(&mut self, code: KeyCode) -> bool {
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
                    // Collect held flags before borrowing rng.
                    let held: Vec<bool> =
                        self.state.dice_pool.dice.iter().map(|d| d.held).collect();
                    let display = held
                        .iter()
                        .map(|&h| {
                            if h {
                                None
                            } else {
                                Some(self.rng.random_range(1u8..=6))
                            }
                        })
                        .collect();
                    self.roll_animation = Some(RollAnimation {
                        frames_remaining: ROLL_ANIM_FRAMES,
                        display,
                    });
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

    fn handle_selecting(&mut self, code: KeyCode, cursor: usize) -> bool {
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
            KeyCode::Char('q') | KeyCode::Char('Q') => false,
            _ => true,
        }
    }

    fn handle_scored(&mut self, _code: KeyCode) -> bool {
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
                    self.state.begin_room();
                    self.state.phase = GamePhase::Boss;
                }
            }
            return true;
        }

        // Regular room: extract reward before mutating.
        let reward_gold: u32 = if success {
            match self.state.dungeon.current_floor().current_room() {
                Some(room::Room::Challenge(t)) => t.reward_gold,
                Some(room::Room::Elite(t)) => t.reward_gold,
                _ => 0,
            }
        } else {
            0
        };

        if success {
            self.state.earn_gold(reward_gold);
        } else {
            self.state.take_damage(10);
            if !matches!(self.state.phase, GamePhase::GameOver) {
                self.state.begin_room();
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

    fn handle_shop(&mut self, code: KeyCode, cursor: usize) -> bool {
        let items_len = match &self.state.phase {
            GamePhase::Shop { items, .. } => items.len(),
            _ => return true,
        };

        match code {
            KeyCode::Up => {
                let new_cursor = if cursor == 0 {
                    items_len.saturating_sub(1)
                } else {
                    cursor - 1
                };
                if let GamePhase::Shop { cursor, .. } = &mut self.state.phase {
                    *cursor = new_cursor;
                }
                true
            }
            KeyCode::Down => {
                let new_cursor = if items_len == 0 || cursor + 1 >= items_len {
                    0
                } else {
                    cursor + 1
                };
                if let GamePhase::Shop { cursor, .. } = &mut self.state.phase {
                    *cursor = new_cursor;
                }
                true
            }
            KeyCode::Enter | KeyCode::Char('s') | KeyCode::Char('S') => {
                let item = match &mut self.state.phase {
                    GamePhase::Shop { items, cursor } if *cursor < items.len() => {
                        Some(items.remove(*cursor))
                    }
                    _ => None,
                };
                if let Some(item) = item {
                    self.state.buy_shop_item(item);
                    // Clamp cursor if it's now past the end.
                    if let GamePhase::Shop { items, cursor } = &mut self.state.phase {
                        if *cursor >= items.len() && !items.is_empty() {
                            *cursor = items.len() - 1;
                        }
                    }
                }
                true
            }
            KeyCode::Char('l') | KeyCode::Char('L') | KeyCode::Esc => {
                self.state.dungeon.current_floor_mut().advance();
                self.transition_after_advance();
                true
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => false,
            _ => true,
        }
    }

    fn handle_rest(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('h') | KeyCode::Char('H') => {
                self.state.heal(15);
                self.state.dungeon.current_floor_mut().advance();
                self.transition_after_advance();
                true
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => false,
            _ => true,
        }
    }

    fn handle_unlock(&mut self, code: KeyCode) -> bool {
        if self.unlock_options.is_none() {
            // All categories already unlocked: any key descends.
            self.state.descend(&mut self.rng);
            self.state.phase = GamePhase::Rolling;
            return true;
        }

        let chosen = match (code, &self.unlock_options) {
            (KeyCode::Char('1'), Some(opts)) => Some(opts[0].clone()),
            (KeyCode::Char('2'), Some(opts)) => Some(opts[1].clone()),
            _ => return true,
        };

        if let Some(cat) = chosen {
            self.state.unlock_category(cat);
            self.unlock_options = None;
            self.state.descend(&mut self.rng);
            self.state.phase = GamePhase::Rolling;
        }

        true
    }

    fn handle_game_over(&mut self, code: KeyCode) -> bool {
        matches!(
            code,
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Enter
        )
        .then(|| false)
        .unwrap_or(true)
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn roll_hint(&self) -> &'static str {
        if self.state.dice_pool.rolls_remaining == self.state.dice_pool.max_rolls {
            "[R] Roll  [Q] Quit"
        } else {
            "[<Arrow Keys>] Select Die  [<Space>] Hold  [R] Roll  [S] Score  [Q] Quit"
        }
    }

    // Advance the roll animation by one tick (called once per render loop iteration).
    // When the frame count reaches zero, clears the animation so real values render.
    fn tick_animation(&mut self) {
        // Decrement and check completion; borrow of roll_animation ends after this block.
        let is_active = match self.roll_animation.as_mut() {
            None => return,
            Some(anim) => {
                anim.frames_remaining -= 1;
                anim.frames_remaining > 0
            }
        };

        if !is_active {
            self.roll_animation = None;
            return;
        }

        // Build fresh random display values. Collect held flags first so the
        // borrow on dice_pool ends before we borrow rng.
        let held: Vec<bool> = self.state.dice_pool.dice.iter().map(|d| d.held).collect();
        let new_display: Vec<Option<u8>> = held
            .iter()
            .map(|&h| {
                if h {
                    None
                } else {
                    Some(self.rng.random_range(1u8..=6))
                }
            })
            .collect();

        if let Some(anim) = self.roll_animation.as_mut() {
            anim.display = new_display;
        }
    }

    // Set the correct phase after the floor's current_room index has been advanced.
    fn transition_after_advance(&mut self) {
        match self.state.dungeon.current_floor().current_room() {
            Some(room::Room::Rest) => {
                self.state.phase = GamePhase::Rest;
            }
            Some(_) | None => {
                let is_boss_next = self.state.dungeon.current_floor().boss_next();
                self.state.begin_room();
                self.state.phase = if is_boss_next {
                    GamePhase::Boss
                } else {
                    GamePhase::Rolling
                };
            }
        }
    }

    // Pick two unique categories from those not yet unlocked. Returns None when
    // fewer than two remain (all categories have been unlocked).
    fn pick_unlock_options(&mut self) -> Option<[ScoreCategory; 2]> {
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
