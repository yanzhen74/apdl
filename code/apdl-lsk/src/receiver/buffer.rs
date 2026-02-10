//! 接收缓存实现
//!
//! 提供流式数据接收和缓存管理功能

use apdl_core::ProtocolError;
use std::collections::VecDeque;

use super::sync::FrameSynchronizer;

/// 接收缓存
///
/// 用于流式接收数据、搜索同步字、提取完整帧
pub struct ReceiveBuffer {
    /// 数据缓冲区（使用VecDeque支持高效的前端移除）
    buffer: VecDeque<u8>,
    /// 最大帧大小（避免无限增长）
    max_frame_size: usize,
    /// 帧同步器
    synchronizer: Option<FrameSynchronizer>,
}

impl ReceiveBuffer {
    /// 创建新的接收缓存
    ///
    /// # 参数
    /// - `max_frame_size`: 最大帧大小（字节）
    pub fn new(max_frame_size: usize) -> Self {
        Self {
            buffer: VecDeque::new(),
            max_frame_size,
            synchronizer: None,
        }
    }

    /// 设置帧同步器
    pub fn set_synchronizer(&mut self, synchronizer: FrameSynchronizer) {
        self.synchronizer = Some(synchronizer);
    }

    /// 追加接收数据
    ///
    /// # 参数
    /// - `data`: 新接收的数据
    pub fn append(&mut self, data: &[u8]) {
        self.buffer.extend(data);

        // 如果缓冲区超过最大值，移除最旧的数据
        while self.buffer.len() > self.max_frame_size * 2 {
            self.buffer.pop_front();
        }
    }

    /// 搜索同步字，返回同步字起始位置
    ///
    /// # 返回
    /// - `Some(offset)`: 找到同步字，返回偏移量
    /// - `None`: 未找到同步字
    pub fn find_sync_marker(&self) -> Option<usize> {
        if let Some(ref sync) = self.synchronizer {
            sync.search_sync(&self.buffer)
        } else {
            None
        }
    }

    /// 基于长度字段计算帧长度
    ///
    /// # 参数
    /// - `length_field_offset`: 长度字段的字节偏移（从帧起始位置）
    /// - `length_field_size`: 长度字段的字节大小（1, 2, 或 4）
    /// - `length_includes_header`: 长度值是否包含帧头部
    /// - `header_size`: 帧头部大小（字节）
    ///
    /// # 返回
    /// - `Some(frame_length)`: 计算出的完整帧长度
    /// - `None`: 缓冲区数据不足以读取长度字段
    pub fn calculate_frame_length(
        &self,
        length_field_offset: usize,
        length_field_size: usize,
        length_includes_header: bool,
        header_size: usize,
    ) -> Option<usize> {
        if self.buffer.len() < length_field_offset + length_field_size {
            return None;
        }

        // 读取长度字段值
        let mut length_value = 0usize;
        for i in 0..length_field_size {
            let byte_val = *self.buffer.get(length_field_offset + i)?;
            length_value = (length_value << 8) | (byte_val as usize);
        }

        // 计算完整帧长度
        let frame_length = if length_includes_header {
            length_value
        } else {
            header_size + length_value
        };

        Some(frame_length)
    }

    /// 提取完整帧
    ///
    /// # 参数
    /// - `length`: 帧长度（字节）
    ///
    /// # 返回
    /// - `Some(frame)`: 提取的完整帧数据
    /// - `None`: 缓冲区数据不足
    pub fn extract_frame(&mut self, length: usize) -> Option<Vec<u8>> {
        if self.buffer.len() < length {
            return None;
        }

        // 提取帧数据
        let frame: Vec<u8> = self.buffer.drain(..length).collect();
        Some(frame)
    }

    /// 尝试提取下一个完整帧（自动搜索同步字和计算长度）
    ///
    /// # 参数
    /// - `length_field_offset`: 长度字段偏移（相对于帧起始）
    /// - `length_field_size`: 长度字段大小
    /// - `length_includes_header`: 长度值是否包含头部
    /// - `header_size`: 头部大小
    ///
    /// # 返回
    /// - `Ok(Some(frame))`: 成功提取完整帧
    /// - `Ok(None)`: 数据不足，需要继续接收
    /// - `Err(ProtocolError)`: 同步失败或格式错误
    pub fn extract_next_frame(
        &mut self,
        length_field_offset: usize,
        length_field_size: usize,
        length_includes_header: bool,
        header_size: usize,
    ) -> Result<Option<Vec<u8>>, ProtocolError> {
        // 1. 搜索同步字
        if let Some(sync_offset) = self.find_sync_marker() {
            // 找到同步字，丢弃之前的数据
            if sync_offset > 0 {
                self.buffer.drain(..sync_offset);
            }

            // 2. 计算帧长度
            if let Some(frame_length) = self.calculate_frame_length(
                length_field_offset,
                length_field_size,
                length_includes_header,
                header_size,
            ) {
                // 验证帧长度合理性
                if frame_length > self.max_frame_size {
                    return Err(ProtocolError::InvalidFrameFormat(format!(
                        "Frame length {} exceeds maximum {}",
                        frame_length, self.max_frame_size
                    )));
                }

                // 3. 提取完整帧
                if let Some(frame) = self.extract_frame(frame_length) {
                    return Ok(Some(frame));
                }
            }
        }

        // 数据不足或未找到同步字
        Ok(None)
    }

    /// 丢弃指定长度的数据
    pub fn discard(&mut self, length: usize) {
        let actual_length = length.min(self.buffer.len());
        self.buffer.drain(..actual_length);
    }

    /// 获取当前缓冲区长度
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// 检查缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// 查看缓冲区内容（不移除）
    pub fn peek(&self, length: usize) -> Option<Vec<u8>> {
        if self.buffer.len() < length {
            return None;
        }
        Some(self.buffer.iter().take(length).copied().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receiver::sync::{FrameSynchronizer, SyncMode};

    #[test]
    fn test_receive_buffer_basic() {
        let mut buffer = ReceiveBuffer::new(1024);

        // 测试追加数据
        buffer.append(&[0x01, 0x02, 0x03]);
        assert_eq!(buffer.len(), 3);

        // 测试提取数据
        let frame = buffer.extract_frame(2).unwrap();
        assert_eq!(frame, vec![0x01, 0x02]);
        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn test_sync_marker_search() {
        let mut buffer = ReceiveBuffer::new(1024);
        let synchronizer = FrameSynchronizer::new(SyncMode::FixedMarker(vec![0xEB, 0x90]));
        buffer.set_synchronizer(synchronizer);

        // 添加包含同步字的数据
        buffer.append(&[0x01, 0x02, 0xEB, 0x90, 0x03, 0x04]);

        // 应该找到同步字在偏移2处
        let sync_pos = buffer.find_sync_marker();
        assert_eq!(sync_pos, Some(2));
    }

    #[test]
    fn test_calculate_frame_length() {
        let mut buffer = ReceiveBuffer::new(1024);

        // CCSDS TM Frame: sync(4) + frame_id(2) + length(2) + ...
        // 假设length字段在偏移6，大小2字节，值为0x0064（100字节数据）
        // 长度不包含头部，头部大小为15字节
        buffer.append(&[
            0xEB, 0x90, 0x00, 0x00, // sync marker
            0x01, 0x02, // frame_id
            0x00, 0x64, // length = 100
        ]);

        let frame_length = buffer.calculate_frame_length(6, 2, false, 15).unwrap();
        assert_eq!(frame_length, 115); // 15 + 100
    }

    #[test]
    fn test_extract_next_frame() {
        let mut buffer = ReceiveBuffer::new(1024);
        let synchronizer = FrameSynchronizer::new(SyncMode::FixedMarker(vec![0xEB, 0x90]));
        buffer.set_synchronizer(synchronizer);

        // 构造一个简单的帧：sync(2) + length(2) + data
        // length字段偏移2，大小2，不包含头部，头部4字节
        buffer.append(&[
            0xEB, 0x90, // sync
            0x00, 0x04, // length = 4 (data only)
            0x01, 0x02, 0x03, 0x04, // data
        ]);

        let frame = buffer.extract_next_frame(2, 2, false, 4).unwrap().unwrap();
        assert_eq!(frame.len(), 8); // 4 + 4
        assert_eq!(frame[0], 0xEB);
        assert_eq!(frame[1], 0x90);
    }

    #[test]
    fn test_buffer_overflow_protection() {
        let mut buffer = ReceiveBuffer::new(100);

        // 添加超过最大值2倍的数据
        for _ in 0..250 {
            buffer.append(&[0xFF]);
        }

        // 缓冲区应该被限制在最大值2倍以内
        assert!(buffer.len() <= 200);
    }

    #[test]
    fn test_peek() {
        let mut buffer = ReceiveBuffer::new(1024);
        buffer.append(&[0x01, 0x02, 0x03, 0x04, 0x05]);

        // 查看数据但不移除
        let peeked = buffer.peek(3).unwrap();
        assert_eq!(peeked, vec![0x01, 0x02, 0x03]);
        assert_eq!(buffer.len(), 5); // 长度不变
    }
}
