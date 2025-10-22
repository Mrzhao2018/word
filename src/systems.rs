use bevy::prelude::*;
use rand::Rng;
use crate::components::*;
use crate::resources::*;
use crate::world::*;

/// 矮人移动系统
pub fn dwarf_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut GridPosition, &Velocity), With<Dwarf>>,
) {
    for (mut transform, mut grid_pos, velocity) in query.iter_mut() {
        // 更新位置
        transform.translation.x += velocity.x * time.delta_secs() * 50.0;
        transform.translation.y += velocity.y * time.delta_secs() * 50.0;
        
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
    _time: Res<Time>,
    mut query: Query<(&mut WorkState, &GridPosition, &mut Velocity, &Dwarf)>,
    _terrain_query: Query<(&GridPosition, &Terrain)>,
) {
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
                    velocity.x = dx.signum() as f32 * 0.5;
                    velocity.y = dy.signum() as f32 * 0.5;
                } else {
                    // 到达目标,完成任务
                    velocity.x = 0.0;
                    velocity.y = 0.0;
                    work_state.current_task = Some(Task::Idle);
                }
            }
            _ => {}
        }
    }
}

/// 资源采集系统
pub fn resource_gathering_system(
    _commands: Commands,
    query: Query<(&WorkState, &GridPosition), With<Dwarf>>,
    mut inventory: ResMut<GlobalInventory>,
) {
    for (work_state, _pos) in query.iter() {
        if let Some(Task::Gathering(_)) = work_state.current_task {
            // 简化:直接增加资源
            if rand::thread_rng().gen_ratio(1, 60) {
                inventory.food += 1;
            }
        }
        
        if let Some(Task::Mining(_)) = work_state.current_task {
            if rand::thread_rng().gen_ratio(1, 120) {
                inventory.stone += 1;
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

/// UI设置
pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 加载支持中文的字体
    let font = asset_server.load("fonts/SourceHanSansCN-Regular.otf");
    
    // UI背景面板 - 左上角
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.7),
            custom_size: Some(Vec2::new(650.0, 60.0)),
            ..default()
        },
        Transform::from_xyz(-375.0, 360.0, 100.0),
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
        ResourceDisplay,  // 标记组件
    ));
    
    // 游戏标题背景
    commands.spawn((
        Sprite {
            color: Color::srgba(0.1, 0.1, 0.15, 0.85),
            custom_size: Some(Vec2::new(400.0, 55.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 360.0, 100.0),
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
        TitleDisplay,  // 标记组件
    ));
    
    // 帮助信息背景
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.7),
            custom_size: Some(Vec2::new(500.0, 90.0)),
            ..default()
        },
        Transform::from_xyz(350.0, -330.0, 100.0),
    ));
    
    // 帮助信息
    commands.spawn((
        Text::new("操作说明:\nWASD/方向键: 移动视角\n矮人会自动工作采集资源\n观察矮人移动并收集资源"),
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
        **text = format!(
            "第{}天 {}时 | 石头: {} | 木材: {} | 食物: {} | 金属: {}\n矮人状态: 空闲{} 采集{} 挖矿{}",
            game_time.day,
            game_time.hour,
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
                // 根据任务类型改变颜色
                sprite.color = match &work_state.current_task {
                    Some(Task::Idle) => Color::srgba(0.5, 0.5, 0.5, 0.6), // 灰色 = 空闲
                    Some(Task::Gathering(_)) => Color::srgba(0.0, 1.0, 0.0, 0.8), // 绿色 = 采集
                    Some(Task::Mining(_)) => Color::srgba(1.0, 0.5, 0.0, 0.8), // 橙色 = 挖矿
                    _ => Color::srgba(1.0, 1.0, 1.0, 0.6),
                };
            }
        }
    }
}
