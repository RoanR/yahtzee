// Main game screen: rolling and holding dice (Rolling and Boss phases).
//
//   [main]    Min(0)     horizontal split:
//     [left]  Fill(2)    DiceView (die boxes + rolls indicator below)
//     [right] Fill(3)    ScoreView (title + category list)
// The Fill(2)/Fill(3) ratio gives the left panel ~40% and the right ~60% of
// the width. On an 80-col terminal the left gets ~32 cols (enough for 5 dice
// at 5 chars each) and the right gets ~48 cols (enough for long category names).

use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout};

use super::{App, Phase, is_quit, roll_animation::RollAnimation};

// Shared by GamePhase::Rolling and GamePhase::Boss, which behave identically.
pub(super) struct RollingPhase;

impl Phase for RollingPhase {
    fn render(&self, app: &App, frame: &mut ratatui::Frame) {
        let main_area = app.vertical_layout(frame, roll_hint(app));

        let [left_area, right_area] =
            Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
                .areas(main_area);

        frame.render_widget(app.dice_widget(), left_area);
        frame.render_widget(app.score_view_widget(), right_area);
    }

    fn handle_key(&self, app: &mut App, code: KeyCode) -> bool {
        if is_quit(code) {
            return false;
        }
        match (
            app.state.dice_pool.max_rolls != app.state.dice_pool.rolls_remaining,
            code,
        ) {
            (true, KeyCode::Right) => {
                app.state.dice_pool.next_die();
                true
            }
            (true, KeyCode::Left) => {
                app.state.dice_pool.prev_die();
                true
            }
            (true, KeyCode::Char(' ')) => {
                app.state.dice_pool.toggle_selected();
                true
            }
            (_, KeyCode::Char('r') | KeyCode::Char('R')) => {
                if app.state.roll() {
                    // Roll committed; start display animation.
                    app.roll_animation =
                        Some(RollAnimation::new(&app.state.dice_pool, &mut app.rng));
                }
                true
            }
            (true, KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter) => {
                app.state.begin_scoring();
                true
            }
            _ => true,
        }
    }
}

fn roll_hint(app: &App) -> &'static str {
    if app.state.dice_pool.rolls_remaining == app.state.dice_pool.max_rolls {
        "[R] Roll  [Q] Quit"
    } else {
        "[<Arrow Keys>] Select Die  [<Space>] Hold  [R] Roll  [S] Score  [Q] Quit"
    }
}
