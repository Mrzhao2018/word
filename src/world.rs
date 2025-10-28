use crate::components::*;
use crate::resources::{ActiveLocalMap, WorldSeed};
use crate::world_map_data::{WorldAtlas, WorldBiome, WorldCell};
use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::{rngs::SmallRng, Rng, SeedableRng};

/// 世界大小常量
pub const WORLD_WIDTH: i32 = 50;
pub const WORLD_HEIGHT: i32 = 30;
pub const TILE_SIZE: f32 = 32.0;

/// 游戏世界资源
#[derive(Resource)]
pub struct GameWorld {
    #[allow(dead_code)] // 保留用于未来世界扩展
    pub width: i32,
    #[allow(dead_code)] // 保留用于未来世界扩展
    pub height: i32,
}

impl Default for GameWorld {
    fn default() -> Self {
        Self {
            width: WORLD_WIDTH,
            height: WORLD_HEIGHT,
        }
    }
}

/// 地形生成器 - 使用多层噪声创建自然地形
struct TerrainGenerator {
    elevation: Perlin,   // 高度噪声
    moisture: Perlin,    // 湿度噪声
    temperature: Perlin, // 温度噪声
    detail: Perlin,      // 细节噪声
    elevation_bias: f64,
    moisture_bias: f64,
    temperature_bias: f64,
    water_cutoff: f64,
    mountain_cutoff: f64,
    tree_moisture_threshold: f64,
    tree_min_elevation: f64,
    stone_moisture_threshold: f64,
    river_band_multiplier: f64,
    biome: Option<WorldBiome>,
}

impl TerrainGenerator {
    fn new(seed: u32, context: Option<&WorldCell>) -> Self {
        let mut generator = Self {
            elevation: Perlin::new(seed),
            moisture: Perlin::new(seed + 1),
            temperature: Perlin::new(seed + 2),
            detail: Perlin::new(seed + 3),
            elevation_bias: 0.0,
            moisture_bias: 0.0,
            temperature_bias: 0.0,
            water_cutoff: -0.3,
            mountain_cutoff: 0.5,
            tree_moisture_threshold: 0.3,
            tree_min_elevation: -0.1,
            stone_moisture_threshold: -0.2,
            river_band_multiplier: 1.0,
            biome: context.map(|cell| cell.biome),
        };

        if let Some(cell) = context {
            generator.elevation_bias = (cell.elevation as f64) * 0.35;
            generator.moisture_bias = (cell.moisture as f64) * 0.45;
            generator.temperature_bias = (cell.temperature as f64) * 0.3;
            generator.apply_biome_modifiers(cell.biome);
        }

        generator.clamp_biases();
        generator
    }

    /// 获取指定位置的地形类型
    fn get_terrain(&self, x: i32, y: i32) -> TerrainType {
        let scale = 0.1; // 噪声缩放因子，值越小地形变化越平缓

        // 多层噪声采样
        let elevation = self.elevation.get([x as f64 * scale, y as f64 * scale]);
        let moisture = self
            .moisture
            .get([x as f64 * scale * 0.8, y as f64 * scale * 0.8]);
        let temperature = self
            .temperature
            .get([x as f64 * scale * 1.2, y as f64 * scale * 1.2]);
        let detail = self
            .detail
            .get([x as f64 * scale * 3.0, y as f64 * scale * 3.0])
            * 0.1;

        let final_elevation = (elevation + detail + self.elevation_bias).clamp(-1.0, 1.0);
        let final_moisture = (moisture + detail * 0.5 + self.moisture_bias).clamp(-1.0, 1.0);
        let final_temperature = (temperature + self.temperature_bias).clamp(-1.0, 1.0);

        let base = if final_elevation < self.water_cutoff {
            TerrainType::Water
        } else if final_elevation > self.mountain_cutoff {
            if final_temperature > 0.3 {
                TerrainType::Mountain
            } else {
                TerrainType::Stone
            }
        } else if final_moisture > self.tree_moisture_threshold
            && final_elevation > self.tree_min_elevation
        {
            TerrainType::Tree
        } else if final_moisture < self.stone_moisture_threshold {
            TerrainType::Stone
        } else {
            TerrainType::Grass
        };

        self.adjust_for_biome(base, final_elevation, final_moisture, final_temperature)
    }

    /// 检查是否应该生成河流
    fn is_river(&self, x: i32, y: i32) -> bool {
        if matches!(self.biome, Some(WorldBiome::Ocean)) {
            return false;
        }

        let river_scale = 0.05;
        let river_noise = self
            .moisture
            .get([x as f64 * river_scale, y as f64 * river_scale]);
        let band = 0.05 * self.river_band_multiplier;
        let elevation_sample =
            self.elevation.get([x as f64 * 0.1, y as f64 * 0.1]) + self.elevation_bias;

        (river_noise.abs() < band) && (elevation_sample < self.mountain_cutoff - 0.1)
    }

    fn apply_biome_modifiers(&mut self, biome: WorldBiome) {
        match biome {
            WorldBiome::Grassland => {}
            WorldBiome::Forest => {
                self.tree_moisture_threshold = 0.05;
                self.tree_min_elevation = -0.25;
                self.stone_moisture_threshold = -0.35;
                self.moisture_bias += 0.15;
                self.river_band_multiplier = 1.2;
            }
            WorldBiome::Desert => {
                self.water_cutoff = -0.55;
                self.mountain_cutoff = 0.45;
                self.tree_moisture_threshold = 0.65;
                self.tree_min_elevation = 0.1;
                self.stone_moisture_threshold = 0.1;
                self.moisture_bias -= 0.3;
                self.temperature_bias += 0.2;
                self.river_band_multiplier = 0.25;
            }
            WorldBiome::Mountain => {
                self.mountain_cutoff = 0.35;
                self.tree_moisture_threshold = 0.45;
                self.stone_moisture_threshold = -0.05;
                self.elevation_bias += 0.25;
                self.water_cutoff = -0.4;
                self.river_band_multiplier = 0.6;
            }
            WorldBiome::Tundra => {
                self.mountain_cutoff = 0.4;
                self.tree_moisture_threshold = 0.55;
                self.stone_moisture_threshold = 0.0;
                self.temperature_bias -= 0.2;
                self.moisture_bias -= 0.05;
                self.river_band_multiplier = 0.8;
            }
            WorldBiome::Ocean => {
                self.water_cutoff = -0.15;
                self.tree_moisture_threshold = 0.8;
                self.tree_min_elevation = 0.2;
                self.stone_moisture_threshold = 0.25;
                self.moisture_bias += 0.3;
                self.elevation_bias -= 0.25;
                self.river_band_multiplier = 1.5;
            }
            WorldBiome::River => {
                self.water_cutoff = -0.25;
                self.tree_moisture_threshold = 0.1;
                self.stone_moisture_threshold = -0.35;
                self.moisture_bias += 0.2;
                self.river_band_multiplier = 1.8;
            }
            WorldBiome::Swamp => {
                self.water_cutoff = -0.25;
                self.tree_moisture_threshold = -0.05;
                self.tree_min_elevation = -0.25;
                self.stone_moisture_threshold = -0.3;
                self.moisture_bias += 0.25;
                self.river_band_multiplier = 2.0;
            }
        }
    }

    fn adjust_for_biome(
        &self,
        terrain: TerrainType,
        elevation: f64,
        moisture: f64,
        temperature: f64,
    ) -> TerrainType {
        match self.biome {
            Some(WorldBiome::Desert) => match terrain {
                TerrainType::Tree if moisture < 0.6 => TerrainType::Grass,
                TerrainType::Grass if temperature > 0.2 => TerrainType::Stone,
                TerrainType::Water if moisture < 0.2 => TerrainType::Stone,
                _ => terrain,
            },
            Some(WorldBiome::Forest) => match terrain {
                TerrainType::Grass if moisture > -0.2 => TerrainType::Tree,
                TerrainType::Stone if moisture > -0.1 => TerrainType::Grass,
                _ => terrain,
            },
            Some(WorldBiome::Mountain) => match terrain {
                TerrainType::Grass => TerrainType::Stone,
                TerrainType::Tree if elevation > 0.1 => TerrainType::Stone,
                TerrainType::Water if elevation > 0.0 => TerrainType::Stone,
                _ => terrain,
            },
            Some(WorldBiome::Tundra) => match terrain {
                TerrainType::Tree => TerrainType::Grass,
                TerrainType::Grass if temperature < -0.1 => TerrainType::Stone,
                _ => terrain,
            },
            Some(WorldBiome::Ocean) => match terrain {
                TerrainType::Tree => TerrainType::Water,
                TerrainType::Grass if elevation < 0.05 => TerrainType::Water,
                TerrainType::Stone if elevation < 0.02 => TerrainType::Water,
                _ => terrain,
            },
            Some(WorldBiome::River) => match terrain {
                TerrainType::Grass if moisture > 0.0 => TerrainType::Tree,
                TerrainType::Stone if moisture > 0.15 => TerrainType::Grass,
                _ => terrain,
            },
            Some(WorldBiome::Swamp) => match terrain {
                TerrainType::Grass if moisture > -0.05 => TerrainType::Tree,
                TerrainType::Stone if moisture > 0.05 => TerrainType::Water,
                TerrainType::Tree if moisture > 0.4 => TerrainType::Water,
                _ => terrain,
            },
            _ => terrain,
        }
    }

    fn clamp_biases(&mut self) {
        self.elevation_bias = self.elevation_bias.clamp(-0.9, 0.9);
        self.moisture_bias = self.moisture_bias.clamp(-0.9, 0.9);
        self.temperature_bias = self.temperature_bias.clamp(-0.9, 0.9);
    }
}

/// 生成世界地形 - 改进版，使用噪声生成
pub fn setup_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    world_seed: Res<WorldSeed>,
    active_local: Res<ActiveLocalMap>,
    world_atlas: Res<WorldAtlas>,
) {
    let mut rng = SmallRng::seed_from_u64(world_seed.seed as u64);
    let font = asset_server.load("fonts/sarasa-gothic-sc-regular.ttf");

    let selected_cell = active_local
        .coord
        .and_then(|coord| world_atlas.cell_at(coord))
        .cloned();

    // 创建地形生成器（使用资源中的种子及选中世界格子上下文）
    let generator = TerrainGenerator::new(world_seed.seed, selected_cell.as_ref());
    let biome = generator.biome;
    let richness_bias = match biome {
        Some(WorldBiome::Forest) => 1.1,
        Some(WorldBiome::Desert) => 0.85,
        Some(WorldBiome::Mountain) => 1.2,
        Some(WorldBiome::Swamp) => 1.05,
        Some(WorldBiome::River) => 1.05,
        Some(WorldBiome::Ocean) => 0.9,
        Some(WorldBiome::Tundra) => 0.9,
        _ => 1.0,
    };

    for x in 0..WORLD_WIDTH {
        for y in 0..WORLD_HEIGHT {
            // 使用噪声生成地形
            let mut terrain_type = generator.get_terrain(x, y);

            // 检查河流覆盖
            if generator.is_river(x, y) && terrain_type != TerrainType::Mountain {
                terrain_type = TerrainType::Water;
            }

            let walkable = !matches!(terrain_type, TerrainType::Water | TerrainType::Mountain);

            // 更好看的颜色和随机变化
            let color_variation = rng.gen_range(-0.05..0.05);
            let (color, ascii_char, char_color) =
                pick_tile_visual(&mut rng, terrain_type, biome, color_variation);

            // 修正坐标计算，让地块中心对齐网格点
            let pos_x =
                x as f32 * TILE_SIZE - (WORLD_WIDTH as f32 * TILE_SIZE / 2.0) + (TILE_SIZE / 2.0);
            let pos_y =
                y as f32 * TILE_SIZE - (WORLD_HEIGHT as f32 * TILE_SIZE / 2.0) + (TILE_SIZE / 2.0);

            // 计算资源丰富度（基于细节噪声并结合世界格子偏好）
            let detail_noise = generator.detail.get([x as f64 * 0.3, y as f64 * 0.3]);
            let base_richness = 0.8 + (detail_noise as f32 + 1.0) * 0.35;
            let mut resource_richness = (base_richness * richness_bias).clamp(0.4, 1.8);

            if let Some(biome_kind) = biome {
                resource_richness *= match (biome_kind, terrain_type) {
                    (WorldBiome::Forest, TerrainType::Tree) => 1.2,
                    (WorldBiome::Forest, TerrainType::Grass) => 1.1,
                    (WorldBiome::Desert, TerrainType::Stone) => 1.15,
                    (WorldBiome::Desert, TerrainType::Grass) => 0.8,
                    (WorldBiome::Desert, TerrainType::Water) => 0.7,
                    (WorldBiome::Mountain, TerrainType::Mountain) => 1.4,
                    (WorldBiome::Mountain, TerrainType::Stone) => 1.2,
                    (WorldBiome::Swamp, TerrainType::Water) => 1.2,
                    (WorldBiome::Swamp, TerrainType::Tree) => 1.1,
                    (WorldBiome::River, TerrainType::Water) => 1.25,
                    (WorldBiome::River, TerrainType::Grass) => 1.1,
                    (WorldBiome::Ocean, TerrainType::Water) => 1.15,
                    (WorldBiome::Tundra, TerrainType::Grass) => 0.85,
                    (WorldBiome::Tundra, TerrainType::Stone) => 1.1,
                    _ => 1.0,
                };
            }
            let resource_richness = resource_richness.clamp(0.4, 1.8);

            // 主地形方块(背景) - 添加渐变效果
            let gradient_offset = rng.gen_range(-0.02..0.02);
            let color_srgba = color.to_srgba();
            commands.spawn((
                Sprite {
                    color: Color::srgb(
                        (color_srgba.red + gradient_offset).clamp(0.0, 1.0),
                        (color_srgba.green + gradient_offset).clamp(0.0, 1.0),
                        (color_srgba.blue + gradient_offset).clamp(0.0, 1.0),
                    ),
                    custom_size: Some(Vec2::new(TILE_SIZE - 1.0, TILE_SIZE - 1.0)),
                    ..default()
                },
                Transform::from_xyz(pos_x, pos_y, 0.0),
                Terrain {
                    terrain_type,
                    walkable,
                    resource_richness,
                },
                GridPosition { x, y },
            ));

            // ASCII字符层
            let mut entity = commands.spawn((
                Text2d::new(ascii_char.to_string()),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(char_color),
                Transform::from_xyz(pos_x, pos_y, 0.05),
                AsciiChar {
                    character: ascii_char,
                },
                GridPosition { x, y },
            ));

            // 为水和树添加动画组件
            match terrain_type {
                TerrainType::Water => {
                    entity.insert(WaterAnimation {
                        phase: rng.gen_range(0.0..6.28),
                    });
                }
                TerrainType::Tree => {
                    entity.insert(TreeSway {
                        offset: rng.gen_range(0.0..6.28),
                    });
                }
                _ => {}
            }

            // 添加网格线效果（更细腻）
            commands.spawn((
                Sprite {
                    color: Color::srgba(0.0, 0.0, 0.0, 0.08),
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                    ..default()
                },
                Transform::from_xyz(pos_x, pos_y, 0.1),
                GridLine,
            ));
        }
    }
}

/// 生成矮人 - 改进版，直接查询已生成的地形实体
pub fn spawn_dwarves(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    terrain_query: Query<(&GridPosition, &Terrain)>,
) {
    let dwarf_names = vec!["乌里克", "索林", "巴林", "朵莉", "芬恩", "格洛因", "诺力"];
    let font = asset_server.load("fonts/sarasa-gothic-sc-regular.ttf");

    let mut rng = rand::thread_rng();

    // 寻找世界中心附近的可行走位置
    let center_x = WORLD_WIDTH / 2;
    let center_y = WORLD_HEIGHT / 2;

    // 为每个矮人单独寻找生成位置
    for (_i, name) in dwarf_names.iter().enumerate() {
        let mut grid_x = center_x;
        let mut grid_y = center_y;
        let mut found_safe_spot = false;

        // 尝试多次随机寻找安全位置
        for _ in 0..200 {
            // 在中心附近随机选择位置
            let test_x = center_x + rng.gen_range(-20..20);
            let test_y = center_y + rng.gen_range(-20..20);

            // 边界检查
            if test_x < 2 || test_x >= WORLD_WIDTH - 2 || test_y < 2 || test_y >= WORLD_HEIGHT - 2 {
                continue;
            }

            // 检查该位置及周围3x3区域是否全部可行走（查询实际地形）
            let mut all_safe = true;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let check_x = test_x + dx;
                    let check_y = test_y + dy;

                    let mut is_walkable = false;
                    for (terrain_pos, terrain) in terrain_query.iter() {
                        if terrain_pos.x == check_x && terrain_pos.y == check_y {
                            is_walkable = terrain.walkable;
                            break;
                        }
                    }

                    if !is_walkable {
                        all_safe = false;
                        break;
                    }
                }
                if !all_safe {
                    break;
                }
            }

            if all_safe {
                grid_x = test_x;
                grid_y = test_y;
                found_safe_spot = true;
                break;
            }
        }

        // 如果没找到，螺旋搜索（查询实际地形）
        if !found_safe_spot {
            'spiral: for radius in 1..40 {
                for angle_step in 0..(radius * 8) {
                    let angle =
                        (angle_step as f32 / (radius * 8) as f32) * std::f32::consts::PI * 2.0;
                    let test_x = center_x + (angle.cos() * radius as f32) as i32;
                    let test_y = center_y + (angle.sin() * radius as f32) as i32;

                    // 边界检查
                    if test_x < 0 || test_x >= WORLD_WIDTH || test_y < 0 || test_y >= WORLD_HEIGHT {
                        continue;
                    }

                    // 查询该位置的实际地形
                    let mut is_walkable = false;
                    for (terrain_pos, terrain) in terrain_query.iter() {
                        if terrain_pos.x == test_x && terrain_pos.y == test_y {
                            is_walkable = terrain.walkable;
                            break;
                        }
                    }

                    if is_walkable {
                        grid_x = test_x;
                        grid_y = test_y;
                        found_safe_spot = true;
                        break 'spiral;
                    }
                }
            }
        }

        // 最终后备：全局搜索第一个可行走位置
        if !found_safe_spot {
            warn!("矮人 {} 无法在中心附近找到位置，使用全局搜索", name);
            'global: for search_x in 0..WORLD_WIDTH {
                for search_y in 0..WORLD_HEIGHT {
                    for (terrain_pos, terrain) in terrain_query.iter() {
                        if terrain_pos.x == search_x
                            && terrain_pos.y == search_y
                            && terrain.walkable
                        {
                            grid_x = search_x;
                            grid_y = search_y;
                            found_safe_spot = true;
                            break 'global;
                        }
                    }
                }
            }
        }

        if !found_safe_spot {
            error!("矮人 {} 无法找到任何可行走位置！跳过生成。", name);
            continue;
        }

        // 根据网格位置计算世界坐标（与地形对齐）
        let x_pos =
            grid_x as f32 * TILE_SIZE - (WORLD_WIDTH as f32 * TILE_SIZE / 2.0) + (TILE_SIZE / 2.0);
        let y_pos =
            grid_y as f32 * TILE_SIZE - (WORLD_HEIGHT as f32 * TILE_SIZE / 2.0) + (TILE_SIZE / 2.0);

        // 创建矮人主体(父实体) - 背景圆
        commands
            .spawn((
                Sprite {
                    color: Color::srgb(0.85, 0.65, 0.45),
                    custom_size: Some(Vec2::new(TILE_SIZE * 0.8, TILE_SIZE * 0.8)),
                    ..default()
                },
                Transform::from_xyz(x_pos, y_pos, 2.0),
                Dwarf::new(name.to_string()),
                GridPosition {
                    x: grid_x,
                    y: grid_y,
                },
                Velocity { x: 0.0, y: 0.0 },
                WorkState {
                    current_task: Some(Task::Idle),
                    work_progress: 0.0,
                    cached_path: Vec::new(),
                    path_index: 0,
                    path_recalc_timer: 0.0,
                    task_cooldown: 0.0,
                    task_duration: 0.0,
                },
            ))
            .with_children(|parent| {
                // 阴影
                parent.spawn((
                    Sprite {
                        color: Color::srgba(0.0, 0.0, 0.0, 0.3),
                        custom_size: Some(Vec2::new(TILE_SIZE * 0.9, TILE_SIZE * 0.3)),
                        ..default()
                    },
                    Transform::from_xyz(1.0, -2.0, -0.5),
                ));

                // 边框
                parent.spawn((
                    Sprite {
                        color: Color::srgb(0.4, 0.3, 0.2),
                        custom_size: Some(Vec2::new(TILE_SIZE * 0.85, TILE_SIZE * 0.85)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, -0.1),
                ));

                // ASCII矮人字符 - 使用@符号(经典Roguelike)
                parent.spawn((
                    Text2d::new("@"),
                    TextFont {
                        font: font.clone(),
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.3, 0.2, 0.1)),
                    Transform::from_xyz(0.0, 0.0, 0.05),
                    AsciiChar { character: '@' },
                ));

                // 工作状态指示器
                parent.spawn((
                    Sprite {
                        color: Color::srgba(1.0, 1.0, 0.0, 0.6),
                        custom_size: Some(Vec2::new(6.0, 6.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 15.0, 0.2),
                    WorkIndicator,
                ));

                // 选择指示器(默认不可见) - 使用方框字符
                parent.spawn((
                    Text2d::new("[  ]"),
                    TextFont {
                        font: font.clone(),
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::srgba(1.0, 1.0, 0.0, 0.0)),
                    Transform::from_xyz(0.0, 0.0, -0.15),
                    SelectionIndicator,
                ));
            });
    }
}

fn pick_tile_visual(
    rng: &mut SmallRng,
    terrain: TerrainType,
    biome: Option<WorldBiome>,
    variation: f32,
) -> (Color, char, Color) {
    match terrain {
        TerrainType::Grass => match biome {
            Some(WorldBiome::Desert) => (
                color_from_base((0.62, 0.56, 0.38), variation, (0.4, 0.3, 0.2)),
                if rng.gen_ratio(1, 12) { '`' } else { '.' },
                Color::srgba(0.45, 0.35, 0.18, 0.35),
            ),
            Some(WorldBiome::Tundra) => (
                color_from_base((0.75, 0.8, 0.82), variation, (0.3, 0.25, 0.25)),
                if rng.gen_ratio(1, 10) { '*' } else { '.' },
                Color::srgba(0.78, 0.82, 0.86, 0.4),
            ),
            Some(WorldBiome::Swamp) => (
                color_from_base((0.25, 0.5, 0.25), variation, (0.7, 0.8, 0.5)),
                if rng.gen_ratio(1, 8) { ',' } else { ';' },
                Color::srgba(0.08, 0.25, 0.08, 0.45),
            ),
            Some(WorldBiome::Forest) => (
                color_from_base((0.22, 0.6, 0.22), variation, (0.8, 1.4, 0.8)),
                if rng.gen_ratio(1, 12) { '"' } else { ',' },
                Color::srgba(0.08, 0.28, 0.1, 0.5),
            ),
            Some(WorldBiome::River) => (
                color_from_base((0.3, 0.7, 0.25), variation, (0.9, 1.6, 0.8)),
                if rng.gen_ratio(1, 10) { '"' } else { ',' },
                Color::srgba(0.12, 0.35, 0.12, 0.45),
            ),
            _ => (
                color_from_base((0.25, 0.65, 0.2), variation, (1.0, 1.5, 1.0)),
                if rng.gen_ratio(1, 20) {
                    '"'
                } else if rng.gen_ratio(1, 15) {
                    '.'
                } else {
                    ','
                },
                Color::srgba(0.1, 0.3, 0.1, 0.4),
            ),
        },
        TerrainType::Stone => match biome {
            Some(WorldBiome::Desert) => (
                color_from_base((0.7, 0.62, 0.4), variation, (0.4, 0.3, 0.2)),
                if rng.gen_ratio(1, 4) { ':' } else { '~' },
                Color::srgba(0.45, 0.35, 0.2, 0.4),
            ),
            Some(WorldBiome::Mountain) => (
                color_from_base((0.5, 0.48, 0.46), variation, (0.6, 0.5, 0.5)),
                if rng.gen_ratio(1, 4) { '^' } else { '#' },
                Color::srgba(0.25, 0.2, 0.18, 0.5),
            ),
            Some(WorldBiome::Tundra) => (
                color_from_base((0.62, 0.64, 0.68), variation, (0.3, 0.3, 0.3)),
                if rng.gen_ratio(1, 3) { '%' } else { '#' },
                Color::srgba(0.55, 0.58, 0.6, 0.45),
            ),
            _ => (
                color_from_base((0.45, 0.45, 0.5), variation, (1.0, 1.0, 1.0)),
                if rng.gen_ratio(1, 3) { '#' } else { '%' },
                Color::srgba(0.2, 0.2, 0.25, 0.5),
            ),
        },
        TerrainType::Tree => match biome {
            Some(WorldBiome::Desert) => (
                color_from_base((0.4, 0.5, 0.28), variation, (0.6, 0.5, 0.4)),
                if rng.gen_ratio(1, 5) { 'v' } else { 'Y' },
                Color::srgba(0.25, 0.35, 0.15, 0.5),
            ),
            Some(WorldBiome::Swamp) => (
                color_from_base((0.2, 0.4, 0.2), variation, (0.6, 0.8, 0.6)),
                if rng.gen_ratio(1, 4) { '&' } else { ';' },
                Color::srgba(0.05, 0.2, 0.08, 0.6),
            ),
            Some(WorldBiome::Tundra) => (
                color_from_base((0.45, 0.55, 0.48), variation, (0.4, 0.5, 0.4)),
                if rng.gen_ratio(1, 6) { 'x' } else { '^' },
                Color::srgba(0.25, 0.3, 0.25, 0.45),
            ),
            _ => {
                let tree_char = if rng.gen_ratio(1, 5) { '♣' } else { '&' };
                (
                    color_from_base((0.2, 0.55, 0.2), variation, (1.0, 1.0, 1.0)),
                    tree_char,
                    Color::srgba(0.05, 0.2, 0.05, 0.6),
                )
            }
        },
        TerrainType::Water => match biome {
            Some(WorldBiome::Swamp) => (
                color_from_base((0.12, 0.28, 0.25), variation, (0.5, 0.5, 0.5)),
                if rng.gen_ratio(1, 3) { '~' } else { '≈' },
                Color::srgba(0.05, 0.18, 0.12, 0.55),
            ),
            Some(WorldBiome::River) => (
                color_from_base((0.2, 0.5, 0.85), variation, (0.8, 0.8, 0.9)),
                if rng.gen_ratio(1, 3) { '≈' } else { '=' },
                Color::srgba(0.1, 0.35, 0.7, 0.5),
            ),
            Some(WorldBiome::Ocean) => (
                color_from_base((0.1, 0.28, 0.65), variation, (0.6, 0.6, 0.8)),
                if rng.gen_ratio(1, 4) { '≈' } else { '~' },
                Color::srgba(0.04, 0.16, 0.4, 0.55),
            ),
            _ => (
                color_from_base((0.15, 0.35, 0.75), variation, (1.0, 1.0, 1.0)),
                if rng.gen_ratio(1, 3) { '≈' } else { '~' },
                Color::srgba(0.05, 0.15, 0.4, 0.5),
            ),
        },
        TerrainType::Mountain => match biome {
            Some(WorldBiome::Mountain) => (
                color_from_base((0.55, 0.48, 0.4), variation, (0.7, 0.5, 0.4)),
                if rng.gen_ratio(1, 3) { '▲' } else { '^' },
                Color::srgba(0.3, 0.2, 0.15, 0.6),
            ),
            Some(WorldBiome::Tundra) => (
                color_from_base((0.65, 0.6, 0.58), variation, (0.4, 0.4, 0.4)),
                if rng.gen_ratio(1, 3) { '△' } else { '^' },
                Color::srgba(0.35, 0.28, 0.22, 0.55),
            ),
            _ => (
                color_from_base((0.5, 0.4, 0.3), variation, (1.0, 1.0, 1.0)),
                if rng.gen_ratio(1, 3) { '▲' } else { '^' },
                Color::srgba(0.25, 0.15, 0.1, 0.6),
            ),
        },
    }
}

fn color_from_base(base: (f32, f32, f32), variation: f32, scale: (f32, f32, f32)) -> Color {
    Color::srgb(
        (base.0 + variation * scale.0).clamp(0.0, 1.0),
        (base.1 + variation * scale.1).clamp(0.0, 1.0),
        (base.2 + variation * scale.2).clamp(0.0, 1.0),
    )
}
