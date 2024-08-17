use std::{borrow::BorrowMut, f32::consts::PI};

use astoria_ml::Network;
use bevy::{
    asset::Assets,
    color::palettes::css::{DARK_ORANGE, GREY, LIGHT_GOLDENROD_YELLOW},
    math::bounding::*,
    prelude::*,
    sprite::{ColorMaterial, MaterialMesh2dBundle, Mesh2dHandle},
};
use rand::prelude::*;

const FOOD_NUM: usize = 4;
const FOOD_RADI: f32 = 12.0;
const BRAIN_LAYOUT: [usize; 5] = [6, 12, 18, 6, 3];
const MAP_SIZE: i32 = 500;
const MOUSE_VELOCITY: f32 = 4.0;
const MOUSE_TURN_ANGLE: f32 = 2.0;
const MOUSE_SIGHT_DIST: f32 = 300.0;
const MOUSE_SIGHT_LINES: usize = 20;
const MOUSE_SIGHT_ANGLE: f32 = 90.0_f32;
const MOUSE_NOSE_DIST: f32 = 500.0;

const DEBUG: bool = true;
const PLAYER: bool = true; 

#[derive(Component)]
pub struct Mouse {
    position: Vec3,
    rotation: Quat,
    sight: Vec<f32>,
    fitness: usize,
    nose: f32,
    brain: Network,
}

#[derive(Component)]
pub struct Food {
    position: Vec3,
}

pub fn mouse_create(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mouse_mesh = meshes
        .add(Triangle2d::new(
            Vec2::new(-1.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 3.0),
        ))
        .into();

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: mouse_mesh,
            transform: Transform::default().with_scale(Vec3::splat(12.)),
            material: materials.add(Color::from(LIGHT_GOLDENROD_YELLOW)),
            ..default()
        },
        Mouse {
            position: Vec3::new(0.0, 0.0, 1.0),
            rotation: Quat::from_rotation_z(0.0),
            sight: vec![0.0; MOUSE_SIGHT_LINES],
            nose: 0.0,
            fitness: 0,
            brain: Network::new(BRAIN_LAYOUT.to_vec()),
        },
    ));
}

fn mouse_nose(
    mouse: &mut Mouse,
    food_query: &Query<(Entity, &mut Food)>,
    ) {
    let closest_distance = food_query
        .iter()
        .map(|food| mouse.position.distance(food.1.position))
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(MOUSE_NOSE_DIST);
    mouse.nose = (1.0 - (closest_distance / MOUSE_NOSE_DIST)).clamp(0.0, 1.0);
}


pub fn mouse_update(
    mut commands: Commands,
    mut mouse_query: Query<(&mut Mouse, &mut Transform)>,
    mut food_query: Query<(Entity, &mut Food)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut gizmo: Gizmos,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Text, With<Debug>>,
) {
  for (mut mouse, mut transform) in mouse_query.iter_mut() {
    mouse_vision(&mut mouse, &food_query, &mut gizmo);
    mouse_nose(&mut mouse, &food_query);
    update_food(&mut commands, &mut mouse, &mut food_query, &mut meshes, &mut materials);
    update_mouse_transform(&mut mouse, &mut transform);
    if PLAYER {mouse_player(&keyboard_input, &mut mouse)}
    if DEBUG {mouse_debug(&mut mouse, &mut query)}
  }
}

fn mouse_vision(
  mouse: &mut Mouse,
  food_query: &Query<(Entity, &mut Food)>,
  gizmo: &mut Gizmos,
) {
    let angles = (0..MOUSE_SIGHT_LINES)
        .map(|i| {
            (-MOUSE_SIGHT_ANGLE / 2.0 + i as f32 * MOUSE_SIGHT_ANGLE / (MOUSE_SIGHT_LINES - 1) as f32)
                .to_radians()
        })
        .collect::<Vec<f32>>();

    let mouse_rotation = mouse.rotation.to_euler(EulerRot::XYZ).2;
    let ray_start = mouse.position.xy();
    let sight_distances: Vec<f32> = angles
        .iter()
        .map(|&angle| {
            let ray_direction =
                Vec2::new(-(mouse_rotation + angle).sin(), (mouse_rotation + angle).cos());
            let ray_end = ray_start + ray_direction * MOUSE_SIGHT_DIST;
            if DEBUG {gizmo.line_2d(ray_start, ray_end, Color::from(GREY));}
            food_query
                .iter()
                .filter_map(|(_,food)| {
                    ray_intersects_aabb(
                        ray_start,
                        ray_end,
                        Aabb2d::new(food.position.truncate(), Vec2::new(FOOD_RADI, FOOD_RADI)),
                    )
                })
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(MOUSE_SIGHT_DIST)
        })
        .collect();
    
    mouse.sight = sight_distances
        .iter()
        .map(|&d| (d.clamp(0.0, 1.0) - 1.0).abs().sqrt())
        .collect();
    println!("{}", mouse.fitness);
}

fn ray_intersects_aabb(ray_start: Vec2, ray_end: Vec2, aabb: Aabb2d) -> Option<f32> {
    let ray_dir = ray_end - ray_start;
    let inv_dir = Vec2::new(1.0 / ray_dir.x, 1.0 / ray_dir.y);
    let (min, max) = (aabb.min, aabb.max);

    let t1 = (min.x - ray_start.x) * inv_dir.x;
    let t2 = (max.x - ray_start.x) * inv_dir.x;
    let t3 = (min.y - ray_start.y) * inv_dir.y;
    let t4 = (max.y - ray_start.y) * inv_dir.y;

    let tmin = t1.min(t2).max(t3.min(t4));
    let tmax = t1.max(t2).min(t3.max(t4));

    if tmax < 0.0 || tmin > tmax {
        None
    } else {
        Some(tmin.max(0.0))
    }
}

fn update_food(
    commands: &mut Commands,
    mouse: &mut Mouse,
    food_query: &mut Query<(Entity, &mut Food)>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
  let food_mesh: Mesh2dHandle = meshes.add(Circle::new(FOOD_RADI)).into();
  let existing_food = food_query.iter().count();
  let food_needed = FOOD_NUM - existing_food;
  
  for _ in 0..food_needed {
    let (x , y) = random_position_in_map();
    commands.spawn((
      MaterialMesh2dBundle {
        mesh: food_mesh.clone(),
        transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
        material: materials.add(Color::from(DARK_ORANGE)),
        ..Default::default()
      },
      Food {
        position: Vec3::new(x, y, 0.0),
      },
    ));
  }
  for (food_entity, food) in food_query.iter_mut() {
    if mouse.position.distance(food.position) < FOOD_RADI * 2.0 {
      commands.entity(food_entity).despawn();
      mouse.fitness += 1;
    }
  }
}

fn mouse_player(keyboard_input: &Res<ButtonInput<KeyCode>>, mouse: &mut Mouse) {
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        mouse_move(1.0, mouse);
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        mouse_turn(1.0, mouse);
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        mouse_turn(-1.0, mouse);
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        mouse_move(-1.0, mouse);
    }
    
}

fn update_mouse_transform(mouse: &mut Mouse, transform: &mut Transform) {
    transform.translation = mouse.position;
    transform.rotation = Quat::from_rotation_z(mouse.rotation.to_euler(EulerRot::XYZ).2);
}


fn mouse_move(input: f32, mouse: &mut Mouse) {
    let forward = Quat::from_rotation_z(mouse.rotation.to_euler(EulerRot::XYZ).2) * Vec3::Y;
    mouse.position += forward * MOUSE_VELOCITY * input;
}

fn mouse_turn(input: f32, mouse: &mut Mouse) {
    let mut z_rotation = mouse.rotation.to_euler(EulerRot::XYZ).2 + MOUSE_TURN_ANGLE.to_radians() * input;
    z_rotation %= 2.0 * PI;
    mouse.rotation = Quat::from_rotation_z(z_rotation);
}

fn random_position_in_map() -> (f32, f32) {
    let mut rng = rand::thread_rng();
    (
        rng.gen_range((-MAP_SIZE / 2)..(MAP_SIZE / 2)) as f32,
        rng.gen_range((-MAP_SIZE / 2)..(MAP_SIZE / 2)) as f32,
    )
}

#[derive(Component, Clone)]
pub struct Debug {
    pub output: String,
}

fn mouse_debug(
    mouse: &Mouse,
    query: &mut Query<&mut Text, With<Debug>>,
) {
    println!("eee");
    let mut sight_output: Vec<&str> = Vec::new();
    for &sight in &mouse.sight {
        let sight_line = match sight {
            0.05..=0.25 => "░",
            0.25..=0.50 => "▒",
            0.50..=0.75 => "▓",
            0.75..=1.00 => "█",
            _           => "   ",
        };
        sight_output.push(sight_line);
    }

    let new_text = format!(
        "{}\nPosition: {:?}\nRotation: {:.2} degrees\nFitness: {}\nNose: {}",
        sight_output.join(""),
        mouse.position,
        quat_to_degrees(mouse.rotation),
        mouse.fitness,
        mouse.nose,
    );

    for mut text in query.iter_mut() {
        println!("{:?}", text.clone());
        text.sections[0].value = new_text.clone();
    }
}

fn quat_to_degrees(quat: Quat) -> Vec3 {
    let (axis, angle_rad) = quat.to_axis_angle();
    let angle_deg = angle_rad.to_degrees();
    axis * angle_deg
}