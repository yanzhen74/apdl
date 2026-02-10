//! 乱序重排缓冲区
//!
//! 基于序列号对PDU进行排序，处理乱序接收的情况

use std::collections::BTreeMap;

/// 乱序重排缓冲区
///
/// 使用BTreeMap自动按序列号排序，提供滑动窗口管理
pub struct ReorderBuffer {
    /// 缓冲区（sequence -> PDU）
    buffer: BTreeMap<u32, Vec<u8>>,
    /// 下一个期望的序列号
    next_expected: u32,
    /// 滑动窗口大小
    window_size: usize,
    /// 序列号模数
    modulo: u32,
    /// 接收统计
    stats: ReorderStatistics,
}

/// 重排统计信息
#[derive(Debug, Clone, Default)]
pub struct ReorderStatistics {
    /// 接收的总PDU数
    pub total_received: u64,
    /// 按序输出的PDU数
    pub in_order_output: u64,
    /// 重排后输出的PDU数
    pub reordered_output: u64,
    /// 丢弃的PDU数（超出窗口）
    pub discarded: u64,
    /// 当前缓冲区大小
    pub buffer_size: usize,
}

impl ReorderBuffer {
    /// 创建新的乱序重排缓冲区
    ///
    /// # 参数
    /// - `window_size`: 滑动窗口大小（允许的最大乱序范围）
    /// - `modulo`: 序列号模数（如CCSDS的0x4000）
    ///
    /// # 示例
    /// ```
    /// use apdl_lsk::demultiplex::ReorderBuffer;
    ///
    /// // 创建窗口大小为16的重排缓冲区
    /// let buffer = ReorderBuffer::new(16, 0x4000);
    /// ```
    pub fn new(window_size: usize, modulo: u32) -> Self {
        Self {
            buffer: BTreeMap::new(),
            next_expected: 0,
            window_size,
            modulo,
            stats: ReorderStatistics::default(),
        }
    }

    /// 插入PDU并尝试输出连续序列
    ///
    /// # 参数
    /// - `sequence`: PDU的序列号
    /// - `pdu`: PDU数据
    ///
    /// # 返回
    /// - `Vec<Vec<u8>>`: 可以按序输出的PDU列表
    ///
    /// # 示例
    /// ```
    /// use apdl_lsk::demultiplex::ReorderBuffer;
    ///
    /// let mut buffer = ReorderBuffer::new(16, 0x4000);
    ///
    /// // 按序接收
    /// let output = buffer.insert(0, vec![0x01]);
    /// assert_eq!(output.len(), 1);
    ///
    /// // 乱序接收（收到序列号2，但还没收到1）
    /// let output = buffer.insert(2, vec![0x03]);
    /// assert_eq!(output.len(), 0); // 缓冲等待
    ///
    /// // 收到序列号1，触发批量输出
    /// let output = buffer.insert(1, vec![0x02]);
    /// assert_eq!(output.len(), 2); // 输出1和2
    /// ```
    pub fn insert(&mut self, sequence: u32, pdu: Vec<u8>) -> Vec<Vec<u8>> {
        self.stats.total_received += 1;

        // 检查是否是期望的序列号
        if sequence == self.next_expected {
            // 直接输出，并检查缓冲区中是否有连续的
            self.stats.in_order_output += 1;
            self.next_expected = self.increment_sequence(self.next_expected);

            let mut result = vec![pdu];
            result.extend(self.drain_ordered());
            result
        } else if self.is_in_window(sequence) {
            // 在窗口范围内，插入缓冲区
            self.buffer.insert(sequence, pdu);
            self.stats.buffer_size = self.buffer.len();

            // 检查是否超出窗口大小限制
            if self.buffer.len() > self.window_size {
                self.clean_old_entries();
            }

            vec![]
        } else {
            // 超出窗口范围，丢弃
            self.stats.discarded += 1;
            vec![]
        }
    }

    /// 提取所有连续的PDU
    fn drain_ordered(&mut self) -> Vec<Vec<u8>> {
        let mut result = Vec::new();

        while let Some(pdu) = self.buffer.remove(&self.next_expected) {
            result.push(pdu);
            self.stats.reordered_output += 1;
            self.next_expected = self.increment_sequence(self.next_expected);
        }

        self.stats.buffer_size = self.buffer.len();
        result
    }

    /// 检查序列号是否在窗口范围内
    fn is_in_window(&self, sequence: u32) -> bool {
        let distance = self.calculate_distance(self.next_expected, sequence);
        distance < self.window_size
    }

    /// 计算两个序列号之间的距离（考虑回绕）
    fn calculate_distance(&self, from: u32, to: u32) -> usize {
        if to >= from {
            (to - from) as usize
        } else {
            // 序列号回绕
            ((self.modulo - from) + to) as usize
        }
    }

    /// 递增序列号（处理回绕）
    fn increment_sequence(&self, seq: u32) -> u32 {
        (seq + 1) % self.modulo
    }

    /// 清理超出窗口的旧条目
    fn clean_old_entries(&mut self) {
        // 只保留最新的window_size个条目
        while self.buffer.len() > self.window_size {
            if let Some(&first_key) = self.buffer.keys().next() {
                self.buffer.remove(&first_key);
                self.stats.discarded += 1;
            }
        }
        self.stats.buffer_size = self.buffer.len();
    }

    /// 强制输出所有缓冲的PDU（不管顺序）
    ///
    /// 用于超时或关闭场景
    pub fn flush(&mut self) -> Vec<Vec<u8>> {
        let result: Vec<Vec<u8>> = self.buffer.values().cloned().collect();
        self.stats.reordered_output += result.len() as u64;
        self.buffer.clear();
        self.stats.buffer_size = 0;
        result
    }

    /// 重置缓冲区
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.next_expected = 0;
        self.stats = ReorderStatistics::default();
    }

    /// 设置下一个期望的序列号
    pub fn set_next_expected(&mut self, sequence: u32) {
        self.next_expected = sequence;
    }

    /// 获取当前缓冲区大小
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> &ReorderStatistics {
        &self.stats
    }

    /// 计算重排率
    pub fn get_reorder_rate(&self) -> f64 {
        if self.stats.total_received == 0 {
            0.0
        } else {
            self.stats.reordered_output as f64 / self.stats.total_received as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reorder_buffer_in_order() {
        let mut buffer = ReorderBuffer::new(16, 0x4000);

        // 按序接收
        let output = buffer.insert(0, vec![0x00]);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], vec![0x00]);

        let output = buffer.insert(1, vec![0x01]);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], vec![0x01]);

        let output = buffer.insert(2, vec![0x02]);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], vec![0x02]);

        // 验证统计
        let stats = buffer.get_statistics();
        assert_eq!(stats.in_order_output, 3);
        assert_eq!(stats.reordered_output, 0);
    }

    #[test]
    fn test_reorder_buffer_out_of_order() {
        let mut buffer = ReorderBuffer::new(16, 0x4000);

        // 先接收序列号0
        let output = buffer.insert(0, vec![0x00]);
        assert_eq!(output.len(), 1);

        // 乱序接收：先收到3
        let output = buffer.insert(3, vec![0x03]);
        assert_eq!(output.len(), 0); // 缓冲等待
        assert_eq!(buffer.buffer_size(), 1);

        // 收到2
        let output = buffer.insert(2, vec![0x02]);
        assert_eq!(output.len(), 0); // 仍然等待序列号1
        assert_eq!(buffer.buffer_size(), 2);

        // 收到1，触发批量输出1、2、3
        let output = buffer.insert(1, vec![0x01]);
        assert_eq!(output.len(), 3);
        assert_eq!(output[0], vec![0x01]);
        assert_eq!(output[1], vec![0x02]);
        assert_eq!(output[2], vec![0x03]);
        assert_eq!(buffer.buffer_size(), 0);
    }

    #[test]
    fn test_reorder_buffer_window_limit() {
        let mut buffer = ReorderBuffer::new(4, 0x4000);

        buffer.insert(0, vec![0x00]);

        // 在窗口内
        buffer.insert(2, vec![0x02]);
        buffer.insert(3, vec![0x03]);
        assert_eq!(buffer.buffer_size(), 2);

        // 超出窗口（距离为5，窗口为4）
        let output = buffer.insert(5, vec![0x05]);
        assert_eq!(output.len(), 0);

        // 验证丢弃统计
        let stats = buffer.get_statistics();
        assert_eq!(stats.discarded, 1);
    }

    #[test]
    fn test_reorder_buffer_wraparound() {
        let mut buffer = ReorderBuffer::new(16, 0x4000);

        // 接近序列号上限
        buffer.set_next_expected(0x3FFE);

        // 按序接收
        let output = buffer.insert(0x3FFE, vec![0xFE]);
        assert_eq!(output.len(), 1);

        let output = buffer.insert(0x3FFF, vec![0xFF]);
        assert_eq!(output.len(), 1);

        // 序列号回绕到0
        let output = buffer.insert(0, vec![0x00]);
        assert_eq!(output.len(), 1);

        // 乱序接收回绕后的序列
        buffer.insert(3, vec![0x03]);
        buffer.insert(2, vec![0x02]);
        let output = buffer.insert(1, vec![0x01]);
        assert_eq!(output.len(), 3); // 输出1、2、3
    }

    #[test]
    fn test_reorder_buffer_flush() {
        let mut buffer = ReorderBuffer::new(16, 0x4000);

        buffer.insert(0, vec![0x00]);

        // 缓冲一些乱序的PDU
        buffer.insert(2, vec![0x02]);
        buffer.insert(3, vec![0x03]);
        buffer.insert(5, vec![0x05]);

        assert_eq!(buffer.buffer_size(), 3);

        // 强制输出所有缓冲的PDU
        let output = buffer.flush();
        assert_eq!(output.len(), 3);
        assert_eq!(buffer.buffer_size(), 0);
    }

    #[test]
    fn test_reorder_buffer_statistics() {
        let mut buffer = ReorderBuffer::new(16, 0x4000);

        // 按序
        buffer.insert(0, vec![0x00]);
        buffer.insert(1, vec![0x01]);

        // 乱序：先收到3，再收到2（触发输出2和3）
        buffer.insert(3, vec![0x03]);
        buffer.insert(2, vec![0x02]);

        // 超出窗口
        buffer.insert(100, vec![0xFF]);

        let stats = buffer.get_statistics();
        assert_eq!(stats.total_received, 5);
        assert_eq!(stats.in_order_output, 3); // 0、1和2（2触发了输出）
        assert_eq!(stats.reordered_output, 1); // 3（被2触发输出）
        assert_eq!(stats.discarded, 1); // 100
    }

    #[test]
    fn test_reorder_buffer_reset() {
        let mut buffer = ReorderBuffer::new(16, 0x4000);

        buffer.insert(0, vec![0x00]);
        buffer.insert(2, vec![0x02]);

        assert_eq!(buffer.buffer_size(), 1);

        // 重置
        buffer.reset();
        assert_eq!(buffer.buffer_size(), 0);
        assert_eq!(buffer.get_statistics().total_received, 0);

        // 重置后应该从0开始
        let output = buffer.insert(0, vec![0x00]);
        assert_eq!(output.len(), 1);
    }

    #[test]
    fn test_reorder_rate_calculation() {
        let mut buffer = ReorderBuffer::new(16, 0x4000);

        // 完全按序
        buffer.insert(0, vec![0x00]);
        buffer.insert(1, vec![0x01]);
        buffer.insert(2, vec![0x02]);

        // 重排率应该为0
        assert_eq!(buffer.get_reorder_rate(), 0.0);

        // 乱序接收：先收到4，再收到3（触发输出3和4）
        buffer.insert(4, vec![0x04]);
        buffer.insert(3, vec![0x03]);

        // 实际统计：in_order=4（0,1,2,3）, reordered=1（4）, total=5
        // 重排率 = 1/5 = 0.2
        let rate = buffer.get_reorder_rate();
        assert!((rate - 0.2).abs() < 0.01);
    }
}
