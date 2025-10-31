use bevy::log::LogPlugin;
use bevy::prelude::*;

mod components;
mod debug_config;
mod pathfinding;
mod resources;
mod systems;
mod ui_framework;
mod world;
mod world_map_data;

use resources::*;
use systems::*;
use world::*;
use world_map_data::*;

fn main() {
    App::new()
        // Bevy默认插件 - 配置日志过滤器
        .add_plugins(DefaultPlugins
            .set(LogPlugin {
                // 完全屏蔽wgpu_hal的所有日志，只保留关键信息
                filter: "wgpu=warn,wgpu_core=warn,wgpu_hal=off,bevy_render=info,dwarf_fortress_game=debug".into(),
                level: bevy::log::Level::INFO,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "矮人要塞式游戏".to_string(),
                    resolution: (1400, 800).into(),
                    ..default()
                }),
                ..default()
            })
        )
        // 状态管理
        .init_state::<GameState>()
        // 资源
        .init_resource::<GameWorld>()
        .init_resource::<GameTime>()
        .init_resource::<SelectedDwarf>()
        .init_resource::<GlobalInventory>()
        .init_resource::<GameInitialized>()
    .init_resource::<WorldSeed>()  // 世界生成种子
    .init_resource::<AtlasSelection>()
        .init_resource::<ActiveLocalMap>()
        .init_resource::<GeneratedMapsRegistry>()  // 已生成地图注册表
        // 启动系统（总是执行）
        .add_systems(Startup, (setup_camera, init_world_atlas))
        // 进入主菜单时的系统
        .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
        // 退出主菜单时的系统
        .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
        // 主菜单状态下的更新系统
        .add_systems(Update, menu_button_system.run_if(in_state(GameState::MainMenu)))
        // 世界视图相关系统
        .add_systems(OnEnter(GameState::WorldView), (
            save_dwarves_state,
            simulate_all_offscreen_dwarves, // 模拟所有地块的后台工作
            cleanup_local_map,
            reset_game_initialized,
        ))
        .add_systems(OnEnter(GameState::WorldView), (
            prepare_world_atlas,
            setup_world_atlas_scene,
        ).chain())
        .add_systems(OnExit(GameState::WorldView), cleanup_world_atlas_scene)
        .add_systems(Update, (
            world_atlas_input_system,
            world_atlas_selection_system,
        ).chain().run_if(in_state(GameState::WorldView)))
        // 进入局部地图时的系统（只在首次初始化时生成）
        .add_systems(OnEnter(GameState::LocalView), (
            setup_world,
            spawn_dwarves,
            setup_ui,
            mark_game_initialized,  // 放在链的最后，确保在地图生成后才标记
        ).chain().run_if(game_not_initialized))
        // 进入局部地图时的模拟系统（只在重新进入已有地图时运行）
        .add_systems(OnEnter(GameState::LocalView), 
            simulate_offscreen_dwarves.run_if(game_initialized)
        )
        // 进入主菜单时的系统（从游戏返回主菜单时清理）
        .add_systems(OnEnter(GameState::MainMenu), (
            cleanup_game_on_menu_return,
            cleanup_world_data,
        ))
        // 进入暂停菜单时的系统
        .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
        // 退出暂停菜单时的系统
        .add_systems(OnExit(GameState::Paused), cleanup_pause_menu)
        // 暂停菜单状态下的更新系统
        .add_systems(Update, (
            pause_game_system,
            pause_menu_button_system,
        ).run_if(in_state(GameState::Paused)))
        // 游戏状态下的更新系统
        .add_systems(Update, (
            pause_game_system,  // ESC暂停检测
            local_view_return_to_world_system,
            ui_hotkey_system,  // UI快捷键系统
            dwarf_work_system,    // 先决策
            dwarf_movement_system, // 后执行移动
            resource_gathering_system,
            building_system,
            time_system,
            time_control_system,
            ui_update_system,
            input_system,
        ).run_if(in_state(GameState::LocalView)))
        .add_systems(Update, (
            update_work_indicators,
            mouse_selection_system,
            update_selection_indicator,
            mouse_control_system,
            update_dwarf_panel,
            dwarf_name_hover_system,
            terrain_info_hover_system,  // 地形信息悬停
        ).run_if(in_state(GameState::LocalView)))
        .add_systems(Update, (
            // 动画系统
            water_animation_system,
            tree_sway_system,
            daylight_cycle_system,
            spawn_particle_system,
            particle_system,
        ).run_if(in_state(GameState::LocalView)))
        .run();
}

fn setup_camera(mut commands: Commands) {
    // 2D相机
    commands.spawn(Camera2d);
}
