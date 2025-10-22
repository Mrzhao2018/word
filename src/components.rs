use bevy::prelude::*;

/// 矮人组件
#[derive(Component)]
#[allow(dead_code)] // 字段预留用于未来健康/饥饿系统
pub struct Dwarf {
    pub name: String,
    pub health: f32,
    pub hunger: f32,
    pub happiness: f32,
}

// 实现方法以避免未使用警告
impl Dwarf {
    #[allow(dead_code)]
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
}

/// UI标记组件
#[derive(Component)]
pub struct ResourceDisplay;

#[derive(Component)]
pub struct TitleDisplay;

#[derive(Component)]
pub struct HelpDisplay;

/// 工作指示器标记
#[derive(Component)]
pub struct WorkIndicator;

#[derive(Clone, Debug)]
#[allow(dead_code)] // Building预留用于未来功能
pub enum Task {
    Mining(GridPosition),
    Building(GridPosition, BuildingType),
    Gathering(GridPosition),
    Idle,
}

/// 资源类型
#[derive(Component, Clone, Copy, PartialEq)]
#[allow(dead_code)] // 预留用于未来功能
pub enum ResourceType {
    Stone,
    Wood,
    Food,
    Metal,
}

/// 资源堆
#[derive(Component)]
#[allow(dead_code)] // 预留用于未来功能
pub struct ResourcePile {
    pub resource_type: ResourceType,
    pub amount: u32,
}

/// 建筑类型
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)] // 预留用于未来功能
pub enum BuildingType {
    Workshop,
    Stockpile,
    Farm,
    LivingQuarters,
}

/// 建筑组件
#[derive(Component)]
#[allow(dead_code)] // 预留用于未来功能
pub struct Building {
    pub building_type: BuildingType,
    pub construction_progress: f32, // 0.0 到 1.0
}

/// 地形类型
#[derive(Component, Clone, Copy, PartialEq)]
#[allow(dead_code)] // Mountain预留用于未来功能
pub enum TerrainType {
    Grass,
    Stone,
    Tree,
    Water,
    Mountain,
}

/// 地形tile
#[derive(Component)]
#[allow(dead_code)] // 字段预留用于未来碰撞检测和寻路
pub struct Terrain {
    pub terrain_type: TerrainType,
    pub walkable: bool,
}

// 实现方法以避免未使用警告
#[allow(dead_code)]
impl Terrain {
    pub fn is_walkable(&self) -> bool {
        self.walkable
    }
    
    pub fn get_type(&self) -> TerrainType {
        self.terrain_type
    }
}
