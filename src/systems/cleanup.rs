use crate::components::*;
use crate::resources::*;
use crate::ui_framework::*;
use crate::world_map_data::*;
use bevy::prelude::*;

macro_rules! despawn_entities_safe {
    ($commands:expr, $query:expr) => {
        for entity in $query.iter() {
            if let Ok(mut entity_commands) = $commands.get_entity(entity) {
                entity_commands.despawn();
            }
        }
    };
}

fn cleanup_local_entities(
    commands: &mut Commands,
    terrain_query: &Query<Entity, With<Terrain>>,
    terrain_ascii_query: &Query<Entity, (With<AsciiChar>, With<GridPosition>)>,
    grid_line_query: &Query<Entity, With<GridLine>>,
    dwarf_query: &Query<Entity, With<Dwarf>>,
    ui_query: &Query<Entity, With<DwarfPanel>>,
    particle_query: &Query<Entity, With<Particle>>,
    daylight_query: &Query<Entity, With<DaylightOverlay>>,
    name_tag_query: &Query<Entity, With<DwarfNameTag>>,
    terrain_info_query: &Query<Entity, With<TerrainInfoLabel>>,
    resource_display_query: &Query<Entity, With<ResourceDisplay>>,
    title_display_query: &Query<Entity, With<TitleDisplay>>,
    help_display_query: &Query<Entity, With<HelpDisplay>>,
    ui_panel_query: &Query<Entity, With<UIPanel>>,
) {
    despawn_entities_safe!(commands, terrain_query);
    despawn_entities_safe!(commands, terrain_ascii_query);
    despawn_entities_safe!(commands, grid_line_query);
    despawn_entities_safe!(commands, dwarf_query);
    despawn_entities_safe!(commands, ui_query);
    despawn_entities_safe!(commands, particle_query);
    despawn_entities_safe!(commands, daylight_query);
    despawn_entities_safe!(commands, name_tag_query);
    despawn_entities_safe!(commands, terrain_info_query);
    despawn_entities_safe!(commands, resource_display_query);
    despawn_entities_safe!(commands, title_display_query);
    despawn_entities_safe!(commands, help_display_query);
    despawn_entities_safe!(commands, ui_panel_query);
}

/// 保存矮人状态
pub fn save_dwarves_state(
    dwarf_data_query: Query<(&Dwarf, &GridPosition, &WorkState)>,
    mut map_registry: ResMut<GeneratedMapsRegistry>,
    active_local: Res<ActiveLocalMap>,
    game_time: Res<GameTime>,
    mut logger: ResMut<crate::logger::GameLogger>,
) {
    if let Some(coord) = active_local.coord {
        let mut stored_dwarves = Vec::new();
        for (dwarf, pos, work) in dwarf_data_query.iter() {
            stored_dwarves.push(StoredDwarf {
                name: dwarf.name.clone(),
                grid_x: pos.x,
                grid_y: pos.y,
                health: dwarf.health,
                hunger: dwarf.hunger,
                happiness: dwarf.happiness,
                current_task: work.current_task.clone(),
                work_progress: work.work_progress,
                last_update_day: game_time.day,
                last_update_hour: game_time.hour,
            });
        }
        if !stored_dwarves.is_empty() {
            let count = stored_dwarves.len();
            map_registry.dwarves.insert(coord, stored_dwarves);
            logger.info(format!("保存 {} 个矮人到地块 {:?}", count, coord));
        }
    }
}

/// 清理局部地图中的所有实体（用于状态切换回大地图）
#[allow(clippy::too_many_arguments)]
pub fn cleanup_local_map(
    mut commands: Commands,
    terrain_query: Query<Entity, With<Terrain>>,
    terrain_ascii_query: Query<Entity, (With<AsciiChar>, With<GridPosition>)>,
    grid_line_query: Query<Entity, With<GridLine>>,
    dwarf_query: Query<Entity, With<Dwarf>>,
    ui_query: Query<Entity, With<DwarfPanel>>,
    particle_query: Query<Entity, With<Particle>>,
    daylight_query: Query<Entity, With<DaylightOverlay>>,
    name_tag_query: Query<Entity, With<DwarfNameTag>>,
    terrain_info_query: Query<Entity, With<TerrainInfoLabel>>,
    resource_display_query: Query<Entity, With<ResourceDisplay>>,
    title_display_query: Query<Entity, With<TitleDisplay>>,
    help_display_query: Query<Entity, With<HelpDisplay>>,
    ui_panel_query: Query<Entity, With<UIPanel>>,
) {
    cleanup_local_entities(
        &mut commands,
        &terrain_query,
        &terrain_ascii_query,
        &grid_line_query,
        &dwarf_query,
        &ui_query,
        &particle_query,
        &daylight_query,
        &name_tag_query,
        &terrain_info_query,
        &resource_display_query,
        &title_display_query,
        &help_display_query,
        &ui_panel_query,
    );
}

/// 当返回主菜单时清理游戏（只在游戏已初始化时才清理）
#[allow(clippy::too_many_arguments)]
pub fn cleanup_game_on_menu_return(
    mut commands: Commands,
    mut game_initialized: ResMut<GameInitialized>,
    terrain_query: Query<Entity, With<Terrain>>,
    terrain_ascii_query: Query<Entity, (With<AsciiChar>, With<GridPosition>)>,
    grid_line_query: Query<Entity, With<GridLine>>,
    dwarf_query: Query<Entity, With<Dwarf>>,
    ui_query: Query<Entity, With<DwarfPanel>>,
    particle_query: Query<Entity, With<Particle>>,
    daylight_query: Query<Entity, With<DaylightOverlay>>,
    name_tag_query: Query<Entity, With<DwarfNameTag>>,
    terrain_info_query: Query<Entity, With<TerrainInfoLabel>>,
    resource_display_query: Query<Entity, With<ResourceDisplay>>,
    title_display_query: Query<Entity, With<TitleDisplay>>,
    help_display_query: Query<Entity, With<HelpDisplay>>,
    ui_panel_query: Query<Entity, With<UIPanel>>,
) {
    if !game_initialized.initialized {
        return;
    }

    game_initialized.initialized = false;

    cleanup_local_entities(
        &mut commands,
        &terrain_query,
        &terrain_ascii_query,
        &grid_line_query,
        &dwarf_query,
        &ui_query,
        &particle_query,
        &daylight_query,
        &name_tag_query,
        &terrain_info_query,
        &resource_display_query,
        &title_display_query,
        &help_display_query,
        &ui_panel_query,
    );
}

/// 清理世界线数据（在返回主菜单时）
pub fn cleanup_world_data(
    mut commands: Commands,
    mut map_registry: ResMut<GeneratedMapsRegistry>,
    mut world_seed: ResMut<WorldSeed>,
    mut game_time: ResMut<GameTime>,
    mut inventory: ResMut<GlobalInventory>,
    world_atlas: Option<ResMut<WorldAtlas>>,
    mut logger: ResMut<crate::logger::GameLogger>,
) {
    // 清理地图数据
    map_registry.maps.clear();
    map_registry.dwarves.clear();
    map_registry.spawn_location = None;
    map_registry.dwarves_spawned = false;
    
    // 重新生成世界种子
    world_seed.seed = rand::random();
    
    // 重新生成世界地图
    if let Some(mut atlas) = world_atlas {
        *atlas = WorldAtlas::generate(
            world_seed.seed as u64,
            WORLD_ATLAS_DEFAULT_WIDTH,
            WORLD_ATLAS_DEFAULT_HEIGHT,
        );
        logger.info(format!("重新生成世界地图，种子: {}", world_seed.seed));
    } else {
        // 如果 WorldAtlas 还不存在，创建它
        let atlas = WorldAtlas::generate(
            world_seed.seed as u64,
            WORLD_ATLAS_DEFAULT_WIDTH,
            WORLD_ATLAS_DEFAULT_HEIGHT,
        );
        commands.insert_resource(atlas);
        logger.info(format!("首次创建世界地图，种子: {}", world_seed.seed));
    }
    
    // 重置游戏时间
    game_time.day = 0;
    game_time.hour = 6;
    game_time.elapsed = 0.0;
    game_time.time_scale = 1.0;
    
    // 重置资源
    inventory.stone = 50;
    inventory.wood = 30;
    inventory.food = 100;
    inventory.metal = 10;
    
    logger.info("返回主菜单，游戏数据已重置".to_string());
}

/// 将游戏初始化标记重置为未初始化
pub fn reset_game_initialized(mut game_initialized: ResMut<GameInitialized>) {
    game_initialized.initialized = false;
}
