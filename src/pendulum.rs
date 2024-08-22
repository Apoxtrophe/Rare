use bevy::{color::palettes::css::{DARK_GREY, GHOST_WHITE, RED}, prelude::*};
use astoria_ml::*;

const CAMERA_SCALE: f32 = 0.5;
const GRAVITY: f32 = 98.1;
const NETWORK_LAYOUT: [usize; 7] = [4, 8, 6, 4, 2, 1, 1];
const RAIL_RADI: f32 = 100.0;
const CART_SIZE: Vec2 = Vec2::new(10.0, 4.0);
const PENDULUM_SIZE: Vec2 = Vec2::new(3.0, 3.0);
const POPULATION: usize = 16;
const MUTATION: f32 = 10.0;
const SIMULATION_TIME: f32 = 10.0;
const POWER_FACTOR: f32 = 100.0;
const LENGTH: f32 = 50.0;
const START_ANGLE: f32 = 180.0_f32;

#[derive(Component, Debug, Clone)]
pub struct PendulumCart {
    angle: f32,
    angular_velocity: f32,
    cart_position: Vec3,
    cart_velocity: Vec3,
    length: f32,
    gravity: f32,
    brain: Network,
    fitness: f32,
    offset: Vec2, // New field for 
    color: Color,
}

#[derive(Component)]
pub struct PendulumLinks {
    cart: Entity,
    pendulum_ball: Entity,
}

#[derive(Resource)]
pub struct Generation {
    epoch: usize,
    max_fitness: f32,
    average_fitness: f32,
}

#[derive(Resource)]
pub struct GenerationTimer(Timer);


impl PendulumCart {
    fn new(
        length: f32,
        gravity: f32,
        offset: Vec2,
    ) -> Self {
        Self {
            angle: START_ANGLE.to_radians(),
            angular_velocity: 0.0,
            cart_position: Vec3::new(0.0, 0.0, 1.0),
            cart_velocity: Vec3::new(0.0, 0.0, 1.0),
            length,
            gravity,
            brain: Network::new(NETWORK_LAYOUT.to_vec(), ActivationFunction::ReLU, ActivationFunction::Tanh),
            fitness: 0.0,
            offset,
            color: Color::srgba(1.0, 1.0, 1.0, 0.05)
        }
    }
    fn update(&mut self, delta_time: f32) {
        let angular_acceleration = (-self.gravity / self.length) * self.angle.sin()
            - (self.cart_velocity.x / self.length) * self.angle.cos();
        self.angular_velocity += angular_acceleration * delta_time;
        self.angle += self.angular_velocity * delta_time;
        
        // Normalize angle
        self.angle = self.angle % (2.0 * std::f32::consts::PI);
        if self.angle > std::f32::consts::PI {
            self.angle -= 2.0 * std::f32::consts::PI;
        } else if self.angle < -std::f32::consts::PI {
            self.angle += 2.0 * std::f32::consts::PI;
        }
        
        // Bind the cart to rail
        if self.cart_position.x < -RAIL_RADI {
            self.cart_position.x = -RAIL_RADI;
            self.cart_velocity.x = 0.0; // Stop the cart if it reaches the minimum bound
        } else if self.cart_position.x > RAIL_RADI {
            self.cart_position.x = RAIL_RADI;
            self.cart_velocity.x = 0.0; // Stop the cart if it reaches the maximum bound
        }
        self.angular_velocity *= 0.999;
        self.cart_position += self.cart_velocity * delta_time;
        self.fitness += normalize_to_range(self.angle.to_degrees(), -180.0, 180.0).abs() * (1.0 / (self.cart_position.x.abs()  + 1.0));
    }
    fn pendulum_position(
        &self,
    ) -> Vec3{
        Vec3::new
        (
            self.cart_position.x + self.length * self.angle.sin(),
            self.cart_position.y - self.length * self.angle.cos(),
            1.0,
        )
    }
    fn reset(
        &mut self,
    ) {
        self.angle = START_ANGLE.to_radians();
        self.angular_velocity = 0.0;
        self.cart_position = Vec3::new(0.0, 0.0, 1.0);
        self.cart_velocity = Vec3::new(0.0, 0.0, 1.0);
        self.fitness = 0.0;
    }
}

pub fn camera_zoomies(
    mut query: Query<&mut OrthographicProjection, With<Camera>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for mut projection in query.iter_mut() {
        projection.viewport_origin = Vec2::new(0.0, 0.0);
        if keyboard.just_pressed(KeyCode::Equal) {
            projection.scale += 0.1;
            println!("Camera scale: {}", projection.scale);
        }
        if keyboard.just_pressed(KeyCode::Minus) {
            projection.scale -= 0.1;
            println!("Camera scale: {}", projection.scale);
        }

    }
}

pub fn pendulum_setup(
    mut commands: Commands, 
) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: CAMERA_SCALE, // Zoom out (values less than 1.0 zoom out, values greater than 1.0 zoom in)
            near: -1000.0, // Ensure it encompasses your z-range
            far: 1000.0,   // Ensure it encompasses your z-range
            ..Default::default()
        },
        ..Default::default()
    });
    commands.insert_resource(GenerationTimer(Timer::from_seconds(SIMULATION_TIME, TimerMode::Repeating)));
    commands.insert_resource(Generation{
        epoch: 0,
        max_fitness: 0.0,
        average_fitness: 0.0,
    });

    let shift = 200.0;
    for i in 0..POPULATION {
        for j in 0..POPULATION {
            let cart_entity = commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(1.0, 1.0, 1.0),
                    custom_size: Some(CART_SIZE), // Cart size
                    ..Default::default()
                },
                ..Default::default()
            })
            .id();
            
            let pendulum_ball_entity = commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::srgba(1.0, 1.0, 1.0, 0.05),
                    custom_size: Some(PENDULUM_SIZE), // Pendulum ball size
                    ..Default::default()
                },
                ..Default::default()
            })
            .id();
            
            commands.entity(cart_entity).insert(PendulumCart::new(LENGTH, GRAVITY, Vec2::new(shift, shift)));
            commands.entity(cart_entity).insert(PendulumLinks {
                cart: cart_entity,
                pendulum_ball: pendulum_ball_entity,
            });
        }
    }
}

pub fn update_pendulum(
    mut query: Query<&mut PendulumCart>,
    time: Res<Time>,
) {
    for mut pendulum_cart in query.iter_mut() {
        pendulum_cart.update(time.delta_seconds());
    }
}

pub fn pendulum_network(
    mut query: Query<&mut PendulumCart>,
    time: Res<Time>,
) {
    for mut pendulum_cart in query.iter_mut() {
        let mut inputs: Vec<f32> = Vec::new();
        
        inputs.push(normalize_to_range(pendulum_cart.angle.to_degrees(), -180.0, 180.0));
        inputs.push(normalize_to_range(pendulum_cart.cart_position.x, -RAIL_RADI, RAIL_RADI));
        inputs.push(normalize_to_range(pendulum_cart.cart_velocity.x, -100.0, 100.0));
        inputs.push(normalize_to_range(pendulum_cart.angular_velocity, -10.0, 10.0));
        let outputs = pendulum_cart.brain.forward(inputs);
        pendulum_cart.cart_velocity.x += outputs[0] * (time.delta_seconds()) * POWER_FACTOR;
    }
}

fn normalize_to_range(value: f32, min: f32, max: f32) -> f32 {
    // Ensure that the value is clamped within the range
    let clamped_value = value.clamp(min, max);

    // Normalize the value to the range [0.0, 1.0]
    let normalized_value = (clamped_value - min) / (max - min);

    // Map the normalized value to the range [-1.0, 1.0]
    2.0 * normalized_value - 1.0
}

pub fn render_pendulum(
    mut commands: Commands,
    mut query: Query<(&PendulumCart, &PendulumLinks)>,
    mut transform_query: Query<&mut Transform>,
    mut gizmo: Gizmos,
) {
    let mut starting: Vec3 = Vec3::ZERO;
    let mut ending: Vec3 = Vec3::ZERO;
    for (pendulum_cart, links) in query.iter_mut() {
        let offset_x = pendulum_cart.offset.x;
        let offset_y = pendulum_cart.offset.y;

        if let Ok(mut cart_transform) = transform_query.get_mut(links.cart) {
            cart_transform.translation = Vec3::new(
                pendulum_cart.cart_position.x + offset_x, 
                pendulum_cart.cart_position.y + offset_y,
                0.0,
            );
            starting = cart_transform.translation;
        }
        if let Ok(mut pendulum_transform) = transform_query.get_mut(links.pendulum_ball) {
            let pendulum_pos = pendulum_cart.pendulum_position();
            pendulum_transform.translation = Vec3::new(pendulum_pos.x + offset_x, pendulum_pos.y + offset_y, 0.0);
            ending = pendulum_transform.translation;
        }
        gizmo.line(starting, ending, pendulum_cart.color);
        gizmo.rect_2d(pendulum_cart.offset, 0., Vec2::new((RAIL_RADI * 2.0) + CART_SIZE.x, 2.0), pendulum_cart.color.darker(0.5))
    }
}

pub fn pendulum_generation(
    mut query: Query<(&mut PendulumCart)>,
    mut generation: ResMut<Generation>,
    mut gen_timer: ResMut<GenerationTimer>,
    time: ResMut<Time>,
) {
    if gen_timer.0.tick(time.delta()).just_finished() {
        generation.epoch += 1;
        generation.max_fitness = 0.0;
        generation.average_fitness = 0.0;
        let mut total = 0.0;

        // Calculate total fitness for average calculation
        for pendulum in query.iter_mut() {
            total += pendulum.fitness;
        }
        generation.average_fitness = total / (POPULATION * POPULATION) as f32;

        // Find the best pendulum
        if let Some(mut best_pendulum) = query.iter_mut().max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap()) {
            let best_brain = best_pendulum.brain.clone();
            generation.max_fitness = best_pendulum.fitness;

            // Set the color of the best pendulum to opaque
            best_pendulum.color = Color::rgba(1.0, 1.0, 0.0, 1.0); // Example: fully opaque green

            // Mutate the rest of the pendulums based on the best one
            for mut pendulum in query.iter_mut() {
                // Skip the best pendulum
                if pendulum.fitness == generation.max_fitness{
                    pendulum.reset();
                    continue;
                }

                // Mutate and update the pendulum's brain
                let mut new_brain = best_brain.clone();
                new_brain.mutate(MUTATION / (generation.epoch as f32));
                pendulum.brain = new_brain;

                // Set the color of the mutated pendulums to nearly transparent
                pendulum.color = Color::rgba(0.0, 1.0, 0.0, 0.02); // Example: nearly transparent green

                // Reset the pendulum
                pendulum.reset();
            }
        }
        println!(
            "Generation: {}, Average: {} Max: {}",
            generation.epoch, generation.average_fitness, generation.max_fitness
        );
    }
}