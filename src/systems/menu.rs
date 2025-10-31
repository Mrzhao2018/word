use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;

/// 设置主菜单
pub fn setup_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/sarasa-gothic-sc-regular.ttf");

    // 菜单根容器
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.15, 0.2)),
            MainMenuUI,
        ))
        .with_children(|parent| {
            // 游戏标题
            parent.spawn((
                Text::new("◆ 矮人要塞式游戏 ◆"),
                TextFont {
                    font: font.clone(),
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.5)),
                Node {
                    margin: UiRect::all(Val::Px(50.0)),
                    ..default()
                },
            ));

            // 副标题
            parent.spawn((
                Text::new("Dwarf Fortress Style Game"),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(80.0)),
                    ..default()
                },
            ));

            // 开始游戏按钮
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(80.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.4, 0.5)),
                    StartButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("开始游戏"),
                        TextFont {
                            font: font.clone(),
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 1.0, 1.0)),
                    ));
                });

            // 游戏说明
            parent.spawn((
                Text::new("操作提示:\n\n• WASD/方向键: 移动视角\n• 鼠标左键: 选择矮人\n• 鼠标右键: 指挥矮人移动\n• 空格: 暂停/继续\n• 数字键1-5: 调节时间速度"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.9)),
                Node {
                    margin: UiRect::top(Val::Px(60.0)),
                    ..default()
                },
            ));
        });
}

/// 检查游戏是否未初始化（用于条件系统）
pub fn game_not_initialized(game_initialized: Res<GameInitialized>) -> bool {
    !game_initialized.initialized
}

/// 检查游戏是否已初始化（用于条件系统）
pub fn game_initialized(game_initialized: Res<GameInitialized>) -> bool {
    game_initialized.initialized
}

/// 标记游戏已初始化
pub fn mark_game_initialized(mut game_initialized: ResMut<GameInitialized>) {
    game_initialized.initialized = true;
}

/// 清理主菜单
pub fn cleanup_main_menu(mut commands: Commands, menu_query: Query<Entity, With<MainMenuUI>>) {
    // 删除所有菜单UI实体（子实体会自动删除）
    for entity in menu_query.iter() {
        if let Ok(mut entity_cmd) = commands.get_entity(entity) {
            entity_cmd.despawn();
        }
    }
}

/// 菜单按钮交互系统
pub fn menu_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<StartButton>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // 点击开始游戏
                *color = BackgroundColor(Color::srgb(0.2, 0.5, 0.3));
                next_state.set(GameState::WorldView);
            }
            Interaction::Hovered => {
                // 鼠标悬停
                *color = BackgroundColor(Color::srgb(0.4, 0.5, 0.6));
            }
            Interaction::None => {
                // 正常状态
                *color = BackgroundColor(Color::srgb(0.3, 0.4, 0.5));
            }
        }
    }
}

/// ESC键暂停检测系统
pub fn pause_game_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    use bevy::input::keyboard::KeyCode;

    if keyboard.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            GameState::LocalView => {
                // 游戏中按ESC暂停
                next_state.set(GameState::Paused);
            }
            GameState::Paused => {
                // 暂停中按ESC继续
                next_state.set(GameState::LocalView);
            }
            _ => {}
        }
    }
}

/// 设置暂停菜单
pub fn setup_pause_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/sarasa-gothic-sc-regular.ttf");

    // 半透明背景遮罩
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            PauseMenuUI,
        ))
        .with_children(|parent| {
            // 暂停标题
            parent.spawn((
                Text::new("⏸ 游戏已暂停 ⏸"),
                TextFont {
                    font: font.clone(),
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    margin: UiRect::all(Val::Px(40.0)),
                    ..default()
                },
            ));

            // 继续游戏按钮
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(70.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.5, 0.3)),
                    ResumeButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("继续游戏 (ESC)"),
                        TextFont {
                            font: font.clone(),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 1.0, 1.0)),
                    ));
                });

            // 返回主菜单按钮
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(70.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.5, 0.3, 0.2)),
                    BackToMenuButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("返回主菜单"),
                        TextFont {
                            font: font.clone(),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 1.0, 1.0)),
                    ));
                });

            // 操作提示
            parent.spawn((
                Text::new("\n按 ESC 继续游戏"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::top(Val::Px(30.0)),
                    ..default()
                },
            ));
        });
}

/// 清理暂停菜单
pub fn cleanup_pause_menu(mut commands: Commands, menu_query: Query<Entity, With<PauseMenuUI>>) {
    // 删除所有暂停菜单UI实体（子实体会自动删除）
    for entity in menu_query.iter() {
        if let Ok(mut entity_cmd) = commands.get_entity(entity) {
            entity_cmd.despawn();
        }
    }
}

/// 暂停菜单按钮交互系统
pub fn pause_menu_button_system(
    mut resume_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ResumeButton>),
    >,
    mut back_query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<BackToMenuButton>,
            Without<ResumeButton>,
        ),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // 继续游戏按钮
    for (interaction, mut color) in resume_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.1, 0.4, 0.2));
                next_state.set(GameState::LocalView);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.3, 0.6, 0.4));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.2, 0.5, 0.3));
            }
        }
    }

    // 返回主菜单按钮
    for (interaction, mut color) in back_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.4, 0.2, 0.1));
                next_state.set(GameState::MainMenu);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.6, 0.4, 0.3));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.5, 0.3, 0.2));
            }
        }
    }
}
