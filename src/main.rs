
use bevy::{prelude::*, text::FontAtlas};

mod mouse;
use mouse::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Startup, mouse_setup)
        .add_systems(Update, mouse_update)
        .add_systems(Update, mouse_upkeep)
        .add_systems(Update, update_food)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(Camera2dBundle {
            projection: OrthographicProjection {
                scale: VIEW_SCALE, // Zoom out (values less than 1.0 zoom out, values greater than 1.0 zoom in)
                near: -1000.0, // Ensure it encompasses your z-range
                far: 1000.0,   // Ensure it encompasses your z-range
                
                ..Default::default()
            },
            ..Default::default()
        });
    
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(200.0),
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
                    font_size: 16.0,
                    ..default()
                },
            ),
            Debug {
                output: "Debug Off".to_string(),
            },
        ));
    });
}