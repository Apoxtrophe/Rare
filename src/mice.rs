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

use crate::mouse;

// CAMERA DEFAULTS
const CAMERA_SCALE: f32 = 0.5;
// MICE DEFUALTS
const BRAIN: [usize; 5] = [10,12,8,6,2];
const VISION_RANGE: f32 = 100.0;
const VISION_ANGLE: f32 = 90.0;
const VISION_LINES: usize = 10;
const COLOR_DEFAULT: [f32; 3] = [1.0, 1.0, 1.0];
const MICE_VELOCITY: f32 = 5.0;
const MICE_ROTATION: f32 = 50.0;
// SIMULATION DEFAULTS
const DEBUG: bool = false;
const POLULATION: usize = 100;
const MAP_SIZE: f32 = 1000.0;
const MIN_RADIUS: f32 = 400.0;
const MUTATION: f32 = 0.1;
const SIMULATION_TIME: f32 = 10.0;
// FOOD DEFAULTS
const FOOD_COUNT: usize = 100;
const FOOD_RADIUS: f32 = 2.0;

#[derive(Resource)]
pub struct Generation {
    epoch: usize,
    max_fitness: usize,
}

#[derive(Component)]
pub struct Cheese; 

#[derive(Component)]
pub struct Mice {
    position: Vec3,
    direction: Quat, 
    sight: Vec<f32>,
    fitness: usize,
    color: [f32; 3],
    brain: Network,
}

#[derive(Resource)]
pub struct GenerationTimer(Timer);


impl Default for Mice {
    fn default() -> Self {
        let mut rnd = rand::thread_rng();
        let mice_positon = Vec3::new(0.0, 0.0, 1.0);
        let mice_direction = Quat::from_rotation_z(rnd.gen_range(0.0..360.0_f32).to_radians());
        Mice {
            position: mice_positon,
            direction: mice_direction,
            sight: vec![0.0; VISION_LINES],
            fitness: 0,
            color: COLOR_DEFAULT,
            brain: Network::new(BRAIN.to_vec()),
        }
    }
}

pub fn mice_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(Generation{
        epoch: 0,
        max_fitness: 0,
    });
    commands.insert_resource(GenerationTimer(Timer::from_seconds(SIMULATION_TIME, TimerMode::Repeating)));
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: CAMERA_SCALE, // Zoom out (values less than 1.0 zoom out, values greater than 1.0 zoom in)
            near: -1000.0, // Ensure it encompasses your z-range
            far: 1000.0,   // Ensure it encompasses your z-range
            
            ..Default::default()
        },
        ..Default::default()
    });
    let mice_mesh: Mesh2dHandle = meshes
        .add(Triangle2d::new(
            Vec2::new(-1.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 3.0),
    )).into();
    for i in 0..POLULATION {
        commands.spawn((MaterialMesh2dBundle {
            mesh: mice_mesh.clone(),
            material: materials.add(Color::srgb_from_array(COLOR_DEFAULT)),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            ..default()
        },
        Mice::default()));
    }
    let cheese_mesh: Mesh2dHandle = meshes
        .add(Circle {
            radius: FOOD_RADIUS,
            ..Default::default()
        }).into();
    for i in 0..FOOD_COUNT {
        let mut rng = rand::thread_rng();
        
        let angle = rng.gen_range(0.0..360.0_f32).to_radians();
        
        let radius = rng.gen_range((MIN_RADIUS/2.0)..(MAP_SIZE/2.0) as f32);
        
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: cheese_mesh.clone(),
                material: materials.add(Color::Srgba(DARK_ORANGE.into())),
                transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
                ..default()
            },
            Cheese,
        ));
    }
    
}

fn new_food_pos(
) -> Vec3{
    let mut rng = rand::thread_rng();
    let angle = rng.gen_range(0.0..360.0_f32).to_radians();
    
    let radius = rng.gen_range((MIN_RADIUS/2.0)..(MAP_SIZE/2.0) as f32);
    
    let x = radius * angle.cos();
    let y = radius * angle.sin();
    let new_pos = Vec3::new(x, y, 0.0);
    new_pos
}

// Collect changes to the mice
pub fn mice_collect(
    mut mice: Query<&mut Mice>,
    mut food_query: Query<&mut Transform, With<Cheese>>,
    mut gizmo: Gizmos,
) {
    for mut mice in mice.iter_mut() {
        mice.sight = mice_vision(&mut mice, &food_query, &mut gizmo);
        let neura_outputs = mice_neura(&mice);
        mice.position = neura_outputs.0;
        mice.direction = neura_outputs.1;
        for mut transform in food_query.iter_mut() {
            let food_output = food_move(&mut mice, &mut transform);
            mice.fitness = food_output.0;
            transform.translation = food_output.1;
        }
    }
}

// Apply changes to the mice
pub fn mice_apply(
    mut mouse_query: Query<(&mut Mice, &mut Transform, &mut Handle<ColorMaterial>), With<Mice>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (mut mice, mut transform, mut color) in mouse_query.iter_mut() {
        let material = materials.get_mut(color.id()).unwrap();
        material.color = Color::srgb_from_array(mice.color);
        transform.translation = mice.position;
        transform.rotation = mice.direction;
    }
}

pub fn camera_zoom(
    mut query: Query<&mut OrthographicProjection, With<Camera>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for mut projection in query.iter_mut() {
        if keyboard.just_pressed(KeyCode::Equal) {
            projection.scale += 0.1;
        }
        if keyboard.just_pressed(KeyCode::Minus) {
            projection.scale -= 0.1;
        }
    }
}

fn mice_vision(
  mice: &mut Mice,
  food_query: &Query<&mut Transform, With<Cheese>>,
  gizmo: &mut Gizmos,
) -> Vec<f32>{
    let angles = (0..VISION_LINES)
        .map(|i| {
            (-VISION_ANGLE / 2.0 + i as f32 * VISION_ANGLE / (VISION_LINES - 1) as f32).to_radians()
        }).collect::<Vec<f32>>();

    let mice_rotation = mice.direction.to_euler(EulerRot::XYZ).2;
    let ray_start = mice.position.xy();
    let sight_distances: Vec<f32> = angles
        .iter()
        .map(|&angle| {
            let ray_direction =
                Vec2::new(-(mice_rotation + angle).sin(), (mice_rotation + angle).cos());
            let ray_end = ray_start + ray_direction * VISION_RANGE;
            if DEBUG {gizmo.line_2d(ray_start, ray_end, Color::from(GREY));}
            food_query
                .iter()
                .filter_map(|transform| {
                    ray_intersects_aabb(
                        ray_start,
                        ray_end,
                        Aabb2d::new(transform.translation.truncate(), Vec2::new(FOOD_RADIUS, FOOD_RADIUS)),
                    )
                }).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(VISION_RANGE)
        }) .collect();
    
    let sight_output: Vec<f32> = sight_distances
        .iter()
        .map(|&d| (d.clamp(0.0, 1.0) - 1.0).abs().sqrt())
        .collect();
    sight_output
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

fn mice_move(input: f32, mice: &Mice) -> Vec3 {
    let forward = Quat::from_rotation_z(mice.direction.to_euler(EulerRot::XYZ).2) * Vec3::Y;
    let position = (forward * MICE_VELOCITY * input.abs()) + mice.position;
    position
}

fn mice_turn(input: f32, mice: &Mice) -> Quat{
    let mut z_rotation = mice.direction.to_euler(EulerRot::XYZ).2 + MICE_ROTATION.to_radians() * input;
    z_rotation %= 2.0 * PI;
    let direction = Quat::from_rotation_z(z_rotation);
    direction
}  

fn mice_neura(
    mice: &Mice,
) -> (Vec3, Quat ) {
    let inputs = mice.sight.clone();
    let outputs = mice.brain.forward(inputs);
    let movement = mice_move(outputs[0], mice); 
    let direction = mice_turn(outputs[1], mice);
    (movement, direction)
}

fn food_move(
    mice: &mut Mice,
    food_transform: &mut Transform,
) -> (usize, Vec3) {
    let mut mice_fitness = mice.fitness;
    let mut food_position = food_transform.translation;
    if mice.position.distance(food_transform.translation) < FOOD_RADIUS * 2.0 {
        mice_fitness += 1;
        food_position = new_food_pos();
    }
    (mice_fitness, food_position)
}

pub fn mice_generation(
    mut query: Query<(&mut Mice, &mut Transform, Entity), With<Mice>>,
    mut generation: ResMut<Generation>,
    mut gen_timer: ResMut<GenerationTimer>,
    time: ResMut<Time>,
) {
    if gen_timer.0.tick(time.delta()).just_finished() {
        generation.epoch += 1;
    
        if let Some((best_mice, _, _)) = query.iter().max_by_key(|(mice, _,_)| mice.fitness) {
            let best_brain = best_mice.brain.clone();
        
            println!("{} *** Fitness: {}", generation.epoch, best_mice.fitness);
            for (mut mice, _, _) in query.iter_mut() {
                let mut new_brain = best_brain.clone();
                new_brain.mutate(MUTATION);
                mice.position = Mice::default().position;
                mice.direction = Mice::default().direction;
                mice.fitness = Mice::default().fitness;
                mice.brain = new_brain;
            }
        }
    }
    
}