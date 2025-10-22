use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;
use crate::components::GridPosition;
use bevy::prelude::*;
use crate::components::Terrain;
use crate::world::*;

/// A*寻路节点
#[derive(Clone, Eq, PartialEq)]
struct PathNode {
    position: (i32, i32),
    g_cost: i32,  // 从起点到当前点的实际代价
    h_cost: i32,  // 从当前点到终点的启发式代价
    parent: Option<(i32, i32)>,
}

impl PathNode {
    fn f_cost(&self) -> i32 {
        self.g_cost + self.h_cost
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // 反转比较，因为BinaryHeap是最大堆，我们需要最小堆
        other.f_cost().cmp(&self.f_cost())
            .then_with(|| other.h_cost.cmp(&self.h_cost))
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// 曼哈顿距离启发式函数
fn heuristic(from: (i32, i32), to: (i32, i32)) -> i32 {
    (from.0 - to.0).abs() + (from.1 - to.1).abs()
}

/// 获取邻居节点（只允许4个正交方向，禁用对角线）
fn get_neighbors(pos: (i32, i32)) -> Vec<(i32, i32)> {
    vec![
        // 只有四个主方向（上下左右）
        (pos.0 + 1, pos.1),   // 右
        (pos.0 - 1, pos.1),   // 左
        (pos.0, pos.1 + 1),   // 上
        (pos.0, pos.1 - 1),   // 下
    ]
}

/// 检查位置是否可行走
fn is_walkable(pos: (i32, i32), terrain_query: &Query<(&GridPosition, &Terrain)>) -> bool {
    // 边界检查
    if pos.0 < 0 || pos.0 >= WORLD_WIDTH || pos.1 < 0 || pos.1 >= WORLD_HEIGHT {
        return false;
    }
    
    // 地形检查
    for (terrain_pos, terrain) in terrain_query.iter() {
        if terrain_pos.x == pos.0 && terrain_pos.y == pos.1 {
            return terrain.walkable;
        }
    }
    
    false
}

/// A*寻路算法实现
/// 返回从start到goal的路径（不包含起点）
pub fn find_path(
    start: (i32, i32),
    goal: (i32, i32),
    terrain_query: &Query<(&GridPosition, &Terrain)>,
) -> Option<Vec<(i32, i32)>> {
    // 检查目标是否可达
    if !is_walkable(goal, terrain_query) {
        return None;
    }
    
    // 如果已经在目标位置
    if start == goal {
        return Some(Vec::new());
    }
    
    let mut open_set = BinaryHeap::new();
    let mut closed_set = HashSet::new();
    let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut g_scores: HashMap<(i32, i32), i32> = HashMap::new();
    
    // 初始化起点
    g_scores.insert(start, 0);
    open_set.push(PathNode {
        position: start,
        g_cost: 0,
        h_cost: heuristic(start, goal),
        parent: None,
    });
    
    // 最大迭代次数，防止死循环
    let mut iterations = 0;
    let max_iterations = 1000;
    
    while let Some(current) = open_set.pop() {
        iterations += 1;
        if iterations > max_iterations {
            return None; // 超时，认为不可达
        }
        
        let current_pos = current.position;
        
        // 到达目标
        if current_pos == goal {
            // 重建路径
            let mut path = Vec::new();
            let mut current = goal;
            
            while let Some(&parent) = came_from.get(&current) {
                path.push(current);
                current = parent;
            }
            
            path.reverse();
            return Some(path);
        }
        
        // 已经访问过
        if closed_set.contains(&current_pos) {
            continue;
        }
        
        closed_set.insert(current_pos);
        
        // 检查所有邻居
        for neighbor in get_neighbors(current_pos) {
            if closed_set.contains(&neighbor) {
                continue;
            }
            
            if !is_walkable(neighbor, terrain_query) {
                continue;
            }
            
            // 计算移动代价（只有正交移动，统一代价10）
            let move_cost = 10;
            
            let tentative_g = g_scores.get(&current_pos).unwrap_or(&i32::MAX) + move_cost;
            let neighbor_g = *g_scores.get(&neighbor).unwrap_or(&i32::MAX);
            
            if tentative_g < neighbor_g {
                // 找到更好的路径
                came_from.insert(neighbor, current_pos);
                g_scores.insert(neighbor, tentative_g);
                
                open_set.push(PathNode {
                    position: neighbor,
                    g_cost: tentative_g,
                    h_cost: heuristic(neighbor, goal),
                    parent: Some(current_pos),
                });
            }
        }
    }
    
    // 没有找到路径
    None
}

/// 简化路径：只移除完全冗余的中间点（必须是相邻格子且方向一致）
/// 这个版本更保守，确保简化后的路径点仍然是逐步相邻的
/// 例如：(0,0) -> (1,0) -> (2,0) -> (3,0) 简化为 (0,0) -> (3,0)
pub fn simplify_path(path: Vec<(i32, i32)>) -> Vec<(i32, i32)> {
    if path.len() <= 2 {
        return path;
    }
    
    let mut simplified = Vec::new();
    simplified.push(path[0]);
    
    let mut i = 0;
    while i < path.len() - 1 {
        // 获取当前方向
        let current = path[i];
        let next = path[i + 1];
        let dir_x = (next.0 - current.0).signum();
        let dir_y = (next.1 - current.1).signum();
        
        // 找到沿着同一方向的最远点（但必须保证每一步都相邻）
        let mut j = i + 1;
        while j < path.len() {
            let step = path[j];
            let step_dir_x = (step.0 - path[j-1].0).signum();
            let step_dir_y = (step.1 - path[j-1].1).signum();
            
            // 检查是否是相邻格子（曼哈顿距离必须为1）
            let dx = (step.0 - path[j-1].0).abs();
            let dy = (step.1 - path[j-1].1).abs();
            let is_adjacent = (dx == 1 && dy == 0) || (dx == 0 && dy == 1);
            
            // 检查方向是否一致
            let same_direction = step_dir_x == dir_x && step_dir_y == dir_y;
            
            if !is_adjacent || !same_direction {
                break;
            }
            
            j += 1;
        }
        
        // 添加这段直线的终点
        simplified.push(path[j - 1]);
        i = j - 1;
    }
    
    simplified
}
