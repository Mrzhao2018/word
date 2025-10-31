use crate::components::*;
use crate::resources::*;
use crate::ui_framework::*;
use bevy::prelude::*;

/// UI设置 - 使用新的UI框架
pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 加载字体和主题
    let font = asset_server.load("fonts/sarasa-gothic-sc-regular.ttf");
    let theme = UITheme::default();
    
    // 初始化主题资源
    commands.insert_resource(theme.clone());

    // 创建昼夜光照覆盖层
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(2000.0, 1200.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 50.0),
        DaylightOverlay,
    ));

    let mut builder = PanelBuilder::new(commands.reborrow(), font.clone(), theme.clone());

    // 1. 资源统计面板（左上角）
    let resource_config = PanelConfig {
        anchor: PanelAnchor::TopLeft,
        offset: Vec2::new(15.0, 15.0),
        min_width: 400.0,
        min_height: 80.0,
        background_color: theme.background_dark,
        border_color: Some(theme.border_color),
        padding: theme.padding_medium,
    };
    let resource_panel = builder.create_panel("resource_stats", resource_config, ResourcePanel);
    builder.add_text(resource_panel, "资源统计...", ResourceDisplay);

    // 2. 游戏标题面板（顶部居中）
    let title_config = PanelConfig {
        anchor: PanelAnchor::TopCenter,
        offset: Vec2::new(0.0, 15.0),
        min_width: 300.0,
        min_height: 50.0,
        background_color: theme.background_light,
        border_color: Some(theme.accent_color),
        padding: theme.padding_small,
    };
    let title_panel = builder.create_panel("game_title", title_config, TitlePanel);
    builder.add_title(title_panel, "◆ 矮人要塞式游戏 ◆");

    // 3. 帮助信息面板（右下角）
    let help_config = PanelConfig {
        anchor: PanelAnchor::BottomRight,
        offset: Vec2::new(15.0, 15.0),
        min_width: 350.0,
        min_height: 200.0,
        background_color: theme.background_dark,
        border_color: Some(theme.border_color),
        padding: theme.padding_medium,
    };
    let help_panel = builder.create_panel("help_info", help_config, HelpPanel);
    builder.add_text(
        help_panel,
        "操作说明:\nWASD/方向键: 移动视角\n鼠标滚轮: 缩放视角\n鼠标左键: 选择矮人\n鼠标右键: 指挥矮人移动\nM: 返回世界地图\n黄色边框 = 选中的矮人\n\n时间控制:\n空格: 暂停/继续\n1: 暂停 | 2: 半速 | 3: 正常\n4: 2倍速 | 5: 5倍速\n\nF1: 切换帮助显示\nF2: 切换调试模式 | F4: 消息面板 | F5: 清除日志\nF3: 切换调试面板",
        HelpDisplay,
    );

    // 4. 矮人详情面板（左下角，初始隐藏）
    let dwarf_detail_config = PanelConfig {
        anchor: PanelAnchor::BottomLeft,
        offset: Vec2::new(15.0, 15.0),
        min_width: 320.0,
        min_height: 280.0,
        background_color: Color::srgba(0.08, 0.08, 0.15, 0.92),
        border_color: Some(Color::srgba(0.8, 0.7, 0.3, 0.8)),
        padding: theme.padding_large,
    };
    let dwarf_detail_panel = builder.create_hidden_panel(
        "dwarf_detail",
        dwarf_detail_config,
        DwarfDetailPanel,
    );
    builder.add_title(dwarf_detail_panel, "◆ 矮人详情 ◆");
    builder.add_text(
        dwarf_detail_panel,
        "选择一个矮人查看详情",
        DwarfPanel,
    );
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

/// 更新矮人详情面板
pub fn update_dwarf_panel(
    selected: Res<SelectedDwarf>,
    dwarves: Query<(&Dwarf, &WorkState, &GridPosition)>,
    mut text_query: Query<&mut Text, With<DwarfPanel>>,
    mut panel_query: Query<(&mut UIPanel, &mut Node), With<DwarfDetailPanel>>,
) {
    // 如果没有选中矮人，隐藏面板
    let Some(selected_entity) = selected.entity else {
        for (mut panel, mut node) in panel_query.iter_mut() {
            if panel.state != PanelState::Hidden {
                node.display = Display::None;
                panel.state = PanelState::Hidden;
            }
        }
        return;
    };

    // 如果无法获取矮人数据，隐藏面板
    let Ok((dwarf, work_state, pos)) = dwarves.get(selected_entity) else {
        for (mut panel, mut node) in panel_query.iter_mut() {
            if panel.state != PanelState::Hidden {
                node.display = Display::None;
                panel.state = PanelState::Hidden;
            }
        }
        return;
    };

    // 显示面板
    for (mut panel, mut node) in panel_query.iter_mut() {
        if panel.state == PanelState::Hidden {
            node.display = Display::Flex;
            panel.state = PanelState::Visible;
        }
    }

    // 更新面板内容
    for mut text in text_query.iter_mut() {
        // 构建任务信息
        let (task_name, task_detail) = match &work_state.current_task {
            Some(Task::Idle) => ("空闲", "正在休息".to_string()),
            Some(Task::Wandering(target)) => (
                "闲逛",
                format!("目标位置: ({}, {})", target.x, target.y),
            ),
            Some(Task::Gathering(target)) => {
                let progress = (work_state.work_progress * 100.0) as i32;
                (
                    "采集资源",
                    format!("位置: ({}, {})\n进度: {}%", target.x, target.y, progress),
                )
            }
            Some(Task::Mining(target)) => {
                let progress = (work_state.work_progress * 100.0) as i32;
                (
                    "挖矿采石",
                    format!("位置: ({}, {})\n进度: {}%", target.x, target.y, progress),
                )
            }
            Some(Task::Building(target, building_type)) => (
                "建造建筑",
                format!(
                    "位置: ({}, {})\n类型: {:?}",
                    target.x, target.y, building_type
                ),
            ),
            None => ("无任务", "等待指令".to_string()),
        };

        // 计算健康状态
        let health_status = if dwarf.health >= 80.0 {
            "健康"
        } else if dwarf.health >= 50.0 {
            "受伤"
        } else {
            "危险"
        };

        // 计算饥饿状态
        let hunger_status = if dwarf.hunger < 30.0 {
            "饱腹"
        } else if dwarf.hunger < 70.0 {
            "正常"
        } else {
            "饥饿"
        };

        // 计算快乐状态
        let happiness_status = if dwarf.happiness >= 75.0 {
            "愉快"
        } else if dwarf.happiness >= 50.0 {
            "一般"
        } else if dwarf.happiness >= 25.0 {
            "沮丧"
        } else {
            "痛苦"
        };

        **text = format!(
            "姓名: {}\n位置: ({}, {})\n\n━━━ 状态 ━━━\n健康: {:.0}% ({})\n饥饿: {:.0}% ({})\n快乐: {:.0}% ({})\n\n━━━ 任务 ━━━\n{}\n{}",
            dwarf.name,
            pos.x,
            pos.y,
            dwarf.health,
            health_status,
            dwarf.hunger,
            hunger_status,
            dwarf.happiness,
            happiness_status,
            task_name,
            task_detail,
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
                    Some(Task::Wandering(_)) => Color::srgba(0.7, 0.7, 1.0, 0.5), // 淡蓝色 = 闲逛
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
