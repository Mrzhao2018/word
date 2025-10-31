/// 全局调试配置
/// 
/// 通过修改这个文件中的常量来控制各种调试输出

/// 主调试开关 - 设为 false 可以关闭所有调试输出
pub const DEBUG_ENABLED: bool = true;

/// 世界地图交互调试
pub const DEBUG_WORLD_MAP_INPUT: bool = false;

/// 世界地图选择状态调试
pub const DEBUG_WORLD_MAP_SELECTION: bool = false;

/// 地形生成调试
#[allow(dead_code)]
pub const DEBUG_TERRAIN_GENERATION: bool = false;

/// 实体生成调试
#[allow(dead_code)]
pub const DEBUG_ENTITY_SPAWN: bool = false;

/// 系统执行调试
#[allow(dead_code)]
pub const DEBUG_SYSTEM_TIMING: bool = false;

/// 调试宏 - 只在调试开关打开时输出
#[macro_export]
macro_rules! debug_log {
    ($flag:expr, $($arg:tt)*) => {
        if $crate::debug_config::DEBUG_ENABLED && $flag {
            println!("[DEBUG] {}", format!($($arg)*));
        }
    };
}

/// 世界地图输入调试宏
#[macro_export]
macro_rules! debug_world_input {
    ($($arg:tt)*) => {
        $crate::debug_log!($crate::debug_config::DEBUG_WORLD_MAP_INPUT, $($arg)*);
    };
}

/// 世界地图选择调试宏
#[macro_export]
macro_rules! debug_world_selection {
    ($($arg:tt)*) => {
        $crate::debug_log!($crate::debug_config::DEBUG_WORLD_MAP_SELECTION, $($arg)*);
    };
}

/// 地形生成调试宏
#[macro_export]
macro_rules! debug_terrain {
    ($($arg:tt)*) => {
        $crate::debug_log!($crate::debug_config::DEBUG_TERRAIN_GENERATION, $($arg)*);
    };
}

/// 实体生成调试宏
#[macro_export]
macro_rules! debug_entity {
    ($($arg:tt)*) => {
        $crate::debug_log!($crate::debug_config::DEBUG_ENTITY_SPAWN, $($arg)*);
    };
}

/// 系统时序调试宏
#[macro_export]
macro_rules! debug_timing {
    ($($arg:tt)*) => {
        $crate::debug_log!($crate::debug_config::DEBUG_SYSTEM_TIMING, $($arg)*);
    };
}
