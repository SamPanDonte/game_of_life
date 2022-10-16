#![forbid(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::unwrap_used)]
use clap::Parser;
use game_of_life::{GameOfLife, Config};

fn main() {
    let game = GameOfLife::new(&Config::parse());
    game.run();
}
