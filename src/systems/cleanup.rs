use crate::components::*;
use crate::resources::*;
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
    );
}

/// 将游戏初始化标记重置为未初始化
pub fn reset_game_initialized(mut game_initialized: ResMut<GameInitialized>) {
    game_initialized.initialized = false;
}
