use crate::resources::*;
use bevy::prelude::*;

/// 时间系统
pub fn time_system(time: Res<Time>, mut game_time: ResMut<GameTime>) {
    // 全局时间缩放会自动影响 delta_secs()
    game_time.elapsed += time.delta_secs();

    if game_time.elapsed >= 10.0 {
        // 每10秒 = 1游戏小时
        game_time.elapsed = 0.0;
        game_time.hour += 1;

        if game_time.hour >= 24 {
            game_time.hour = 0;
            game_time.day += 1;
        }
    }
}

/// 时间控制系统 - 按键调节全局游戏速度
pub fn time_control_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut time: ResMut<Time<Virtual>>,
    mut game_time: ResMut<GameTime>,
) {
    use bevy::input::keyboard::KeyCode;

    let mut new_scale: Option<f32> = None;

    // 数字键 1-5 设置时间倍率
    if keyboard.just_pressed(KeyCode::Digit1) {
        new_scale = Some(0.0); // 暂停
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        new_scale = Some(0.5); // 半速
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        new_scale = Some(1.0); // 正常速度
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        new_scale = Some(2.0); // 2倍速
    }
    if keyboard.just_pressed(KeyCode::Digit5) {
        new_scale = Some(5.0); // 5倍速
    }

    // 空格键快速切换暂停/正常
    if keyboard.just_pressed(KeyCode::Space) {
        if game_time.time_scale > 0.0 {
            new_scale = Some(0.0); // 暂停
        } else {
            new_scale = Some(1.0); // 恢复正常
        }
    }

    // 应用新的时间缩放到全局时间和游戏时间
    if let Some(scale) = new_scale {
        time.set_relative_speed(scale);
        game_time.time_scale = scale;
    }
}
