# 矮人要塞式游戏 (Dwarf Fortress-like Game)

使用Bevy游戏引擎和Rust编写的类矮人要塞游戏。

## 功能特性

### 已实现
- ✅ 基础世界生成(草地、石头、树木、水域)
- ✅ 矮人实体系统(7个矮人)
- ✅ 矮人自动工作(采集、挖矿)
- ✅ 资源系统(石头、木材、食物、金属)
- ✅ 时间系统(日夜循环)
- ✅ 相机控制(WASD或方向键移动)
- ✅ UI显示(资源统计、时间)

### 计划实现
- 🔲 建筑系统(工坊、仓库、农场、宿舍)
- 🔲 需求系统(饥饿、疲劳、快乐度)
- 🔲 AI路径寻找
- 🔲 地下挖掘
- 🔲 战斗系统
- 🔲 贸易系统

## 快速开始

### 1. 安装Rust
如果还没安装Rust,访问 https://rustup.rs/ 下载安装。

### 2. 运行游戏
```bash
cargo run
```

首次运行会下载依赖,可能需要几分钟。

### 3. 操作说明
- **WASD** 或 **方向键**: 移动相机
- **ESC**: 退出游戏

## 项目结构

```
src/
├── main.rs          # 游戏入口,插件配置
├── components.rs    # 游戏组件(矮人、地形、建筑等)
├── resources.rs     # 全局资源(时间、库存等)
├── systems.rs       # 游戏系统(移动、工作、UI等)
└── world.rs         # 世界生成和常量
```

## Rust基础概念说明

### 1. 组件(Component)
组件是附加到实体的数据。例如:
```rust
#[derive(Component)]
pub struct Dwarf {
    pub name: String,
    pub health: f32,
}
```

### 2. 系统(System)
系统是处理组件的逻辑函数:
```rust
fn dwarf_movement_system(query: Query<&mut Transform, With<Dwarf>>) {
    // 处理所有有Dwarf组件的实体
}
```

### 3. 资源(Resource)
全局共享的数据:
```rust
#[derive(Resource)]
pub struct GameTime {
    pub day: u32,
}
```

## 扩展建议

1. **添加新的矮人技能**:
   - 在 `components.rs` 中添加技能枚举
   - 在 `systems.rs` 中实现技能逻辑

2. **创建新建筑**:
   - 在 `BuildingType` 枚举中添加类型
   - 实现建筑建造逻辑

3. **改进AI**:
   - 实现A*寻路算法
   - 添加任务优先级系统

## 常见问题

### 编译太慢?
首次编译会比较慢,后续会快很多。可以使用 `cargo run --release` 获得更好的性能(但编译更久)。

### 游戏卡顿?
当前是开发模式,使用 `cargo run --release` 可获得更好性能。

## 学习资源

- [Rust官方教程](https://doc.rust-lang.org/book/)
- [Bevy官方文档](https://bevyengine.org/learn/)
- [Bevy示例](https://github.com/bevyengine/bevy/tree/main/examples)

## 许可证

MIT
