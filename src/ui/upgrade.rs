// Upgrade screen widgets: die selection prompt and face selection grid.
//
// Two-step campfire upgrade flow:
//
// Step 1 (UpgradeSelectDie): DiceView with yellow cursor + UpgradeDiePrompt on the right.
// Step 2 (UpgradeSelectFace): FaceSelectView showing all faces of the chosen die,
//   with the cursor face highlighted in cyan.

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    dice::{Die, DieUpgrade},
    game::{GamePhase, UpgradeKind},
};

use super::{App, Phase, dice_view, is_quit};

// ─── UpgradeDiePrompt ─────────────────────────────────────────────────────────

// Right-panel text shown during die selection.
pub struct UpgradeDiePrompt {
    kind: UpgradeKind,
}

impl UpgradeDiePrompt {
    pub fn new(kind: UpgradeKind) -> Self {
        Self { kind }
    }
}

impl Widget for UpgradeDiePrompt {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let label = match self.kind {
            UpgradeKind::Augment => "Augment\n(+1 value)",
            UpgradeKind::Enchant => "Enchant\n(+5 score)",
        };
        Paragraph::new(format!("Pick a die to\n{}", label)).render(area, buf);
    }
}

// ─── FaceSelectView ───────────────────────────────────────────────────────────

// Shows all faces of a single die as small boxes; cursor face highlighted cyan.
//
//  Die 2 (D6) -- Augment (+1 value):
//
//  [1] [2] [3] [4] [5] [6]
//       ^-- cursor in cyan

const FACE_W: u16 = 5;
const FACE_H: u16 = 3;
const FACE_GAP: u16 = 1;

pub struct FaceSelectView<'a> {
    die: &'a Die,
    die_index: usize,
    face_cursor: usize,
    kind: UpgradeKind,
}

impl<'a> FaceSelectView<'a> {
    pub fn new(die: &'a Die, die_index: usize, face_cursor: usize, kind: UpgradeKind) -> Self {
        Self {
            die,
            die_index,
            face_cursor,
            kind,
        }
    }
}

impl Widget for FaceSelectView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let kind_label = match self.kind {
            UpgradeKind::Augment => "Augment (+1 value)",
            UpgradeKind::Enchant => "Enchant (+5 score)",
        };
        let title = format!(
            "Die {} ({}) -- {}:",
            self.die_index + 1,
            self.die.label(),
            kind_label
        );

        let title_area = Rect::new(area.x, area.y, area.width, 1);
        Paragraph::new(title).render(title_area, buf);

        let faces_y = area.y + 2;
        for (i, face) in self.die.faces().iter().enumerate() {
            let x = area.x + i as u16 * (FACE_W + FACE_GAP);
            if x + FACE_W > area.right() || faces_y + FACE_H > area.bottom() {
                break;
            }
            let value_str = if face.get_value() == crate::dice::WILD {
                "W".to_string()
            } else {
                face.get_value().to_string()
            };
            let mut block = Block::bordered().border_set(symbols::border::PLAIN);
            if i == self.face_cursor {
                block = block.style(Style::new().cyan());
            }
            Paragraph::new(Line::from(format!("{:>2}", value_str)))
                .block(block)
                .render(Rect::new(x, faces_y, FACE_W, FACE_H), buf);
        }
    }
}

// ─── UpgradeSelectDiePhase ──────────────────────────────────────────────────

// pending_die isn't stored here: it's owned by GameState::phase and isn't
// Clone-cheap (Die holds a Vec<DieFace>), so it's re-read from
// GameState::phase in render/handle_key instead (same pattern ShopPhase uses
// for `items`) rather than cloned into this struct on every frame.
pub(super) struct UpgradeSelectDiePhase {
    pub die_cursor: usize,
    pub kind: UpgradeKind,
    pub from_shop: bool,
}

impl Phase for UpgradeSelectDiePhase {
    fn render(&self, app: &App, frame: &mut ratatui::Frame) {
        let pending_die = match &app.state.phase {
            GamePhase::UpgradeSelectDie { pending_die, .. } => pending_die.as_ref(),
            _ => None,
        };
        let hint = if pending_die.is_some() {
            "[Left/Right] Select  [Enter] Replace  [Q] Quit"
        } else {
            "[Left/Right] Select  [Enter] Confirm  [Esc] Back  [Q] Quit"
        };
        let main_area = app.vertical_layout(frame, hint);
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
                .areas(main_area);

        frame.render_widget(
            dice_view::DiceView::with_upgrade_cursor(&app.state.dice_pool, self.die_cursor),
            left_area,
        );
        if let Some(die) = pending_die {
            frame.render_widget(
                Paragraph::new(format!("Replacing with\n{}", die.label())),
                right_area,
            );
        } else {
            frame.render_widget(UpgradeDiePrompt::new(self.kind), right_area);
        }
    }

    fn handle_key(&self, app: &mut App, code: KeyCode) -> bool {
        if is_quit(code) {
            return false;
        }
        let pool_len = app.state.dice_pool.dice.len();
        let has_pending_die = matches!(
            &app.state.phase,
            GamePhase::UpgradeSelectDie {
                pending_die: Some(_),
                ..
            }
        );
        match code {
            KeyCode::Left => {
                app.state.phase.cycle_die_cursor(-1, pool_len);
                true
            }
            KeyCode::Right => {
                app.state.phase.cycle_die_cursor(1, pool_len);
                true
            }
            KeyCode::Enter | KeyCode::Char('s') | KeyCode::Char('S') => {
                let pending_die = match &mut app.state.phase {
                    GamePhase::UpgradeSelectDie { pending_die, .. } => pending_die.take(),
                    _ => None,
                };
                if let Some(die) = pending_die {
                    app.state.replace_die_at(self.die_cursor, die);
                    let (items, cursor) = app.stashed_shop.take().unwrap_or_default();
                    app.state.phase = GamePhase::Shop { items, cursor };
                } else {
                    app.state.phase = GamePhase::UpgradeSelectFace {
                        die_index: self.die_cursor,
                        face_cursor: 0,
                        kind: self.kind,
                        from_shop: self.from_shop,
                    };
                }
                true
            }
            KeyCode::Esc if !self.from_shop && !has_pending_die => {
                app.state.phase = GamePhase::Rest { cursor: 0 };
                true
            }
            _ => true,
        }
    }
}

// ─── UpgradeSelectFacePhase ─────────────────────────────────────────────────

pub(super) struct UpgradeSelectFacePhase {
    pub die_index: usize,
    pub face_cursor: usize,
    pub kind: UpgradeKind,
    pub from_shop: bool,
}

impl Phase for UpgradeSelectFacePhase {
    fn render(&self, app: &App, frame: &mut ratatui::Frame) {
        let main_area = app.vertical_layout(
            frame,
            "[Left/Right] Select  [Enter] Upgrade  [Esc] Back  [Q] Quit",
        );
        frame.render_widget(
            FaceSelectView::new(
                &app.state.dice_pool.dice[self.die_index],
                self.die_index,
                self.face_cursor,
                self.kind,
            ),
            main_area,
        );
    }

    fn handle_key(&self, app: &mut App, code: KeyCode) -> bool {
        if is_quit(code) {
            return false;
        }
        let faces_len = app.state.dice_pool.dice[self.die_index].faces().len();
        match code {
            KeyCode::Left => {
                app.state.phase.cycle_face_cursor(-1, faces_len);
                true
            }
            KeyCode::Right => {
                app.state.phase.cycle_face_cursor(1, faces_len);
                true
            }
            KeyCode::Enter | KeyCode::Char('s') | KeyCode::Char('S') => {
                let upgrade = match self.kind {
                    UpgradeKind::Augment => DieUpgrade::Augment {
                        face_index: self.face_cursor,
                    },
                    UpgradeKind::Enchant => DieUpgrade::Enchant {
                        face_index: self.face_cursor,
                        bonus_score: 5,
                    },
                };
                app.state.upgrade_die(self.die_index, upgrade);
                if self.from_shop {
                    // Return to the shop with any remaining items.
                    let (items, cursor) = app.stashed_shop.take().unwrap_or_default();
                    app.state.phase = GamePhase::Shop { items, cursor };
                } else {
                    app.state.dungeon.current_floor_mut().advance();
                    app.transition_after_advance();
                }
                true
            }
            KeyCode::Esc => {
                app.state.phase = GamePhase::UpgradeSelectDie {
                    die_cursor: self.die_index,
                    kind: self.kind,
                    from_shop: self.from_shop,
                    pending_die: None,
                };
                true
            }
            _ => true,
        }
    }
}
