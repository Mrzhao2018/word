use crate::logger::*;
use crate::ui_framework::*;
use bevy::prelude::*;

/// 通知消息文本标记
#[derive(Component)]
pub struct NotificationText;

/// 设置通知消息面板
pub fn setup_notification_panel(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    theme: Res<UITheme>,
) {
    let font = asset_server.load("fonts/sarasa-gothic-sc-regular.ttf");
    let mut builder = PanelBuilder::new(commands.reborrow(), font.clone(), theme.clone());

    // 创建通知消息面板（底部中间）
    let notification_config = PanelConfig {
        anchor: PanelAnchor::BottomCenter,
        offset: Vec2::new(0.0, 15.0),
        min_width: 600.0,
        min_height: 150.0,
        background_color: Color::srgba(0.05, 0.05, 0.1, 0.9),
        border_color: Some(Color::srgba(0.4, 0.6, 0.8, 0.7)),
        padding: 12.0,
    };

    let notification_panel = builder.create_hidden_panel(
        "notifications",
        notification_config,
        NotificationPanel,
    );
    
    builder.add_title(notification_panel, "◆ 消息通知 ◆");

    // 添加消息文本容器
    let text_entity = commands
        .spawn((
            Text::new("等待消息..."),
            TextFont {
                font: font.clone(),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Node {
                max_height: Val::Px(100.0),
                overflow: Overflow::clip(),
                ..default()
            },
            NotificationText,
        ))
        .id();

    commands.entity(notification_panel).add_child(text_entity);
}

/// 更新通知消息面板
pub fn update_notification_panel(
    logger: Res<GameLogger>,
    mut text_query: Query<&mut Text, With<NotificationText>>,
) {
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    if logger.messages.is_empty() {
        **text = "无消息".to_string();
        return;
    }

    // 显示最近的10条消息
    let recent_messages: Vec<_> = logger.messages.iter().rev().take(10).collect();
    
    let mut display_text = String::new();
    for (i, msg) in recent_messages.iter().rev().enumerate() {
        if i > 0 {
            display_text.push('\n');
        }
        display_text.push_str(&format!(
            "[{}] {} {}",
            msg.timestamp,
            msg.level.as_str(),
            msg.message
        ));
    }

    **text = display_text;
}

/// 切换通知面板显示/隐藏（F4键）
pub fn toggle_notification_panel(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut panel_query: Query<(&mut UIPanel, &mut Node), With<NotificationPanel>>,
) {
    if keyboard.just_pressed(KeyCode::F4) {
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

/// 游戏内调试控制系统
pub fn debug_control_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut logger: ResMut<GameLogger>,
) {
    // F2 切换调试模式
    if keyboard.just_pressed(KeyCode::F2) {
        logger.toggle_debug();
    }

    // F5 清空日志
    if keyboard.just_pressed(KeyCode::F5) {
        logger.messages.clear();
        logger.clear_log_file();
        logger.info("日志已清空".to_string());
    }
}
