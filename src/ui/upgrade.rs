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

use super::{App, dice_view};

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

impl App {
    pub(super) fn render_upgrade_select_die(
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
            frame.render_widget(UpgradeDiePrompt::new(kind), right_area);
        }
    }

    pub(super) fn render_upgrade_select_face(
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
            FaceSelectView::new(
                &self.state.dice_pool.dice[die_index],
                die_index,
                face_cursor,
                kind,
            ),
            main_area,
        );
    }

    pub(super) fn handle_upgrade_select_die(
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
                self.state.phase.cycle_die_cursor(-1, pool_len);
                true
            }
            KeyCode::Right => {
                self.state.phase.cycle_die_cursor(1, pool_len);
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

    pub(super) fn handle_upgrade_select_face(
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
                self.state.phase.cycle_face_cursor(-1, faces_len);
                true
            }
            KeyCode::Right => {
                self.state.phase.cycle_face_cursor(1, faces_len);
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
}
