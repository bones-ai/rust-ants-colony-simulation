use crate::*;
use bevy::{
    math::{vec2, vec3},
    prelude::*,
};
use rand::{thread_rng, Rng};
use std::f32::consts::PI;

// Function to find the n points with max z values
pub fn find_n_points_with_max_z(points: &mut [(i32, i32, f32)], n: usize) -> Vec<(i32, i32, f32)> {
    quickselect(points, 0, points.len() - 1, n);
    points[points.len().saturating_sub(n)..].to_vec()
}

pub fn calc_weighted_midpoint(points: &[(i32, i32, f32)]) -> Vec2 {
    let mut total_weight = 0.0;
    let mut weighted_sum_x = 0.0;
    let mut weighted_sum_y = 0.0;

    points.iter().for_each(|(p0, p1, p2)| {
        total_weight += p2;
        weighted_sum_x += *p0 as f32 * p2;
        weighted_sum_y += *p1 as f32 * p2;
    });

    let total_weight_recip = total_weight.recip();
    let weighted_midpoint_x = weighted_sum_x * total_weight_recip;
    let weighted_midpoint_y = weighted_sum_y * total_weight_recip;

    vec2(weighted_midpoint_x, weighted_midpoint_y)
}

pub fn calc_rotation_angle(v1: Vec3, v2: Vec3) -> f32 {
    let dx = v1.x - v2.x;
    let dy = v1.y - v2.y;

    // Calculate the angle using arctangent (atan2) function
    let mut angle_rad = dy.atan2(dx);

    // Ensure the angle is within [0, 2*PI) range
    if angle_rad < 0.0 {
        angle_rad += 2.0 * PI;
    }
    angle_rad
}

pub fn angle_between_vectors(a: &Vec2, b: &Vec2) -> f32 {
    let dot_product = a.x * b.x + a.y * b.y;
    let magnitude_a = (a.x * a.x + a.y * a.y).sqrt();
    let magnitude_b = (b.x * b.x + b.y * b.y).sqrt();

    let cos_theta = dot_product / (magnitude_a * magnitude_b);
    cos_theta.acos()
}

pub fn rotate_vector(vector: &Vec2, angle_deg: f32) -> Vec2 {
    let angle_rad = angle_deg.to_radians();
    let sin_theta = angle_rad.sin();
    let cos_theta = angle_rad.cos();

    let x = vector.x * cos_theta - vector.y * sin_theta;
    let y = vector.x * sin_theta + vector.y * cos_theta;

    vec2(x, y)
}

pub fn window_to_grid(x: i32, y: i32) -> (i32, i32) {
    // Convert from center to top left co-ords
    let (tx, ty) = (x + (W as usize / 2) as i32, (H as usize / 2) as i32 - y);
    let (tx, ty) = (tx / PH_UNIT_GRID_SIZE as i32, ty / PH_UNIT_GRID_SIZE as i32);

    (tx, ty)
}

pub fn grid_to_window(tx: i32, ty: i32) -> (i32, i32) {
    let x = tx * PH_UNIT_GRID_SIZE as i32 + PH_UNIT_GRID_SIZE as i32 / 2 - (W as i32 / 2);
    let y = (H as i32 / 2) - ty * PH_UNIT_GRID_SIZE as i32 - PH_UNIT_GRID_SIZE as i32 / 2;

    (x, y)
}

pub fn vector_to_angle_deg(vec: Vec2) -> f32 {
    let angle_rad = vec.y.atan2(vec.x);
    let angle_deg = angle_rad.to_degrees();

    // Adjust the angle to be within the range of 0 to 360 degrees
    if angle_deg < 0.0 {
        angle_deg + 360.0
    } else {
        angle_deg
    }
}

pub fn get_rand_unit_vec3() -> Vec3 {
    let mut rng = thread_rng();
    vec3(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0).normalize()
}

pub fn get_rand_vec2() -> Vec2 {
    let mut rng = thread_rng();
    vec2(rng.gen_range(-W..W), rng.gen_range(-H..H))
}

pub fn get_rand_unit_vec2() -> Vec2 {
    let rand_vec3 = get_rand_unit_vec3();
    vec2(rand_vec3.x, rand_vec3.y)
}

// Function to partition the array based on the pivot (max z value)
fn partition(points: &mut [(i32, i32, f32)], low: usize, high: usize) -> usize {
    let pivot = points[high].2;
    let mut i = low;

    for j in low..high {
        if points[j].2 >= pivot {
            points.swap(i, j);
            i += 1;
        }
    }

    points.swap(i, high);
    i
}

// Modified Quickselect algorithm to find n points with max z values
fn quickselect(points: &mut [(i32, i32, f32)], low: usize, high: usize, n: usize) {
    if low < high {
        let pivot_index = partition(points, low, high);

        if pivot_index == n - 1 {
            return;
        } else if pivot_index > n - 1 {
            quickselect(points, low, pivot_index - 1, n);
        } else {
            quickselect(points, pivot_index + 1, high, n);
        }
    }
}
