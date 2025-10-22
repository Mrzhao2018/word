use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

/// 当返回主菜单时清理游戏（只在游戏已初始化时才清理）
pub fn cleanup_game_on_menu_return(
    mut commands: Commands,
    mut game_initialized: ResMut<GameInitialized>,
    terrain_query: Query<Entity, With<Terrain>>,
    terrain_ascii_query: Query<Entity, (With<AsciiChar>, With<GridPosition>)>,  // 地形的ASCII字符（有GridPosition）
    grid_line_query: Query<Entity, With<GridLine>>,  // 网格线
    dwarf_query: Query<Entity, With<Dwarf>>,
    ui_query: Query<Entity, With<DwarfPanel>>,
    particle_query: Query<Entity, With<Particle>>,
    daylight_query: Query<Entity, With<DaylightOverlay>>,
    name_tag_query: Query<Entity, With<DwarfNameTag>>,
    terrain_info_query: Query<Entity, With<TerrainInfoLabel>>,  // 地形信息标签
    resource_display_query: Query<Entity, With<ResourceDisplay>>,
    title_display_query: Query<Entity, With<TitleDisplay>>,
    help_display_query: Query<Entity, With<HelpDisplay>>,
) {
    // 只在游戏已初始化时才清理
    if !game_initialized.initialized {
        return;
    }
    
    // 重置初始化标记
    game_initialized.initialized = false;
    
    // 安全删除实体的宏（检查实体是否存在）
    macro_rules! despawn_entities_safe {
        ($query:expr) => {
            for entity in $query.iter() {
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.despawn();
                }
            }
        };
    }
    
    // 清理所有游戏实体
    despawn_entities_safe!(terrain_query);  // 地形方块
    despawn_entities_safe!(terrain_ascii_query);  // 地形ASCII字符
    despawn_entities_safe!(grid_line_query);  // 网格线
    despawn_entities_safe!(dwarf_query);    // 矮人（包括其子实体：ASCII字符、工作指示器、选择指示器）
    despawn_entities_safe!(ui_query);
    despawn_entities_safe!(particle_query);
    despawn_entities_safe!(daylight_query);
    despawn_entities_safe!(name_tag_query);
    despawn_entities_safe!(terrain_info_query);  // 地形信息标签
    despawn_entities_safe!(resource_display_query);
    despawn_entities_safe!(title_display_query);
    despawn_entities_safe!(help_display_query);
}
