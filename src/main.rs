use bevy::prelude::*;

extern crate physics_engine;
use physics_engine::physics::PhsyicsEngine;

fn main() {
    App::new()
        .add_plugins(PhsyicsEngine)
        .run();
}

