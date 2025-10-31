# 调试系统使用指南

## 概述

本项目包含一个全局调试配置系统，可以方便地控制各个模块的调试输出。

## 配置文件

调试配置位于 `src/debug_config.rs`，包含以下开关：

### 主开关
- `DEBUG_ENABLED`: 全局调试总开关，设为 `false` 可关闭所有调试输出

### 分类开关
- `DEBUG_WORLD_MAP_INPUT`: 世界地图鼠标/键盘输入调试
- `DEBUG_WORLD_MAP_SELECTION`: 世界地图选择状态变化调试
- `DEBUG_TERRAIN_GENERATION`: 地形生成过程调试
- `DEBUG_ENTITY_SPAWN`: 实体创建调试
- `DEBUG_SYSTEM_TIMING`: 系统执行时间调试

## 使用方法

### 1. 基础用法

在需要调试输出的文件顶部导入对应的宏：

```rust
use crate::{debug_world_input, debug_world_selection};
```

### 2. 输出调试信息

使用对应的宏输出信息：

```rust
// 世界地图输入调试
debug_world_input!("鼠标坐标: {:?}", cursor_pos);

// 世界地图选择调试
debug_world_selection!("选中地块变化: {:?} -> {:?}", old_coord, new_coord);

// 地形生成调试
debug_terrain!("生成地块 ({}, {}): {:?}", x, y, tile_type);

// 实体生成调试
debug_entity!("生成实体: {} at {:?}", entity_name, position);

// 系统时序调试
debug_timing!("系统 {} 耗时: {:?}", system_name, duration);
```

### 3. 通用调试宏

如果需要自定义条件的调试输出：

```rust
use crate::debug_log;
use crate::debug_config::DEBUG_MY_FEATURE;

debug_log!(DEBUG_MY_FEATURE, "自定义调试信息: {}", value);
```

## 添加新的调试类别

### 1. 在 `debug_config.rs` 中添加常量

```rust
pub const DEBUG_NEW_FEATURE: bool = false;
```

### 2. 添加对应的宏

```rust
#[macro_export]
macro_rules! debug_new_feature {
    ($($arg:tt)*) => {
        $crate::debug_log!($crate::debug_config::DEBUG_NEW_FEATURE, $($arg)*);
    };
}
```

### 3. 在需要的地方导入使用

```rust
use crate::debug_new_feature;

debug_new_feature!("新功能调试信息");
```

## 性能说明

- 当对应的调试开关为 `false` 时，调试代码会被完全跳过
- 字符串格式化只在开关打开时才执行
- 对正式版本性能影响极小

## 开发建议

1. **开发阶段**: 将需要关注的模块开关设为 `true`
2. **调试特定功能**: 只打开相关模块的开关
3. **性能测试**: 关闭所有调试开关（`DEBUG_ENABLED = false`）
4. **发布版本**: 确保 `DEBUG_ENABLED = false`

## 当前已集成的模块

- ✅ 世界地图输入系统 (`world_map_view.rs`)
- ✅ 世界地图选择系统 (`world_map_view.rs`)
- ⚠️ 地形生成系统（已预留接口）
- ⚠️ 实体生成系统（已预留接口）
- ⚠️ 系统时序分析（已预留接口）

## 示例输出

当 `DEBUG_WORLD_MAP_INPUT` 和 `DEBUG_WORLD_MAP_SELECTION` 开启时：

```
[DEBUG] input_system 更新悬停: Some(IVec2(10, 6)) -> Some(IVec2(10, 5))
[DEBUG] 状态变化 - 悬停: Some(IVec2(10, 5)), 选中: Some(IVec2(10, 6))
[DEBUG] input_system 更新悬停: Some(IVec2(10, 5)) -> Some(IVec2(9, 5))
[DEBUG] 状态变化 - 悬停: Some(IVec2(9, 5)), 选中: Some(IVec2(10, 6))
```
