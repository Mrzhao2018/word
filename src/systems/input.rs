use crate::components::*;
use crate::resources::*;
use crate::world::*;
use bevy::prelude::*;

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

    // 限制相机移动范围（基于地图大小）
    let half_world_width = (WORLD_WIDTH as f32 * TILE_SIZE) / 2.0;
    let half_world_height = (WORLD_HEIGHT as f32 * TILE_SIZE) / 2.0;
    
    // 添加边界缓冲区，防止相机移出地图太远
    let margin = 200.0;
    camera_transform.translation.x = camera_transform.translation.x
        .clamp(-half_world_width - margin, half_world_width + margin);
    camera_transform.translation.y = camera_transform.translation.y
        .clamp(-half_world_height - margin, half_world_height + margin);
}

/// 相机缩放系统
pub fn camera_zoom_system(
    mut scroll_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    use bevy::input::mouse::MouseScrollUnit;
    
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    for event in scroll_events.read() {
        // 计算缩放增量
        let zoom_delta = match event.unit {
            MouseScrollUnit::Line => event.y * 0.1,  // 每行滚动缩放10%
            MouseScrollUnit::Pixel => event.y * 0.002, // 像素模式缩放更细腻
        };

        // 更新缩放比例（scale值越小，画面越大）
        // 限制缩放范围：0.3x (放大3.3倍) 到 2.5x (缩小到40%)
        let new_scale = (camera_transform.scale.x - zoom_delta).clamp(0.3, 2.5);
        camera_transform.scale = Vec3::splat(new_scale);
    }
}

/// 鼠标选择矮人系统
pub fn mouse_selection_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    dwarves: Query<(Entity, &Transform), With<Dwarf>>,
    mut selected: ResMut<SelectedDwarf>,
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

    // 更新选中状态（矮人详情面板会在 update_dwarf_panel 系统中自动显示/隐藏）
    selected.entity = closest_dwarf;
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
    terrain_query: Query<(&GridPosition, &Terrain)>,
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

    // 检查目标地形是否可行走
    let mut is_walkable = false;
    for (terrain_pos, terrain) in terrain_query.iter() {
        if terrain_pos.x == grid_x && terrain_pos.y == grid_y {
            is_walkable = terrain.walkable;
            break;
        }
    }

    // 只有当目标位置可行走时才分配任务
    if is_walkable {
        if let Ok(mut work_state) = dwarves.get_mut(selected_entity) {
            work_state.current_task = Some(Task::Gathering(GridPosition {
                x: grid_x,
                y: grid_y,
            }));
        }
    }
}

/// 矮人名字悬停系统
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
    let font = asset_server.load("fonts/sarasa-gothic-sc-regular.ttf");

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

/// 地形信息悬停系统 - 显示鼠标下方的地形信息
pub fn terrain_info_hover_system(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    terrain_query: Query<(&GridPosition, &Terrain)>,
    existing_labels: Query<Entity, With<TerrainInfoLabel>>,
    asset_server: Res<AssetServer>,
) {
    // 清除所有现有地形标签
    for entity in existing_labels.iter() {
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

    // 转换为网格坐标
    let grid_x = ((world_position.x + (WORLD_WIDTH as f32 * TILE_SIZE / 2.0)) / TILE_SIZE) as i32;
    let grid_y = ((world_position.y + (WORLD_HEIGHT as f32 * TILE_SIZE / 2.0)) / TILE_SIZE) as i32;

    // 边界检查
    if grid_x < 0 || grid_x >= WORLD_WIDTH || grid_y < 0 || grid_y >= WORLD_HEIGHT {
        return;
    }

    // 查找对应位置的地形
    for (terrain_pos, terrain) in terrain_query.iter() {
        if terrain_pos.x == grid_x && terrain_pos.y == grid_y {
            let font = asset_server.load("fonts/sarasa-gothic-sc-regular.ttf");

            // 构建地形信息文本
            let terrain_info = format!(
                "{}\n资源产出: {:.0}%\n丰富度: {:.1}x\n移动速度: {:.0}%",
                terrain.terrain_type.description(),
                terrain.terrain_type.resource_multiplier() * 100.0,
                terrain.resource_richness,
                terrain.terrain_type.movement_speed() * 100.0
            );

            // 在鼠标位置附近显示信息
            commands.spawn((
                Text::new(&terrain_info),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.9)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(cursor_position.y + 10.0),
                    left: Val::Px(cursor_position.x + 15.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.95)),
                TerrainInfoLabel,
            ));

            break;
        }
    }
}
