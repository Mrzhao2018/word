# 大地图系统设计文档

## 1. 设计目标
- 将现有 50×30 网格的 "小地图" 定位为单个局部区域 (Local Map)，可被嵌入到更大的世界版图中。
- 支持多个区域的生成、加载、卸载，并保持与全局资源、时间、剧情事件的同步。
- 为后续功能（旅行、贸易、外交、扩张、随机事件等）提供统一的数据与系统接口。
- 保持现有局部地图玩法的完整性，避免大幅重构现有寻路、AI、资源等逻辑。

## 2. 系统层次概览
```
+--------------------------------------------------------------+
|                      World Simulation (宏观层)               |
| - WorldMapResource (全局大地图)                              |
| - RegionController 系统 (区域状态与事件驱动)                |
| - GlobalTime / GlobalInventory (全局资源、时间)             |
| - TravelManager (队伍/矮人跨区域移动)                        |
+-------------------------切换视图------------------------------+
|                      Local Simulation (局部层)               |
| - LocalMapState (当前局部地图实体集合)                      |
| - 现有系统: 寻路、AI、资源采集、UI 等                       |
| - LocalTime (可选，与全局时间同步或加速)                     |
+--------------------------------------------------------------+
```

### 2.1 模块划分
- **World Layer**：只保留抽象数据（地形摘要、气候、城镇、事件），不实例化具体实体。
- **Local Layer**：使用当前的 ECS 结构，管理具体地块、单位、建筑。
- **Shared Layer**：跨层共享的资源与系统，如全局时间、全局库存、任务系统。

## 3. 核心数据模型

### 3.1 大地图资源结构 `WorldMap`
```rust
pub struct WorldMap {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<WorldTile>,
}

pub struct WorldTile {
    pub coord: IVec2,
    pub biome: BiomeKind,
    pub elevation: i16,
    pub temperature: i8,
    pub rainfall: i8,
    pub discovered: bool,
    pub settlements: Vec<SettlementId>,
    pub resource_summary: ResourceSummary,
    pub local_map: Option<LocalMapId>,
    pub world_events: Vec<WorldEventId>,
}

pub enum BiomeKind {
    Grassland,
    Forest,
    Mountain,
    Desert,
    Tundra,
    Ocean,
    River,
    Swamp,
}

pub struct ResourceSummary {
    pub wood: f32,
    pub stone: f32,
    pub metal: f32,
    pub food: f32,
    pub special: Vec<SpecialResource>,
}
```

- `tiles` 为稀疏存储仍使用 `Vec` + 索引（`index = x + y * width`）。若未来地图极大，可换为 `HashMap` 或四叉树。
- `local_map` 指向持久化的局部地图 ID，用于懒加载。

### 3.2 局部地图索引
```rust
pub type LocalMapId = u64; // 由 WorldGenerator 生成

pub struct LocalMapRegistry {
    pub loaded: HashMap<LocalMapId, LocalMapHandle>,
    pub cache_dir: PathBuf,
}

pub struct LocalMapHandle {
    pub seed: u64,
    pub last_visit: GameTimestamp,
    pub persistent_path: PathBuf,
    pub is_dirty: bool,
}
```

- `seed` 与现有 `WorldSeed` 类似，用于重建局部地图。
- `is_dirty` 用于离开局部地图时决定是否写回磁盘。

### 3.3 状态资源
- `GameState::WorldView`：显示大地图，停用局部系统。
- `GameState::LocalView(LocalMapId)`：加载局部地图，启用现有系统。
- `GlobalTime`：主时间轴。局部时间可与其同步或设置倍率（如进入局部时放慢）。
- `GlobalInventory`：全局资源池，局部活动可读写。
- `TravelPlan`：记录队伍跨区域移动路径（可用于队伍 AI 和事件）。

## 4. 系统设计

### 4.1 世界初始化
1. 载入或生成 `WorldMap`：
   - 使用多层噪声（海陆、高度、湿度、温度）。
   - 生成河流、湖泊、特殊资源点。
   - 为每个 `WorldTile` 指派一个 `seed`（用于局部地图生成）。
2. 构建 `LocalMapRegistry`，设置缓存目录（例如 `saves/regions/`）。
3. 设置初始 `GameState::WorldView`，生成大地图 UI。

### 4.2 进入局部地图流程
```
WorldView -> 用户选择 tile -> 请求 LocalMapLoad(tile)
    ├─ 检查 LocalMapRegistry.loaded 是否已加载
    ├─ 若未加载：
    │    ├─ 若存在持久化文件 -> deserialize
    │    └─ 否则调用 LocalGenerator(seed) -> 生成实体数据
    ├─ 清空当前局部实体 (despawn)
    ├─ 通过 commands.spawn 批量创建新的局部地图实体
    ├─ 设置 GameState::LocalView(local_id)
    └─ 切换 UI / 摄像机 / 输入绑定
```

### 4.3 离开局部地图
1. 捕获当前局部状态：
   - 资源节点剩余量、建筑状态、单位位置和数据。
2. 序列化写入 `LocalMapHandle.persistent_path`（如 RON/JSON/bincode）。
3. 更新 `LocalMapHandle.last_visit`、`is_dirty = false`。
4. 清空局部实体，释放内存。
5. 回到 `GameState::WorldView`，恢复大地图 UI。

### 4.4 大地图系统 (WorldView)
- **输入系统**：处理缩放、平移、鼠标悬停、点击。
- **渲染系统**：为每个 `WorldTile` 绘制一个符号/图标（基于 `BiomeKind`, `discovered`, `resource_summary`）。
- **信息 UI**：显示选中格子的概况、已建造的建筑、驻扎矮人、资源加成等。
- **事件系统**：
  - 时间推移时触发宏观事件（季节变化、贸易路线刷新、怪物迁移）。
  - `WorldEvent` 可以在大地图上显示标记，进入对应局部地图处理。

### 4.5 跨区域移动
- 定义 `TravelTask`：包含队伍成员、路线、出发时间、预计到达时间。
- 大地图周期性更新 `TravelTask`：
  - 根据地形、天气、敌对势力调整速度。
  - 到达目的地后触发事件（自动进入局部地图或生成战斗场景）。
- 局部地图离开时，将矮人从局部实体转化为抽象的 `TravelUnit`。

## 5. 生成与持久化

### 5.1 世界生成流程
1. **噪声生成**：
   - `continent_noise` 确定海陆分布。
   - `elevation_noise` + `detail_noise` 细化地形。
   - `temperature_noise`, `rainfall_noise` 决定气候分类。
2. **Biomes**：
   ```rust
   fn classify_biome(elevation, temperature, rainfall) -> BiomeKind
   ```
3. **资源分布**：
   - 基于 `biome` + `elevation` + 随机扰动。
   - 记录在 `ResourceSummary` 中。
4. **局部种子**：
   ```rust
   tile.local_map = Some(LocalMapId::new(global_seed, tile.coord));
   tile.seed = hash(global_seed, tile.coord);
   ```

### 5.2 局部生成流程
- `LocalGenerator` 接受 `LocalMapId` 与 `seed`，返回：
  - 地形网格 (与现有 `setup_world` 一致，可复用函数)。
  - 初始单位、建筑、资源节点。
  - 特殊事件脚本（基于大地图事件同步）。

### 5.3 保存 / 加载
- 新增 `SaveData` 结构：
  ```rust
  pub struct SaveData {
      pub world_map: WorldMap,
      pub local_registry: LocalRegistryMetadata,
      pub global_inventory: GlobalInventory,
      pub global_time: GlobalTime,
      pub active_local: Option<ActiveLocalState>,
  }
  ```
- `LocalRegistryMetadata` 记录每个 `LocalMapHandle` 的 `last_visit`, `is_dirty`, `path`。
- `ActiveLocalState` 在保存时直接序列化 ECS World (当前局部) 或制作镜像结构。

## 6. 系统交互图
```
+------------------------+      +---------------------------+
|      WorldMapResource  |      |  LocalMapRegistry         |
| - tiles                |<---->| - loaded handles          |
| - events               |      | - serialization metadata  |
+------------------------+      +---------------------------+
          ^                                   ^
          |                                   |
          v                                   v
+------------------------+      +---------------------------+
|  WorldView Systems     |      | LocalView Systems         |
| - map rendering        |      | - setup_world (existing)  |
| - event scheduler      |      | - dwarf systems (existing)|
| - travel updates       |      | - ui_update_system        |
+------------------------+      +---------------------------+
```

## 7. 渐进式实现路径

### 阶段 1：基础大地图框架
- [ ] 定义 `WorldMap` 数据结构与序列化。
- [ ] 创建 `GameState::WorldView / LocalView`。
- [ ] 在 WorldView 中渲染一个简单的大地图（例如静态噪声结果）。
- [ ] 支持从 WorldView 切换到现有 LocalView 并返回。

### 阶段 2：局部地图懒加载
- [ ] 引入 `LocalMapRegistry` 与持久化目录。
- [ ] 支持保存/加载局部状态。
- [ ] 离开局部地图时清除实体，释放内存。

### 阶段 3：跨区域功能
- [ ] 实现 TravelTask / 队伍系统。
- [ ] 大地图事件调度（季节、灾害、贸易）。
- [ ] 在 WorldView UI 中显示资源与事件信息。

### 阶段 4：扩展与优化
- [ ] LOD / 大地图缩放。
- [ ] 多线程生成（地形、事件预计算）。
- [ ] 网络或多人准备（可选）。
- [ ] AI 派系 / 外部势力。

## 8. 与现有代码的融合点
- `setup_world` 和 `spawn_dwarves` 需要拆分为：
  - `LocalMapBuilder::generate(seed)` 返回纯数据。
  - `LocalMapSpawner::spawn(LocalMapData)` 根据数据生成实体。
- 现有系统（移动、寻路、AI、UI）在 `LocalView` 保持不变；只需在切换局部地图时正确初始化。
- `WorldSeed` 资源升级为 `GlobalSeed`，用于大地图与局部地图一致的重建。
- 在 `main.rs` 的状态切换中按 `GameState` 决定加载哪些系统组。

## 9. 风险与缓解策略
| 风险 | 缓解方案 |
|------|-----------|
| 内存泄漏 / 实体残留 | 构建统一的 `despawn_local_map_entities()` 工具函数 |
| IO 阻塞 | 保存/加载使用异步任务或在切换界面显示进度条 |
| 数据膨胀 | 为每个局部地图定期清理历史快照，仅保留最新状态 |
| 世界生成过慢 | 使用渐进式生成（进入局部时才生成细节） |
| 设计耦合度过高 | 通过资源/事件总线解耦，局部系统不直接访问大地图结构 |

## 10. 后续工作
- 制定精确的 `LocalMapData` 序列化格式。
- 统一资源名称与ID，确保大地图与局部地图的资源同步。
- 设计大地图 UI 原型（缩放、标记、事件提示）。
- 编写测试用例：
  - 切换 10+ 个局部地图，检查内存增长。
  - 随机生成世界后多次保存/加载，验证数据一致性。
  - 模拟不同天气、派系事件对局部地图的影响。

---

本文档将作为后续实现大地图系统的设计基线。若接下来要实现某个阶段，请在任务开始前确认细节是否需要调整。
