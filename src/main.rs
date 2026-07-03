mod dice;
mod dungeon;
mod game;
mod relics;
mod scoring;
mod ui;

fn main() {
    let mut rng = rand::rng();
    let state = game::GameState::new(&mut rng);
    ui::App::new(state).run().expect("TUI error");
}
