/// UI交互系统 - 处理快捷键和面板操作

use crate::ui_framework::*;
use bevy::prelude::*;

/// 快捷键系统 - 处理F1-F12等快捷键
pub fn ui_hotkey_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut panel_query: Query<(&mut UIPanel, &mut Node)>,
) {
    // F1: 切换帮助面板
    if keyboard.just_pressed(KeyCode::F1) {
        toggle_panel_visibility(&mut panel_query, "help_info");
    }
    
    // F2: 切换资源面板（预留）
    if keyboard.just_pressed(KeyCode::F2) {
        toggle_panel_visibility(&mut panel_query, "resource_stats");
    }
    
    // F3: 切换小地图（预留）
    if keyboard.just_pressed(KeyCode::F3) {
        // 未来功能
    }
    
    // Tab: 切换所有面板显示/隐藏
    if keyboard.just_pressed(KeyCode::Tab) {
        for (mut panel, mut node) in panel_query.iter_mut() {
            // 不影响游戏标题
            if panel.id != "game_title" {
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
}

/// 面板拖拽系统（未来功能）
pub fn panel_drag_system() {
    // 预留：实现面板拖拽功能
}

/// 面板缩放系统（未来功能）
pub fn panel_resize_system() {
    // 预留：实现面板大小调整
}

/// 面板动画系统（未来功能）
pub fn panel_animation_system() {
    // 预留：实现面板淡入淡出等动画
}
