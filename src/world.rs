use bevy::prelude::*;
use rand::Rng;
use noise::{NoiseFn, Perlin};
use crate::components::*;

/// 世界大小常量
pub const WORLD_WIDTH: i32 = 50;
pub const WORLD_HEIGHT: i32 = 30;
pub const TILE_SIZE: f32 = 32.0;

/// 游戏世界资源
#[derive(Resource)]
pub struct GameWorld {
    #[allow(dead_code)]  // 保留用于未来世界扩展
    pub width: i32,
    #[allow(dead_code)]  // 保留用于未来世界扩展
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
    elevation: Perlin,  // 高度噪声
    moisture: Perlin,   // 湿度噪声
    temperature: Perlin, // 温度噪声
    detail: Perlin,     // 细节噪声
}

impl TerrainGenerator {
    fn new(seed: u32) -> Self {
        Self {
            elevation: Perlin::new(seed),
            moisture: Perlin::new(seed + 1),
            temperature: Perlin::new(seed + 2),
            detail: Perlin::new(seed + 3),
        }
    }
    
    /// 获取指定位置的地形类型
    fn get_terrain(&self, x: i32, y: i32) -> TerrainType {
        let scale = 0.1; // 噪声缩放因子，值越小地形变化越平缓
        
        // 多层噪声采样
        let elevation = self.elevation.get([x as f64 * scale, y as f64 * scale]);
        let moisture = self.moisture.get([x as f64 * scale * 0.8, y as f64 * scale * 0.8]);
        let temperature = self.temperature.get([x as f64 * scale * 1.2, y as f64 * scale * 1.2]);
        let detail = self.detail.get([x as f64 * scale * 3.0, y as f64 * scale * 3.0]) * 0.1;
        
        // 组合噪声值
        let final_elevation = elevation + detail;
        let final_moisture = moisture + detail * 0.5;
        
        // 基于高度和湿度决定地形类型
        if final_elevation < -0.3 {
            TerrainType::Water  // 低洼区域 = 水
        } else if final_elevation > 0.5 {
            if temperature > 0.3 {
                TerrainType::Mountain  // 高海拔 + 高温 = 山脉
            } else {
                TerrainType::Stone     // 高海拔 + 低温 = 石地
            }
        } else if final_moisture > 0.3 && final_elevation > -0.1 {
            TerrainType::Tree  // 湿润 + 中等海拔 = 森林
        } else if final_moisture < -0.2 {
            TerrainType::Stone  // 干燥区域 = 石地
        } else {
            TerrainType::Grass  // 默认草地
        }
    }
    
    /// 检查是否应该生成河流
    fn is_river(&self, x: i32, y: i32) -> bool {
        let river_scale = 0.05;
        let river_noise = self.moisture.get([x as f64 * river_scale, y as f64 * river_scale]);
        
        // 河流沿着湿度噪声的特定等值线
        (river_noise.abs() < 0.05) && (self.elevation.get([x as f64 * 0.1, y as f64 * 0.1]) < 0.3)
    }
}

/// 生成世界地形 - 改进版，使用噪声生成
pub fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>, mut world_seed: ResMut<crate::resources::WorldSeed>) {
    let mut rng = rand::thread_rng();
    let font = asset_server.load("fonts/sarasa-gothic-sc-regular.ttf");
    
    // 创建地形生成器（使用资源中的种子）
    world_seed.seed = rng.gen();
    let generator = TerrainGenerator::new(world_seed.seed);
    
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
            let (color, ascii_char, char_color) = match terrain_type {
                TerrainType::Grass => (
                    Color::srgb(0.25 + color_variation, 0.65 + color_variation * 1.5, 0.2 + color_variation),
                    if rng.gen_ratio(1, 20) { '"' } else if rng.gen_ratio(1, 15) { '.' } else { ',' },
                    Color::srgba(0.1, 0.3, 0.1, 0.4),
                ),
                TerrainType::Stone => (
                    Color::srgb(0.45 + color_variation, 0.45 + color_variation, 0.5 + color_variation),
                    if rng.gen_ratio(1, 3) { '#' } else { '%' },
                    Color::srgba(0.2, 0.2, 0.25, 0.5),
                ),
                TerrainType::Tree => {
                    // 树木有更多变化
                    let tree_char = if rng.gen_ratio(1, 5) { '♣' } else { '&' };
                    (
                        Color::srgb(0.2 + color_variation, 0.55 + color_variation, 0.2 + color_variation),
                        tree_char,
                        Color::srgba(0.05, 0.2, 0.05, 0.6),
                    )
                },
                TerrainType::Water => (
                    Color::srgb(0.15 + color_variation, 0.35 + color_variation, 0.75 + color_variation),
                    if rng.gen_ratio(1, 3) { '≈' } else { '~' },
                    Color::srgba(0.05, 0.15, 0.4, 0.5),
                ),
                TerrainType::Mountain => (
                    Color::srgb(0.5 + color_variation, 0.4 + color_variation, 0.3 + color_variation),
                    if rng.gen_ratio(1, 3) { '▲' } else { '^' },
                    Color::srgba(0.25, 0.15, 0.1, 0.6),
                ),
            };
            
            // 修正坐标计算，让地块中心对齐网格点
            let pos_x = x as f32 * TILE_SIZE - (WORLD_WIDTH as f32 * TILE_SIZE / 2.0) + (TILE_SIZE / 2.0);
            let pos_y = y as f32 * TILE_SIZE - (WORLD_HEIGHT as f32 * TILE_SIZE / 2.0) + (TILE_SIZE / 2.0);
            
            // 计算资源丰富度（基于细节噪声）
            let detail_noise = generator.detail.get([x as f64 * 0.3, y as f64 * 0.3]);
            let resource_richness = 0.8 + (detail_noise as f32 + 1.0) * 0.35; // 0.8 - 1.5
            
            // 主地形方块(背景) - 添加渐变效果
            let gradient_offset = rng.gen_range(-0.02..0.02);
            commands.spawn((
                Sprite {
                    color: Color::srgb(
                        (color.to_srgba().red + gradient_offset).clamp(0.0, 1.0),
                        (color.to_srgba().green + gradient_offset).clamp(0.0, 1.0),
                        (color.to_srgba().blue + gradient_offset).clamp(0.0, 1.0),
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
                AsciiChar { character: ascii_char },
                GridPosition { x, y },
            ));
            
            // 为水和树添加动画组件
            match terrain_type {
                TerrainType::Water => {
                    entity.insert(WaterAnimation { phase: rng.gen_range(0.0..6.28) });
                }
                TerrainType::Tree => {
                    entity.insert(TreeSway { offset: rng.gen_range(0.0..6.28) });
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

/// 生成矮人 - 改进版，确保生成在可行走地形上
pub fn spawn_dwarves(mut commands: Commands, asset_server: Res<AssetServer>, world_seed: Res<crate::resources::WorldSeed>) {
    let dwarf_names = vec!["乌里克", "索林", "巴林", "朵莉", "芬恩", "格洛因", "诺力"];
    let font = asset_server.load("fonts/sarasa-gothic-sc-regular.ttf");
    
    // 使用与世界生成相同的种子创建地形生成器
    let generator = TerrainGenerator::new(world_seed.seed);
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
        for _ in 0..100 {
            // 在中心附近随机选择位置
            let test_x = center_x + rng.gen_range(-15..15);
            let test_y = center_y + rng.gen_range(-15..15);
            
            // 边界检查（留出2格边距）
            if test_x < 2 || test_x >= WORLD_WIDTH - 2 || test_y < 2 || test_y >= WORLD_HEIGHT - 2 {
                continue;
            }
            
            // 检查该位置及周围是否全部可行走
            let mut all_safe = true;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let check_x = test_x + dx;
                    let check_y = test_y + dy;
                    
                    let terrain = generator.get_terrain(check_x, check_y);
                    let is_river = generator.is_river(check_x, check_y);
                    
                    if matches!(terrain, TerrainType::Water | TerrainType::Mountain) || is_river {
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
        
        // 如果没找到，使用中心位置作为后备
        if !found_safe_spot {
            grid_x = center_x;
            grid_y = center_y;
        }
        
        // 根据网格位置计算世界坐标（与地形对齐）
        let x_pos = grid_x as f32 * TILE_SIZE - (WORLD_WIDTH as f32 * TILE_SIZE / 2.0) + (TILE_SIZE / 2.0);
        let y_pos = grid_y as f32 * TILE_SIZE - (WORLD_HEIGHT as f32 * TILE_SIZE / 2.0) + (TILE_SIZE / 2.0);
        
        // 创建矮人主体(父实体) - 背景圆
        commands.spawn((
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
