// Global
pub const W: f32 = 1920.0;
pub const H: f32 = 1080.0;
pub const BG_COLOR: (u8, u8, u8) = (0, 0, 0);

// Ants
pub const NUM_ANTS: u32 = 5000;
pub const ANT_SPEED: f32 = 1.5;
pub const ANT_DIRECTION_RANDOMNESS_DEG: f32 = 300.0;
pub const ANT_DIRECTION_UPDATE_INTERVAL: f32 = 0.5;
pub const ANT_SPRITE_SCALE: f32 = 0.3;
pub const ANT_Z_INDEX: f32 = 3.0;
pub const ANT_INITIAL_PH_STRENGTH: f32 = 32.0;
pub const ANT_PH_STRENGTH_DECAY_RATE: f32 = 0.7;
pub const ANT_PH_STRENGTH_DECAY_INTERVAL: f32 = 0.5;
pub const ANT_PH_DROP_INTERVAL: f32 = 0.7;
pub const INITIAL_ANT_PH_SCAN_RADIUS: f32 = 15.0;
pub const ANT_PH_SCAN_RADIUS_INCREMENT: f32 = 0.1;
pub const ANT_PH_SCAN_RADIUS_SCALE: f32 = 1.8;
pub const ANT_STEERING_FORCE_FACTOR: f32 = 0.7;
pub const ANT_TARGET_AUTO_PULL_RADIUS: f32 = 100.0;

// Ant Colony
pub const HOME_LOCATION: (f32, f32) = (759.0, -350.0);
// pub const HOME_LOCATION: (f32, f32) = (300.0, -250.0);
pub const HOME_SPRITE_SCALE: f32 = 2.5;
pub const HOME_RADIUS: f32 = 30.0;

// Pheromones
pub const MAX_PHEROMONE_STRENGTH: f32 = 500.0;
pub const PH_DECAY_RATE: f32 = 0.08;
pub const PH_DECAY_INTERVAL: f32 = 0.1;
pub const PH_IMG_UPDATE_SEC: f32 = 0.1;
pub const PH_GRID_VIZ_MIN_STRENGTH: u8 = 1;
pub const PH_UNIT_GRID_SIZE: usize = 5;
pub const PH_KD_TREE_UPDATE_INTERVAL: f32 = 2.0;
// Larger grid size causes clumps of ants following signals
pub const PH_CACHE_GRID_SIZE: i32 = 10;
pub const PH_COLOR_TO_FOOD: (u8, u8, u8) = (2, 79, 2);
pub const PH_COLOR_TO_HOME: (u8, u8, u8) = (200, 81, 112);
pub const PH_GRID_OPACITY: u8 = 255;

// Path Viz
pub const VIZ_COLOR_TO_HOME: (u8, u8, u8) = (17, 106, 123);
pub const VIZ_COLOR_TO_FOOD: (u8, u8, u8) = (92, 46, 126);
pub const VIZ_DECAY_RATE: f32 = 0.3;
pub const VIZ_COLOR_STRENGTH: f32 = 8.0;
pub const VIZ_MAX_COLOR_STRENGTH: f32 = 50.0;

// Food
// pub const FOOD_LOCATION: (f32, f32) = (-400.0, 300.0);
pub const FOOD_LOCATION: (f32, f32) = (-750.0, 400.0);
pub const FOOD_PICKUP_RADIUS: f32 = 30.0;
pub const FOOD_SPRITE_SCALE: f32 = 2.0;

// Sprites
pub const SPRITE_ANT: &str = "ant.png";
pub const SPRITE_ANT_WITH_FOOD: &str = "ant_with_food.png";
pub const SPRITE_ANT_COLONY: &str = "nest.png";
pub const SPRITE_FOOD: &str = "food.png";
