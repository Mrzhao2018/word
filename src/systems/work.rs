use bevy::prelude::*;
use rand::Rng;
use crate::components::*;
use crate::resources::*;
use crate::world::*;
use crate::pathfinding::{find_path, simplify_path};

/// 矮人工作系统 - 完全重写版，使用A*寻路算法
pub fn dwarf_work_system(
    time: Res<Time>,
    mut query: Query<(&mut WorkState, &GridPosition, &mut Velocity, &Dwarf)>,
    terrain_query: Query<(&GridPosition, &Terrain)>,
) {
    // 如果时间暂停,AI不做决策
    if time.delta_secs() <= 0.0001 {
        return;
    }
    
    let mut rng = rand::thread_rng();
    
    for (mut work_state, pos, mut velocity, _dwarf) in query.iter_mut() {
        // 更新路径重计算计时器
        work_state.path_recalc_timer += time.delta_secs();
        
        match &work_state.current_task {
            Some(Task::Idle) => {
                // 空闲状态：随机分配新任务
                if rng.gen_ratio(1, 100) {
                    let target_x = rng.gen_range(0..WORLD_WIDTH);
                    let target_y = rng.gen_range(0..WORLD_HEIGHT);
                    
                    // 检查目标是否可行走
                    let mut is_walkable = false;
                    for (terrain_pos, terrain) in terrain_query.iter() {
                        if terrain_pos.x == target_x && terrain_pos.y == target_y {
                            is_walkable = terrain.walkable;
                            break;
                        }
                    }
                    
                    if is_walkable {
                        let task_type = rng.gen_range(0..3);
                        let new_task = match task_type {
                            0 => Task::Gathering(GridPosition { x: target_x, y: target_y }),
                            1 => Task::Mining(GridPosition { x: target_x, y: target_y }),
                            _ => Task::Idle,
                        };
                        
                        work_state.current_task = Some(new_task);
                        work_state.cached_path.clear();
                        work_state.path_index = 0;
                    }
                }
            }
            Some(Task::Gathering(target)) | Some(Task::Mining(target)) => {
                let current_pos = (pos.x, pos.y);
                let target_pos = (target.x, target.y);
                
                // 检查是否已到达目标
                if current_pos == target_pos {
                    // 到达目标，停止移动
                    velocity.x = 0.0;
                    velocity.y = 0.0;
                    work_state.cached_path.clear();
                    work_state.path_index = 0;
                    // 工作进度在 resource_gathering_system 中累积
                    continue;
                }
                
                // 检查是否需要重新计算路径
                let need_recalc = work_state.cached_path.is_empty() 
                    || work_state.path_index >= work_state.cached_path.len()
                    || work_state.path_recalc_timer > 2.0; // 每2秒重新计算一次
                
                if need_recalc {
                    // 使用A*算法计算路径
                    match find_path(current_pos, target_pos, &terrain_query) {
                        Some(path) => {
                            // 直接使用原始路径（暂时禁用简化，因为会导致抽搐）
                            work_state.cached_path = path;
                            work_state.path_index = 0;
                            work_state.path_recalc_timer = 0.0;
                        }
                        None => {
                            // 找不到路径，放弃任务
                            velocity.x = 0.0;
                            velocity.y = 0.0;
                            work_state.current_task = Some(Task::Idle);
                            work_state.cached_path.clear();
                            work_state.path_index = 0;
                            continue;
                        }
                    }
                }
                
                // 沿着路径移动
                if !work_state.cached_path.is_empty() && work_state.path_index < work_state.cached_path.len() {
                    let next_waypoint = work_state.cached_path[work_state.path_index];
                    
                    // 检查下一个路径点是否仍然可行走
                    let mut waypoint_walkable = false;
                    for (terrain_pos, terrain) in terrain_query.iter() {
                        if terrain_pos.x == next_waypoint.0 && terrain_pos.y == next_waypoint.1 {
                            waypoint_walkable = terrain.walkable;
                            break;
                        }
                    }
                    
                    if !waypoint_walkable {
                        // 路径点变得不可行走（例如动态障碍），重新计算路径
                        work_state.cached_path.clear();
                        work_state.path_index = 0;
                        work_state.path_recalc_timer = 999.0; // 强制重新计算
                        continue;
                    }
                    
                    // 检查是否到达当前路径点（使用距离阈值，因为GridPosition可能还没更新）
                    let dx = (current_pos.0 - next_waypoint.0).abs();
                    let dy = (current_pos.1 - next_waypoint.1).abs();
                    if dx == 0 && dy == 0 {
                        // 已经在目标格子上，前进到下一个路径点
                        work_state.path_index += 1;
                        
                        // 如果还有下一个路径点
                        if work_state.path_index < work_state.cached_path.len() {
                            let next = work_state.cached_path[work_state.path_index];
                            velocity.x = (next.0 - current_pos.0) as f32;
                            velocity.y = (next.1 - current_pos.1) as f32;
                        } else {
                            // 路径走完了，但还没到目标？重新计算
                            work_state.cached_path.clear();
                            work_state.path_index = 0;
                        }
                    } else {
                        // 向当前路径点移动
                        let dx = next_waypoint.0 - current_pos.0;
                        let dy = next_waypoint.1 - current_pos.1;
                        velocity.x = dx as f32;
                        velocity.y = dy as f32;
                    }
                } else {
                    // 没有路径，停止
                    velocity.x = 0.0;
                    velocity.y = 0.0;
                }
            }
            _ => {
                velocity.x = 0.0;
                velocity.y = 0.0;
            }
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
                    let mut resource_richness = 1.0;
                    
                    for (terrain_pos, terrain) in terrain_query.iter() {
                        if terrain_pos.x == pos.x && terrain_pos.y == pos.y {
                            terrain_multiplier = terrain.terrain_type.resource_multiplier();
                            resource_richness = terrain.resource_richness;
                            break;
                        }
                    }
                    
                    // 累积工作进度，考虑地形和资源丰富度
                    let progress_speed = 0.2 * terrain_multiplier * resource_richness;
                    work_state.work_progress += time.delta_secs() * progress_speed;
                    
                    // 完成采集
                    if work_state.work_progress >= 1.0 {
                        // 根据地形类型添加资源
                        let base_amount = 1;
                        let amount = (base_amount as f32 * terrain_multiplier * resource_richness) as u32;
                        
                        for (terrain_pos, terrain) in terrain_query.iter() {
                            if terrain_pos.x == pos.x && terrain_pos.y == pos.y {
                                match terrain.terrain_type {
                                    crate::components::TerrainType::Tree => inventory.wood += amount,
                                    crate::components::TerrainType::Stone => inventory.stone += amount,
                                    _ => inventory.food += amount,
                                }
                                break;
                            }
                        }
                        
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
                    let mut resource_richness = 1.0;
                    
                    for (terrain_pos, terrain) in terrain_query.iter() {
                        if terrain_pos.x == pos.x && terrain_pos.y == pos.y {
                            terrain_multiplier = terrain.terrain_type.resource_multiplier();
                            resource_richness = terrain.resource_richness;
                            break;
                        }
                    }
                    
                    // 累积工作进度
                    let progress_speed = 0.15 * terrain_multiplier * resource_richness;
                    work_state.work_progress += time.delta_secs() * progress_speed;
                    
                    // 完成挖矿
                    if work_state.work_progress >= 1.0 {
                        let base_amount = 2;
                        let amount = (base_amount as f32 * terrain_multiplier * resource_richness) as u32;
                        inventory.metal += amount;
                        
                        work_state.work_progress = 0.0;
                        work_state.current_task = Some(Task::Idle);
                    }
                }
            }
            _ => {}
        }
    }
}

/// 建筑系统占位
pub fn building_system() {
    // 建筑系统暂未实现
}
