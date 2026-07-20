// Upgrade screen widgets: die selection prompt and face selection grid.
//
// Two-step campfire upgrade flow:
//
// Step 1 (UpgradeSelectDie): DiceView with yellow cursor + UpgradeDiePrompt on the right.
// Step 2 (UpgradeSelectFace): FaceSelectView showing all faces of the chosen die,
//   with the cursor face highlighted in cyan.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::{dice::Die, game::UpgradeKind};

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

const FACE_W: u16 = 4;
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
            Paragraph::new(Line::from(format!("{:>1}", value_str)))
                .block(block)
                .render(Rect::new(x, faces_y, FACE_W, FACE_H), buf);
        }
    }
}
