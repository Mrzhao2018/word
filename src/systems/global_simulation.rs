use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;

/// 全局模拟系统 - 模拟不在当前地图的矮人工作
/// 这个系统在每次进入地图前运行，计算矮人在离开期间完成的工作
pub fn simulate_offscreen_dwarves(
    mut map_registry: ResMut<GeneratedMapsRegistry>,
    mut inventory: ResMut<GlobalInventory>,
    game_time: Res<GameTime>,
    active_local: Res<ActiveLocalMap>,
    mut logger: ResMut<crate::logger::GameLogger>,
) {
    let current_coord = match active_local.coord {
        Some(coord) => coord,
        None => return,
    };
    
    // 获取当前地块的矮人数据
    let stored_dwarves = match map_registry.dwarves.get_mut(&current_coord) {
        Some(dwarves) => dwarves,
        None => return,
    };
    
    let current_day = game_time.day;
    let current_hour = game_time.hour;
    
    let mut total_resources_gathered = (0u32, 0u32, 0u32, 0u32); // (wood, stone, food, metal)
    let mut max_hours_passed = 0u32; // 记录最长的时间差用于日志
    
    for dwarf in stored_dwarves.iter_mut() {
        // 计算时间差（以小时为单位）
        let time_passed_hours = if current_day > dwarf.last_update_day {
            (current_day - dwarf.last_update_day) * 24 + current_hour - dwarf.last_update_hour
        } else {
            current_hour.saturating_sub(dwarf.last_update_hour)
        };
        
        if time_passed_hours == 0 {
            continue;
        }
        
        max_hours_passed = max_hours_passed.max(time_passed_hours);
        
        // 更新矮人的时间戳
        dwarf.last_update_day = current_day;
        dwarf.last_update_hour = current_hour;
        
        // 根据任务类型模拟工作
        let resources = simulate_dwarf_work(dwarf, time_passed_hours);
        
        total_resources_gathered.0 += resources.0;
        total_resources_gathered.1 += resources.1;
        total_resources_gathered.2 += resources.2;
        total_resources_gathered.3 += resources.3;
    }
    
    // 累加资源到全局库存
    if total_resources_gathered.0 > 0 || total_resources_gathered.1 > 0 
        || total_resources_gathered.2 > 0 || total_resources_gathered.3 > 0 {
        inventory.wood += total_resources_gathered.0;
        inventory.stone += total_resources_gathered.1;
        inventory.food += total_resources_gathered.2;
        inventory.metal += total_resources_gathered.3;
        
        logger.info(format!(
            "地块 {:?} 离线采集: 木材+{}, 石头+{}, 食物+{}, 金属+{}",
            current_coord,
            total_resources_gathered.0,
            total_resources_gathered.1,
            total_resources_gathered.2,
            total_resources_gathered.3
        ));
    }
}

/// 模拟单个矮人的工作，返回采集的资源 (wood, stone, food, metal)
fn simulate_dwarf_work(dwarf: &mut StoredDwarf, hours_passed: u32) -> (u32, u32, u32, u32) {
    let mut resources = (0u32, 0u32, 0u32, 0u32);
    
    // 根据当前任务类型决定工作效率
    let (work_hours_per_cycle, task_type) = match &dwarf.current_task {
        Some(Task::Gathering(_)) => (2.0, "gathering"), // 采集任务平均2小时完成一次
        Some(Task::Mining(_)) => (3.0, "mining"),       // 挖矿任务平均3小时完成一次
        Some(Task::Idle) => (4.0, "idle"),              // 空闲状态偶尔也会工作
        Some(Task::Wandering(_)) => (6.0, "wandering"), // 闲逛时很少工作
        _ => return resources,
    };
    
    // 计算完成的工作周期数
    let cycles_completed = (hours_passed as f32 / work_hours_per_cycle).floor() as u32;
    
    if cycles_completed == 0 {
        return resources;
    }
    
    // 根据任务类型分配资源
    match task_type {
        "gathering" => {
            // 采集主要产出木材和食物
            resources.0 = cycles_completed * 2; // 木材
            resources.2 = cycles_completed;     // 食物
        }
        "mining" => {
            // 挖矿主要产出石头和金属
            resources.1 = cycles_completed * 2; // 石头
            resources.3 = cycles_completed;     // 金属
        }
        "idle" => {
            // 空闲状态下随机采集一些资源（效率较低）
            resources.0 = cycles_completed / 2;
            resources.2 = cycles_completed / 2;
        }
        "wandering" => {
            // 闲逛时偶尔采集少量资源
            resources.2 = cycles_completed / 3;
        }
        _ => {}
    }
    
    // 更新工作进度（模拟部分完成的工作）
    let remaining_time = hours_passed as f32 - (cycles_completed as f32 * work_hours_per_cycle);
    dwarf.work_progress = (remaining_time / work_hours_per_cycle).min(0.99);
    
    // 模拟基本需求变化（每小时轻微变化）
    let hours_f = hours_passed as f32;
    dwarf.hunger = (dwarf.hunger - hours_f * 0.5).max(0.0);     // 饥饿度下降
    dwarf.happiness = (dwarf.happiness - hours_f * 0.2).max(20.0); // 幸福度略微下降，但保持最低值
    
    resources
}

/// 模拟所有未加载地图的矮人（在WorldView状态下调用）
pub fn simulate_all_offscreen_dwarves(
    mut map_registry: ResMut<GeneratedMapsRegistry>,
    mut inventory: ResMut<GlobalInventory>,
    game_time: Res<GameTime>,
    mut logger: ResMut<crate::logger::GameLogger>,
) {
    let current_day = game_time.day;
    let current_hour = game_time.hour;
    
    let mut total_wood = 0u32;
    let mut total_stone = 0u32;
    let mut total_food = 0u32;
    let mut total_metal = 0u32;
    let mut tiles_processed = 0;
    
    // 遍历所有有矮人的地块
    for (coord, dwarves) in map_registry.dwarves.iter_mut() {
        let mut tile_resources = (0u32, 0u32, 0u32, 0u32);
        
        for dwarf in dwarves.iter_mut() {
            // 计算时间差
            let time_passed_hours = if current_day > dwarf.last_update_day {
                (current_day - dwarf.last_update_day) * 24 + current_hour - dwarf.last_update_hour
            } else {
                current_hour.saturating_sub(dwarf.last_update_hour)
            };
            
            if time_passed_hours == 0 {
                continue;
            }
            
            // 更新时间戳
            dwarf.last_update_day = current_day;
            dwarf.last_update_hour = current_hour;
            
            // 模拟工作
            let resources = simulate_dwarf_work(dwarf, time_passed_hours);
            tile_resources.0 += resources.0;
            tile_resources.1 += resources.1;
            tile_resources.2 += resources.2;
            tile_resources.3 += resources.3;
        }
        
        if tile_resources.0 > 0 || tile_resources.1 > 0 
            || tile_resources.2 > 0 || tile_resources.3 > 0 {
            total_wood += tile_resources.0;
            total_stone += tile_resources.1;
            total_food += tile_resources.2;
            total_metal += tile_resources.3;
            tiles_processed += 1;
            
            logger.debug(format!(
                "地块 {:?} 后台生产: 木材+{}, 石头+{}, 食物+{}, 金属+{}",
                coord, tile_resources.0, tile_resources.1, tile_resources.2, tile_resources.3
            ));
        }
    }
    
    // 累加所有资源
    if tiles_processed > 0 {
        inventory.wood += total_wood;
        inventory.stone += total_stone;
        inventory.food += total_food;
        inventory.metal += total_metal;
        
        logger.info(format!(
            "全局模拟: {} 个地块后台运行，采集 木材:{}, 石头:{}, 食物:{}, 金属:{}",
            tiles_processed, total_wood, total_stone, total_food, total_metal
        ));
    }
}
