use bevy::prelude::*;

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
#[derive(Resource, Default)]
pub struct GameTime {
    pub day: u32,
    pub hour: u32,
    pub elapsed: f32,
}

/// 选中的矮人
#[derive(Resource, Default)]
pub struct SelectedDwarf {
    pub entity: Option<Entity>,
}

// 实现方法以避免未使用警告
#[allow(dead_code)]
impl SelectedDwarf {
    pub fn select(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }
    
    pub fn deselect(&mut self) {
        self.entity = None;
    }
    
    pub fn is_selected(&self, entity: Entity) -> bool {
        self.entity == Some(entity)
    }
}
