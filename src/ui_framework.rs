/// UI框架 - 模块化、可扩展的UI系统
/// 
/// 设计理念：
/// 1. 面板系统：所有UI元素组织为独立的面板
/// 2. 布局管理：支持多种布局模式（固定、锚点、弹性）
/// 3. 主题系统：统一的颜色、字体和样式
/// 4. 事件系统：支持交互和动画
/// 5. 状态管理：面板可以显示/隐藏/最小化

use bevy::prelude::*;

// ============ UI主题配置 ============

/// UI主题 - 统一的颜色和样式配置
#[derive(Resource, Clone)]
pub struct UITheme {
    // 颜色方案
    pub primary_color: Color,       // 主色调
    pub secondary_color: Color,     // 次要色
    pub accent_color: Color,        // 强调色
    pub background_dark: Color,     // 深色背景
    pub background_light: Color,    // 浅色背景
    pub text_primary: Color,        // 主文本颜色
    pub text_secondary: Color,      // 次要文本
    pub text_highlight: Color,      // 高亮文本
    pub border_color: Color,        // 边框颜色
    
    // 字体大小
    pub font_size_title: f32,       // 标题字体
    pub font_size_normal: f32,      // 正常文本
    pub font_size_small: f32,       // 小字体
    
    // 间距
    pub padding_small: f32,
    pub padding_medium: f32,
    pub padding_large: f32,
    
    // 面板样式
    pub panel_alpha: f32,           // 面板透明度
    pub panel_border_width: f32,    // 边框宽度
}

impl Default for UITheme {
    fn default() -> Self {
        Self {
            // 深色主题配色
            primary_color: Color::srgb(0.3, 0.5, 0.8),
            secondary_color: Color::srgb(0.4, 0.6, 0.7),
            accent_color: Color::srgb(1.0, 0.85, 0.3),
            background_dark: Color::srgba(0.05, 0.05, 0.1, 0.85),
            background_light: Color::srgba(0.1, 0.1, 0.15, 0.75),
            text_primary: Color::srgb(1.0, 1.0, 1.0),
            text_secondary: Color::srgb(0.8, 0.8, 0.8),
            text_highlight: Color::srgb(1.0, 0.9, 0.5),
            border_color: Color::srgba(0.5, 0.5, 0.6, 0.5),
            
            font_size_title: 26.0,
            font_size_normal: 18.0,
            font_size_small: 14.0,
            
            padding_small: 8.0,
            padding_medium: 12.0,
            padding_large: 20.0,
            
            panel_alpha: 0.85,
            panel_border_width: 2.0,
        }
    }
}

// ============ 面板系统 ============

/// 面板位置锚点
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PanelAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    Center,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl PanelAnchor {
    pub fn to_position(&self, offset: Vec2) -> (Val, Val) {
        match self {
            PanelAnchor::TopLeft => (Val::Px(offset.x), Val::Px(offset.y)),
            PanelAnchor::TopCenter => (Val::Percent(50.0), Val::Px(offset.y)),
            PanelAnchor::TopRight => (Val::Auto, Val::Px(offset.y)),
            PanelAnchor::MiddleLeft => (Val::Px(offset.x), Val::Percent(50.0)),
            PanelAnchor::Center => (Val::Percent(50.0), Val::Percent(50.0)),
            PanelAnchor::MiddleRight => (Val::Auto, Val::Percent(50.0)),
            PanelAnchor::BottomLeft => (Val::Px(offset.x), Val::Auto),
            PanelAnchor::BottomCenter => (Val::Percent(50.0), Val::Auto),
            PanelAnchor::BottomRight => (Val::Auto, Val::Auto),
        }
    }
}

/// 面板配置
#[derive(Clone)]
pub struct PanelConfig {
    pub anchor: PanelAnchor,
    pub offset: Vec2,
    pub min_width: f32,
    pub min_height: f32,
    pub background_color: Color,
    pub border_color: Option<Color>,
    pub padding: f32,
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            anchor: PanelAnchor::TopLeft,
            offset: Vec2::new(15.0, 15.0),
            min_width: 200.0,
            min_height: 100.0,
            background_color: Color::srgba(0.05, 0.05, 0.1, 0.85),
            border_color: Some(Color::srgba(0.5, 0.5, 0.6, 0.5)),
            padding: 12.0,
        }
    }
}

/// 面板状态
#[derive(Component, Clone, Copy, PartialEq)]
pub enum PanelState {
    Visible,
    Hidden,
    Minimized,
}

/// 面板组件 - 标记所有UI面板
#[derive(Component)]
pub struct UIPanel {
    pub id: String,
    pub state: PanelState,
    pub config: PanelConfig,
}

/// 面板内容组件 - 用于标记面板内部的内容区域
#[derive(Component)]
pub struct PanelContent {
    pub panel_id: String,
}

// ============ 具体面板类型标记 ============

/// 资源统计面板
#[derive(Component)]
pub struct ResourcePanel;

/// 游戏标题面板
#[derive(Component)]
pub struct TitlePanel;

/// 帮助信息面板
#[derive(Component)]
pub struct HelpPanel;

/// 矮人详情面板
#[derive(Component)]
pub struct DwarfDetailPanel;

/// 小地图面板
#[derive(Component)]
pub struct MinimapPanel;

/// 建筑菜单面板
#[derive(Component)]
pub struct BuildingMenuPanel;

/// 通知消息面板
#[derive(Component)]
pub struct NotificationPanel;

/// 调试信息面板
#[derive(Component)]
pub struct DebugPanel;

// ============ UI构建器 ============

/// UI面板构建器 - 提供流式API构建面板
pub struct PanelBuilder<'a> {
    commands: Commands<'a, 'a>,
    font: Handle<Font>,
    theme: UITheme,
}

impl<'a> PanelBuilder<'a> {
    pub fn new(commands: Commands<'a, 'a>, font: Handle<Font>, theme: UITheme) -> Self {
        Self {
            commands,
            font,
            theme,
        }
    }

    /// 创建基础面板容器
    pub fn create_panel(
        &mut self,
        id: &str,
        config: PanelConfig,
        marker: impl Component,
    ) -> Entity {
        self.create_panel_with_state(id, config, marker, PanelState::Visible)
    }

    /// 创建初始隐藏的面板容器
    pub fn create_hidden_panel(
        &mut self,
        id: &str,
        config: PanelConfig,
        marker: impl Component,
    ) -> Entity {
        self.create_panel_with_state(id, config, marker, PanelState::Hidden)
    }

    /// 创建面板容器（指定初始状态）
    fn create_panel_with_state(
        &mut self,
        id: &str,
        config: PanelConfig,
        marker: impl Component,
        initial_state: PanelState,
    ) -> Entity {
        let (left, top) = config.anchor.to_position(config.offset);
        let (right, bottom) = if matches!(config.anchor, PanelAnchor::TopRight | PanelAnchor::MiddleRight | PanelAnchor::BottomRight) {
            (Val::Px(config.offset.x), Val::Auto)
        } else if matches!(config.anchor, PanelAnchor::BottomLeft | PanelAnchor::BottomCenter) {
            (Val::Auto, Val::Px(config.offset.y))
        } else {
            (Val::Auto, Val::Auto)
        };

        let display = if initial_state == PanelState::Hidden {
            Display::None
        } else {
            Display::Flex
        };

        self.commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left,
                    top,
                    right,
                    bottom,
                    padding: UiRect::all(Val::Px(config.padding)),
                    min_width: Val::Px(config.min_width),
                    min_height: Val::Px(config.min_height),
                    flex_direction: FlexDirection::Column,
                    display,
                    ..default()
                },
                BackgroundColor(config.background_color),
                UIPanel {
                    id: id.to_string(),
                    state: initial_state,
                    config: config.clone(),
                },
                marker,
            ))
            .id()
    }

    /// 向面板添加标题
    pub fn add_title(&mut self, parent: Entity, title: &str) {
        let text_entity = self.commands
            .spawn((
                Text::new(title),
                TextFont {
                    font: self.font.clone(),
                    font_size: self.theme.font_size_title,
                    ..default()
                },
                TextColor(self.theme.text_highlight),
                Node {
                    margin: UiRect::bottom(Val::Px(self.theme.padding_small)),
                    ..default()
                },
            ))
            .id();
        
        self.commands.entity(parent).add_child(text_entity);
    }

    /// 向面板添加普通文本
    pub fn add_text(&mut self, parent: Entity, text: &str, marker: impl Component) {
        let text_entity = self.commands
            .spawn((
                Text::new(text),
                TextFont {
                    font: self.font.clone(),
                    font_size: self.theme.font_size_normal,
                    ..default()
                },
                TextColor(self.theme.text_primary),
                marker,
            ))
            .id();
        
        self.commands.entity(parent).add_child(text_entity);
    }

    /// 向面板添加小号文本
    pub fn add_small_text(&mut self, parent: Entity, text: &str) {
        let text_entity = self.commands
            .spawn((
                Text::new(text),
                TextFont {
                    font: self.font.clone(),
                    font_size: self.theme.font_size_small,
                    ..default()
                },
                TextColor(self.theme.text_secondary),
            ))
            .id();
        
        self.commands.entity(parent).add_child(text_entity);
    }
}

// ============ UI工具函数 ============

/// 切换面板显示/隐藏
pub fn toggle_panel_visibility(
    panel_query: &mut Query<(&mut UIPanel, &mut Node)>,
    panel_id: &str,
) {
    for (mut panel, mut node) in panel_query.iter_mut() {
        if panel.id == panel_id {
            panel.state = match panel.state {
                PanelState::Visible => {
                    node.display = Display::None;
                    PanelState::Hidden
                }
                PanelState::Hidden => {
                    node.display = Display::Flex;
                    PanelState::Visible
                }
                PanelState::Minimized => {
                    node.display = Display::Flex;
                    PanelState::Visible
                }
            };
        }
    }
}

/// 显示面板
pub fn show_panel(panel_query: &mut Query<(&mut UIPanel, &mut Node)>, panel_id: &str) {
    for (mut panel, mut node) in panel_query.iter_mut() {
        if panel.id == panel_id {
            node.display = Display::Flex;
            panel.state = PanelState::Visible;
        }
    }
}

/// 隐藏面板
pub fn hide_panel(panel_query: &mut Query<(&mut UIPanel, &mut Node)>, panel_id: &str) {
    for (mut panel, mut node) in panel_query.iter_mut() {
        if panel.id == panel_id {
            node.display = Display::None;
            panel.state = PanelState::Hidden;
        }
    }
}
