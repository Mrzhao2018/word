# 矮人详情面板实现说明

## 功能概述

矮人详情面板是一个动态显示的UI面板，显示当前选中矮人的详细信息。

## 主要特性

### 1. 动态显示/隐藏
- **有矮人选中时**：面板自动显示
- **无矮人选中时**：面板自动隐藏
- 无需手动控制，完全自动化

### 2. 显示内容

#### 基本信息
- 矮人姓名
- 当前位置坐标 (x, y)

#### 状态信息
每个状态显示数值和文字描述：

**健康状态**
- 80%+ → "健康"
- 50%-80% → "受伤"
- 50%以下 → "危险"

**饥饿状态**
- 30%以下 → "饱腹"
- 30%-70% → "正常"
- 70%以上 → "饥饿"

**快乐状态**
- 75%+ → "愉快"
- 50%-75% → "一般"
- 25%-50% → "沮丧"
- 25%以下 → "痛苦"

#### 任务信息
根据当前任务类型显示不同内容：

- **空闲**：显示"正在休息"
- **闲逛**：显示目标位置
- **采集资源**：显示目标位置和进度百分比
- **挖矿采石**：显示目标位置和进度百分比
- **建造建筑**：显示目标位置和建筑类型
- **无任务**：显示"等待指令"

### 3. UI设计

**位置**：左侧中间 (MiddleLeft anchor)
**尺寸**：320x280 像素（最小）
**背景色**：深蓝灰色半透明 (rgba: 0.08, 0.08, 0.15, 0.92)
**边框色**：金黄色半透明 (rgba: 0.8, 0.7, 0.3, 0.8)
**内边距**：大号间距

## 技术实现

### 相关文件
- `src/ui_framework.rs` - UI框架基础设施
- `src/systems/ui.rs` - UI系统实现

### 核心函数

#### setup_ui
```rust
// 在setup_ui中创建矮人详情面板
let dwarf_detail_panel = builder.create_hidden_panel(
    "dwarf_detail",
    dwarf_detail_config,
    DwarfDetailPanel,
);
```

#### update_dwarf_panel
```rust
pub fn update_dwarf_panel(
    selected: Res<SelectedDwarf>,
    dwarves: Query<(&Dwarf, &WorkState, &GridPosition)>,
    mut text_query: Query<&mut Text, With<DwarfPanel>>,
    mut panel_query: Query<(&mut UIPanel, &mut Node), With<DwarfDetailPanel>>,
)
```

**执行逻辑**：
1. 检查是否有选中的矮人
2. 如果没有，隐藏面板并返回
3. 如果有但数据无效，隐藏面板并返回
4. 如果有且数据有效，显示面板并更新内容

### 系统注册

面板更新系统在`LocalView`状态下每帧执行：
```rust
.add_systems(Update, 
    update_dwarf_panel
    .run_if(in_state(GameState::LocalView))
)
```

## 使用方法

### 对玩家
1. 用鼠标左键点击任意矮人
2. 矮人详情面板自动在左侧显示
3. 点击其他矮人切换显示内容
4. 点击空地取消选择，面板自动隐藏

### 对开发者
如需修改面板样式或内容：

1. **修改样式**：编辑 `setup_ui` 中的 `dwarf_detail_config`
2. **修改内容格式**：编辑 `update_dwarf_panel` 中的文本格式化逻辑
3. **添加新信息**：在 `update_dwarf_panel` 中查询更多组件并添加到显示文本中

## 未来扩展

可能的增强功能：
- [ ] 添加矮人技能等级显示
- [ ] 添加装备信息
- [ ] 添加关系网络（朋友/敌人）
- [ ] 添加历史记录（完成的任务）
- [ ] 添加特质/性格描述
- [ ] 点击面板按钮直接发送指令
- [ ] 显示矮人的思想/情绪动态
- [ ] 显示健康详情（受伤部位等）

## 注意事项

1. 面板使用 `DwarfDetailPanel` 组件标记
2. 面板内的文本使用 `DwarfPanel` 组件标记
3. 面板通过 `UIPanel.id = "dwarf_detail"` 识别
4. 必须确保 `SelectedDwarf` 资源已正确初始化
5. 面板依赖字体文件 `fonts/sarasa-gothic-sc-regular.ttf`
