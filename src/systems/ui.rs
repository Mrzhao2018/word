use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;

/// UI设置
pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 加载支持中文、emoji和特殊符号的字体
    let font = asset_server.load("fonts/sarasa-gothic-sc-regular.ttf");

    // 创建昼夜光照覆盖层 - 全屏半透明层
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),      // 初始透明
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
        ResourceDisplay, // 标记组件
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
        TitleDisplay, // 标记组件
    ));

    // 帮助信息
    commands.spawn((
        Text::new("操作说明:\nWASD/方向键: 移动视角\n鼠标左键: 选择矮人\n鼠标右键: 指挥选中的矮人移动到目标位置\nM: 返回世界地图\n黄色边框 = 选中的矮人\n\n时间控制:\n空格: 暂停/继续\n1: 暂停 | 2: 半速 | 3: 正常 | 4: 2倍速 | 5: 5倍速"),
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
            dwarf.name, pos.x, pos.y, dwarf.health, dwarf.hunger, dwarf.happiness, task_text,
        );
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
