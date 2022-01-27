use bevy::prelude::*;

// the `bevy_main` proc_macro generates the required ios boilerplate
#[bevy_main]
fn main() {
    bevy_bird_lib::start_game();
}
