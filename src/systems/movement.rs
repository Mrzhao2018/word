use bevy::prelude::*;
use crate::components::*;
use crate::world::*;

/// 矮人移动系统 - 改进版,包含缓动和动画
pub fn dwarf_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut GridPosition, &Velocity), With<Dwarf>>,
) {
    for (mut transform, mut grid_pos, velocity) in query.iter_mut() {
        // 只有在有速度时才移动
        if velocity.x.abs() > 0.01 || velocity.y.abs() > 0.01 {
            // 计算目标位置(基于速度)
            transform.translation.x += velocity.x * time.delta_secs() * 100.0;
            transform.translation.y += velocity.y * time.delta_secs() * 100.0;
        }
        
        // 更新网格位置
        let new_x = ((transform.translation.x + (WORLD_WIDTH as f32 * TILE_SIZE / 2.0)) / TILE_SIZE) as i32;
        let new_y = ((transform.translation.y + (WORLD_HEIGHT as f32 * TILE_SIZE / 2.0)) / TILE_SIZE) as i32;
        
        if new_x >= 0 && new_x < WORLD_WIDTH && new_y >= 0 && new_y < WORLD_HEIGHT {
            grid_pos.x = new_x;
            grid_pos.y = new_y;
        }
    }
}
