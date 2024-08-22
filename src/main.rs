
use bevy::{prelude::*, text::FontAtlas};
mod mice;
use mice::*;

mod pendulum;
use pendulum::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //.add_systems(Startup, mice_setup)
        //.add_systems(Update, mice_collect)
        //.add_systems(Update, mice_apply)
        //.add_systems(Update, mice_generation)
        .add_systems(Startup, pendulum_setup)
        .add_systems(Update, camera_zoomies)
        .add_systems(Update, update_pendulum)
        .add_systems(Update, render_pendulum)
        .add_systems(Update, pendulum_network)
        .add_systems(Update, pendulum_generation)
        .run();
}

//meshes: &mut ResMut<Assets<Mesh>>,
//materials: &mut ResMut<Assets<ColorMaterial>>,
