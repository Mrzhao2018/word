use crate::components::*;
use crate::ui_framework::*;
use crate::world::*;
use bevy::prelude::*;

/// 小地图显示的文本标记
#[derive(Component)]
pub struct MinimapText;

/// 小地图内容容器标记
#[derive(Component)]
pub struct MinimapContent;

/// 小地图地形像素标记
#[derive(Component)]
pub struct MinimapTerrain;

/// 设置小地图UI
pub fn setup_minimap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    theme: Res<UITheme>,
    terrain_query: Query<(&GridPosition, &Terrain)>,
) {
    let font = asset_server.load("fonts/sarasa-gothic-sc-regular.ttf");
    let mut builder = PanelBuilder::new(commands.reborrow(), font.clone(), theme.clone());

    // 创建小地图面板（右上角）
    let minimap_config = PanelConfig {
        anchor: PanelAnchor::TopRight,
        offset: Vec2::new(15.0, 100.0),
        min_width: 200.0,
        min_height: 180.0,
        background_color: Color::srgba(0.05, 0.05, 0.1, 0.9),
        border_color: Some(Color::srgba(0.5, 0.5, 0.6, 0.7)),
        padding: 10.0,
    };

    let minimap_panel = builder.create_panel("minimap", minimap_config, MinimapPanel);
    builder.add_title(minimap_panel, "◆ 小地图 ◆");

    // 添加小地图内容容器
    let minimap_content = commands
        .spawn((
            Node {
                width: Val::Px(160.0),
                height: Val::Px(100.0), // 保持地图比例 80:50 = 160:100
                margin: UiRect::top(Val::Px(5.0)),
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
            MinimapContent,
        ))
        .id();

    commands.entity(minimap_panel).add_child(minimap_content);

    // 绘制地形（每个方块2x2像素）
    let pixel_width = 160.0 / WORLD_WIDTH as f32;
    let pixel_height = 100.0 / WORLD_HEIGHT as f32;

    for (pos, terrain) in terrain_query.iter() {
        let color = match terrain.terrain_type {
            TerrainType::Grass => Color::srgb(0.3, 0.6, 0.3),
            TerrainType::Tree => Color::srgb(0.2, 0.5, 0.2),
            TerrainType::Stone => Color::srgb(0.5, 0.5, 0.5),
            TerrainType::Water => Color::srgb(0.2, 0.4, 0.8),
            TerrainType::Mountain => Color::srgb(0.4, 0.4, 0.4),
        };

        let terrain_pixel = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(pixel_width),
                    height: Val::Px(pixel_height),
                    left: Val::Px(pos.x as f32 * pixel_width),
                    top: Val::Px(pos.y as f32 * pixel_height),
                    ..default()
                },
                BackgroundColor(color),
                MinimapTerrain,
            ))
            .id();

        commands.entity(minimap_content).add_child(terrain_pixel);
    }

    // 创建视口指示器（表示当前相机位置）
    let viewport_indicator = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(30.0),
                height: Val::Px(30.0),
                border: UiRect::all(Val::Px(2.0)),
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            BorderColor::all(Color::srgb(1.0, 1.0, 0.0)),
            BackgroundColor(Color::NONE),
            MinimapViewport,
        ))
        .id();

    commands.entity(minimap_content).add_child(viewport_indicator);

    // 添加矮人数量显示
    let dwarf_count_text = commands
        .spawn((
            Text::new("矮人: 0"),
            TextFont {
                font: font.clone(),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Node {
                margin: UiRect::top(Val::Px(5.0)),
                ..default()
            },
            MinimapText,
        ))
        .id();

    commands.entity(minimap_panel).add_child(dwarf_count_text);
}

/// 更新小地图视口指示器
pub fn update_minimap_viewport(
    camera_query: Query<&Transform, With<Camera2d>>,
    mut viewport_query: Query<&mut Node, With<MinimapViewport>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let Ok(mut viewport_node) = viewport_query.single_mut() else {
        return;
    };

    // 小地图尺寸
    let minimap_width = 160.0;
    let minimap_height = 100.0;
    
    let world_width = WORLD_WIDTH as f32 * TILE_SIZE;
    let world_height = WORLD_HEIGHT as f32 * TILE_SIZE;

    // 计算视口框大小（基于相机缩放）
    // camera.scale 越小（放大），视口框越小；scale 越大（缩小），视口框越大
    let camera_scale = camera_transform.scale.x;
    
    // 假设屏幕尺寸约为 1280x720，计算可见的世界范围
    let screen_width = 1280.0;
    let screen_height = 720.0;
    let visible_world_width = screen_width * camera_scale;
    let visible_world_height = screen_height * camera_scale;
    
    // 将可见世界范围映射到小地图上
    let viewport_width = (visible_world_width / world_width * minimap_width).clamp(10.0, minimap_width);
    let viewport_height = (visible_world_height / world_height * minimap_height).clamp(10.0, minimap_height);

    // 将世界坐标转换为小地图坐标（0-1范围）
    let normalized_x = (camera_transform.translation.x + world_width / 2.0) / world_width;
    let normalized_y = 1.0 - (camera_transform.translation.y + world_height / 2.0) / world_height;

    // 转换为小地图像素坐标（视口框居中在相机位置）
    let minimap_x = (normalized_x * minimap_width - viewport_width / 2.0)
        .clamp(0.0, minimap_width - viewport_width);
    let minimap_y = (normalized_y * minimap_height - viewport_height / 2.0)
        .clamp(0.0, minimap_height - viewport_height);

    // 更新视口框的位置和大小
    viewport_node.left = Val::Px(minimap_x);
    viewport_node.top = Val::Px(minimap_y);
    viewport_node.width = Val::Px(viewport_width);
    viewport_node.height = Val::Px(viewport_height);
}

/// 更新小地图文本信息
pub fn update_minimap_text(
    dwarves: Query<&Dwarf>,
    mut text_query: Query<&mut Text, With<MinimapText>>,
) {
    let dwarf_count = dwarves.iter().count();

    for mut text in text_query.iter_mut() {
        **text = format!("矮人: {}", dwarf_count);
    }
}

/// 更新小地图上的矮人位置标记
pub fn update_minimap_dwarves(
    mut commands: Commands,
    dwarves: Query<&GridPosition, With<Dwarf>>,
    minimap_content: Query<Entity, With<MinimapContent>>,
    existing_markers: Query<Entity, With<MinimapDwarfMarker>>,
) {
    let Ok(content_entity) = minimap_content.single() else {
        return;
    };

    // 清除旧的矮人标记
    for marker in existing_markers.iter() {
        commands.entity(marker).despawn();
    }

    // 小地图上每个格子的像素大小
    let pixel_width = 160.0 / WORLD_WIDTH as f32;
    let pixel_height = 100.0 / WORLD_HEIGHT as f32;

    // 为每个矮人创建标记
    for pos in dwarves.iter() {
        let dwarf_marker = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(pixel_width * 1.5),  // 稍大一点方便看见
                    height: Val::Px(pixel_height * 1.5),
                    left: Val::Px(pos.x as f32 * pixel_width),
                    top: Val::Px(pos.y as f32 * pixel_height),
                    ..default()
                },
                BackgroundColor(Color::srgb(1.0, 0.6, 0.2)), // 橙色标记矮人
                MinimapDwarfMarker,
            ))
            .id();

        commands.entity(content_entity).add_child(dwarf_marker);
    }
}
