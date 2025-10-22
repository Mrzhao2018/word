use bevy::prelude::*;

mod components;
mod systems;
mod resources;
mod world;
mod pathfinding;

use systems::*;
use resources::*;
use world::*;

fn main() {
    App::new()
        // Bevy默认插件
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "矮人要塞式游戏".to_string(),
                resolution: (1400, 800).into(),
                ..default()
            }),
            ..default()
        }))
        // 状态管理
        .init_state::<GameState>()
        // 资源
        .init_resource::<GameWorld>()
        .init_resource::<GameTime>()
        .init_resource::<SelectedDwarf>()
        .init_resource::<GlobalInventory>()
        .init_resource::<GameInitialized>()
        .init_resource::<WorldSeed>()  // 世界生成种子
        // 启动系统（总是执行）
        .add_systems(Startup, setup_camera)
        // 进入主菜单时的系统
        .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
        // 退出主菜单时的系统
        .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
        // 主菜单状态下的更新系统
        .add_systems(Update, menu_button_system.run_if(in_state(GameState::MainMenu)))
        // 进入游戏时的系统（只在首次初始化时生成世界）
        .add_systems(OnEnter(GameState::Playing), (
            setup_world,
            spawn_dwarves,
            setup_ui,
        ).chain().run_if(game_not_initialized))
        // 标记游戏已初始化
        .add_systems(OnEnter(GameState::Playing), mark_game_initialized)
        // 进入主菜单时的系统（从游戏返回主菜单时清理）
        .add_systems(OnEnter(GameState::MainMenu), cleanup_game_on_menu_return)
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
            dwarf_work_system,    // 先决策
            dwarf_movement_system, // 后执行移动
            resource_gathering_system,
            building_system,
            time_system,
            time_control_system,
            ui_update_system,
            input_system,
        ).run_if(in_state(GameState::Playing)))
        .add_systems(Update, (
            update_work_indicators,
            mouse_selection_system,
            update_selection_indicator,
            mouse_control_system,
            update_dwarf_panel,
            dwarf_name_hover_system,
            terrain_info_hover_system,  // 地形信息悬停
        ).run_if(in_state(GameState::Playing)))
        .add_systems(Update, (
            // 动画系统
            water_animation_system,
            tree_sway_system,
            daylight_cycle_system,
            spawn_particle_system,
            particle_system,
        ).run_if(in_state(GameState::Playing)))
        .run();
}

fn setup_camera(mut commands: Commands) {
    // 2D相机
    commands.spawn(Camera2d);
}
