use bevy::prelude::*;

/// 矮人组件
#[derive(Component)]
pub struct Dwarf {
    pub name: String,
    pub health: f32,
    pub hunger: f32,
    pub happiness: f32,
}

impl Dwarf {
    pub fn new(name: String) -> Self {
        Self {
            name,
            health: 100.0,
            hunger: 50.0,
            happiness: 75.0,
        }
    }
}

/// 位置组件(网格坐标)
#[derive(Component, Clone, Debug, PartialEq)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

/// 速度组件
#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

/// 工作状态
#[derive(Component)]
pub struct WorkState {
    pub current_task: Option<Task>,
    pub work_progress: f32,           // 工作进度 0.0-1.0
    pub cached_path: Vec<(i32, i32)>, // 缓存的路径
    pub path_index: usize,            // 当前路径点索引
    pub path_recalc_timer: f32,       // 路径重新计算计时器
    pub task_cooldown: f32,           // 任务冷却时间（防止频繁切换）
    pub task_duration: f32,           // 当前任务持续时间（防止卡住）
}

/// UI标记组件
#[derive(Component)]
pub struct ResourceDisplay;

#[derive(Component)]
pub struct TitleDisplay;

#[derive(Component)]
pub struct HelpDisplay;

#[derive(Component)]
pub struct DwarfPanel;

/// 工作指示器标记
#[derive(Component)]
pub struct WorkIndicator;

/// 选择指示器标记
#[derive(Component)]
pub struct SelectionIndicator;

/// 鼠标悬停名字标签
#[derive(Component)]
pub struct DwarfNameTag;

/// 地形信息标签
#[derive(Component)]
pub struct TerrainInfoLabel;

/// 主菜单UI标记
#[derive(Component)]
pub struct MainMenuUI;

/// 开始游戏按钮
#[derive(Component)]
pub struct StartButton;

/// 暂停菜单UI标记
#[derive(Component)]
pub struct PauseMenuUI;

/// 继续游戏按钮
#[derive(Component)]
pub struct ResumeButton;

/// 返回主菜单按钮
#[derive(Component)]
pub struct BackToMenuButton;

/// ASCII字符显示组件
#[derive(Component)]
pub struct AsciiChar {
    #[allow(dead_code)] // 保留用于未来可能的字符判断逻辑
    pub character: char,
}

/// 水面动画
#[derive(Component)]
pub struct WaterAnimation {
    pub phase: f32,
}

/// 树木摇摆动画
#[derive(Component)]
pub struct TreeSway {
    pub offset: f32,
}

/// 粒子效果
#[derive(Component)]
pub struct Particle {
    pub lifetime: f32,
    pub velocity: Vec2,
}

/// 昼夜光照覆盖层标记
#[derive(Component)]
pub struct DaylightOverlay;

/// 网格线标记
#[derive(Component)]
pub struct GridLine;

#[derive(Clone, Debug, PartialEq)]
pub enum Task {
    Mining(GridPosition),
    #[allow(dead_code)] // 保留用于未来建筑系统
    Building(GridPosition, BuildingType),
    Gathering(GridPosition),
    Wandering(GridPosition), // 闲逛 - 随机走动但不工作
    Idle,
}

/// 建筑类型
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)] // 保留用于未来建筑系统扩展
pub enum BuildingType {
    Workshop,
    Stockpile,
    Farm,
    LivingQuarters,
}

/// 建筑组件
#[derive(Component)]
#[allow(dead_code)]
pub struct Building {
    // 保留用于未来建筑系统扩展
    pub building_type: BuildingType,
    pub construction_progress: f32, // 0.0 到 1.0
}

/// 地形类型
#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub enum TerrainType {
    Grass,
    Stone,
    Tree,
    Water,
    Mountain,
}

impl TerrainType {
    /// 获取地形的资源产出倍率
    pub fn resource_multiplier(&self) -> f32 {
        match self {
            TerrainType::Tree => 1.5,     // 森林采集效率高
            TerrainType::Stone => 1.2,    // 石地挖矿效率较高
            TerrainType::Mountain => 1.8, // 山脉挖矿效率最高
            TerrainType::Water => 0.8,    // 水边采集效率略低
            TerrainType::Grass => 1.0,    // 草地标准效率
        }
    }

    /// 获取地形的移动速度倍率
    pub fn movement_speed(&self) -> f32 {
        match self {
            TerrainType::Grass => 1.0,    // 草地正常速度
            TerrainType::Stone => 0.9,    // 石地略慢
            TerrainType::Tree => 0.8,     // 森林较慢
            TerrainType::Water => 0.0,    // 水域无法通行
            TerrainType::Mountain => 0.0, // 山脉无法通行
        }
    }

    /// 获取地形的描述
    pub fn description(&self) -> &'static str {
        match self {
            TerrainType::Grass => "草地 - 适合采集食物",
            TerrainType::Stone => "石地 - 适合采集石头",
            TerrainType::Tree => "森林 - 富含木材和食物",
            TerrainType::Water => "水域 - 可以钓鱼",
            TerrainType::Mountain => "山脉 - 富含矿石和金属",
        }
    }
}

/// 地形tile
#[derive(Component)]
pub struct Terrain {
    pub terrain_type: TerrainType,
    pub walkable: bool,
    pub resource_richness: f32, // 资源丰富度 0.5-1.5
}
