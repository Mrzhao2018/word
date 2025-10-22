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
#[derive(Component, Clone, Debug)]
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
    pub work_progress: f32, // 工作进度 0.0-1.0
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
    #[allow(dead_code)]  // 保留用于未来可能的字符判断逻辑
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

#[derive(Clone, Debug)]
pub enum Task {
    Mining(GridPosition),
    #[allow(dead_code)]  // 保留用于未来建筑系统
    Building(GridPosition, BuildingType),
    Gathering(GridPosition),
    Idle,
}

/// 建筑类型
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]  // 保留用于未来建筑系统扩展
pub enum BuildingType {
    Workshop,
    Stockpile,
    Farm,
    LivingQuarters,
}

/// 建筑组件
#[derive(Component)]
pub struct Building {
    #[allow(dead_code)]  // 保留用于未来建筑系统扩展
    pub building_type: BuildingType,
    pub construction_progress: f32, // 0.0 到 1.0
}

/// 地形类型
#[derive(Component, Clone, Copy, PartialEq)]
pub enum TerrainType {
    Grass,
    Stone,
    Tree,
    Water,
    Mountain,
}

/// 地形tile
#[derive(Component)]
pub struct Terrain {
    pub terrain_type: TerrainType,
    #[allow(dead_code)]  // 保留用于未来寻路系统
    pub walkable: bool,
}
