use bevy::prelude::*;

/// 游戏状态
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    WorldView,
    LocalView,
    Paused,
}

/// 全局资源库存
#[derive(Resource)]
pub struct GlobalInventory {
    pub stone: u32,
    pub wood: u32,
    pub food: u32,
    pub metal: u32,
}

impl Default for GlobalInventory {
    fn default() -> Self {
        Self {
            stone: 50,
            wood: 30,
            food: 100,
            metal: 10,
        }
    }
}

/// 游戏时间
#[derive(Resource)]
pub struct GameTime {
    pub day: u32,
    pub hour: u32,
    pub elapsed: f32,
    pub time_scale: f32, // 时间流逝速度倍率 (0.0 = 暂停, 1.0 = 正常, 2.0 = 2倍速)
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            day: 0,
            hour: 6,
            elapsed: 0.0,
            time_scale: 1.0,
        }
    }
}

impl GameTime {
    /// 获取当前时间的光照强度 (0.0 = 黑夜, 1.0 = 白天)
    #[allow(dead_code)] // 保留用于未来更复杂的昼夜系统
    pub fn get_daylight(&self) -> f32 {
        // 6点日出,18点日落
        if self.hour < 6 {
            0.6 // 夜晚 - 调亮了
        } else if self.hour < 8 {
            // 日出渐变 6-8点
            0.6 + (self.hour - 6) as f32 * 0.2
        } else if self.hour < 18 {
            1.0 // 白天
        } else if self.hour < 20 {
            // 日落渐变 18-20点
            1.0 - (self.hour - 18) as f32 * 0.2
        } else {
            0.6 // 夜晚 - 调亮了
        }
    }

    /// 获取环境光颜色
    #[allow(dead_code)] // 保留用于未来更复杂的光照系统
    pub fn get_ambient_color(&self) -> Color {
        let daylight = self.get_daylight();
        if self.hour >= 6 && self.hour < 8 {
            // 日出 - 温暖的橙色
            Color::srgb(1.0, 0.85, 0.7)
        } else if self.hour >= 18 && self.hour < 20 {
            // 日落 - 温暖的橙色
            Color::srgb(1.0, 0.85, 0.7)
        } else if daylight < 0.8 {
            // 夜晚 - 轻微的蓝色调
            Color::srgb(0.85, 0.85, 0.95)
        } else {
            // 白天 - 正常光
            Color::srgb(1.0, 1.0, 1.0)
        }
    }
}

/// 选中的矮人
#[derive(Resource, Default)]
pub struct SelectedDwarf {
    pub entity: Option<Entity>,
}

/// 游戏是否已初始化（用于区分首次进入和从暂停恢复）
#[derive(Resource, Default)]
pub struct GameInitialized {
    pub initialized: bool,
}

/// 世界生成种子（确保地形和矮人生成使用相同的地形数据）
#[derive(Resource)]
pub struct WorldSeed {
    pub seed: u32,
}

impl Default for WorldSeed {
    fn default() -> Self {
        Self {
            seed: rand::random(),
        }
    }
}

/// 当前激活的局部地图（从大地图进入时设置）
#[derive(Resource, Default)]
pub struct ActiveLocalMap {
    pub coord: Option<IVec2>,
}

/// 存储的地图块数据
#[derive(Clone)]
pub struct StoredMapTile {
    pub x: i32,
    pub y: i32,
    pub terrain_type: crate::components::TerrainType,
    pub walkable: bool,
    pub resource_richness: f32,
    pub color: bevy::color::Color,
    pub ascii_char: char,
    pub char_color: bevy::color::Color,
    pub has_water_animation: bool,
    pub has_tree_sway: bool,
    pub water_phase: f32,
    pub tree_offset: f32,
}

/// 存储的矮人数据
#[derive(Clone)]
pub struct StoredDwarf {
    pub name: String,
    pub grid_x: i32,
    pub grid_y: i32,
    pub health: f32,
    pub hunger: f32,
    pub happiness: f32,
    pub current_task: Option<crate::components::Task>,
    pub work_progress: f32,
    /// 上次更新时的游戏时间（用于全局模拟）
    pub last_update_day: u32,
    pub last_update_hour: u32,
}

/// 已生成的局部地图注册表（世界线持久化）
#[derive(Resource, Default)]
pub struct GeneratedMapsRegistry {
    /// 存储每个地块的地图数据 - key: 世界坐标(x,y), value: 一维数组，使用x和y索引
    pub maps: std::collections::HashMap<IVec2, Vec<StoredMapTile>>,
    /// 存储每个地块的矮人数据 - key: 世界坐标(x,y)
    pub dwarves: std::collections::HashMap<IVec2, Vec<StoredDwarf>>,
    /// 初始出生地块（矮人只在这里生成）
    pub spawn_location: Option<IVec2>,
    /// 矮人是否已经生成（防止重复生成）
    pub dwarves_spawned: bool,
}
