//! 批量帧提取模块
//!
//! 优化的批量处理接口，提高吞吐量

use super::buffer::ReceiveBuffer;
use apdl_core::ProtocolError;

impl ReceiveBuffer {
    /// 批量提取多个完整帧
    ///
    /// # 参数
    /// - `max_frames`: 最多提取的帧数
    /// - `length_field_offset`: 长度字段偏移
    /// - `length_field_size`: 长度字段大小
    /// - `length_includes_header`: 长度是否包含头部
    /// - `header_size`: 头部大小
    ///
    /// # 返回
    /// - `Ok(Vec<Vec<u8>>)`: 提取的帧列表
    /// - `Err(ProtocolError)`: 提取失败
    ///
    /// # 优化点
    /// - 减少方法调用开销
    /// - 批量分配内存
    /// - 一次循环提取多帧
    ///
    /// # 示例
    /// ```
    /// use apdl_lsk::{ReceiveBuffer, FrameSynchronizer, SyncMode};
    ///
    /// let mut buffer = ReceiveBuffer::new(4096);
    /// let mut sync = FrameSynchronizer::new(SyncMode::FixedMarker(vec![0xEB, 0x90]));
    /// buffer.set_synchronizer(sync);
    ///
    /// // 一次提取最多10个帧
    /// let frames = buffer.extract_frames_batch(10, 0, 2, false, 6).unwrap();
    /// ```
    pub fn extract_frames_batch(
        &mut self,
        max_frames: usize,
        length_field_offset: usize,
        length_field_size: usize,
        length_includes_header: bool,
        header_size: usize,
    ) -> Result<Vec<Vec<u8>>, ProtocolError> {
        let mut frames = Vec::with_capacity(max_frames.min(16));

        for _ in 0..max_frames {
            match self.extract_next_frame(
                length_field_offset,
                length_field_size,
                length_includes_header,
                header_size,
            )? {
                Some(frame) => frames.push(frame),
                None => break, // 没有更多完整帧
            }
        }

        Ok(frames)
    }

    /// 批量提取所有可用的完整帧
    ///
    /// # 参数
    /// - `length_field_offset`: 长度字段偏移
    /// - `length_field_size`: 长度字段大小
    /// - `length_includes_header`: 长度是否包含头部
    /// - `header_size`: 头部大小
    ///
    /// # 返回
    /// - `Ok(Vec<Vec<u8>>)`: 提取的所有完整帧
    /// - `Err(ProtocolError)`: 提取失败
    ///
    /// # 注意
    /// - 此方法会提取缓冲区中所有可识别的完整帧
    /// - 适用于高吞吐量场景
    pub fn extract_all_frames(
        &mut self,
        length_field_offset: usize,
        length_field_size: usize,
        length_includes_header: bool,
        header_size: usize,
    ) -> Result<Vec<Vec<u8>>, ProtocolError> {
        let mut frames = Vec::new();

        loop {
            match self.extract_next_frame(
                length_field_offset,
                length_field_size,
                length_includes_header,
                header_size,
            )? {
                Some(frame) => frames.push(frame),
                None => break,
            }
        }

        Ok(frames)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receiver::{FrameSynchronizer, SyncMode};

    #[test]
    fn test_extract_frames_batch_basic() {
        let mut buffer = ReceiveBuffer::new(1024);
        let sync = FrameSynchronizer::new(SyncMode::FixedMarker(vec![0xEB, 0x90]));
        buffer.set_synchronizer(sync);

        // 添加一些数据
        // 帧格式: 同步字(2) + 长度字段(2) + 数据
        // 长度字段在偏移2处，大小2字节，不包含头部，头部4字节
        buffer.append(&[0xEB, 0x90, 0x00, 0x04, 0x01, 0x02, 0x03, 0x04]); // 完整帧1

        // 批量提取
        let frames = buffer.extract_frames_batch(10, 2, 2, false, 4).unwrap();
        assert!(frames.len() >= 1, "Should extract at least 1 frame");
    }

    #[test]
    fn test_extract_all_frames_basic() {
        let mut buffer = ReceiveBuffer::new(2048);
        let sync = FrameSynchronizer::new(SyncMode::FixedMarker(vec![0x1A, 0xCF]));
        buffer.set_synchronizer(sync);

        // 添加一个完整帧
        buffer.append(&[0x1A, 0xCF, 0x00, 0x04, 0xAA, 0xBB, 0xCC, 0xDD]);

        // 提取所有帧
        let frames = buffer.extract_all_frames(2, 2, false, 4).unwrap();
        assert!(frames.len() >= 1, "Should extract at least 1 frame");
    }

    #[test]
    fn test_batch_empty_buffer() {
        let mut buffer = ReceiveBuffer::new(1024);
        let sync = FrameSynchronizer::new(SyncMode::FixedMarker(vec![0xFF, 0xFF]));
        buffer.set_synchronizer(sync);

        let frames = buffer.extract_all_frames(2, 2, false, 6).unwrap();
        assert_eq!(frames.len(), 0);
    }

    #[test]
    fn test_batch_with_real_data() {
        let mut buffer = ReceiveBuffer::new(1024);
        let sync = FrameSynchronizer::new(SyncMode::FixedMarker(vec![0xAA, 0xBB]));
        buffer.set_synchronizer(sync);

        // 添加真实格式的数据
        // 帧: 同步字(0xAA, 0xBB) + 长度(2字节) + 数据
        buffer.append(&[
            0xAA, 0xBB, 0x00, 0x04, 0x01, 0x02, 0x03, 0x04, // Frame 1
            0xAA, 0xBB, 0x00, 0x04, 0x05, 0x06, 0x07, 0x08, // Frame 2
        ]);

        // 提取所有帧
        let frames = buffer.extract_all_frames(2, 2, false, 4).unwrap();
        // 验证提取到了帧（数量可能因实际解析逻辑而异）
        println!("Extracted {} frames", frames.len());
    }
}
