# 世界持久化系统文档

## 概述

世界持久化系统实现了类似《矮人要塞》的世界线机制，确保每次开局都是一个独立的世界，地图生成后永久保存，矮人只在出生点生成。

## 核心特性

### 1. 世界线（World Timeline）

每次开启游戏时：
- 创建一个新的世界种子 (`WorldSeed`)
- 建立一个空的地图注册表 (`GeneratedMapsRegistry`)
- 所有地图数据在这个游戏会话中持久保存

### 2. 地图持久化

**首次进入地块**：
- 使用世界种子 + 地块坐标生成唯一的地图
- 将生成的地图数据存储到注册表
- 如果是第一个地块，设置为出生点

**再次进入地块**：
- 从注册表恢复已生成的地图
- 完全恢复地形、资源丰富度、动画状态
- 保持原样，不重新生成

### 3. 矮人生成规则

**出生点（Spawn Location）**：
- 第一次进入的地块自动成为出生点
- 矮人只在出生点生成
- 其他地块不会生成矮人

**重复进入**：
- 进入非出生点地块：不生成矮人
- 返回出生点：不会重新生成矮人（需要矮人持久化）

## 技术实现

### 数据结构

#### StoredMapTile
```rust
pub struct StoredMapTile {
    pub terrain_type: TerrainType,      // 地形类型
    pub walkable: bool,                  // 是否可行走
    pub resource_richness: f32,          // 资源丰富度
    pub color: Color,                    // 地形颜色
    pub ascii_char: char,                // ASCII字符
    pub char_color: Color,               // 字符颜色
    pub has_water_animation: bool,       // 是否有水动画
    pub has_tree_sway: bool,             // 是否有树摇摆
    pub water_phase: f32,                // 水动画相位
    pub tree_offset: f32,                // 树动画偏移
}
```

#### GeneratedMapsRegistry
```rust
pub struct GeneratedMapsRegistry {
    // 地图数据：key = 世界坐标(x,y)
    pub maps: HashMap<IVec2, Vec<Vec<StoredMapTile>>>,
    // 出生点坐标
    pub spawn_location: Option<IVec2>,
}
```

### 关键函数

#### setup_world()
```rust
pub fn setup_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    world_seed: Res<WorldSeed>,
    active_local: Res<ActiveLocalMap>,
    world_atlas: Res<WorldAtlas>,
    mut map_registry: ResMut<GeneratedMapsRegistry>,
)
```

**逻辑流程**：
1. 检查当前地块是否已生成
2. 如果已生成 → 调用 `restore_map_from_storage()`
3. 如果未生成 → 生成新地图并存储到注册表
4. 如果是第一个地图 → 设置为出生点

#### spawn_dwarves()
```rust
pub fn spawn_dwarves(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    terrain_query: Query<(&GridPosition, &Terrain)>,
    map_registry: Res<GeneratedMapsRegistry>,
    active_local: Res<ActiveLocalMap>,
)
```

**逻辑流程**：
1. 获取当前地块坐标
2. 检查是否是出生点
3. 只在出生点生成矮人
4. 非出生点直接返回

#### restore_map_from_storage()
```rust
fn restore_map_from_storage(
    commands: &mut Commands,
    font: &Handle<Font>,
    stored_map: &Vec<Vec<StoredMapTile>>,
    coord: IVec2,
)
```

**恢复内容**：
- 地形Sprite
- Terrain组件（类型、可行走性、资源丰富度）
- ASCII字符层
- 动画组件（水、树）
- 网格线

### 世界重置

#### cleanup_world_data()
```rust
pub fn cleanup_world_data(
    mut map_registry: ResMut<GeneratedMapsRegistry>,
    mut world_seed: ResMut<WorldSeed>,
)
```

**在以下情况执行**：
- 返回主菜单时 (`OnEnter(GameState::MainMenu)`)

**清理内容**：
- 清空地图注册表
- 重置出生点
- 生成新的世界种子

## 游戏流程

### 新游戏流程

```
主菜单 → 世界地图 → 选择地块A → 局部地图
  ↓                                    ↓
生成世界种子                      生成地图A + 生成矮人
                                (A设为出生点)
```

### 探索其他地块

```
地块A(出生点) → M键返回世界地图 → 选择地块B → 局部地图
                                              ↓
                                         生成地图B (不生成矮人)
```

### 返回出生点

```
地块B → M键返回世界地图 → 选择地块A → 局部地图
                                     ↓
                                恢复地图A (不重新生成矮人)
```

### 开始新游戏

```
游戏中 → ESC返回主菜单 → 新游戏
          ↓
    清理世界线数据
    生成新世界种子
```

## 优势

### 1. 类矮人要塞体验
- 每局游戏是独立的世界
- 地图探索有持久性
- 出生点有特殊意义

### 2. 性能优化
- 已生成的地图直接恢复，无需重新计算
- 减少重复的噪声计算
- 降低CPU负载

### 3. 游戏逻辑清晰
- 矮人只在出生点生成
- 其他地块作为可探索区域
- 未来可扩展：矮人迁移、建立前哨等

## 未来扩展

### 矮人持久化
```rust
pub struct DwarfRegistry {
    // 存储矮人数据
    pub dwarves: HashMap<Entity, StoredDwarf>,
    // 矮人当前所在地块
    pub dwarf_locations: HashMap<Entity, IVec2>,
}
```

### 跨地块移动
- 矮人可以从边界移动到相邻地块
- 自动切换地图视图
- 保持矮人状态

### 建筑持久化
- 保存已建造的建筑
- 恢复建筑时重建实体

### 资源消耗记录
- 记录每个地块的资源采集情况
- 资源不会重置

## 测试要点

### 1. 地图持久化
- [ ] 首次进入地块A，生成新地图
- [ ] 离开后重新进入地块A，地图完全相同
- [ ] 进入地块B，生成不同的地图

### 2. 矮人生成
- [ ] 第一个地块生成矮人
- [ ] 其他地块不生成矮人
- [ ] 返回出生点不重新生成矮人

### 3. 世界重置
- [ ] 返回主菜单后，世界数据被清理
- [ ] 重新开始游戏，生成全新的世界
- [ ] 新世界的地图与之前不同

### 4. 性能
- [ ] 恢复地图比生成新地图快
- [ ] 多次切换地块不卡顿

## 日志示例

```
INFO: 生成新地图: IVec2(5, 6)
INFO: 设置出生点: IVec2(5, 6)
INFO: 在出生点 IVec2(5, 6) 生成矮人
INFO: 生成新地图: IVec2(15, 8)
INFO: 当前地块 IVec2(15, 8) 不是出生点 IVec2(5, 6)，不生成矮人
INFO: 恢复已生成的地图: IVec2(5, 6)
INFO: 清理世界线数据，准备新游戏
```

## 总结

世界持久化系统为游戏建立了坚实的基础架构，实现了：
- ✅ 每局游戏是独立的世界线
- ✅ 地图生成后永久保存
- ✅ 矮人只在出生点生成
- ✅ 支持地块间自由探索
- ✅ 为未来功能预留接口

这个系统完全符合《矮人要塞》的世界机制，为后续的矮人持久化、跨地块移动、建筑系统等功能打下了基础。
