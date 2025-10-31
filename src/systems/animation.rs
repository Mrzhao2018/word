use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;
use rand::Rng;

/// 水面波光动画
pub fn water_animation_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut WaterAnimation, &mut TextColor)>,
) {
    for (mut transform, mut water, mut color) in query.iter_mut() {
        water.phase += time.delta_secs() * 2.0;

        // 上下波动 - 使用基准位置
        let wave = water.phase.sin() * 0.8;
        transform.translation.y = water.base_y + wave;

        // 颜色闪烁(模拟波光)
        let brightness = 0.5 + water.phase.sin() * 0.3;
        color.0 = Color::srgba(0.05 * brightness, 0.15 * brightness, 0.4, 0.5);
    }
}

/// 树木摇摆动画
pub fn tree_sway_system(time: Res<Time>, mut query: Query<(&mut Transform, &mut TreeSway)>) {
    for (mut transform, mut sway) in query.iter_mut() {
        sway.offset += time.delta_secs() * 1.5;

        // 轻微的左右摇摆 - 使用基准位置
        let sway_amount = sway.offset.sin() * 0.6;
        transform.translation.x = sway.base_x + sway_amount;
    }
}

/// 改进的昼夜循环光照效果 - 只影响颜色叠加层
pub fn daylight_cycle_system(
    time_res: Res<GameTime>,
    mut overlay_query: Query<&mut Sprite, With<DaylightOverlay>>,
) {
    // 计算精确的时间（包含小数部分）
    let time_of_day = time_res.hour as f32 + (time_res.elapsed / 10.0);

    // 定义关键时间点
    const SUNRISE_START: f32 = 5.0; // 日出开始
    const SUNRISE_END: f32 = 7.0; // 日出结束
    const SUNSET_START: f32 = 17.0; // 日落开始
    const SUNSET_END: f32 = 19.0; // 日落结束

    // 不同时段的颜色和透明度
    let (color, alpha) = if time_of_day >= SUNRISE_END && time_of_day < SUNSET_START {
        // 白天 (7:00-17:00) - 无覆盖
        (Color::srgb(0.1, 0.15, 0.3), 0.0)
    } else if time_of_day >= SUNSET_END || time_of_day < SUNRISE_START {
        // 深夜 (19:00-5:00) - 深蓝色覆盖
        (Color::srgb(0.05, 0.1, 0.25), 0.6)
    } else if time_of_day >= SUNRISE_START && time_of_day < SUNRISE_END {
        // 日出过渡 (5:00-7:00) - 从深夜到白天
        let progress = (time_of_day - SUNRISE_START) / (SUNRISE_END - SUNRISE_START);
        let smooth_progress = smooth_step(progress); // 使用平滑插值

        // 从深蓝夜色过渡到温暖晨光
        let sunrise_color = Color::srgb(
            0.05 + smooth_progress * 0.15, // 轻微橙色
            0.1 + smooth_progress * 0.1,
            0.25 - smooth_progress * 0.1, // 减少蓝色
        );
        let alpha = 0.6 - smooth_progress * 0.6; // 从0.6渐变到0
        (sunrise_color, alpha)
    } else {
        // 日落过渡 (17:00-19:00) - 从白天到深夜
        let progress = (time_of_day - SUNSET_START) / (SUNSET_END - SUNSET_START);
        let smooth_progress = smooth_step(progress); // 使用平滑插值

        // 从温暖夕阳过渡到深蓝夜色
        let sunset_color = Color::srgb(
            0.2 - smooth_progress * 0.15, // 渐少橙色
            0.15 - smooth_progress * 0.05,
            0.15 + smooth_progress * 0.1, // 增加蓝色
        );
        let alpha = smooth_progress * 0.6; // 从0渐变到0.6
        (sunset_color, alpha)
    };

    for mut sprite in overlay_query.iter_mut() {
        sprite.color = color.with_alpha(alpha);
    }
}

// 平滑步进函数 - 提供更自然的过渡曲线
fn smooth_step(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t) // Hermite插值
}

/// 生成粒子效果 - 只在工作进行中生成
pub fn spawn_particle_system(
    mut commands: Commands,
    time: Res<Time>,
    dwarves: Query<(&Transform, &WorkState, &GridPosition), With<Dwarf>>,
) {
    // 如果时间暂停,不生成粒子
    if time.delta_secs() <= 0.0001 {
        return;
    }

    for (transform, work_state, pos) in dwarves.iter() {
        // 只在矮人到达目标位置并且正在工作时生成粒子
        let should_spawn = match &work_state.current_task {
            Some(Task::Mining(target)) => {
                pos.x == target.x && pos.y == target.y && work_state.work_progress > 0.0
            }
            Some(Task::Gathering(target)) => {
                pos.x == target.x && pos.y == target.y && work_state.work_progress > 0.0
            }
            _ => false,
        };

        if !should_spawn {
            continue;
        }

        // 降低粒子生成频率
        if rand::thread_rng().gen_ratio(1, 10) {
            continue;
        }

        match &work_state.current_task {
            Some(Task::Mining(_)) => {
                // 挖矿粉尘
                let angle = rand::random::<f32>() * std::f32::consts::PI * 2.0;
                let speed = rand::random::<f32>() * 20.0 + 10.0;

                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.6, 0.5, 0.4, 0.8),
                        custom_size: Some(Vec2::new(3.0, 3.0)),
                        ..default()
                    },
                    Transform::from_xyz(transform.translation.x, transform.translation.y, 3.0),
                    Particle {
                        lifetime: 1.0,
                        velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                    },
                ));
            }
            Some(Task::Gathering(_)) => {
                // 采集特效
                let angle = rand::random::<f32>() * std::f32::consts::PI * 2.0;
                let speed = rand::random::<f32>() * 15.0 + 5.0;

                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.2, 0.8, 0.2, 0.9),
                        custom_size: Some(Vec2::new(4.0, 4.0)),
                        ..default()
                    },
                    Transform::from_xyz(
                        transform.translation.x,
                        transform.translation.y + 10.0,
                        3.0,
                    ),
                    Particle {
                        lifetime: 0.8,
                        velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed + 20.0),
                    },
                ));
            }
            _ => {}
        }
    }
}

/// 更新粒子
pub fn particle_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Particle, &mut Sprite)>,
) {
    for (entity, mut transform, mut particle, mut sprite) in query.iter_mut() {
        particle.lifetime -= time.delta_secs();

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // 更新位置
        transform.translation.x += particle.velocity.x * time.delta_secs();
        transform.translation.y += particle.velocity.y * time.delta_secs();

        // 重力
        particle.velocity.y -= 50.0 * time.delta_secs();

        // 淡出
        sprite.color = sprite.color.with_alpha(particle.lifetime);
    }
}
