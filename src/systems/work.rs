use crate::components::*;
use crate::pathfinding::{find_path, simplify_path};
use crate::resources::*;
use crate::world::*;
use crate::debug_entity;
use bevy::prelude::*;
use rand::Rng;

/// 矮人工作系统 - 优化版，智能目标选择和持续工作
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
        // 更新计时器
        work_state.path_recalc_timer += time.delta_secs();
        work_state.task_cooldown -= time.delta_secs();
        work_state.task_duration += time.delta_secs();

        match &work_state.current_task {
            Some(Task::Idle) => {
                // 空闲状态：30%概率寻找工作，70%概率闲逛
                if work_state.task_cooldown <= 0.0 {
                    let should_work = rng.gen_ratio(3, 10); // 30%概率工作
                    let mut candidates: Vec<(GridPosition, TerrainType, f32)> = Vec::new();
                    
                    if should_work {
                        // 寻找工作目标
                        
                        for (terrain_pos, terrain) in terrain_query.iter() {
                            if !terrain.walkable {
                                continue;
                            }
                            
                            // 计算距离
                            let dx = (terrain_pos.x - pos.x).abs();
                            let dy = (terrain_pos.y - pos.y).abs();
                            let distance = ((dx * dx + dy * dy) as f32).sqrt();
                            
                            // 优先选择附近20格内的目标
                            if distance <= 20.0 {
                                // 根据地形类型和资源丰富度评分
                                let terrain_score = match terrain.terrain_type {
                                    TerrainType::Tree => 3.0,      // 树木高优先级
                                    TerrainType::Stone => 2.5,     // 石头较高优先级
                                    TerrainType::Mountain => 2.0,  // 山脉（如果可走）
                                    TerrainType::Grass => 1.5,     // 草地中等优先级
                                    TerrainType::Water => 0.5,     // 水域低优先级
                                };
                                
                                // 综合评分：地形分 * 资源丰富度 / (距离 + 1)
                                let score = terrain_score * terrain.resource_richness / (distance + 1.0);
                                
                                candidates.push((terrain_pos.clone(), terrain.terrain_type, score));
                            }
                        }
                    }
                    
                    // 如果找到候选目标，按评分排序并选择最佳目标
                    if !candidates.is_empty() {
                        candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
                        
                        // 从前30%的候选中随机选择，增加多样性
                        let top_count = (candidates.len() / 3).max(1);
                        
                        // 尝试找一个可达的目标（最多尝试5次）
                        let mut found_reachable = false;
                        for attempt in 0..5.min(top_count) {
                            let chosen_idx = if attempt == 0 {
                                rng.gen_range(0..top_count)
                            } else {
                                rng.gen_range(0..candidates.len())
                            };
                            
                            let (target_pos, terrain_type, _) = &candidates[chosen_idx];
                            let current_pos = (pos.x, pos.y);
                            let target = (target_pos.x, target_pos.y);
                            
                            // 快速路径验证
                            if let Some(_path) = find_path(current_pos, target, &terrain_query) {
                                // 路径存在，分配任务
                                let new_task = match terrain_type {
                                    TerrainType::Tree | TerrainType::Grass => Task::Gathering(target_pos.clone()),
                                    TerrainType::Stone | TerrainType::Mountain => Task::Mining(target_pos.clone()),
                                    _ => Task::Gathering(target_pos.clone()),
                                };
                                
                                debug_entity!("矮人找到工作目标: {:?} at {:?}", terrain_type, target);
                                work_state.current_task = Some(new_task);
                                work_state.cached_path.clear();
                                work_state.path_index = 0;
                                work_state.task_cooldown = 1.0;
                                work_state.task_duration = 0.0;
                                found_reachable = true;
                                break;
                            }
                        }
                        
                        if !found_reachable {
                            debug_entity!("矮人找不到可达的工作目标，开始闲逛");
                            work_state.task_cooldown = 2.0;
                        }
                    } else {
                        // 不寻找工作，开始闲逛
                        debug_entity!("矮人选择闲逛而非工作");
                    }
                    
                    // 无论是否找到工作，都可能开始闲逛
                    if work_state.current_task == Some(Task::Idle) {
                        // 在附近随机选择闲逛目标（5-8格范围）
                        let wander_distance = rng.gen_range(5..=8);
                        let target_x = (pos.x + rng.gen_range(-wander_distance..=wander_distance))
                            .clamp(0, WORLD_WIDTH - 1);
                        let target_y = (pos.y + rng.gen_range(-wander_distance..=wander_distance))
                            .clamp(0, WORLD_HEIGHT - 1);
                        
                        // 检查闲逛目标是否可行走
                        for (terrain_pos, terrain) in terrain_query.iter() {
                            if terrain_pos.x == target_x && terrain_pos.y == target_y && terrain.walkable {
                                work_state.current_task = Some(Task::Wandering(GridPosition {
                                    x: target_x,
                                    y: target_y,
                                }));
                                work_state.cached_path.clear();
                                work_state.path_index = 0;
                                work_state.task_cooldown = 3.0; // 闲逛后3秒再决定下一步
                                work_state.task_duration = 0.0;
                                break;
                            }
                        }
                    }
                }
            }
            Some(Task::Wandering(target)) => {
                // 闲逛：移动到目标但不工作
                let current_pos = (pos.x, pos.y);
                let target_pos = (target.x, target.y);

                // 闲逛超时（10秒后停止）
                if work_state.task_duration > 10.0 {
                    velocity.x = 0.0;
                    velocity.y = 0.0;
                    work_state.current_task = Some(Task::Idle);
                    work_state.cached_path.clear();
                    work_state.path_index = 0;
                    work_state.task_cooldown = 1.0;
                    work_state.task_duration = 0.0;
                    continue;
                }

                // 到达目标，停止闲逛
                if current_pos == target_pos {
                    velocity.x = 0.0;
                    velocity.y = 0.0;
                    work_state.current_task = Some(Task::Idle);
                    work_state.cached_path.clear();
                    work_state.path_index = 0;
                    work_state.task_cooldown = 1.0; // 闲逛结束后快速决定下一步
                    work_state.task_duration = 0.0;
                    continue;
                }

                // 使用与工作相同的寻路逻辑
                let need_recalc = work_state.cached_path.is_empty()
                    || work_state.path_index >= work_state.cached_path.len()
                    || work_state.path_recalc_timer > 5.0;

                if need_recalc {
                    match find_path(current_pos, target_pos, &terrain_query) {
                        Some(path) => {
                            let simplified = simplify_path(path);
                            work_state.cached_path = simplified;
                            work_state.path_index = 0;
                            work_state.path_recalc_timer = 0.0;
                        }
                        None => {
                            // 找不到路径，放弃闲逛
                            velocity.x = 0.0;
                            velocity.y = 0.0;
                            work_state.current_task = Some(Task::Idle);
                            work_state.cached_path.clear();
                            work_state.path_index = 0;
                            work_state.task_cooldown = 1.0;
                            work_state.task_duration = 0.0;
                            continue;
                        }
                    }
                }

                // 沿路径移动
                if !work_state.cached_path.is_empty()
                    && work_state.path_index < work_state.cached_path.len()
                {
                    let next_waypoint = work_state.cached_path[work_state.path_index];

                    let dx = (current_pos.0 - next_waypoint.0).abs();
                    let dy = (current_pos.1 - next_waypoint.1).abs();
                    if dx == 0 && dy == 0 {
                        work_state.path_index += 1;

                        if work_state.path_index < work_state.cached_path.len() {
                            let next = work_state.cached_path[work_state.path_index];
                            let dx = next.0 - current_pos.0;
                            let dy = next.1 - current_pos.1;
                            let distance = ((dx * dx + dy * dy) as f32).sqrt();

                            if distance > 0.01 {
                                velocity.x = (dx as f32 / distance).round();
                                velocity.y = (dy as f32 / distance).round();
                            } else {
                                velocity.x = 0.0;
                                velocity.y = 0.0;
                            }
                        } else {
                            work_state.cached_path.clear();
                            work_state.path_index = 0;
                        }
                    } else {
                        let dx = next_waypoint.0 - current_pos.0;
                        let dy = next_waypoint.1 - current_pos.1;
                        let distance = ((dx * dx + dy * dy) as f32).sqrt();
                        if distance > 0.01 {
                            velocity.x = (dx as f32 / distance).round();
                            velocity.y = (dy as f32 / distance).round();
                        } else {
                            velocity.x = 0.0;
                            velocity.y = 0.0;
                        }
                    }
                } else {
                    velocity.x = 0.0;
                    velocity.y = 0.0;
                }
            }
            Some(Task::Gathering(target)) | Some(Task::Mining(target)) => {
                let current_pos = (pos.x, pos.y);
                let target_pos = (target.x, target.y);

                // 任务超时检测（20秒后放弃）
                if work_state.task_duration > 20.0 {
                    debug_entity!("矮人任务超时，放弃目标 {:?}", target_pos);
                    velocity.x = 0.0;
                    velocity.y = 0.0;
                    work_state.current_task = Some(Task::Idle);
                    work_state.cached_path.clear();
                    work_state.path_index = 0;
                    work_state.task_cooldown = 1.0; // 缩短冷却，快速寻找新目标
                    work_state.task_duration = 0.0;
                    continue;
                }

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

                // 检查是否需要重新计算路径（延长重计算间隔）
                let need_recalc = work_state.cached_path.is_empty()
                    || work_state.path_index >= work_state.cached_path.len()
                    || work_state.path_recalc_timer > 5.0; // 每5秒重新计算一次（原来是2秒）

                if need_recalc {
                    // 使用A*算法计算路径
                    match find_path(current_pos, target_pos, &terrain_query) {
                        Some(path) => {
                            // 使用改进的路径简化算法（已验证相邻性和方向一致性）
                            let simplified = simplify_path(path);
                            work_state.cached_path = simplified;
                            work_state.path_index = 0;
                            work_state.path_recalc_timer = 0.0;
                        }
                        None => {
                            // 找不到路径，放弃任务
                            debug_entity!("矮人无法到达目标 {:?}，寻找新目标", target_pos);
                            velocity.x = 0.0;
                            velocity.y = 0.0;
                            work_state.current_task = Some(Task::Idle);
                            work_state.cached_path.clear();
                            work_state.path_index = 0;
                            work_state.task_cooldown = 0.5; // 快速寻找可达目标
                            work_state.task_duration = 0.0;
                            continue;
                        }
                    }
                }

                // 沿着路径移动
                if !work_state.cached_path.is_empty()
                    && work_state.path_index < work_state.cached_path.len()
                {
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

                            // 计算方向（支持简化路径的非相邻点）
                            let dx = next.0 - current_pos.0;
                            let dy = next.1 - current_pos.1;
                            let distance = ((dx * dx + dy * dy) as f32).sqrt();

                            if distance > 0.01 {
                                velocity.x = (dx as f32 / distance).round();
                                velocity.y = (dy as f32 / distance).round();
                            } else {
                                velocity.x = 0.0;
                                velocity.y = 0.0;
                            }
                        } else {
                            // 路径走完了，但还没到目标？重新计算
                            work_state.cached_path.clear();
                            work_state.path_index = 0;
                        }
                    } else {
                        // 向当前路径点移动（标准化方向，支持简化路径）
                        let dx = next_waypoint.0 - current_pos.0;
                        let dy = next_waypoint.1 - current_pos.1;

                        // 计算单位方向向量（支持任意距离的路径点）
                        let distance = ((dx * dx + dy * dy) as f32).sqrt();
                        if distance > 0.01 {
                            velocity.x = (dx as f32 / distance).round();
                            velocity.y = (dy as f32 / distance).round();
                        } else {
                            velocity.x = 0.0;
                            velocity.y = 0.0;
                        }
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
                        let amount =
                            (base_amount as f32 * terrain_multiplier * resource_richness) as u32;

                        for (terrain_pos, terrain) in terrain_query.iter() {
                            if terrain_pos.x == pos.x && terrain_pos.y == pos.y {
                                match terrain.terrain_type {
                                    crate::components::TerrainType::Tree => {
                                        inventory.wood += amount
                                    }
                                    crate::components::TerrainType::Stone => {
                                        inventory.stone += amount
                                    }
                                    _ => inventory.food += amount,
                                }
                                break;
                            }
                        }

                        work_state.work_progress = 0.0;
                        work_state.current_task = Some(Task::Idle);
                        work_state.task_cooldown = 0.5; // 快速寻找下一个任务
                        work_state.task_duration = 0.0;
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
                        let amount =
                            (base_amount as f32 * terrain_multiplier * resource_richness) as u32;
                        inventory.metal += amount;

                        work_state.work_progress = 0.0;
                        work_state.current_task = Some(Task::Idle);
                        work_state.task_cooldown = 0.5; // 快速寻找下一个任务
                        work_state.task_duration = 0.0;
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
