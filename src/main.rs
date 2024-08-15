use astoria_ml::*;
use bevy::{color::palettes::{basic::WHITE, css::PURPLE}, prelude::*, sprite::MaterialMesh2dBundle};

mod mouse;
use mouse::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, create_mouse)
        .add_systems(Update, create_food)
        .add_systems(Update, mouse_vision)
        .add_systems(Update, mouse_update)
        .add_systems(Update, mouse_player)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    
}