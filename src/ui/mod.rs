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
pub mod rest_view;
pub mod roll_animation;
pub mod room_select;
pub mod score_view;
pub mod scored;
pub mod shop_view;
pub mod unlock;
pub mod upgrade_view;

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
    dice::{Die, DieUpgrade},
    game::{GamePhase, GameState, UpgradeKind},
    scoring::ScoreCategory,
    shop,
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
    stashed_shop: Option<(Vec<shop::ShopItem>, usize)>,
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
            GamePhase::Shop { items, cursor } => self.render_rest_shop(
                frame,
                shop_view::ShopView::new(items, self.state.gold, *cursor),
            ),
            GamePhase::Rest { cursor } => {
                self.render_rest_shop(frame, rest_view::RestView::new(*cursor))
            }
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

    // Main game screen
    //   [main]    Min(0)     horizontal split:
    //     [left]  Fill(2)    DiceView (die boxes + rolls indicator below)
    //     [right] Fill(3)    ScoreView (title + category list)
    // The Fill(2)/Fill(3) ratio gives the left panel ~40% and the right ~60% of
    // the width. On an 80-col terminal the left gets ~32 cols (enough for 5 dice
    // at 5 chars each) and the right gets ~48 cols (enough for long category names).
    fn render_game(&self, frame: &mut ratatui::Frame) {
        let main_area = self.vertical_layout(frame, self.roll_hint());

        // Inner horizontal split inside main_area.
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
                .areas(main_area);

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
    }

    fn render_selecting(&self, frame: &mut ratatui::Frame, cursor: usize) {
        let main_area = self.vertical_layout(
            frame,
            "[Up/Down] Select Category  [S/Enter] Confirm [R/Esc] To Roll [Q] Quit",
        );

        let [left_area, right_area] =
            Layout::horizontal([Constraint::Fill(2), Constraint::Fill(3)]).areas(main_area);

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
    }

    // Used for shop and rest sites
    fn render_rest_shop(&self, frame: &mut ratatui::Frame, widget: impl Widget) {
        let main_area = self.vertical_layout(
            frame,
            "[Up/Down] Select  [Enter] Confirm  [L] Leave [Q] Quit",
        );
        frame.render_widget(widget, main_area);
    }

    fn render_upgrade_select_die(
        &self,
        frame: &mut ratatui::Frame,
        die_cursor: usize,
        kind: UpgradeKind,
        pending_die: Option<&Die>,
    ) {
        let hint = if pending_die.is_some() {
            "[Left/Right] Select  [Enter] Replace  [Q] Quit"
        } else {
            "[Left/Right] Select  [Enter] Confirm  [Esc] Back  [Q] Quit"
        };
        let main_area = self.vertical_layout(frame, hint);
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
                .areas(main_area);

        frame.render_widget(
            dice_view::DiceView::with_upgrade_cursor(&self.state.dice_pool, die_cursor),
            left_area,
        );
        if let Some(die) = pending_die {
            frame.render_widget(
                Paragraph::new(format!("Replacing with\n{}", die.label())),
                right_area,
            );
        } else {
            frame.render_widget(upgrade_view::UpgradeDiePrompt::new(kind), right_area);
        }
    }

    fn render_upgrade_select_face(
        &self,
        frame: &mut ratatui::Frame,
        die_index: usize,
        face_cursor: usize,
        kind: UpgradeKind,
    ) {
        let main_area = self.vertical_layout(
            frame,
            "[Left/Right] Select  [Enter] Upgrade  [Esc] Back  [Q] Quit",
        );
        frame.render_widget(
            upgrade_view::FaceSelectView::new(
                &self.state.dice_pool.dice[die_index],
                die_index,
                face_cursor,
                kind,
            ),
            main_area,
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

        let cursor = cursor.min(available.len() - 1);

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
            KeyCode::Esc | KeyCode::Char('r') | KeyCode::Char('R') => {
                self.state.back_room();
                true
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => false,
            _ => true,
        }
    }

    fn handle_shop(&mut self, cursor: usize) {
        // Only proceed if the player can afford the selected item.
        if !match &self.state.phase {
            GamePhase::Shop { items, .. } => items
                .get(cursor)
                .is_some_and(|it| self.state.gold >= it.price),
            _ => false,
        } {
            return;
        }

        // Remove the item from the available list
        let item = match &mut self.state.phase {
            GamePhase::Shop { items, .. } if cursor < items.len() => Some(items.remove(cursor)),
            _ => None,
        };

        //
        if let Some(item) = item {
            match item.kind {
                shop::ShopItemKind::DieUpgrade(kind) => {
                    self.state.spend_gold(item.price);
                    // Extract remaining shop state and stash it while the player
                    // picks which die and face to upgrade.
                    let old = std::mem::replace(&mut self.state.phase, GamePhase::GameOver);
                    if let GamePhase::Shop { items, cursor } = old {
                        self.stashed_shop = Some((items, cursor));
                    }
                    self.state.phase = GamePhase::UpgradeSelectDie {
                        die_cursor: 0,
                        kind,
                        from_shop: true,
                        pending_die: None,
                    };
                }
                shop::ShopItemKind::SpecialDie(kind) => {
                    self.state.spend_gold(item.price);
                    let pending_die = Some(kind.create_die());
                    let old = std::mem::replace(&mut self.state.phase, GamePhase::GameOver);
                    if let GamePhase::Shop { items, cursor } = old {
                        self.stashed_shop = Some((items, cursor));
                    }
                    self.state.phase = GamePhase::UpgradeSelectDie {
                        die_cursor: 0,
                        kind: UpgradeKind::Augment,
                        from_shop: true,
                        pending_die,
                    };
                }
                _ => {
                    self.state.buy_shop_item(item);
                    // Clamp cursor if it's now past the end.
                    if let GamePhase::Shop { items, cursor } = &mut self.state.phase
                        && *cursor >= items.len()
                        && !items.is_empty()
                    {
                        *cursor = items.len() - 1;
                    }
                }
            }
        }
    }

    fn handle_rest(&mut self, cursor: usize) {
        match cursor {
            0 => {
                self.state.heal(15);
                self.state.dungeon.current_floor_mut().advance();
                self.transition_after_advance();
            }
            1 => {
                self.state.phase = GamePhase::UpgradeSelectDie {
                    die_cursor: 0,
                    kind: UpgradeKind::Augment,
                    from_shop: false,
                    pending_die: None,
                };
            }
            _ => {
                self.state.phase = GamePhase::UpgradeSelectDie {
                    die_cursor: 0,
                    kind: UpgradeKind::Enchant,
                    from_shop: false,
                    pending_die: None,
                };
            }
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

    fn handle_upgrade_select_die(
        &mut self,
        code: KeyCode,
        die_cursor: usize,
        kind: UpgradeKind,
        from_shop: bool,
        pending_die: Option<Die>,
    ) -> bool {
        let pool_len = self.state.dice_pool.dice.len();
        match code {
            KeyCode::Left => {
                self.state.phase = GamePhase::UpgradeSelectDie {
                    die_cursor: (die_cursor + pool_len - 1) % pool_len,
                    kind,
                    from_shop,
                    pending_die,
                };
                true
            }
            KeyCode::Right => {
                self.state.phase = GamePhase::UpgradeSelectDie {
                    die_cursor: (die_cursor + 1) % pool_len,
                    kind,
                    from_shop,
                    pending_die,
                };
                true
            }
            KeyCode::Enter | KeyCode::Char('s') | KeyCode::Char('S') => {
                if let Some(die) = pending_die {
                    self.state.replace_die_at(die_cursor, die);
                    let (items, cursor) = self.stashed_shop.take().unwrap_or_default();
                    self.state.phase = GamePhase::Shop { items, cursor };
                } else {
                    self.state.phase = GamePhase::UpgradeSelectFace {
                        die_index: die_cursor,
                        face_cursor: 0,
                        kind,
                        from_shop,
                    };
                }
                true
            }
            KeyCode::Esc if !from_shop && pending_die.is_none() => {
                self.state.phase = GamePhase::Rest { cursor: 0 };
                true
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => false,
            _ => true,
        }
    }

    fn handle_upgrade_select_face(
        &mut self,
        code: KeyCode,
        die_index: usize,
        face_cursor: usize,
        kind: UpgradeKind,
        from_shop: bool,
    ) -> bool {
        let faces_len = self.state.dice_pool.dice[die_index].faces().len();
        match code {
            KeyCode::Left => {
                self.state.phase = GamePhase::UpgradeSelectFace {
                    die_index,
                    face_cursor: (face_cursor + faces_len - 1) % faces_len,
                    kind,
                    from_shop,
                };
                true
            }
            KeyCode::Right => {
                self.state.phase = GamePhase::UpgradeSelectFace {
                    die_index,
                    face_cursor: (face_cursor + 1) % faces_len,
                    kind,
                    from_shop,
                };
                true
            }
            KeyCode::Enter | KeyCode::Char('s') | KeyCode::Char('S') => {
                let upgrade = match kind {
                    UpgradeKind::Augment => DieUpgrade::Augment {
                        face_index: face_cursor,
                    },
                    UpgradeKind::Enchant => DieUpgrade::Enchant {
                        face_index: face_cursor,
                        bonus_score: 5,
                    },
                };
                self.state.upgrade_die(die_index, upgrade);
                if from_shop {
                    // Return to the shop with any remaining items.
                    let (items, cursor) = self.stashed_shop.take().unwrap_or_default();
                    self.state.phase = GamePhase::Shop { items, cursor };
                } else {
                    self.state.dungeon.current_floor_mut().advance();
                    self.transition_after_advance();
                }
                true
            }
            KeyCode::Esc => {
                self.state.phase = GamePhase::UpgradeSelectDie {
                    die_cursor: die_index,
                    kind,
                    from_shop,
                    pending_die: None,
                };
                true
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => false,
            _ => true,
        }
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
