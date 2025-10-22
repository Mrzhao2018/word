use bevy::prelude::*;
use rand::Rng;
use crate::components::*;
use crate::resources::*;
use crate::world::*;

/// 矮人移动系统 - 改进版,包含缓动和动画
pub fn dwarf_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut GridPosition, &Velocity), With<Dwarf>>,
) {
    for (mut transform, mut grid_pos, velocity) in query.iter_mut() {
        // 只有在有速度时才移动
        if velocity.x.abs() > 0.01 || velocity.y.abs() > 0.01 {
            // 计算目标位置(基于速度)
            transform.translation.x += velocity.x * time.delta_secs() * 100.0;
            transform.translation.y += velocity.y * time.delta_secs() * 100.0;
        }
        
        // 更新网格位置
        let new_x = ((transform.translation.x + (WORLD_WIDTH as f32 * TILE_SIZE / 2.0)) / TILE_SIZE) as i32;
        let new_y = ((transform.translation.y + (WORLD_HEIGHT as f32 * TILE_SIZE / 2.0)) / TILE_SIZE) as i32;
        
        if new_x >= 0 && new_x < WORLD_WIDTH && new_y >= 0 && new_y < WORLD_HEIGHT {
            grid_pos.x = new_x;
            grid_pos.y = new_y;
        }
    }
}

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

/// 资源采集系统 - 改进版，基于工作进度
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
                    // 累积工作进度
                    work_state.work_progress += time.delta_secs() * 0.5; // 2秒完成一次采集
                    
                    if work_state.work_progress >= 1.0 {
                        // 完成采集，根据地形类型获得资源
                        let mut found_terrain = TerrainType::Grass;
                        for (terrain_pos, terrain) in terrain_query.iter() {
                            if terrain_pos.x == target.x && terrain_pos.y == target.y {
                                found_terrain = terrain.terrain_type;
                                break;
                            }
                        }
                        
                        match found_terrain {
                            TerrainType::Tree => {
                                inventory.wood += 1;
                                inventory.food += 1; // 树木可能有果实
                            }
                            TerrainType::Grass => {
                                inventory.food += 2; // 草地采集食物
                            }
                            TerrainType::Water => {
                                inventory.food += 1; // 水边钓鱼
                            }
                            _ => {
                                inventory.food += 1; // 默认食物
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
                    // 累积工作进度
                    work_state.work_progress += time.delta_secs() * 0.3; // 3.3秒完成一次挖矿
                    
                    if work_state.work_progress >= 1.0 {
                        // 完成挖矿，根据地形类型获得资源
                        let mut found_terrain = TerrainType::Stone;
                        for (terrain_pos, terrain) in terrain_query.iter() {
                            if terrain_pos.x == target.x && terrain_pos.y == target.y {
                                found_terrain = terrain.terrain_type;
                                break;
                            }
                        }
                        
                        match found_terrain {
                            TerrainType::Stone => {
                                inventory.stone += 3; // 石头地形产出更多石头
                            }
                            TerrainType::Mountain => {
                                inventory.stone += 2;
                                inventory.metal += 1; // 山脉可能有金属
                            }
                            _ => {
                                inventory.stone += 1; // 其他地形也能挖到一些石头
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

/// 时间系统
pub fn time_system(
    time: Res<Time>,
    mut game_time: ResMut<GameTime>,
) {
    // 全局时间缩放会自动影响 delta_secs()
    game_time.elapsed += time.delta_secs();
    
    if game_time.elapsed >= 10.0 { // 每10秒 = 1游戏小时
        game_time.elapsed = 0.0;
        game_time.hour += 1;
        
        if game_time.hour >= 24 {
            game_time.hour = 0;
            game_time.day += 1;
        }
    }
}

/// 时间控制系统 - 按键调节全局游戏速度
pub fn time_control_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut time: ResMut<Time<Virtual>>,
    mut game_time: ResMut<GameTime>,
) {
    use bevy::input::keyboard::KeyCode;
    
    let mut new_scale: Option<f32> = None;
    
    // 数字键 1-5 设置时间倍率
    if keyboard.just_pressed(KeyCode::Digit1) {
        new_scale = Some(0.0); // 暂停
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        new_scale = Some(0.5); // 半速
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        new_scale = Some(1.0); // 正常速度
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        new_scale = Some(2.0); // 2倍速
    }
    if keyboard.just_pressed(KeyCode::Digit5) {
        new_scale = Some(5.0); // 5倍速
    }
    
    // 空格键快速切换暂停/正常
    if keyboard.just_pressed(KeyCode::Space) {
        if game_time.time_scale > 0.0 {
            new_scale = Some(0.0); // 暂停
        } else {
            new_scale = Some(1.0); // 恢复正常
        }
    }
    
    // 应用新的时间缩放到全局时间和游戏时间
    if let Some(scale) = new_scale {
        time.set_relative_speed(scale);
        game_time.time_scale = scale;
    }
}

/// UI设置
pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 加载支持中文的字体
    let font = asset_server.load("fonts/SourceHanSansCN-Regular.otf");
    
    // 创建昼夜光照覆盖层 - 全屏半透明层
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0), // 初始透明
            custom_size: Some(Vec2::new(2000.0, 1200.0)), // 覆盖整个屏幕
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 50.0), // z=50,在地形之上,矮人之下
        DaylightOverlay,
    ));
    
    // 资源显示UI - 使用标记组件
    commands.spawn((
        Text::new("资源统计"),
        TextFont {
            font: font.clone(),
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.9, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(15.0),
            left: Val::Px(15.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ResourceDisplay,  // 标记组件
    ));
    
    // 游戏标题
    commands.spawn((
        Text::new("◆ 矮人要塞式游戏 ◆"),
        TextFont {
            font: font.clone(),
            font_size: 26.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.85, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(15.0),
            left: Val::Percent(50.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.85)),
        TitleDisplay,  // 标记组件
    ));
    
    // 帮助信息
    commands.spawn((
        Text::new("操作说明:\nWASD/方向键: 移动视角\n鼠标左键: 选择矮人\n鼠标右键: 指挥选中的矮人移动到目标位置\n黄色边框 = 选中的矮人\n\n时间控制:\n空格: 暂停/继续\n1: 暂停 | 2: 半速 | 3: 正常 | 4: 2倍速 | 5: 5倍速"),
        TextFont {
            font: font.clone(),
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.9, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(15.0),
            right: Val::Px(15.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        HelpDisplay,
    ));
}

/// UI更新系统
pub fn ui_update_system(
    inventory: Res<GlobalInventory>,
    game_time: Res<GameTime>,
    dwarves: Query<(&Dwarf, &WorkState)>,
    mut query: Query<&mut Text, With<ResourceDisplay>>,
) {
    // 统计矮人状态
    let mut idle_count = 0;
    let mut gathering_count = 0;
    let mut mining_count = 0;
    
    for (_dwarf, work_state) in dwarves.iter() {
        match &work_state.current_task {
            Some(Task::Idle) => idle_count += 1,
            Some(Task::Gathering(_)) => gathering_count += 1,
            Some(Task::Mining(_)) => mining_count += 1,
            _ => {}
        }
    }
    
    for mut text in query.iter_mut() {
        // 时间倍率显示
        let speed_text = if game_time.time_scale == 0.0 {
            "⏸暂停"
        } else if game_time.time_scale == 0.5 {
            "▶半速"
        } else if game_time.time_scale == 1.0 {
            "▶正常"
        } else if game_time.time_scale == 2.0 {
            "▶▶2倍速"
        } else if game_time.time_scale >= 5.0 {
            "▶▶▶5倍速"
        } else {
            &format!("▶{}x", game_time.time_scale)
        };
        
        **text = format!(
            "第{}天 {}时 {} | 石头: {} | 木材: {} | 食物: {} | 金属: {}\n矮人状态: 空闲{} 采集{} 挖矿{}",
            game_time.day,
            game_time.hour,
            speed_text,
            inventory.stone,
            inventory.wood,
            inventory.food,
            inventory.metal,
            idle_count,
            gathering_count,
            mining_count,
        );
    }
}

/// 输入处理系统
pub fn input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    use bevy::input::keyboard::KeyCode;
    
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };
    
    let speed = 5.0;
        
        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            camera_transform.translation.y += speed;
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            camera_transform.translation.y -= speed;
        }
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            camera_transform.translation.x -= speed;
        }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        camera_transform.translation.x += speed;
    }
}

/// 更新工作指示器
pub fn update_work_indicators(
    dwarves: Query<(&WorkState, &Children), With<Dwarf>>,
    mut indicators: Query<&mut Sprite, With<WorkIndicator>>,
) {
    for (work_state, children) in dwarves.iter() {
        for child in children.iter() {
            if let Ok(mut sprite) = indicators.get_mut(child) {
                // 根据任务类型和进度改变颜色和透明度
                sprite.color = match &work_state.current_task {
                    Some(Task::Idle) => Color::srgba(0.5, 0.5, 0.5, 0.6), // 灰色 = 空闲
                    Some(Task::Gathering(_)) => {
                        // 绿色，透明度随进度变化
                        let alpha = 0.5 + work_state.work_progress * 0.5;
                        Color::srgba(0.0, 1.0, 0.0, alpha)
                    }
                    Some(Task::Mining(_)) => {
                        // 橙色，透明度随进度变化
                        let alpha = 0.5 + work_state.work_progress * 0.5;
                        Color::srgba(1.0, 0.5, 0.0, alpha)
                    }
                    _ => Color::srgba(1.0, 1.0, 1.0, 0.6),
                };
            }
        }
    }
}

/// 鼠标选择矮人系统
pub fn mouse_selection_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    dwarves: Query<(Entity, &Transform), With<Dwarf>>,
    mut selected: ResMut<SelectedDwarf>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing_panel: Query<Entity, With<DwarfPanel>>,
) {
    // 只在左键点击时处理
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    
    let Ok(window) = windows.single() else {
        return;
    };
    
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    
    // 将屏幕坐标转换为世界坐标
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };
    
    // 查找最近的矮人
    let mut closest_dwarf: Option<Entity> = None;
    let mut closest_distance = f32::MAX;
    
    for (entity, transform) in dwarves.iter() {
        let distance = world_position.distance(transform.translation.truncate());
        if distance < TILE_SIZE && distance < closest_distance {
            closest_distance = distance;
            closest_dwarf = Some(entity);
        }
    }
    
    // 更新选中状态
    selected.entity = closest_dwarf;
    
    // 移除旧的面板
    for entity in existing_panel.iter() {
        commands.entity(entity).despawn();
    }
    
    // 如果选中了矮人,创建新面板
    if closest_dwarf.is_some() {
        let font = asset_server.load("fonts/SourceHanSansCN-Regular.otf");
        
        commands.spawn((
            Text::new("矮人信息"),
            TextFont {
                font: font.clone(),
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.9, 0.6)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(100.0),
                left: Val::Px(15.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.9)),
            DwarfPanel,
        ));
    }
}

/// 更新选择指示器
pub fn update_selection_indicator(
    selected: Res<SelectedDwarf>,
    dwarves: Query<(Entity, &Children), With<Dwarf>>,
    mut indicators: Query<&mut TextColor, With<SelectionIndicator>>,
) {
    for (entity, children) in dwarves.iter() {
        let is_selected = selected.entity == Some(entity);
        
        for child in children.iter() {
            if let Ok(mut text_color) = indicators.get_mut(child) {
                // 选中时显示黄色边框,未选中时透明
                if is_selected {
                    text_color.0 = Color::srgba(1.0, 1.0, 0.0, 0.9);
                } else {
                    text_color.0 = Color::srgba(1.0, 1.0, 0.0, 0.0);
                }
            }
        }
    }
}

/// 鼠标控制矮人系统
pub fn mouse_control_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    selected: Res<SelectedDwarf>,
    mut dwarves: Query<&mut WorkState, With<Dwarf>>,
) {
    // 只在右键点击且有选中矮人时处理
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }
    
    let Some(selected_entity) = selected.entity else {
        return;
    };
    
    let Ok(window) = windows.single() else {
        return;
    };
    
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    
    // 将屏幕坐标转换为世界坐标
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };
    
    // 转换为网格坐标
    let grid_x = ((world_position.x + (WORLD_WIDTH as f32 * TILE_SIZE / 2.0)) / TILE_SIZE) as i32;
    let grid_y = ((world_position.y + (WORLD_HEIGHT as f32 * TILE_SIZE / 2.0)) / TILE_SIZE) as i32;
    
    // 边界检查
    if grid_x < 0 || grid_x >= WORLD_WIDTH || grid_y < 0 || grid_y >= WORLD_HEIGHT {
        return;
    }
    
    // 给选中的矮人分配任务
    if let Ok(mut work_state) = dwarves.get_mut(selected_entity) {
        work_state.current_task = Some(Task::Gathering(GridPosition {
            x: grid_x,
            y: grid_y,
        }));
    }
}

/// 更新矮人面板信息
pub fn update_dwarf_panel(
    selected: Res<SelectedDwarf>,
    dwarves: Query<(&Dwarf, &WorkState, &GridPosition)>,
    mut panel_query: Query<&mut Text, With<DwarfPanel>>,
) {
    let Some(selected_entity) = selected.entity else {
        return;
    };
    
    let Ok((dwarf, work_state, pos)) = dwarves.get(selected_entity) else {
        return;
    };
    
    for mut text in panel_query.iter_mut() {
        let task_text = match &work_state.current_task {
            Some(Task::Idle) => "空闲".to_string(),
            Some(Task::Gathering(target)) => {
                let progress = (work_state.work_progress * 100.0) as i32;
                format!("采集 -> ({}, {}) [{}%]", target.x, target.y, progress)
            }
            Some(Task::Mining(target)) => {
                let progress = (work_state.work_progress * 100.0) as i32;
                format!("挖矿 -> ({}, {}) [{}%]", target.x, target.y, progress)
            }
            Some(Task::Building(target, _)) => format!("建造 -> ({}, {})", target.x, target.y),
            None => "无任务".to_string(),
        };
        
        **text = format!(
            "◆ {} ◆\n\n位置: ({}, {})\n健康: {:.0}%\n饥饿: {:.0}%\n快乐: {:.0}%\n\n当前任务:\n{}",
            dwarf.name,
            pos.x, pos.y,
            dwarf.health,
            dwarf.hunger,
            dwarf.happiness,
            task_text,
        );
    }
}

/// 水面波光动画
pub fn water_animation_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut WaterAnimation, &mut TextColor)>,
) {
    for (mut transform, mut water, mut color) in query.iter_mut() {
        water.phase += time.delta_secs() * 2.0;
        
        // 上下波动
        let wave = (water.phase.sin() * 2.0).round();
        transform.translation.y += wave * 0.1;
        
        // 颜色闪烁(模拟波光)
        let brightness = 0.5 + water.phase.sin() * 0.3;
        color.0 = Color::srgba(
            0.05 * brightness,
            0.15 * brightness,
            0.4,
            0.5,
        );
    }
}

/// 树木摇摆动画
pub fn tree_sway_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut TreeSway)>,
) {
    for (mut transform, mut sway) in query.iter_mut() {
        sway.offset += time.delta_secs() * 1.5;
        
        // 轻微的左右摇摆
        let sway_amount = (sway.offset.sin() * 1.5).round();
        transform.translation.x += sway_amount * 0.1;
    }
}

/// 改进的昼夜循环光照效果 - 只影响颜色叠加层
pub fn daylight_cycle_system(
    time_res: Res<GameTime>,
    mut overlay_query: Query<&mut Sprite, With<DaylightOverlay>>,
) {
    // 计算精确的时间（包含小数部分）
    let time_of_day = time_res.hour as f32 + (time_res.elapsed / 10.0);
    
    // 定义关键时间点
    const SUNRISE_START: f32 = 5.0;  // 日出开始
    const SUNRISE_END: f32 = 7.0;    // 日出结束
    const SUNSET_START: f32 = 17.0;  // 日落开始
    const SUNSET_END: f32 = 19.0;    // 日落结束
    
    // 不同时段的颜色和透明度
    let (color, alpha) = if time_of_day >= SUNRISE_END && time_of_day < SUNSET_START {
        // 白天 (7:00-17:00) - 无覆盖
        (Color::srgb(0.1, 0.15, 0.3), 0.0)
        
    } else if time_of_day >= SUNSET_END || time_of_day < SUNRISE_START {
        // 深夜 (19:00-5:00) - 深蓝色覆盖
        (Color::srgb(0.05, 0.1, 0.25), 0.6)
        
    } else if time_of_day >= SUNRISE_START && time_of_day < SUNRISE_END {
        // 日出过渡 (5:00-7:00) - 从深夜到白天
        let progress = (time_of_day - SUNRISE_START) / (SUNRISE_END - SUNRISE_START);
        let smooth_progress = smooth_step(progress); // 使用平滑插值
        
        // 从深蓝夜色过渡到温暖晨光
        let sunrise_color = Color::srgb(
            0.05 + smooth_progress * 0.15,  // 轻微橙色
            0.1 + smooth_progress * 0.1,
            0.25 - smooth_progress * 0.1,   // 减少蓝色
        );
        let alpha = 0.6 - smooth_progress * 0.6; // 从0.6渐变到0
        (sunrise_color, alpha)
        
    } else {
        // 日落过渡 (17:00-19:00) - 从白天到深夜
        let progress = (time_of_day - SUNSET_START) / (SUNSET_END - SUNSET_START);
        let smooth_progress = smooth_step(progress); // 使用平滑插值
        
        // 从温暖夕阳过渡到深蓝夜色
        let sunset_color = Color::srgb(
            0.2 - smooth_progress * 0.15,   // 渐少橙色
            0.15 - smooth_progress * 0.05,
            0.15 + smooth_progress * 0.1,   // 增加蓝色
        );
        let alpha = smooth_progress * 0.6; // 从0渐变到0.6
        (sunset_color, alpha)
    };
    
    for mut sprite in overlay_query.iter_mut() {
        sprite.color = color.with_alpha(alpha);
    }
}

// 平滑步进函数 - 提供更自然的过渡曲线
fn smooth_step(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t) // Hermite插值
}

/// 生成粒子效果 - 只在工作进行中生成
pub fn spawn_particle_system(
    mut commands: Commands,
    time: Res<Time>,
    dwarves: Query<(&Transform, &WorkState, &GridPosition), With<Dwarf>>,
) {
    // 如果时间暂停,不生成粒子
    if time.delta_secs() <= 0.0001 {
        return;
    }
    
    for (transform, work_state, pos) in dwarves.iter() {
        // 只在矮人到达目标位置并且正在工作时生成粒子
        let should_spawn = match &work_state.current_task {
            Some(Task::Mining(target)) => {
                pos.x == target.x && pos.y == target.y && work_state.work_progress > 0.0
            }
            Some(Task::Gathering(target)) => {
                pos.x == target.x && pos.y == target.y && work_state.work_progress > 0.0
            }
            _ => false,
        };
        
        if !should_spawn {
            continue;
        }
        
        // 降低粒子生成频率
        if rand::thread_rng().gen_ratio(1, 10) {
            continue;
        }
        
        match &work_state.current_task {
            Some(Task::Mining(_)) => {
                // 挖矿粉尘
                let angle = rand::random::<f32>() * std::f32::consts::PI * 2.0;
                let speed = rand::random::<f32>() * 20.0 + 10.0;
                
                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.6, 0.5, 0.4, 0.8),
                        custom_size: Some(Vec2::new(3.0, 3.0)),
                        ..default()
                    },
                    Transform::from_xyz(
                        transform.translation.x,
                        transform.translation.y,
                        3.0,
                    ),
                    Particle {
                        lifetime: 1.0,
                        velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                    },
                ));
            }
            Some(Task::Gathering(_)) => {
                // 采集特效
                let angle = rand::random::<f32>() * std::f32::consts::PI * 2.0;
                let speed = rand::random::<f32>() * 15.0 + 5.0;
                
                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.2, 0.8, 0.2, 0.9),
                        custom_size: Some(Vec2::new(4.0, 4.0)),
                        ..default()
                    },
                    Transform::from_xyz(
                        transform.translation.x,
                        transform.translation.y + 10.0,
                        3.0,
                    ),
                    Particle {
                        lifetime: 0.8,
                        velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed + 20.0),
                    },
                ));
            }
            _ => {}
        }
    }
}

/// 更新粒子
pub fn particle_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Particle, &mut Sprite)>,
) {
    for (entity, mut transform, mut particle, mut sprite) in query.iter_mut() {
        particle.lifetime -= time.delta_secs();
        
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        
        // 更新位置
        transform.translation.x += particle.velocity.x * time.delta_secs();
        transform.translation.y += particle.velocity.y * time.delta_secs();
        
        // 重力
        particle.velocity.y -= 50.0 * time.delta_secs();
        
        // 淡出
        sprite.color = sprite.color.with_alpha(particle.lifetime);
    }
}

/// 鼠标悬停显示矮人名字系统
pub fn dwarf_name_hover_system(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    dwarves: Query<(Entity, &Transform, &Dwarf), With<Dwarf>>,
    existing_tags: Query<Entity, With<DwarfNameTag>>,
    asset_server: Res<AssetServer>,
) {
    // 清除所有现有名字标签
    for entity in existing_tags.iter() {
        commands.entity(entity).despawn();
    }
    
    let Ok(window) = windows.single() else {
        return;
    };
    
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    
    // 将屏幕坐标转换为世界坐标
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };
    
    // 加载字体
    let font = asset_server.load("fonts/SourceHanSansCN-Regular.otf");
    
    // 检查鼠标附近的矮人（半径约50像素）
    const HOVER_RADIUS: f32 = 50.0;
    
    for (_entity, transform, dwarf) in dwarves.iter() {
        let distance = world_position.distance(transform.translation.truncate());
        
        if distance < HOVER_RADIUS {
            // 在矮人上方显示名字
            commands.spawn((
                Text2d::new(&dwarf.name),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.3)),
                Transform::from_xyz(
                    transform.translation.x,
                    transform.translation.y + 25.0,
                    100.0, // 确保在最上层
                ),
                DwarfNameTag,
            ));
        }
    }
}
