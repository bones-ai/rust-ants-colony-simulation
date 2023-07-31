use std::{f32::consts::PI, time::Duration};

use bevy::{
    math::{vec2, vec3},
    prelude::*,
    time::common_conditions::on_timer,
};
use rand::{thread_rng, Rng};

use crate::{
    utils::{calc_rotation_angle, get_rand_unit_vec2},
    *,
};

pub struct AntPlugin;

pub enum AntTask {
    FindFood,
    FindHome,
}

#[derive(Component)]
pub struct Ant;
#[derive(Component)]
struct CurrentTask(AntTask);
#[derive(Component)]
struct Velocity(Vec2);
#[derive(Component)]
struct Acceleration(Vec2);
#[derive(Component)]
struct PhStrength(f32);

#[derive(Resource)]
struct AntScanRadius(f32);
#[derive(Resource)]
pub struct AntFollowCameraPos(pub Vec2);

impl Plugin for AntPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .insert_resource(AntScanRadius(INITIAL_ANT_PH_SCAN_RADIUS))
            .insert_resource(AntFollowCameraPos(Vec2::ZERO))
            .add_systems(
                Update,
                drop_pheromone.run_if(on_timer(Duration::from_secs_f32(PH_DROP_INTERVAL))),
            )
            .add_systems(
                Update,
                check_wall_collision.run_if(on_timer(Duration::from_secs_f32(0.1))),
            )
            .add_systems(
                Update,
                check_home_food_collisions.run_if(on_timer(Duration::from_secs_f32(0.1))),
            )
            .add_systems(Update, update_camera_follow_pos)
            .add_systems(
                Update,
                periodic_direction_update.run_if(on_timer(Duration::from_secs_f32(
                    ANT_DIRECTION_UPDATE_INTERVAL,
                ))),
            )
            .add_systems(
                Update,
                update_scan_radius.run_if(on_timer(Duration::from_secs_f32(1.0))),
            )
            .add_systems(
                Update,
                decay_ph_strength.run_if(on_timer(Duration::from_secs_f32(
                    PH_STRENGTH_DECAY_INTERVAL,
                ))),
            )
            .add_systems(Update, update_position.after(check_wall_collision));
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    for _ in 0..NUM_ANTS {
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load(SPRITE_ANT),
                transform: Transform::from_xyz(HOME_LOCATION.0, HOME_LOCATION.1, ANT_Z_INDEX)
                    .with_scale(Vec3::splat(ANT_SPRITE_SCALE)),
                ..Default::default()
            },
            Ant,
            CurrentTask(AntTask::FindFood),
            Velocity(get_rand_unit_vec2()),
            Acceleration(Vec2::ZERO),
            PhStrength(ANT_INITIAL_PH_STRENGTH),
        ));
    }
}

fn drop_pheromone(
    mut ant_query: Query<(&Transform, &CurrentTask, &PhStrength), With<Ant>>,
    mut pheromones: ResMut<Pheromones>,
) {
    for (transform, ant_task, ph_strength) in ant_query.iter_mut() {
        let x = transform.translation.x as i32;
        let y = transform.translation.y as i32;

        match ant_task.0 {
            AntTask::FindFood => pheromones.to_home.emit(&(x, y), ph_strength.0),
            AntTask::FindHome => pheromones.to_food.emit(&(x, y), ph_strength.0),
        }
    }

    // pheromones.update_qt();
}

fn update_scan_radius(mut scan_radius: ResMut<AntScanRadius>) {
    if scan_radius.0 > INITIAL_ANT_PH_SCAN_RADIUS * PH_SCAN_RADIUS_SCALE {
        return;
    }

    scan_radius.0 += PH_SCAN_RADIUS_INCREMENT;
}

fn update_camera_follow_pos(
    ant_query: Query<&Transform, With<Ant>>,
    mut follow_pos: ResMut<AntFollowCameraPos>,
) {
    for transform in ant_query.iter() {
        follow_pos.0 = transform.translation.truncate();
        break;
    }
}

fn decay_ph_strength(mut ant_query: Query<&mut PhStrength, With<Ant>>) {
    for mut ph_strength in ant_query.iter_mut() {
        if ph_strength.0 <= 0.0 {
            continue;
        }

        ph_strength.0 -= PH_STRENGTH_DECAY_RATE;
    }
}

fn get_steering_force(target: Vec2, current: Vec2, velocity: Vec2) -> Vec2 {
    let desired = target - current;
    let steering = desired - velocity;
    steering * 0.05
}

fn calc_weighted_midpoint(points: &Vec<(i32, i32, f32)>) -> Vec2 {
    let total_weight: f32 = points.iter().map(|point| point.2).sum();

    let weighted_sum_x: f32 = points.iter().map(|point| point.0 as f32 * point.2).sum();
    let weighted_sum_y: f32 = points.iter().map(|point| point.1 as f32 * point.2).sum();

    let weighted_midpoint_x = weighted_sum_x / total_weight;
    let weighted_midpoint_y = weighted_sum_y / total_weight;

    vec2(weighted_midpoint_x, weighted_midpoint_y)
}

fn periodic_direction_update(
    mut ant_query: Query<(&mut Acceleration, &Transform, &CurrentTask, &Velocity), With<Ant>>,
    pheromones: Res<Pheromones>,
    scan_radius: Res<AntScanRadius>,
) {
    for (mut acceleration, transform, current_task, velocity) in ant_query.iter_mut() {
        let surrounding_ph;
        let current_pos = transform.translation;

        match current_task.0 {
            AntTask::FindFood => {
                surrounding_ph = Some(
                    pheromones
                        .to_food
                        .get_ph_in_range(&current_pos, scan_radius.0),
                );
                // surrounding_ph = Some(pheromones.get_pheromone_points(&current_pos, false));
            }
            AntTask::FindHome => {
                surrounding_ph = Some(
                    pheromones
                        .to_home
                        .get_ph_in_range(&current_pos, scan_radius.0),
                );
                // surrounding_ph = Some(pheromones.get_pheromone_points(&current_pos, true));
            }
        }

        let points = surrounding_ph.unwrap();
        if points.is_empty() {
            // Default direction randomization
            acceleration.0 += get_rand_unit_vec2() * 0.2;
            continue;
        }

        let target = calc_weighted_midpoint(&points);
        let steering_force =
            get_steering_force(target, transform.translation.truncate(), velocity.0);

        let mut rng = rand::thread_rng();
        acceleration.0 += steering_force * rng.gen_range(0.2..ANT_STERRING_FORCE_FACTOR);
    }
}

fn check_home_food_collisions(
    mut ant_query: Query<
        (
            &Transform,
            &mut Velocity,
            &mut CurrentTask,
            &mut PhStrength,
            &mut Handle<Image>,
        ),
        With<Ant>,
    >,
    asset_server: Res<AssetServer>,
) {
    for (transform, mut velocity, mut ant_task, mut ph_strength, mut image_handle) in
        ant_query.iter_mut()
    {
        // Home collision
        let dist_to_home =
            transform
                .translation
                .distance(vec3(HOME_LOCATION.0, HOME_LOCATION.1, 0.0));
        if dist_to_home < HOME_RADIUS {
            // rebound only the ants with food
            match ant_task.0 {
                AntTask::FindFood => {}
                AntTask::FindHome => {
                    velocity.0 *= -1.0;
                }
            }
            ant_task.0 = AntTask::FindFood;
            ph_strength.0 = ANT_INITIAL_PH_STRENGTH;
            *image_handle = asset_server.load(SPRITE_ANT);
        }

        // Food Collision
        // Iterate over each food location
        for food_location in FOOD_LOCATIONS {
            let dist_to_food = transform
                .translation
                .distance(vec3(food_location.0, food_location.1, 0.0));

            if dist_to_food < FOOD_PICKUP_RADIUS {
                // rebound only the ants with food
                match ant_task.0 {
                    AntTask::FindFood => {
                        velocity.0 *= -1.0;
                    }
                    AntTask::FindHome => {}
                }
                ant_task.0 = AntTask::FindHome;
                ph_strength.0 = ANT_INITIAL_PH_STRENGTH;
                *image_handle = asset_server.load(SPRITE_ANT_WITH_FOOD);
            }
        }
    }
}

fn check_wall_collision(
    mut ant_query: Query<(&Transform, &Velocity, &mut Acceleration), With<Ant>>,
) {
    for (transform, velocity, mut acceleration) in ant_query.iter_mut() {
        // wall rebound
        let border = 20.0;
        let top_left = (-W / 2.0, H / 2.0);
        let bottom_right = (W / 2.0, -H / 2.0);
        let x_bound = transform.translation.x < top_left.0 + border
            || transform.translation.x >= bottom_right.0 - border;
        let y_bound = transform.translation.y >= top_left.1 - border
            || transform.translation.y < bottom_right.1 + border;
        if x_bound || y_bound {
            let mut rng = thread_rng();
            let target = vec2(rng.gen_range(-200.0..200.0), rng.gen_range(-200.0..200.0));
            acceleration.0 +=
                get_steering_force(target, transform.translation.truncate(), velocity.0);
        }
    }
}

fn update_position(
    mut ant_query: Query<(&mut Transform, &mut Velocity, &mut Acceleration), With<Ant>>,
) {
    for (mut transform, mut velocity, mut acceleration) in ant_query.iter_mut() {
        let old_pos = transform.translation;

        velocity.0 = (velocity.0 + acceleration.0).normalize();
        transform.translation += vec3(velocity.0.x, velocity.0.y, 0.0) * ANT_SPEED;
        acceleration.0 = Vec2::ZERO;
        transform.rotation =
            Quat::from_rotation_z(calc_rotation_angle(&old_pos, &transform.translation) + PI / 2.0);
    }
}
