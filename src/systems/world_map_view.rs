use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::resources::{ActiveLocalMap, GameState, WorldSeed};
use crate::world_map_data::{AtlasSelection, WorldAtlas, WorldCell, WORLD_ATLAS_TILE_SIZE, WORLD_ATLAS_DEFAULT_WIDTH, WORLD_ATLAS_DEFAULT_HEIGHT};
use crate::{debug_world_input, debug_world_selection};

/// 初始化世界地图（使用世界种子）
pub fn init_world_atlas(
    mut commands: Commands,
    world_seed: Res<WorldSeed>,
) {
    let atlas = WorldAtlas::generate(
        world_seed.seed as u64,
        WORLD_ATLAS_DEFAULT_WIDTH,
        WORLD_ATLAS_DEFAULT_HEIGHT,
    );
    commands.insert_resource(atlas);
    info!("初始化世界地图，种子: {}", world_seed.seed);
}

/// 世界地图场景根节点
#[derive(Component)]
pub struct AtlasViewRoot;

/// 宏观世界格子渲染实体
#[derive(Component)]
pub struct AtlasTileVisual {
    #[allow(dead_code)]
    pub coord: IVec2,
}

/// 选中高亮实体
#[derive(Component)]
pub struct AtlasSelectionHighlight;

/// 悬停高亮实体
#[derive(Component)]
pub struct AtlasHoverHighlight;

/// 信息文本实体
#[derive(Component)]
pub struct AtlasInfoText;

/// 操作提示文本实体
#[derive(Component)]
pub struct AtlasInstructionText;

/// 进入大地图视图前的准备逻辑
pub fn prepare_world_atlas(
    world_atlas: Res<WorldAtlas>,
    mut selection: ResMut<AtlasSelection>,
    mut active_local: ResMut<ActiveLocalMap>,
) {
    if let Some(coord) = active_local.coord {
        let clamped = clamp_coord(coord, world_atlas.width, world_atlas.height);
        if selection.selected != Some(clamped) {
            selection.selected = Some(clamped);
        }
        selection.hovered = Some(clamped);
        active_local.coord = Some(clamped);
        return;
    }

    if selection.selected.is_none() {
        let center = IVec2::new(world_atlas.width / 2, world_atlas.height / 2);
        selection.selected = Some(center);
        selection.hovered = Some(center);
        active_local.coord = Some(center);
    }
}

/// 构建宏观世界地图的可视化实体
#[allow(clippy::too_many_arguments)]
pub fn setup_world_atlas_scene(
    mut commands: Commands,
    world_atlas: Res<WorldAtlas>,
    selection: Res<AtlasSelection>,
    asset_server: Res<AssetServer>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        camera_transform.translation.x = 0.0;
        camera_transform.translation.y = 0.0;
        camera_transform.scale = Vec3::ONE;
    }

    let font = asset_server.load("fonts/sarasa-gothic-sc-regular.ttf");

    let root = commands
        .spawn((
            Transform::default(),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            AtlasViewRoot,
        ))
        .id();

    let selected_cell = selection
        .selected
        .and_then(|coord| world_atlas.cell_at(coord));
    let preview_cell = selection
        .hovered
        .and_then(|coord| world_atlas.cell_at(coord));

    commands.entity(root).with_children(|parent| {
        // 绘制所有地块
        for cell in &world_atlas.cells {
            parent.spawn((
                Sprite {
                    color: cell.color(),
                    custom_size: Some(Vec2::splat(WORLD_ATLAS_TILE_SIZE - 2.0)),
                    ..default()
                },
                Transform::from_translation(tile_to_world(
                    cell.coord,
                    world_atlas.width,
                    world_atlas.height,
                )),
                Visibility::default(),
                InheritedVisibility::default(),
                AtlasTileVisual { coord: cell.coord },
            ));
        }

        // 选中高亮
        let highlight_position = selected_cell
            .map(|cell| tile_to_world(cell.coord, world_atlas.width, world_atlas.height))
            .unwrap_or(Vec3::new(0.0, 0.0, -10.0));

        parent.spawn((
            Sprite {
                color: Color::srgba(1.0, 0.9, 0.3, 0.45),
                custom_size: Some(Vec2::splat(WORLD_ATLAS_TILE_SIZE + 6.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(
                highlight_position.x,
                highlight_position.y,
                10.0,
            )),
            Visibility::default(),
            InheritedVisibility::default(),
            AtlasSelectionHighlight,
        ));

        // 悬停高亮
        parent.spawn((
            Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 0.25),
                custom_size: Some(Vec2::splat(WORLD_ATLAS_TILE_SIZE + 2.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 9.0)),
            Visibility::Hidden,
            InheritedVisibility::default(),
            AtlasHoverHighlight,
        ));

        // 选中信息文本
        parent.spawn((
            Text2d::new(build_tile_info(preview_cell, selected_cell)),
            TextFont {
                font: font.clone(),
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 0.9)),
            Transform::from_xyz(
                0.0,
                -((world_atlas.height as f32 * WORLD_ATLAS_TILE_SIZE) / 2.0) - 60.0,
                20.0,
            ),
            Visibility::default(),
            InheritedVisibility::default(),
            AtlasInfoText,
        ));

        // 操作提示
        parent.spawn((
            Text2d::new("操作: 鼠标左键选择 | Enter进入局部地图 | Esc返回主菜单"),
            TextFont {
                font,
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.85, 0.9, 1.0)),
            Transform::from_xyz(
                0.0,
                ((world_atlas.height as f32 * WORLD_ATLAS_TILE_SIZE) / 2.0) + 40.0,
                20.0,
            ),
            Visibility::default(),
            InheritedVisibility::default(),
            AtlasInstructionText,
        ));
    });
}

/// 清理宏观世界地图场景
pub fn cleanup_world_atlas_scene(
    mut commands: Commands,
    roots: Query<Entity, With<AtlasViewRoot>>,
) {
    for entity in roots.iter() {
        if let Ok(mut entity_cmd) = commands.get_entity(entity) {
            entity_cmd.despawn();
        }
    }
}

/// 宏观世界地图输入与状态变更
#[allow(clippy::too_many_arguments)]
pub fn world_atlas_input_system(
    mut selection: ResMut<AtlasSelection>,
    mut active_local: ResMut<ActiveLocalMap>,
    world_atlas: Res<WorldAtlas>,
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut world_seed: ResMut<WorldSeed>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let hovered_coord = windows
        .iter()
        .next()
        .and_then(|window| window.cursor_position())
        .and_then(|cursor| {
            camera_query
                .iter()
                .next()
                .and_then(|(camera, camera_transform)| {
                    camera.viewport_to_world_2d(camera_transform, cursor).ok()
                })
        })
        .and_then(|world_pos| world_to_tile(world_pos, world_atlas.width, world_atlas.height));

    if selection.hovered != hovered_coord {
        debug_world_input!("input_system 更新悬停: {:?} -> {:?}", selection.hovered, hovered_coord);
    }
    
    selection.hovered = hovered_coord;

    if buttons.just_pressed(MouseButton::Left) {
        if let Some(coord) = hovered_coord {
            if selection.selected != Some(coord) {
                selection.selected = Some(coord);
            }
            active_local.coord = Some(coord);
        }
    }

    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter) {
        if let Some(coord) = selection.selected {
            if let Some(cell) = world_atlas.cell_at(coord) {
                world_seed.seed = cell.local_seed;
                active_local.coord = Some(coord);
                next_state.set(GameState::LocalView);
            }
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
    }
}

/// 宏观世界地图的高亮和文本更新
pub fn world_atlas_selection_system(
    world_atlas: Res<WorldAtlas>,
    selection: Res<AtlasSelection>,
    mut last_state: Local<(Option<IVec2>, Option<IVec2>)>,
    mut highlight_query: Query<
        &mut Transform,
        (With<AtlasSelectionHighlight>, Without<AtlasHoverHighlight>),
    >,
    mut hover_highlight_query: Query<
        (&mut Transform, &mut Visibility),
        (With<AtlasHoverHighlight>, Without<AtlasSelectionHighlight>),
    >,
    mut info_text_query: Query<&mut Text2d, With<AtlasInfoText>>,
) {
    // 强制每帧检查，即使没有变更检测
    let current_state = (selection.selected, selection.hovered);
    let state_changed = *last_state != current_state;
    
    if state_changed {
        debug_world_selection!(
            "状态变化 - 悬停: {:?}, 选中: {:?}",
            selection.hovered, selection.selected
        );
    }
    
    *last_state = current_state;

    let selected_cell = selection
        .selected
        .and_then(|coord| world_atlas.cell_at(coord));
    let hovered_cell = selection
        .hovered
        .and_then(|coord| world_atlas.cell_at(coord));

    // 每帧强制更新选中高亮位置
    if let Ok(mut transform) = highlight_query.single_mut() {
        if let Some(cell) = selected_cell {
            let mut position = tile_to_world(cell.coord, world_atlas.width, world_atlas.height);
            position.z = 10.0;
            transform.translation = position;
        } else {
            transform.translation = Vec3::new(0.0, 0.0, -10.0);
        }
    }

    // 每帧强制更新悬停高亮位置
    if let Ok((mut transform, mut visibility)) = hover_highlight_query.single_mut() {
        if let Some(cell) = hovered_cell {
            let mut position = tile_to_world(cell.coord, world_atlas.width, world_atlas.height);
            position.z = 9.0;
            
            if state_changed {
                debug_world_selection!(
                    "更新悬停高亮 - 坐标: {:?}, 世界位置: {:?}",
                    cell.coord, position
                );
            }
            
            transform.translation = position;
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }

    // 每帧强制更新信息文本
    if let Ok(mut text) = info_text_query.single_mut() {
        **text = build_tile_info(hovered_cell, selected_cell);
    }
}

/// 局部地图中的世界地图切换输入
pub fn local_view_return_to_world_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        next_state.set(GameState::WorldView);
    }
}

fn clamp_coord(coord: IVec2, width: i32, height: i32) -> IVec2 {
    IVec2::new(coord.x.clamp(0, width - 1), coord.y.clamp(0, height - 1))
}

fn tile_to_world(coord: IVec2, width: i32, height: i32) -> Vec3 {
    let half_width = width as f32 * WORLD_ATLAS_TILE_SIZE / 2.0;
    let half_height = height as f32 * WORLD_ATLAS_TILE_SIZE / 2.0;

    Vec3::new(
        coord.x as f32 * WORLD_ATLAS_TILE_SIZE - half_width + WORLD_ATLAS_TILE_SIZE / 2.0,
        coord.y as f32 * WORLD_ATLAS_TILE_SIZE - half_height + WORLD_ATLAS_TILE_SIZE / 2.0,
        0.0,
    )
}

fn world_to_tile(world_pos: Vec2, width: i32, height: i32) -> Option<IVec2> {
    let half_width = width as f32 * WORLD_ATLAS_TILE_SIZE / 2.0;
    let half_height = height as f32 * WORLD_ATLAS_TILE_SIZE / 2.0;

    let x = ((world_pos.x + half_width) / WORLD_ATLAS_TILE_SIZE).floor() as i32;
    let y = ((world_pos.y + half_height) / WORLD_ATLAS_TILE_SIZE).floor() as i32;

    if x < 0 || x >= width || y < 0 || y >= height {
        None
    } else {
        Some(IVec2::new(x, y))
    }
}

fn build_tile_info(hover: Option<&WorldCell>, selected: Option<&WorldCell>) -> String {
    match (hover, selected) {
        (Some(hover), Some(selected)) => {
            let suffix = if hover.coord == selected.coord {
                " (同悬停)"
            } else {
                ""
            };
            format!(
                "悬停: {}\n选中: {}{}",
                format_cell_line_core(hover),
                format_cell_line_core(selected),
                suffix
            )
        }
        (Some(hover), None) => format!("悬停: {}", format_cell_line_core(hover)),
        (None, Some(selected)) => format!("选中: {}", format_cell_line_core(selected)),
        (None, None) => "未选择地块".to_string(),
    }
}

fn format_cell_line_core(cell: &WorldCell) -> String {
    format!(
        "({},{}) 群落:{} 海拔:{:.2} 湿度:{:.2} 温度:{:.2} 局部种子:{}",
        cell.coord.x,
        cell.coord.y,
        cell.label(),
        cell.elevation,
        cell.moisture,
        cell.temperature,
        cell.local_seed
    )
}
