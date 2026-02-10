//! 帧同步模块
//!
//! 实现CCSDS标准的帧同步机制

use std::collections::VecDeque;

/// 同步模式
#[derive(Debug, Clone)]
pub enum SyncMode {
    /// 固定同步字模式（如CCSDS的0xEB90）
    FixedMarker(Vec<u8>),
    /// 模式搜索（支持掩码）
    PatternSearch { pattern: Vec<u8>, mask: Vec<u8> },
    /// 伪随机序列锁定（暂不实现）
    PseudoRandomLock,
}

/// 帧同步器
#[derive(Debug, Clone)]
pub struct FrameSynchronizer {
    mode: SyncMode,
    /// 连续检测成功次数（用于锁定判断）
    lock_threshold: usize,
    /// 当前锁定状态
    is_locked: bool,
}

impl FrameSynchronizer {
    /// 创建新的帧同步器
    pub fn new(mode: SyncMode) -> Self {
        Self {
            mode,
            lock_threshold: 3, // 连续3次检测成功才认为锁定
            is_locked: false,
        }
    }

    /// 设置锁定阈值
    pub fn set_lock_threshold(&mut self, threshold: usize) {
        self.lock_threshold = threshold;
    }

    /// 在数据缓冲区中搜索同步字
    ///
    /// # 参数
    /// - `buffer`: 数据缓冲区
    ///
    /// # 返回
    /// - `Some(offset)`: 找到同步字，返回偏移量
    /// - `None`: 未找到同步字
    pub fn search_sync(&self, buffer: &VecDeque<u8>) -> Option<usize> {
        match &self.mode {
            SyncMode::FixedMarker(marker) => self.search_fixed_marker(buffer, marker),
            SyncMode::PatternSearch { pattern, mask } => self.search_pattern(buffer, pattern, mask),
            SyncMode::PseudoRandomLock => {
                // 伪随机序列锁定暂不实现
                None
            }
        }
    }

    /// 搜索固定同步字
    fn search_fixed_marker(&self, buffer: &VecDeque<u8>, marker: &[u8]) -> Option<usize> {
        if marker.is_empty() || buffer.len() < marker.len() {
            return None;
        }

        // 滑动窗口搜索
        for i in 0..=(buffer.len() - marker.len()) {
            let mut matched = true;
            for j in 0..marker.len() {
                if buffer.get(i + j) != Some(&marker[j]) {
                    matched = false;
                    break;
                }
            }
            if matched {
                return Some(i);
            }
        }

        None
    }

    /// 搜索模式（带掩码）
    fn search_pattern(&self, buffer: &VecDeque<u8>, pattern: &[u8], mask: &[u8]) -> Option<usize> {
        if pattern.is_empty() || buffer.len() < pattern.len() {
            return None;
        }

        if mask.len() != pattern.len() {
            // 掩码长度不匹配，回退到固定匹配
            return self.search_fixed_marker(buffer, pattern);
        }

        // 带掩码的滑动窗口搜索
        for i in 0..=(buffer.len() - pattern.len()) {
            let mut matched = true;
            for j in 0..pattern.len() {
                let buffer_byte = *buffer.get(i + j)?;
                let pattern_byte = pattern[j];
                let mask_byte = mask[j];

                if (buffer_byte & mask_byte) != (pattern_byte & mask_byte) {
                    matched = false;
                    break;
                }
            }
            if matched {
                return Some(i);
            }
        }

        None
    }

    /// 检查同步锁定状态
    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    /// 更新锁定状态（基于连续检测）
    pub fn update_lock_state(&mut self, sync_found: bool) {
        if sync_found {
            self.is_locked = true;
        } else {
            self.is_locked = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_marker_sync() {
        let sync = FrameSynchronizer::new(SyncMode::FixedMarker(vec![0xEB, 0x90]));

        // 测试找到同步字
        let mut buffer = VecDeque::new();
        buffer.extend(&[0x01, 0x02, 0xEB, 0x90, 0x03, 0x04]);

        let pos = sync.search_sync(&buffer);
        assert_eq!(pos, Some(2));
    }

    #[test]
    fn test_fixed_marker_not_found() {
        let sync = FrameSynchronizer::new(SyncMode::FixedMarker(vec![0xEB, 0x90]));

        // 测试未找到同步字
        let mut buffer = VecDeque::new();
        buffer.extend(&[0x01, 0x02, 0x03, 0x04]);

        let pos = sync.search_sync(&buffer);
        assert_eq!(pos, None);
    }

    #[test]
    fn test_fixed_marker_at_start() {
        let sync = FrameSynchronizer::new(SyncMode::FixedMarker(vec![0xEB, 0x90]));

        // 测试同步字在起始位置
        let mut buffer = VecDeque::new();
        buffer.extend(&[0xEB, 0x90, 0x01, 0x02]);

        let pos = sync.search_sync(&buffer);
        assert_eq!(pos, Some(0));
    }

    #[test]
    fn test_pattern_search_with_mask() {
        // 测试带掩码的模式搜索
        // 模式：0xEB 0x90，掩码：0xFF 0xF0（只匹配第二字节的高4位）
        let sync = FrameSynchronizer::new(SyncMode::PatternSearch {
            pattern: vec![0xEB, 0x90],
            mask: vec![0xFF, 0xF0],
        });

        let mut buffer = VecDeque::new();
        // 0xEB 0x9F 也应该匹配（因为掩码只检查0x90的高4位）
        buffer.extend(&[0x01, 0x02, 0xEB, 0x9F, 0x03, 0x04]);

        let pos = sync.search_sync(&buffer);
        assert_eq!(pos, Some(2));
    }

    #[test]
    fn test_ccsds_tm_sync_marker() {
        // CCSDS TM帧同步字：0x1ACFFC1D
        let sync = FrameSynchronizer::new(SyncMode::FixedMarker(vec![0x1A, 0xCF, 0xFC, 0x1D]));

        let mut buffer = VecDeque::new();
        buffer.extend(&[0x00, 0x00, 0x1A, 0xCF, 0xFC, 0x1D, 0x01, 0x02]);

        let pos = sync.search_sync(&buffer);
        assert_eq!(pos, Some(2));
    }

    #[test]
    fn test_multiple_markers() {
        // 测试多个同步字出现（应返回第一个）
        let sync = FrameSynchronizer::new(SyncMode::FixedMarker(vec![0xEB, 0x90]));

        let mut buffer = VecDeque::new();
        buffer.extend(&[0xEB, 0x90, 0x01, 0xEB, 0x90, 0x02]);

        let pos = sync.search_sync(&buffer);
        assert_eq!(pos, Some(0)); // 返回第一个
    }

    #[test]
    fn test_empty_marker() {
        let sync = FrameSynchronizer::new(SyncMode::FixedMarker(vec![]));

        let mut buffer = VecDeque::new();
        buffer.extend(&[0x01, 0x02, 0x03]);

        let pos = sync.search_sync(&buffer);
        assert_eq!(pos, None);
    }

    #[test]
    fn test_insufficient_buffer() {
        let sync = FrameSynchronizer::new(SyncMode::FixedMarker(vec![0xEB, 0x90]));

        // 缓冲区长度小于同步字长度
        let mut buffer = VecDeque::new();
        buffer.extend(&[0xEB]);

        let pos = sync.search_sync(&buffer);
        assert_eq!(pos, None);
    }
}
