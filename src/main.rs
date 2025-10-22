use bevy::prelude::*;

mod components;
mod systems;
mod resources;
mod world;

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
        // 资源
        .init_resource::<GameWorld>()
        .init_resource::<GameTime>()
        .init_resource::<SelectedDwarf>()
        .init_resource::<GlobalInventory>()
        // 启动系统
        .add_systems(Startup, (
            setup_camera,
            setup_world,
            spawn_dwarves,
            setup_ui,
        ))
        // 更新系统
        .add_systems(Update, (
            dwarf_movement_system,
            dwarf_work_system,
            resource_gathering_system,
            building_system,
            time_system,
            time_control_system,  // 时间控制系统
            ui_update_system,
            input_system,
            update_work_indicators,
            mouse_selection_system,
            update_selection_indicator,
            mouse_control_system,
            update_dwarf_panel,
            // 动画系统
            water_animation_system,
            tree_sway_system,
            daylight_cycle_system,  // 重新启用,使用覆盖层方式
            spawn_particle_system,
            particle_system,
        ))
        .run();
}

fn setup_camera(mut commands: Commands) {
    // 2D相机
    commands.spawn(Camera2d);
}
