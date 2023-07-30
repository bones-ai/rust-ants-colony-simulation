use std::{cmp, collections::HashMap, time::Duration};

use bevy::{
    math::vec3,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    time::common_conditions::on_timer,
};

use crate::{
    gui::SimSettings,
    quadtree::{Point, QuadTree, Rectangle},
    utils::window_to_grid,
    *,
};

pub struct PheromonePlugin;

#[derive(Resource)]
pub struct Pheromones {
    pub to_home: PheromoneGrid,
    pub to_food: PheromoneGrid,
    pub qt_home: QuadTree,
    pub qt_food: QuadTree,
}

#[derive(Component)]
struct PheromoneImageRender;

pub struct PheromoneGrid {
    strength: HashMap<(i32, i32), f32>,
    color: (u8, u8, u8),
}

impl PheromoneGrid {
    pub fn emit(&mut self, key: &(i32, i32), value: f32) {
        match self.strength.get_mut(key) {
            Some(old_value) => {
                *old_value = (value + *old_value).min(MAX_PHEROMONE_STRENGTH);
            }
            None => {
                self.strength.insert(key.clone(), value);
            }
        }
    }

    pub fn get_ph_in_range(&self, pos: &Vec3, radius: f32) -> Vec<(i32, i32, f32)> {
        let mut ph_items = Vec::new();
        for (k, v) in self.strength.iter() {
            let dist = pos.distance(vec3(k.0 as f32, k.1 as f32, 0.0));
            if dist < radius {
                ph_items.push((k.0, k.1, *v));
            }
        }

        ph_items
    }

    pub fn decay(&mut self) {
        for (_, v) in self.strength.iter_mut() {
            *v -= PHEROMONE_DECAY_RATE;
        }
        self.strength.retain(|_, v| *v > 0.0);
    }

    pub fn get_strength(&self) -> &HashMap<(i32, i32), f32> {
        &self.strength
    }
}

impl Plugin for PheromonePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .insert_resource(Pheromones::new())
            .add_systems(
                Update,
                pheromone_decay.run_if(on_timer(Duration::from_secs_f32(PHEROMONE_DECAY_INTERVAL))),
            )
            .add_systems(
                Update,
                pheromone_image_update
                    .run_if(on_timer(Duration::from_secs_f32(PHEROMONE_IMG_UPDATE_SEC))),
            );
    }
}

fn pheromone_decay(mut pheromones: ResMut<Pheromones>) {
    pheromones.to_food.decay();
    pheromones.to_home.decay();
}

fn pheromone_image_update(
    mut textures: ResMut<Assets<Image>>,
    sim_settings: Res<SimSettings>,
    pheromone: Res<Pheromones>,
    mut image_handle_query: Query<&mut Handle<Image>, With<PheromoneImageRender>>,
) {
    let mut img_handle = image_handle_query.single_mut();
    let (w, h) = (
        W as usize / PHEROMONE_UNIT_GRID_SIZE as usize,
        H as usize / PHEROMONE_UNIT_GRID_SIZE as usize,
    );
    let mut bytes = Vec::with_capacity(w as usize * h as usize * 4);
    for _ in 0..h {
        for _ in 0..w {
            bytes.push(0);
            bytes.push(0);
            bytes.push(0);
            bytes.push(0);
        }
    }

    if sim_settings.is_show_home_ph {
        add_map_to_img(
            &pheromone.to_home.strength,
            &pheromone.to_home.color,
            &mut bytes,
        );
    }
    if sim_settings.is_show_food_ph {
        add_map_to_img(
            &pheromone.to_food.strength,
            &pheromone.to_food.color,
            &mut bytes,
        );
    }

    let pheromone_map = Image::new(
        Extent3d {
            width: w as u32,
            height: h as u32,
            ..Default::default()
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8Unorm,
    );
    *img_handle = textures.add(pheromone_map);
}

fn add_map_to_img(map: &HashMap<(i32, i32), f32>, color: &(u8, u8, u8), img_bytes: &mut Vec<u8>) {
    let w = W as usize / PHEROMONE_UNIT_GRID_SIZE as usize;
    for (k, v) in map.iter() {
        let (x, y) = (k.0, k.1);
        let (x, y) = window_to_grid(x, y);

        let idx = y * w as i32 + x;
        let strength = cmp::min((*v as u32).saturating_mul(5), u8::MAX.into()) as u8;

        let idx = (idx as usize).saturating_mul(4);
        if idx.saturating_add(3) >= img_bytes.len() || strength < PHEROMONE_GRID_VIZ_MIN_STRENGTH {
            continue;
        }

        img_bytes[idx + 3] = cmp::min(img_bytes[idx + 3].saturating_add(strength), PH_GRID_OPACITY);
        // img_bytes[idx] = img_bytes[idx].saturating_add(color.0 / 3);
        // img_bytes[idx + 1] = img_bytes[idx + 1].saturating_add(color.1 / 3);
        // img_bytes[idx + 2] = img_bytes[idx + 2].saturating_add(color.2 / 3);
        img_bytes[idx] = color.0;
        img_bytes[idx + 1] = color.1;
        img_bytes[idx + 2] = color.2;
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0)
                .with_scale(Vec3::splat(PHEROMONE_UNIT_GRID_SIZE as f32)),
            ..Default::default()
        },
        PheromoneImageRender,
    ));
}

impl Pheromones {
    fn new() -> Self {
        let mut to_food_map = HashMap::new();
        let mut to_home_map = HashMap::new();

        // Food and Home have high pheromone strength
        to_food_map.insert((FOOD_LOCATION.0 as i32, FOOD_LOCATION.1 as i32), 100000.0);
        to_home_map.insert((HOME_LOCATION.0 as i32, HOME_LOCATION.1 as i32), 100000.0);

        let boundary = Rectangle::new(-W / 2.0, H / 2.0, W, H);
        Self {
            to_food: PheromoneGrid {
                strength: to_food_map,
                color: (PH_COLOR_TO_FOOD.0, PH_COLOR_TO_FOOD.1, PH_COLOR_TO_FOOD.2),
            },
            to_home: PheromoneGrid {
                strength: to_home_map,
                color: (PH_COLOR_TO_HOME.0, PH_COLOR_TO_HOME.1, PH_COLOR_TO_HOME.2),
            },
            qt_home: QuadTree::new(boundary.clone(), 4),
            qt_food: QuadTree::new(boundary.clone(), 4),
        }
    }

    pub fn get_pheromone_points(&self, pos: &Vec3, is_home: bool) -> Vec<(i32, i32, f32)> {
        let mut ph_items = Vec::new();
        let range = Rectangle::new(
            pos.x,
            pos.y,
            INITIAL_ANT_PH_SCAN_RADIUS,
            INITIAL_ANT_PH_SCAN_RADIUS,
        );

        if is_home {
            let surrounding_points = self.qt_home.query(&range);
            for p in surrounding_points {
                if let Some(v) = self.to_home.strength.get(&(p.x as i32, p.y as i32)) {
                    ph_items.push((p.x as i32, p.y as i32, *v));
                }
            }
        } else {
            let surrounding_points = self.qt_food.query(&range);
            for p in surrounding_points {
                if let Some(v) = self.to_food.strength.get(&(p.x as i32, p.y as i32)) {
                    ph_items.push((p.x as i32, p.y as i32, *v));
                }
            }
        }

        return ph_items;
    }

    pub fn update_qt(&mut self) {
        let boundary = Rectangle::new(-W / 2.0, H / 2.0, W, H);
        let mut new_qt_home = QuadTree::new(boundary.clone(), 4);
        let mut new_qt_food = QuadTree::new(boundary.clone(), 4);

        for (k, _) in self.to_home.strength.iter() {
            let point = Point::new(k.0 as f32, k.1 as f32);
            new_qt_home.insert(&point);
        }
        for (k, _) in self.to_food.strength.iter() {
            let point = Point::new(k.0 as f32, k.1 as f32);
            new_qt_food.insert(&point);
        }

        self.qt_home = new_qt_home;
        self.qt_food = new_qt_food;

        let boundary = Rectangle::new(-W / 2.0, H / 2.0, W, H);
        println!("home qt: {:?}", self.qt_home.query(&boundary).len());
        println!("food qt: {:?}", self.qt_food.query(&boundary).len());

        // self.qt_food.display();
        // println!("food ph: {:?}", self.to_food.strength);
    }
}
