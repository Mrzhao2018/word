use crate::components::*;
use crate::world::*;
use bevy::prelude::*;

/// 矮人移动系统 - 基于网格的离散移动，GridPosition始终反映实际位置
pub fn dwarf_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut GridPosition, &Velocity), With<Dwarf>>,
    terrain_query: Query<(&GridPosition, &Terrain), Without<Dwarf>>,
) {
    for (mut transform, mut grid_pos, velocity) in query.iter_mut() {
        // 只有在有速度时才移动
        if velocity.x.abs() > 0.01 || velocity.y.abs() > 0.01 {
            // 计算移动方向（规范化到-1, 0, 1）
            let dir_x = if velocity.x.abs() < 0.01 {
                0
            } else {
                velocity.x.signum() as i32
            };
            let dir_y = if velocity.y.abs() < 0.01 {
                0
            } else {
                velocity.y.signum() as i32
            };

            // 计算目标网格位置
            let target_grid_x = grid_pos.x + dir_x;
            let target_grid_y = grid_pos.y + dir_y;

            // 检查目标位置是否可行走
            let mut can_move = false;
            let mut terrain_speed = 1.0;

            if target_grid_x >= 0
                && target_grid_x < WORLD_WIDTH
                && target_grid_y >= 0
                && target_grid_y < WORLD_HEIGHT
            {
                for (terrain_pos, terrain) in terrain_query.iter() {
                    if terrain_pos.x == target_grid_x && terrain_pos.y == target_grid_y {
                        can_move = terrain.walkable;
                        terrain_speed = terrain.terrain_type.movement_speed();
                        break;
                    }
                }
            }

            if can_move {
                // 计算目标世界坐标（与地形对齐）
                let target_x = target_grid_x as f32 * TILE_SIZE
                    - (WORLD_WIDTH as f32 * TILE_SIZE / 2.0)
                    + (TILE_SIZE / 2.0);
                let target_y = target_grid_y as f32 * TILE_SIZE
                    - (WORLD_HEIGHT as f32 * TILE_SIZE / 2.0)
                    + (TILE_SIZE / 2.0);

                // 平滑移动到目标位置
                let effective_speed = 100.0 * terrain_speed;
                let move_speed = time.delta_secs() * effective_speed;

                let dx = target_x - transform.translation.x;
                let dy = target_y - transform.translation.y;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance < move_speed || distance < 1.0 {
                    // 已经接近目标，直接对齐到网格
                    transform.translation.x = target_x;
                    transform.translation.y = target_y;
                } else {
                    // 继续移动
                    let move_x = dx / distance * move_speed;
                    let move_y = dy / distance * move_speed;
                    transform.translation.x += move_x;
                    transform.translation.y += move_y;
                }
            }
        }

        // 重要：每帧都根据transform计算GridPosition，确保GridPosition反映实际位置
        // 反向计算：pos = grid * TILE_SIZE - (WIDTH * TILE_SIZE / 2.0) + (TILE_SIZE / 2.0)
        // 所以：grid = (pos + (WIDTH * TILE_SIZE / 2.0) - (TILE_SIZE / 2.0)) / TILE_SIZE
        let calculated_grid_x = ((transform.translation.x + (WORLD_WIDTH as f32 * TILE_SIZE / 2.0)
            - (TILE_SIZE / 2.0))
            / TILE_SIZE)
            .round() as i32;
        let calculated_grid_y = ((transform.translation.y
            + (WORLD_HEIGHT as f32 * TILE_SIZE / 2.0)
            - (TILE_SIZE / 2.0))
            / TILE_SIZE)
            .round() as i32;

        // 只在合理范围内更新GridPosition
        if calculated_grid_x >= 0
            && calculated_grid_x < WORLD_WIDTH
            && calculated_grid_y >= 0
            && calculated_grid_y < WORLD_HEIGHT
        {
            grid_pos.x = calculated_grid_x;
            grid_pos.y = calculated_grid_y;
        }
    }
}
