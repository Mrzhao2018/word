use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::{rngs::SmallRng, Rng, SeedableRng};

/// 默认世界地图宽度
pub const WORLD_ATLAS_DEFAULT_WIDTH: i32 = 20;
/// 默认世界地图高度
pub const WORLD_ATLAS_DEFAULT_HEIGHT: i32 = 12;
/// 世界地图格子的渲染尺寸
pub const WORLD_ATLAS_TILE_SIZE: f32 = 48.0;

/// 宏观世界地图资源，保存大地图抽象数据
#[derive(Resource)]
pub struct WorldAtlas {
    pub width: i32,
    pub height: i32,
    #[allow(dead_code)]
    pub seed: u64,
    pub cells: Vec<WorldCell>,
}

impl Default for WorldAtlas {
    fn default() -> Self {
        let seed = rand::random();
        Self::generate(seed, WORLD_ATLAS_DEFAULT_WIDTH, WORLD_ATLAS_DEFAULT_HEIGHT)
    }
}

impl WorldAtlas {
    /// 重新生成宏观世界地图
    #[allow(dead_code)]
    pub fn regenerate(&mut self, seed: u64, width: i32, height: i32) {
        *self = Self::generate(seed, width, height);
    }

    /// 根据种子与尺寸生成宏观世界地图
    pub fn generate(seed: u64, width: i32, height: i32) -> Self {
        let elevation_noise = Perlin::new(seed as u32);
        let moisture_noise = Perlin::new((seed as u32).wrapping_add(1));
        let temperature_noise = Perlin::new((seed as u32).wrapping_add(2));
        let mut cells = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let coord = IVec2::new(x, y);
                let nx = x as f64 / width as f64 - 0.5;
                let ny = y as f64 / height as f64 - 0.5;

                let elevation = elevation_noise.get([nx * 1.5, ny * 1.5]) as f32;
                let moisture = moisture_noise.get([nx * 1.8, ny * 1.8]) as f32;
                let temperature = temperature_noise.get([nx * 1.2, ny * 1.2]) as f32;

                let biome = classify_biome(elevation, temperature, moisture);
                let mut rng = SmallRng::seed_from_u64(seed ^ ((x as u64) << 32) ^ y as u64);
                let local_seed = rng.gen::<u32>();

                cells.push(WorldCell {
                    coord,
                    biome,
                    elevation,
                    moisture,
                    temperature,
                    local_seed,
                });
            }
        }

        Self {
            width,
            height,
            seed,
            cells,
        }
    }

    /// 根据坐标获取宏观世界格子
    pub fn cell_at(&self, coord: IVec2) -> Option<&WorldCell> {
        if coord.x < 0 || coord.x >= self.width || coord.y < 0 || coord.y >= self.height {
            return None;
        }
        let index = (coord.y * self.width + coord.x) as usize;
        self.cells.get(index)
    }
}

/// 宏观世界地图中的单元格
#[derive(Clone)]
pub struct WorldCell {
    pub coord: IVec2,
    pub biome: WorldBiome,
    pub elevation: f32,
    pub moisture: f32,
    pub temperature: f32,
    pub local_seed: u32,
}

impl WorldCell {
    /// 渲染颜色
    pub fn color(&self) -> Color {
        match self.biome {
            WorldBiome::Grassland => Color::srgb(0.35, 0.7, 0.35),
            WorldBiome::Forest => Color::srgb(0.2, 0.5, 0.25),
            WorldBiome::Mountain => Color::srgb(0.55, 0.5, 0.45),
            WorldBiome::Desert => Color::srgb(0.85, 0.75, 0.4),
            WorldBiome::Tundra => Color::srgb(0.7, 0.75, 0.8),
            WorldBiome::Ocean => Color::srgb(0.15, 0.35, 0.7),
            WorldBiome::River => Color::srgb(0.2, 0.5, 0.85),
            WorldBiome::Swamp => Color::srgb(0.25, 0.45, 0.3),
        }
    }

    /// 简短标签，用于UI显示
    pub fn label(&self) -> &'static str {
        match self.biome {
            WorldBiome::Grassland => "草原",
            WorldBiome::Forest => "森林",
            WorldBiome::Mountain => "山脉",
            WorldBiome::Desert => "沙漠",
            WorldBiome::Tundra => "冻原",
            WorldBiome::Ocean => "海洋",
            WorldBiome::River => "河流",
            WorldBiome::Swamp => "沼泽",
        }
    }
}

/// 宏观世界地图的选中状态
#[derive(Resource, Default)]
pub struct AtlasSelection {
    pub selected: Option<IVec2>,
    pub hovered: Option<IVec2>,
}

/// 宏观世界地图支持的生物群落
#[derive(Clone, Copy, Debug)]
pub enum WorldBiome {
    Grassland,
    Forest,
    Mountain,
    Desert,
    Tundra,
    Ocean,
    River,
    Swamp,
}

fn classify_biome(elevation: f32, temperature: f32, moisture: f32) -> WorldBiome {
    if elevation < -0.2 {
        if moisture > 0.2 {
            WorldBiome::Swamp
        } else {
            WorldBiome::Ocean
        }
    } else if (elevation.abs() < 0.15) && moisture > 0.55 {
        WorldBiome::River
    } else if elevation > 0.6 {
        if temperature < -0.1 {
            WorldBiome::Tundra
        } else {
            WorldBiome::Mountain
        }
    } else if moisture < -0.3 {
        WorldBiome::Desert
    } else if moisture > 0.3 {
        if temperature > 0.3 {
            WorldBiome::Swamp
        } else {
            WorldBiome::Forest
        }
    } else if temperature < -0.2 {
        WorldBiome::Tundra
    } else {
        WorldBiome::Grassland
    }
}
