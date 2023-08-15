use std::{f32::consts::PI, time::Duration};

use bevy::{
    math::{vec2, vec3},
    prelude::*,
    time::common_conditions::on_timer,
};
use rand::{thread_rng, Rng};

use crate::{
    gui::SimStatistics,
    pheromone::Pheromones,
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
pub struct CurrentTask(pub AntTask);
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
                drop_pheromone.run_if(on_timer(Duration::from_secs_f32(ANT_PH_DROP_INTERVAL))),
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
                update_stats.run_if(on_timer(Duration::from_secs_f32(3.0))),
            )
            .add_systems(
                Update,
                update_scan_radius.run_if(on_timer(Duration::from_secs_f32(1.0))),
            )
            .add_systems(
                Update,
                decay_ph_strength.run_if(on_timer(Duration::from_secs_f32(
                    ANT_PH_STRENGTH_DECAY_INTERVAL,
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
            AntTask::FindFood => pheromones.to_home.emit_signal(&(x, y), ph_strength.0),
            AntTask::FindHome => pheromones.to_food.emit_signal(&(x, y), ph_strength.0),
        }
    }
}

fn update_scan_radius(mut scan_radius: ResMut<AntScanRadius>) {
    if scan_radius.0 > INITIAL_ANT_PH_SCAN_RADIUS * ANT_PH_SCAN_RADIUS_SCALE {
        return;
    }

    scan_radius.0 += ANT_PH_SCAN_RADIUS_INCREMENT;
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

fn update_stats(
    mut stats: ResMut<SimStatistics>,
    scan_radius: Res<AntScanRadius>,
    ant_query: Query<With<Ant>>,
) {
    stats.scan_radius = scan_radius.0;
    stats.num_ants = ant_query.iter().len();
}

fn decay_ph_strength(mut ant_query: Query<&mut PhStrength, With<Ant>>) {
    for mut ph_strength in ant_query.iter_mut() {
        ph_strength.0 = f32::max(ph_strength.0 - ANT_PH_STRENGTH_DECAY_RATE, 0.0);
    }
}

fn get_steering_force(target: Vec2, current: Vec2, velocity: Vec2) -> Vec2 {
    let desired = target - current;
    let steering = desired - velocity;
    steering * 0.05
}

fn periodic_direction_update(
    mut ant_query: Query<(&mut Acceleration, &Transform, &CurrentTask, &Velocity), With<Ant>>,
    mut pheromones: ResMut<Pheromones>,
    mut stats: ResMut<SimStatistics>,
    scan_radius: Res<AntScanRadius>,
) {
    (stats.food_cache_size, stats.home_cache_size) = pheromones.clear_cache();

    for (mut acceleration, transform, current_task, velocity) in ant_query.iter_mut() {
        let current_pos = transform.translation;
        let mut target = None;

        // If ant is close to food/home, pull it towards itself
        match current_task.0 {
            AntTask::FindFood => {
                let dist_to_food = transform.translation.distance_squared(vec3(
                    FOOD_LOCATION.0,
                    FOOD_LOCATION.1,
                    0.0,
                ));
                if dist_to_food <= ANT_TARGET_AUTO_PULL_RADIUS * ANT_TARGET_AUTO_PULL_RADIUS {
                    target = Some(vec2(FOOD_LOCATION.0, FOOD_LOCATION.1));
                }
            }
            AntTask::FindHome => {
                let dist_to_home = transform.translation.distance_squared(vec3(
                    HOME_LOCATION.0,
                    HOME_LOCATION.1,
                    0.0,
                ));
                if dist_to_home <= ANT_TARGET_AUTO_PULL_RADIUS * ANT_TARGET_AUTO_PULL_RADIUS {
                    target = Some(vec2(HOME_LOCATION.0, HOME_LOCATION.1));
                }
            }
        }

        if target.is_none() {
            match current_task.0 {
                AntTask::FindFood => {
                    target = pheromones
                        .to_food
                        .get_steer_target(&current_pos, scan_radius.0);
                }
                AntTask::FindHome => {
                    target = pheromones
                        .to_home
                        .get_steer_target(&current_pos, scan_radius.0);
                }
            }
        }

        if target.is_none() {
            // Default direction randomization
            acceleration.0 += get_rand_unit_vec2() * 0.2;
            continue;
        }

        let steering_force = get_steering_force(
            target.unwrap(),
            transform.translation.truncate(),
            velocity.0,
        );

        let mut rng = rand::thread_rng();
        acceleration.0 += steering_force * rng.gen_range(0.4..=ANT_STEERING_FORCE_FACTOR);
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
                .distance_squared(vec3(HOME_LOCATION.0, HOME_LOCATION.1, 0.0));
        if dist_to_home < HOME_RADIUS * HOME_RADIUS {
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
        let dist_to_food =
            transform
                .translation
                .distance_squared(vec3(FOOD_LOCATION.0, FOOD_LOCATION.1, 0.0));
        if dist_to_food < FOOD_PICKUP_RADIUS * FOOD_PICKUP_RADIUS {
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

        if !acceleration.0.is_nan() {
            velocity.0 = (velocity.0 + acceleration.0).normalize();
            let new_translation = transform.translation + vec3(velocity.0.x, velocity.0.y, 0.0) * ANT_SPEED;
            if !new_translation.is_nan() {
                transform.translation = new_translation;
            }
        }

        acceleration.0 = Vec2::ZERO;
        transform.rotation =
            Quat::from_rotation_z(calc_rotation_angle(&old_pos, &transform.translation) + PI / 2.0);
    }
}
