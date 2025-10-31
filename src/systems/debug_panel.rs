use crate::components::*;
use crate::debug_config::*;
use crate::resources::*;
use crate::ui_framework::*;
use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

/// 调试面板文本标记
#[derive(Component)]
pub struct DebugPanelText;

/// 设置调试面板
pub fn setup_debug_panel(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    theme: Res<UITheme>,
) {
    // 只在调试模式开启时创建面板
    if !DEBUG_ENABLED {
        return;
    }

    let font = asset_server.load("fonts/sarasa-gothic-sc-regular.ttf");
    let mut builder = PanelBuilder::new(commands.reborrow(), font.clone(), theme.clone());

    // 创建调试面板（左侧中间）
    let debug_config = PanelConfig {
        anchor: PanelAnchor::MiddleLeft,
        offset: Vec2::new(15.0, 0.0),
        min_width: 280.0,
        min_height: 200.0,
        background_color: Color::srgba(0.1, 0.05, 0.05, 0.85),
        border_color: Some(Color::srgba(0.8, 0.2, 0.2, 0.7)),
        padding: 12.0,
    };

    let debug_panel = builder.create_panel("debug_info", debug_config, DebugPanel);
    builder.add_title(debug_panel, "◆ 调试信息 ◆");

    // 添加调试文本
    builder.add_text(
        debug_panel,
        "调试信息加载中...",
        DebugPanelText,
    );
}

/// 更新调试面板信息
pub fn update_debug_panel(
    diagnostics: Res<DiagnosticsStore>,
    game_time: Res<GameTime>,
    inventory: Res<GlobalInventory>,
    dwarves: Query<(&Dwarf, &WorkState, &GridPosition)>,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut text_query: Query<&mut Text, With<DebugPanelText>>,
) {
    // 如果调试未开启，不更新
    if !DEBUG_ENABLED {
        return;
    }

    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    // 获取 FPS
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .unwrap_or(0.0);

    // 获取帧时间
    let frame_time = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|ft| ft.smoothed())
        .unwrap_or(0.0);

    // 统计矮人状态
    let total_dwarves = dwarves.iter().count();
    let mut idle_count = 0;
    let mut gathering_count = 0;
    let mut mining_count = 0;
    let mut wandering_count = 0;

    for (_dwarf, work_state, _pos) in dwarves.iter() {
        match &work_state.current_task {
            Some(Task::Idle) => idle_count += 1,
            Some(Task::Gathering(_)) => gathering_count += 1,
            Some(Task::Mining(_)) => mining_count += 1,
            Some(Task::Wandering(_)) => wandering_count += 1,
            _ => {}
        }
    }

    // 获取相机信息
    let camera_info = if let Ok(camera_transform) = camera_query.single() {
        format!(
            "相机: ({:.0}, {:.0})\n缩放: {:.2}x",
            camera_transform.translation.x,
            camera_transform.translation.y,
            camera_transform.scale.x
        )
    } else {
        "相机: N/A".to_string()
    };

    // 构建调试信息
    **text = format!(
        "━━━ 性能 ━━━\n\
        FPS: {:.1}\n\
        帧时间: {:.2}ms\n\
        \n\
        ━━━ 游戏状态 ━━━\n\
        时间: 第{}天 {}时\n\
        时间倍率: {:.1}x\n\
        \n\
        ━━━ 资源 ━━━\n\
        石头: {} | 木材: {}\n\
        食物: {} | 金属: {}\n\
        \n\
        ━━━ 矮人 ({}) ━━━\n\
        空闲: {} | 采集: {}\n\
        挖矿: {} | 闲逛: {}\n\
        \n\
        ━━━ 相机 ━━━\n\
        {}",
        fps,
        frame_time,
        game_time.day,
        game_time.hour,
        game_time.time_scale,
        inventory.stone,
        inventory.wood,
        inventory.food,
        inventory.metal,
        total_dwarves,
        idle_count,
        gathering_count,
        mining_count,
        wandering_count,
        camera_info
    );
}

/// 切换调试面板显示/隐藏（F3键）
pub fn toggle_debug_panel(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut panel_query: Query<(&mut UIPanel, &mut Node), With<DebugPanel>>,
) {
    if !DEBUG_ENABLED {
        return;
    }

    if keyboard.just_pressed(KeyCode::F3) {
        for (mut panel, mut node) in panel_query.iter_mut() {
            panel.state = match panel.state {
                PanelState::Visible => {
                    node.display = Display::None;
                    PanelState::Hidden
                }
                PanelState::Hidden => {
                    node.display = Display::Flex;
                    PanelState::Visible
                }
                PanelState::Minimized => {
                    node.display = Display::Flex;
                    PanelState::Visible
                }
            };
        }
    }
}
