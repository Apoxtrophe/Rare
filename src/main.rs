
use bevy::{prelude::*, text::FontAtlas};

mod mouse;
use mouse::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Startup, mouse_create)
        .add_systems(Update, mouse_update)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(Camera2dBundle::default());
    
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(10.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    })
    .with_children(|parent| {
        parent.spawn((
            TextBundle::from_section(
                "Debug Off",
                TextStyle {
                    font_size: 12.0,
                    ..default()
                },
            ),
            Debug {
                output: "Debug Off".to_string(),
            },
        ));
    });
}