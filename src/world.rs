use bevy::prelude::*;
use rand::Rng;
use crate::components::*;

/// 世界大小常量
pub const WORLD_WIDTH: i32 = 50;
pub const WORLD_HEIGHT: i32 = 30;
pub const TILE_SIZE: f32 = 32.0;

/// 游戏世界资源
#[derive(Resource)]
pub struct GameWorld {
    pub width: i32,
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

// 实现方法以避免未使用警告
#[allow(dead_code)]
impl GameWorld {
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }
    
    pub fn get_tile_index(&self, x: i32, y: i32) -> Option<usize> {
        if self.in_bounds(x, y) {
            Some((y * self.width + x) as usize)
        } else {
            None
        }
    }
}

/// 生成世界地形
pub fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut rng = rand::thread_rng();
    let font = asset_server.load("fonts/SourceHanSansCN-Regular.otf");
    
    for x in 0..WORLD_WIDTH {
        for y in 0..WORLD_HEIGHT {
            // 随机生成地形
            let terrain_type = if rng.gen_ratio(1, 10) {
                TerrainType::Tree
            } else if rng.gen_ratio(1, 15) {
                TerrainType::Stone
            } else if rng.gen_ratio(1, 30) {
                TerrainType::Mountain
            } else if y < 5 && rng.gen_ratio(1, 20) {
                TerrainType::Water
            } else {
                TerrainType::Grass
            };
            
            let walkable = !matches!(terrain_type, TerrainType::Water | TerrainType::Mountain);
            
            // 更好看的颜色和随机变化
            let color_variation = rng.gen_range(-0.05..0.05);
            let (color, ascii_char, char_color) = match terrain_type {
                TerrainType::Grass => (
                    Color::srgb(0.25 + color_variation, 0.65 + color_variation * 1.5, 0.2 + color_variation),
                    if rng.gen_ratio(1, 20) { '"' } else { ',' },
                    Color::srgba(0.1, 0.3, 0.1, 0.4),  // 深绿色字符
                ),
                TerrainType::Stone => (
                    Color::srgb(0.45 + color_variation, 0.45 + color_variation, 0.5 + color_variation),
                    if rng.gen_ratio(1, 3) { '#' } else { '%' },
                    Color::srgba(0.2, 0.2, 0.25, 0.5),  // 深灰色字符
                ),
                TerrainType::Tree => (
                    Color::srgb(0.2 + color_variation, 0.55 + color_variation, 0.2 + color_variation),
                    '&',
                    Color::srgba(0.05, 0.2, 0.05, 0.6),  // 深绿色字符
                ),
                TerrainType::Water => (
                    Color::srgb(0.15 + color_variation, 0.35 + color_variation, 0.75 + color_variation),
                    '~',
                    Color::srgba(0.05, 0.15, 0.4, 0.5),  // 深蓝色字符
                ),
                TerrainType::Mountain => (
                    Color::srgb(0.5 + color_variation, 0.4 + color_variation, 0.3 + color_variation),
                    '^',
                    Color::srgba(0.25, 0.15, 0.1, 0.6),  // 深棕色字符
                ),
            };
            
            let pos_x = x as f32 * TILE_SIZE - (WORLD_WIDTH as f32 * TILE_SIZE / 2.0);
            let pos_y = y as f32 * TILE_SIZE - (WORLD_HEIGHT as f32 * TILE_SIZE / 2.0);
            
            // 主地形方块(背景)
            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(TILE_SIZE - 1.0, TILE_SIZE - 1.0)),
                    ..default()
                },
                Transform::from_xyz(pos_x, pos_y, 0.0),
                Terrain {
                    terrain_type,
                    walkable,
                },
                GridPosition { x, y },
            ));
            
            // ASCII字符层
            commands.spawn((
                Text2d::new(ascii_char.to_string()),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(char_color),
                Transform::from_xyz(pos_x, pos_y, 0.05),
                AsciiChar { character: ascii_char },
            ));
            
            // 添加网格线效果
            commands.spawn((
                Sprite {
                    color: Color::srgba(0.0, 0.0, 0.0, 0.1),
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                    ..default()
                },
                Transform::from_xyz(pos_x, pos_y, 0.1),
            ));
        }
    }
}

/// 生成矮人
pub fn spawn_dwarves(mut commands: Commands, asset_server: Res<AssetServer>) {
    let dwarf_names = vec!["乌里克", "索林", "巴林", "朵莉", "芬恩", "格洛因", "诺力"];
    let font = asset_server.load("fonts/SourceHanSansCN-Regular.otf");
    
    for (i, name) in dwarf_names.iter().enumerate() {
        let x_pos = (i as f32 * 50.0) - 150.0;
        let y_pos = 0.0;
        
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
                x: WORLD_WIDTH / 2 + i as i32 - 3,
                y: WORLD_HEIGHT / 2,
            },
            Velocity { x: 0.0, y: 0.0 },
            WorkState {
                current_task: Some(Task::Idle),
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
