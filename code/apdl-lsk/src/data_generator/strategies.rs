//! 数据生成策略模块
//!
//! 提供多种数据生成策略：随机、顺序、固定值、边界值等

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// 数据生成策略枚举
#[derive(Debug, Clone)]
pub enum GenerationStrategy {
    /// 完全随机生成
    Random,
    /// 顺序递增生成
    Sequential,
    /// 固定值生成
    Fixed(Vec<u8>),
    /// 边界值生成（用于测试边界条件）
    BoundaryValues,
}

impl Default for GenerationStrategy {
    fn default() -> Self {
        Self::Random
    }
}

/// 随机数据生成策略
pub struct RandomStrategy {
    rng: StdRng,
}

impl RandomStrategy {
    /// 创建新的随机策略
    pub fn new() -> Self {
        // 使用当前时间作为种子
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// 使用指定种子创建（用于可重复测试）
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// 生成指定长度的随机字节
    pub fn generate_bytes(&mut self, length: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; length];
        self.rng.fill_bytes(&mut bytes);
        bytes
    }

    /// 生成指定范围内的随机u64值
    pub fn generate_u64_in_range(&mut self, min: u64, max: u64) -> u64 {
        if min >= max {
            return min;
        }
        let range = max - min + 1;
        let mut buf = [0u8; 8];
        self.rng.fill_bytes(&mut buf);
        let random_val = u64::from_le_bytes(buf);
        min + (random_val % range)
    }

    /// 生成指定位数的随机值
    pub fn generate_bits(&mut self, bits: usize) -> u64 {
        if bits == 0 {
            return 0;
        }
        let max_value = if bits >= 64 {
            u64::MAX
        } else {
            (1u64 << bits) - 1
        };
        self.generate_u64_in_range(0, max_value)
    }
}

impl Default for RandomStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// 顺序递增生成策略
pub struct SequentialStrategy {
    counter: u64,
    step: u64,
}

impl SequentialStrategy {
    /// 创建新的顺序策略，从0开始
    pub fn new() -> Self {
        Self { counter: 0, step: 1 }
    }

    /// 创建指定起始值和步长的策略
    pub fn with_start_and_step(start: u64, step: u64) -> Self {
        Self { counter: start, step }
    }

    /// 获取下一个值并递增
    pub fn next(&mut self) -> u64 {
        let value = self.counter;
        self.counter = self.counter.wrapping_add(self.step);
        value
    }

    /// 生成指定长度的字节序列
    pub fn generate_bytes(&mut self, length: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(length);
        for _ in 0..length {
            result.push((self.next() % 256) as u8);
        }
        result
    }

    /// 重置计数器
    pub fn reset(&mut self) {
        self.counter = 0;
    }
}

impl Default for SequentialStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// 固定值生成策略
pub struct FixedStrategy {
    value: Vec<u8>,
}

impl FixedStrategy {
    /// 创建新的固定值策略
    pub fn new(value: Vec<u8>) -> Self {
        Self { value }
    }

    /// 获取固定值
    pub fn get_value(&self) -> Vec<u8> {
        self.value.clone()
    }

    /// 获取指定长度的值（截断或填充）
    pub fn get_value_with_length(&self, length: usize) -> Vec<u8> {
        if self.value.len() >= length {
            self.value[..length].to_vec()
        } else {
            let mut result = self.value.clone();
            result.resize(length, 0);
            result
        }
    }
}

/// 边界值生成策略
/// 用于生成测试边界条件的值（最小值、最大值、0、1、-1等）
pub struct BoundaryValueStrategy {
    /// 边界值列表
    boundaries: Vec<u64>,
    /// 当前索引
    current_index: usize,
}

impl BoundaryValueStrategy {
    /// 创建默认的边界值策略
    pub fn new() -> Self {
        Self {
            boundaries: vec![
                0,                    // 零值
                1,                    // 最小正值
                u8::MAX as u64,       // u8最大值
                u16::MAX as u64,      // u16最大值
                u32::MAX as u64,      // u32最大值
                u64::MAX,             // u64最大值
            ],
            current_index: 0,
        }
    }

    /// 创建指定位数的边界值策略
    pub fn for_bits(bits: usize) -> Self {
        let max_value = if bits >= 64 {
            u64::MAX
        } else {
            (1u64 << bits) - 1
        };

        Self {
            boundaries: vec![
                0,          // 零值
                1,          // 最小正值
                max_value,  // 最大值
                max_value - 1, // 最大值-1
            ],
            current_index: 0,
        }
    }

    /// 获取下一个边界值
    pub fn next(&mut self) -> u64 {
        if self.boundaries.is_empty() {
            return 0;
        }
        let value = self.boundaries[self.current_index];
        self.current_index = (self.current_index + 1) % self.boundaries.len();
        value
    }

    /// 生成指定长度的边界值字节
    pub fn generate_bytes(&mut self, length: usize) -> Vec<u8> {
        let value = self.next();
        match length {
            1 => vec![value as u8],
            2 => vec![(value >> 8) as u8, value as u8],
            4 => vec![
                (value >> 24) as u8,
                (value >> 16) as u8,
                (value >> 8) as u8,
                value as u8,
            ],
            8 => vec![
                (value >> 56) as u8,
                (value >> 48) as u8,
                (value >> 40) as u8,
                (value >> 32) as u8,
                (value >> 24) as u8,
                (value >> 16) as u8,
                (value >> 8) as u8,
                value as u8,
            ],
            _ => {
                let mut result = Vec::with_capacity(length);
                for i in 0..length {
                    result.push(((value >> (i * 8)) & 0xFF) as u8);
                }
                result
            }
        }
    }

    /// 重置索引
    pub fn reset(&mut self) {
        self.current_index = 0;
    }
}

impl Default for BoundaryValueStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_strategy() {
        let mut strategy = RandomStrategy::new();
        let bytes = strategy.generate_bytes(10);
        assert_eq!(bytes.len(), 10);

        let value = strategy.generate_bits(8);
        assert!(value <= 255);
    }

    #[test]
    fn test_sequential_strategy() {
        let mut strategy = SequentialStrategy::new();
        assert_eq!(strategy.next(), 0);
        assert_eq!(strategy.next(), 1);
        assert_eq!(strategy.next(), 2);

        strategy.reset();
        assert_eq!(strategy.next(), 0);
    }

    #[test]
    fn test_fixed_strategy() {
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let strategy = FixedStrategy::new(data.clone());
        assert_eq!(strategy.get_value(), data);

        // 测试截断
        let truncated = strategy.get_value_with_length(2);
        assert_eq!(truncated, vec![0xDE, 0xAD]);

        // 测试填充
        let padded = strategy.get_value_with_length(6);
        assert_eq!(padded, vec![0xDE, 0xAD, 0xBE, 0xEF, 0, 0]);
    }

    #[test]
    fn test_boundary_value_strategy() {
        let mut strategy = BoundaryValueStrategy::for_bits(8);
        
        let first = strategy.next();
        let second = strategy.next();
        let third = strategy.next();
        
        // 边界值应该是：0, 1, 255, 254
        assert_eq!(first, 0);
        assert_eq!(second, 1);
        assert!(third == 255 || third == 254);

        // 测试循环
        strategy.reset();
        assert_eq!(strategy.next(), 0);
    }

    #[test]
    fn test_sequential_bytes() {
        let mut strategy = SequentialStrategy::with_start_and_step(0, 1);
        let bytes = strategy.generate_bytes(4);
        assert_eq!(bytes, vec![0, 1, 2, 3]);
    }
}
