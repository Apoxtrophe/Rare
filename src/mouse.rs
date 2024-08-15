use std::{borrow::BorrowMut, f32::consts::PI};

use astoria_ml::Network;
use bevy::{asset::Assets, color::palettes::css::{DARK_GREEN, LIGHT_GOLDENROD_YELLOW, LINEN, ORANGE, PURPLE, RED, WHITE}, input::keyboard::KeyboardInput, prelude::{Commands, Mesh, ResMut}, reflect::Array, sprite::{ColorMaterial, MaterialMesh2dBundle, Mesh2dHandle}};
use bevy::prelude::*;
use rand::prelude::*;
use bevy::math::bounding::*;

const FOOD_NUM: usize = 1;
const FOOD_RADI: f32 = 12.0;
const BRAIN_LAYOUT: [usize;5] = [6, 12, 18, 6, 3];
const MAP_SIZE: i32 = 500;
const MOUSE_VELOCITY: f32 = 4.0; 
const MOUSE_TURN_ANGLE: f32 = 1.0;
const MOUSE_SIGHT_DIST: f32 = 300.0;
const MOUSE_SIGHT_LINES: usize = 10;
const MOUSE_SIGHT_ANGLE: f32 = 30.0_f32;

#[derive(Component)]
pub struct Mouse {
  position: Vec3,
  rotation: Quat,
  sight: Vec<f32>,
  fitness: usize, 
  brain: Network,
}
pub fn create_mouse(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  commands.spawn(Camera2dBundle::default());
  let mouse_mesh: Mesh2dHandle = meshes.add(Triangle2d::new(Vec2 { x: -1.0, y: 0.0 }, Vec2::new(1.0, 0.0), Vec2::new(0.0, 3.0))).into();
    
  commands.spawn((MaterialMesh2dBundle {
    mesh: mouse_mesh,
    transform: Transform::default().with_scale(Vec3::splat(12.)),
    material: materials.add(Color::from(LIGHT_GOLDENROD_YELLOW)),
    ..default()
  },
  Mouse{
    position: Vec3::new(0.0, 0.0, 1.0),
    rotation: Quat::from_rotation_z(0.0),
    sight: vec![0.0;MOUSE_SIGHT_LINES],
    fitness: 0,
    brain: Network::new(BRAIN_LAYOUT.to_vec()),
  }));
}

fn mouse_move(
  input: f32,
  mouse: &mut Mouse,
) {
  let forward = Quat::from_rotation_z(mouse.rotation.z) * Vec3::Y;
  mouse.position += forward * MOUSE_VELOCITY * input;
}

fn mouse_turn (
  input: f32,
  mouse: &mut Mouse,
) {
  mouse.rotation.z += MOUSE_TURN_ANGLE.to_radians() * input;
  if mouse.rotation.z >= 360.0_f32.to_radians() || mouse.rotation.z <= -360.0_f32.to_radians() {
    mouse.rotation = Quat::from_rotation_z(0.0);
  }
}

pub fn mouse_vision(
    mut query: Query<(&mut Mouse, &Transform)>,
    food_query: Query<&Food>,
    mut gizmo: Gizmos,
) {
  let angles: Vec<f32> = (0..MOUSE_SIGHT_LINES)
          .map(|i| (-MOUSE_SIGHT_ANGLE / 2.0 + i as f32 * MOUSE_SIGHT_ANGLE / (MOUSE_SIGHT_LINES - 1) as f32).to_radians())
          .collect();
  for (mut mouse, transform) in query.iter_mut() {
    let mut sight_distances = [MOUSE_SIGHT_DIST; MOUSE_SIGHT_LINES]; // Initialize sight distances

    for food in food_query.iter() {
      let half_extents = Vec2::new(FOOD_RADI, FOOD_RADI);
      let food_aabb = Aabb2d::new(
          food.position.truncate(),  // This is the center of the AABB
          half_extents,              // This is the half size (half extents) of the AABB
      );
      for i in 0..MOUSE_SIGHT_LINES {
        // We're casting a single ray (sight line) in front of the mouse
        let (sin, cos) = (transform.rotation.to_euler(EulerRot::XYZ).2 + angles[i] as f32).sin_cos();
        let ray_direction = Vec2::new(-sin, cos);
        let ray_start = mouse.position.xy();
        let ray_end = ray_start + ray_direction * MOUSE_SIGHT_DIST;
        
        gizmo.line_2d(ray_start, ray_end, Color::from(WHITE));
        
        if let Some(intersection_distance) = ray_intersects_aabb(ray_start, ray_end, food_aabb) {
          if intersection_distance < sight_distances[i] {
            sight_distances[i] = intersection_distance;
          }
        }
      }
    }
    for i in 0..MOUSE_SIGHT_LINES {
      mouse.sight[i] = (sight_distances[i].clamp(0.0, 1.0) -1.0).abs().powf(1.0/2.0);
    }
    mouse_sight_visualize(&mouse.sight);
  }
}

pub fn mouse_sight_visualize(
  mouse_sight: &Vec<f32>,
) {
  let mut output: Vec<&str> = Vec::new();
  for i in 0..mouse_sight.len() {
    match mouse_sight[i] {
      0.15..0.25 =>  output.push("░░░"),
      0.25..0.50 => output.push("▒▒▒"),
      0.50..0.75 => output.push("▓▓▓"),
      0.75..1.00 => output.push("▓▓▓"),
      _ => output.push("   "), 
    }
  }
  println!("{}", output.join(""));
}

fn ray_intersects_aabb(ray_start: Vec2, ray_end: Vec2, aabb: Aabb2d) -> Option<f32> {
    let ray_dir = ray_end - ray_start;

    let inv_dir = Vec2::new(1.0 / ray_dir.x, 1.0 / ray_dir.y);

    let min = aabb.min;
    let max = aabb.max;

    let t1 = (min.x - ray_start.x) * inv_dir.x;
    let t2 = (max.x - ray_start.x) * inv_dir.x;
    let t3 = (min.y - ray_start.y) * inv_dir.y;
    let t4 = (max.y - ray_start.y) * inv_dir.y;

    let tmin = t1.min(t2).max(t3.min(t4));
    let tmax = t1.max(t2).min(t3.max(t4));
    if tmax < 0.0 {
        return None;
    }
    if tmin > tmax {
        return None;
    }
    if tmin < 0.0 {
        return Some(tmax);
    }
    Some(tmin)
}

pub fn mouse_update(
  mut commands: Commands,
  mut query: Query<(&mut Mouse, &mut Transform)>,
  mut food_query: Query<(Entity, &mut Food)>,
) {
  for (mut mouse, mut transform) in query.iter_mut() {
    // Set the position and rotation without affecting scale
    transform.translation = mouse.position;
    transform.rotation = Quat::from_rotation_z(mouse.rotation.z);
    
    for (food_entity, mut food) in food_query.iter_mut() {
      let distance = mouse.position.distance(food.position);
      if distance < FOOD_RADI * 2.0 {
        commands.entity(food_entity).despawn();
        mouse.fitness += 1;
      }
    }
  }
}

pub fn mouse_player(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut query: Query<(&mut Mouse, &mut Transform)>,
) {    
  
  if let (mut mouse, mut transform) = query.single_mut() {
    if keyboard_input.pressed(KeyCode::ArrowUp) {
      mouse_move(1.0, mouse.borrow_mut());
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
      mouse_turn(1.0, mouse.borrow_mut());
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
      mouse_turn(-1.0, mouse.borrow_mut());
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
      mouse_move(-1.0, mouse.borrow_mut());
    }
    transform.translation = mouse.position;
    transform.rotation = Quat::from_rotation_z(mouse.rotation.z);
  }
}

#[derive(Component)]
pub struct Food {
  position: Vec3,
}
  
pub fn create_food(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  mut query: Query<(&mut Food, &mut Transform)>,
) {
  let food_mesh: Mesh2dHandle = meshes.add(Circle::new(FOOD_RADI)).into();
  let mut food_count = 0;
  for (mut food, mut transform) in query.iter_mut(){
    food_count += 1; 
    transform.translation = food.position;
  }
  let food_debt = FOOD_NUM - food_count;
  for i in 0..food_debt {
    let mut rng = rand::thread_rng();
    let x = rng.gen_range((-MAP_SIZE / 2)..(MAP_SIZE /2));
    let y = rng.gen_range((-MAP_SIZE / 2)..(MAP_SIZE /2));
    commands.spawn((MaterialMesh2dBundle {
            mesh: food_mesh.clone(),
            transform: Transform::from_translation(Vec3::new(x as f32, y as f32, 0.0)),
            material: materials.add(Color::from(DARK_GREEN)),
            ..default()
        },
        Food {
          position: Vec3::new(x as f32, y as f32, 0.0),
        }
    ));
  }
}