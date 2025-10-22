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
    
    // 资源显示UI
    commands.spawn((
        Text::new("资源统计"),
        TextFont {
            font: font.clone(),
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

/// UI更新系统
pub fn ui_update_system(
    inventory: Res<GlobalInventory>,
    game_time: Res<GameTime>,
    mut query: Query<&mut Text>,
) {
    for mut text in query.iter_mut() {
        **text = format!(
            "第{}天 {}时\n石头: {} | 木材: {} | 食物: {} | 金属: {}",
            game_time.day,
            game_time.hour,
            inventory.stone,
            inventory.wood,
            inventory.food,
            inventory.metal,
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
