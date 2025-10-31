# 相机缩放功能

## 功能概述

实现了基于鼠标滚轮的2D相机缩放功能，让玩家可以自由调整游戏视角的大小。

## 使用方法

### 玩家操作
- **向上滚动鼠标滚轮**：放大视角（看得更清晰）
- **向下滚动鼠标滚轮**：缩小视角（看得更广阔）

### 缩放范围
- **最大放大**：3.3倍（scale = 0.3）
- **最小缩小**：40%（scale = 2.5）
- **默认比例**：1.0倍（scale = 1.0）

## 技术实现

### 相关文件
- `src/systems/input.rs` - 缩放系统实现
- `src/main.rs` - 系统注册

### 核心函数

#### camera_zoom_system
```rust
pub fn camera_zoom_system(
    mut scroll_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
)
```

**工作原理**：
1. 监听鼠标滚轮事件
2. 根据滚轮方向计算缩放增量
3. 更新相机的 Transform.scale 值
4. 限制缩放范围在 0.3 到 2.5 之间

**缩放计算**：
- **Line模式**（普通鼠标）：每行滚动 ±10%
- **Pixel模式**（精确设备）：每像素 ±0.2%

### 系统注册

系统在 `LocalView` 状态下运行：
```rust
.add_systems(Update, (
    ui_update_system,
    input_system,
    camera_zoom_system,  // 相机缩放
).run_if(in_state(GameState::LocalView)))
```

## 实现细节

### Transform.scale 的含义
- **scale < 1.0**：放大效果（物体显示更大）
- **scale = 1.0**：正常大小
- **scale > 1.0**：缩小效果（物体显示更小）

### 为什么使用 Transform 而不是 OrthographicProjection？
在 Bevy 0.17 中，对于 2D 相机：
- `Transform.scale` 是推荐的缩放方式
- `OrthographicProjection` 不是组件，访问受限
- `Transform.scale` 同时影响 X、Y、Z 轴，使用 `Vec3::splat()` 保持一致

### 滚轮事件处理
```rust
for event in scroll_events.read() {
    let zoom_delta = match event.unit {
        MouseScrollUnit::Line => event.y * 0.1,
        MouseScrollUnit::Pixel => event.y * 0.002,
    };
    
    let new_scale = (camera_transform.scale.x - zoom_delta).clamp(0.3, 2.5);
    camera_transform.scale = Vec3::splat(new_scale);
}
```

**注意**：
- `event.y` 为正时向上滚动
- 缩放增量为负时，scale 减小，画面放大
- 使用 `clamp()` 防止过度缩放

## 用户体验

### 优点
- ✅ 流畅的缩放体验
- ✅ 合理的缩放范围限制
- ✅ 支持多种滚轮设备
- ✅ 与视角移动无冲突

### 缩放范围设计理由

**最大放大（0.3倍）**：
- 可以清楚看到单个矮人的细节
- 适合精确操作和观察

**最小缩小（2.5倍）**：
- 可以看到更大的游戏区域
- 适合全局规划和快速移动

**默认视角（1.0倍）**：
- 平衡细节和视野
- 大多数操作的舒适视角

## 未来扩展

可能的改进方向：
- [ ] 添加缩放动画（平滑过渡）
- [ ] 键盘快捷键缩放（+/- 键）
- [ ] 以鼠标位置为中心缩放
- [ ] 保存和恢复缩放设置
- [ ] 不同游戏状态的独立缩放
- [ ] 缩放级别指示器UI
- [ ] 快速重置到默认缩放（Home键）

## 兼容性

### 设备支持
- ✅ 标准鼠标滚轮
- ✅ 触控板双指滑动
- ✅ 精密触控设备
- ✅ 游戏鼠标

### 平台测试
- ✅ Windows
- ⚠️ macOS（理论支持，未测试）
- ⚠️ Linux（理论支持，未测试）

## 常见问题

### Q: 缩放后相机位置偏移怎么办？
A: 使用 WASD 或方向键重新调整视角位置。

### Q: 能否缩放得更大？
A: 当前限制是为了保持游戏可玩性。如需调整，修改 `clamp(0.3, 2.5)` 的参数。

### Q: 缩放是否影响游戏性能？
A: 不影响。缩放只改变渲染视口，不增加实体数量。

### Q: 能否添加缩放重置功能？
A: 可以添加快捷键（如 Home 键）将 scale 重置为 1.0。

## 调试技巧

查看当前缩放级别：
```rust
// 在任何系统中添加
info!("当前缩放: {}", camera_transform.scale.x);
```

测试极限缩放：
```rust
// 临时修改 clamp 范围进行测试
.clamp(0.1, 5.0)
```
