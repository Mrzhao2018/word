use bevy::prelude::*;
use rand::Rng;
use crate::components::*;
use crate::resources::*;
use crate::world::*;

/// 矮人工作系统
pub fn dwarf_work_system(
    time: Res<Time>,
    mut query: Query<(&mut WorkState, &GridPosition, &mut Velocity, &Dwarf)>,
    _terrain_query: Query<(&GridPosition, &Terrain)>,
) {
    // 如果时间暂停,AI不做决策
    if time.delta_secs() <= 0.0001 {
        return;
    }
    
    let mut rng = rand::thread_rng();
    
    for (mut work_state, pos, mut velocity, _dwarf) in query.iter_mut() {
        match &work_state.current_task {
            Some(Task::Idle) => {
                // 随机分配任务
                if rng.gen_ratio(1, 100) {
                    let new_task = match rng.gen_range(0..3) {
                        0 => Task::Gathering(GridPosition {
                            x: rng.gen_range(0..WORLD_WIDTH),
                            y: rng.gen_range(0..WORLD_HEIGHT),
                        }),
                        1 => Task::Mining(GridPosition {
                            x: rng.gen_range(0..WORLD_WIDTH),
                            y: rng.gen_range(0..WORLD_HEIGHT),
                        }),
                        _ => Task::Idle,
                    };
                    work_state.current_task = Some(new_task);
                }
            }
            Some(Task::Gathering(target)) | Some(Task::Mining(target)) => {
                // 移动到目标位置
                let dx = target.x - pos.x;
                let dy = target.y - pos.y;
                
                if dx.abs() > 0 || dy.abs() > 0 {
                    // 移动中，停止工作
                    velocity.x = dx.signum() as f32 * 0.5;
                    velocity.y = dy.signum() as f32 * 0.5;
                    work_state.work_progress = 0.0; // 移动时重置进度
                } else {
                    // 到达目标位置，停止移动，开始工作
                    velocity.x = 0.0;
                    velocity.y = 0.0;
                    // 工作进度在 resource_gathering_system 中累积
                }
            }
            _ => {}
        }
    }
}

/// 资源采集系统 - 改进版，基于工作进度和地形属性
pub fn resource_gathering_system(
    time: Res<Time>,
    mut query: Query<(&mut WorkState, &GridPosition), With<Dwarf>>,
    terrain_query: Query<(&GridPosition, &Terrain)>,
    mut inventory: ResMut<GlobalInventory>,
) {
    // 如果时间暂停,不采集资源
    if time.delta_secs() <= 0.0001 {
        return;
    }
    
    for (mut work_state, pos) in query.iter_mut() {
        // 先克隆当前任务以避免借用冲突
        let current_task = work_state.current_task.clone();
        
        match &current_task {
            Some(Task::Gathering(target)) => {
                // 到达目标位置才能采集
                if pos.x == target.x && pos.y == target.y {
                    // 获取地形信息
                    let mut terrain_multiplier = 1.0;
                    let mut found_terrain = TerrainType::Grass;
                    
                    for (terrain_pos, terrain) in terrain_query.iter() {
                        if terrain_pos.x == target.x && terrain_pos.y == target.y {
                            found_terrain = terrain.terrain_type;
                            // 采集速度 = 地形倍率 × 资源丰富度
                            terrain_multiplier = terrain.terrain_type.resource_multiplier() * terrain.resource_richness;
                            break;
                        }
                    }
                    
                    // 累积工作进度（考虑地形加成）
                    work_state.work_progress += time.delta_secs() * 0.5 * terrain_multiplier;
                    
                    if work_state.work_progress >= 1.0 {
                        // 完成采集，根据地形类型获得资源
                        match found_terrain {
                            TerrainType::Tree => {
                                let wood_yield = ((1.0 * terrain_multiplier) as i32).max(1) as u32;
                                let food_yield = ((1.0 * terrain_multiplier) as i32).max(1) as u32;
                                inventory.wood += wood_yield;
                                inventory.food += food_yield;
                            }
                            TerrainType::Grass => {
                                let food_yield = ((2.0 * terrain_multiplier) as i32).max(1) as u32;
                                inventory.food += food_yield;
                            }
                            TerrainType::Water => {
                                let food_yield = ((1.0 * terrain_multiplier) as i32).max(1) as u32;
                                inventory.food += food_yield;
                            }
                            _ => {
                                inventory.food += 1;
                            }
                        }
                        
                        // 重置进度，返回空闲
                        work_state.work_progress = 0.0;
                        work_state.current_task = Some(Task::Idle);
                    }
                }
            }
            
            Some(Task::Mining(target)) => {
                // 到达目标位置才能挖矿
                if pos.x == target.x && pos.y == target.y {
                    // 获取地形信息
                    let mut terrain_multiplier = 1.0;
                    let mut found_terrain = TerrainType::Stone;
                    
                    for (terrain_pos, terrain) in terrain_query.iter() {
                        if terrain_pos.x == target.x && terrain_pos.y == target.y {
                            found_terrain = terrain.terrain_type;
                            // 挖矿速度 = 地形倍率 × 资源丰富度
                            terrain_multiplier = terrain.terrain_type.resource_multiplier() * terrain.resource_richness;
                            break;
                        }
                    }
                    
                    // 累积工作进度（考虑地形加成）
                    work_state.work_progress += time.delta_secs() * 0.3 * terrain_multiplier;
                    
                    if work_state.work_progress >= 1.0 {
                        // 完成挖矿，根据地形类型获得资源
                        match found_terrain {
                            TerrainType::Stone => {
                                let stone_yield = ((3.0 * terrain_multiplier) as i32).max(1) as u32;
                                inventory.stone += stone_yield;
                            }
                            TerrainType::Mountain => {
                                let stone_yield = ((2.0 * terrain_multiplier) as i32).max(1) as u32;
                                let metal_yield = ((1.0 * terrain_multiplier) as i32).max(1) as u32;
                                inventory.stone += stone_yield;
                                inventory.metal += metal_yield;
                            }
                            _ => {
                                inventory.stone += 1;
                            }
                        }
                        
                        // 重置进度，返回空闲
                        work_state.work_progress = 0.0;
                        work_state.current_task = Some(Task::Idle);
                    }
                }
            }
            
            _ => {
                // 其他任务重置进度
                work_state.work_progress = 0.0;
            }
        }
    }
}

/// 建筑系统
pub fn building_system(
    time: Res<Time>,
    mut query: Query<&mut Building>,
) {
    for mut building in query.iter_mut() {
        if building.construction_progress < 1.0 {
            building.construction_progress += time.delta_secs() * 0.1;
        }
    }
}
