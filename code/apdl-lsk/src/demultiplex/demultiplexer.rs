//! 解复接器实现
//!
//! 根据VCID/APID等标识将复接流分离为多路独立数据流

use std::collections::{HashMap, VecDeque};
use apdl_core::ProtocolError;

use super::sequence_validator::{SequenceValidator, ValidationResult};

/// 虚拟通道状态
#[derive(Debug, Clone)]
pub struct ChannelState {
    /// 通道ID（VCID或APID）
    pub channel_id: u16,
    /// 接收的帧总数
    pub frame_count: u64,
    /// 丢失的帧总数
    pub lost_frame_count: u64,
    /// 最后接收的序列号
    pub last_sequence: Option<u16>,
    /// 通道是否激活
    pub is_active: bool,
    /// 最后接收时间戳（可选）
    pub last_received_time: Option<std::time::SystemTime>,
}

impl ChannelState {
    /// 创建新的通道状态
    pub fn new(channel_id: u16) -> Self {
        Self {
            channel_id,
            frame_count: 0,
            lost_frame_count: 0,
            last_sequence: None,
            is_active: false,
            last_received_time: None,
        }
    }

    /// 更新接收统计
    pub fn update_receive(&mut self, sequence: u16, lost_count: u64) {
        self.frame_count += 1;
        self.lost_frame_count += lost_count;
        self.last_sequence = Some(sequence);
        self.is_active = true;
        self.last_received_time = Some(std::time::SystemTime::now());
    }

    /// 获取丢帧率
    pub fn get_loss_rate(&self) -> f64 {
        if self.frame_count == 0 {
            0.0
        } else {
            self.lost_frame_count as f64 / (self.frame_count + self.lost_frame_count) as f64
        }
    }
}

/// 解复接器
///
/// 根据虚拟通道ID（VCID）或应用进程ID（APID）将复接流分离为多路
pub struct Demultiplexer {
    /// 各通道的PDU队列（channel_id -> PDU queue）
    channels: HashMap<u16, VecDeque<Vec<u8>>>,
    /// 各通道的状态
    channel_states: HashMap<u16, ChannelState>,
    /// 序列号校验器（每个通道独立）
    sequence_validators: HashMap<u16, SequenceValidator>,
    /// 每个通道的最大队列长度
    max_queue_size: usize,
}

impl Demultiplexer {
    /// 创建新的解复接器
    ///
    /// # 参数
    /// - `max_queue_size`: 每个通道的最大队列长度
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            channels: HashMap::new(),
            channel_states: HashMap::new(),
            sequence_validators: HashMap::new(),
            max_queue_size,
        }
    }

    /// 根据通道ID分发帧到对应通道
    ///
    /// # 参数
    /// - `channel_id`: 通道ID（VCID或APID）
    /// - `sequence`: 帧序列号
    /// - `frame`: 帧数据
    ///
    /// # 返回
    /// - `Ok(ValidationResult)`: 分发成功，返回序列号校验结果
    /// - `Err(ProtocolError)`: 分发失败
    pub fn demultiplex(
        &mut self,
        channel_id: u16,
        sequence: u16,
        frame: Vec<u8>,
    ) -> Result<ValidationResult, ProtocolError> {
        // 获取或创建通道队列
        let queue = self.channels.entry(channel_id).or_insert_with(VecDeque::new);

        // 检查队列是否已满
        if queue.len() >= self.max_queue_size {
            return Err(ProtocolError::Other(format!(
                "Channel {} queue is full (max: {})",
                channel_id, self.max_queue_size
            )));
        }

        // 获取或创建序列号校验器
        let validator = self
            .sequence_validators
            .entry(channel_id)
            .or_insert_with(|| SequenceValidator::new(0x4000)); // CCSDS默认14位序列号

        // 验证序列号
        let validation_result = validator.validate(channel_id, sequence);

        // 统计丢失帧数
        let lost_count = match &validation_result {
            ValidationResult::FrameLost(count) => *count as u64,
            _ => 0,
        };

        // 更新通道状态
        let state = self
            .channel_states
            .entry(channel_id)
            .or_insert_with(|| ChannelState::new(channel_id));
        state.update_receive(sequence, lost_count);

        // 将帧加入队列
        queue.push_back(frame);

        Ok(validation_result)
    }

    /// 从指定通道提取PDU
    ///
    /// # 参数
    /// - `channel_id`: 通道ID
    ///
    /// # 返回
    /// - `Some(pdu)`: 成功提取PDU
    /// - `None`: 通道为空或不存在
    pub fn extract_pdu(&mut self, channel_id: u16) -> Option<Vec<u8>> {
        self.channels.get_mut(&channel_id)?.pop_front()
    }

    /// 获取通道状态
    ///
    /// # 参数
    /// - `channel_id`: 通道ID
    ///
    /// # 返回
    /// - `Some(&ChannelState)`: 通道状态
    /// - `None`: 通道不存在
    pub fn get_channel_state(&self, channel_id: u16) -> Option<&ChannelState> {
        self.channel_states.get(&channel_id)
    }

    /// 获取通道中待处理的PDU数量
    pub fn get_queue_length(&self, channel_id: u16) -> usize {
        self.channels.get(&channel_id).map_or(0, |q| q.len())
    }

    /// 获取所有活跃通道的ID列表
    pub fn get_active_channels(&self) -> Vec<u16> {
        self.channel_states
            .iter()
            .filter(|(_, state)| state.is_active)
            .map(|(id, _)| *id)
            .collect()
    }

    /// 清空指定通道的队列
    pub fn clear_channel(&mut self, channel_id: u16) {
        if let Some(queue) = self.channels.get_mut(&channel_id) {
            queue.clear();
        }
    }

    /// 重置通道状态
    pub fn reset_channel(&mut self, channel_id: u16) {
        if let Some(state) = self.channel_states.get_mut(&channel_id) {
            *state = ChannelState::new(channel_id);
        }
        self.clear_channel(channel_id);
        self.sequence_validators.remove(&channel_id);
    }

    /// 获取所有通道的统计信息
    pub fn get_statistics(&self) -> HashMap<u16, ChannelStatistics> {
        let mut stats = HashMap::new();
        for (channel_id, state) in &self.channel_states {
            stats.insert(
                *channel_id,
                ChannelStatistics {
                    channel_id: *channel_id,
                    frame_count: state.frame_count,
                    lost_frame_count: state.lost_frame_count,
                    loss_rate: state.get_loss_rate(),
                    queue_length: self.get_queue_length(*channel_id),
                },
            );
        }
        stats
    }
}

/// 通道统计信息
#[derive(Debug, Clone)]
pub struct ChannelStatistics {
    pub channel_id: u16,
    pub frame_count: u64,
    pub lost_frame_count: u64,
    pub loss_rate: f64,
    pub queue_length: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demultiplexer_basic() {
        let mut demux = Demultiplexer::new(100);

        // 向通道0发送帧
        let result = demux.demultiplex(0, 0, vec![0x01, 0x02, 0x03]).unwrap();
        assert!(matches!(result, ValidationResult::Ok));

        // 验证队列长度
        assert_eq!(demux.get_queue_length(0), 1);

        // 提取PDU
        let pdu = demux.extract_pdu(0).unwrap();
        assert_eq!(pdu, vec![0x01, 0x02, 0x03]);
        assert_eq!(demux.get_queue_length(0), 0);
    }

    #[test]
    fn test_demultiplexer_multiple_channels() {
        let mut demux = Demultiplexer::new(100);

        // 向不同通道发送数据
        demux.demultiplex(0, 0, vec![0xA0]).unwrap();
        demux.demultiplex(1, 0, vec![0xB0]).unwrap();
        demux.demultiplex(2, 0, vec![0xC0]).unwrap();

        // 验证通道独立性
        assert_eq!(demux.get_queue_length(0), 1);
        assert_eq!(demux.get_queue_length(1), 1);
        assert_eq!(demux.get_queue_length(2), 1);

        // 验证数据正确性
        assert_eq!(demux.extract_pdu(0).unwrap()[0], 0xA0);
        assert_eq!(demux.extract_pdu(1).unwrap()[0], 0xB0);
        assert_eq!(demux.extract_pdu(2).unwrap()[0], 0xC0);
    }

    #[test]
    fn test_sequence_validation() {
        let mut demux = Demultiplexer::new(100);

        // 正常序列
        let result = demux.demultiplex(0, 0, vec![0x01]).unwrap();
        assert!(matches!(result, ValidationResult::Ok));

        let result = demux.demultiplex(0, 1, vec![0x02]).unwrap();
        assert!(matches!(result, ValidationResult::Ok));

        // 跳过序列号2，直接到3（丢失1帧）
        let result = demux.demultiplex(0, 3, vec![0x03]).unwrap();
        assert!(matches!(result, ValidationResult::FrameLost(1)));

        // 检查通道状态
        let state = demux.get_channel_state(0).unwrap();
        assert_eq!(state.frame_count, 3);
        assert_eq!(state.lost_frame_count, 1);
    }

    #[test]
    fn test_channel_statistics() {
        let mut demux = Demultiplexer::new(100);

        // 发送多个帧
        for i in 0..10u16 {
            demux.demultiplex(0, i, vec![i as u8]).unwrap();
        }

        // 跳过几个序列号（模拟丢帧）
        demux.demultiplex(0, 15, vec![0xFF]).unwrap();

        let stats = demux.get_statistics();
        let channel_0_stats = stats.get(&0).unwrap();

        assert_eq!(channel_0_stats.frame_count, 11);
        assert_eq!(channel_0_stats.lost_frame_count, 5); // 丢失10-14共5帧
        assert!(channel_0_stats.loss_rate > 0.0);
    }

    #[test]
    fn test_queue_overflow() {
        let mut demux = Demultiplexer::new(3); // 最大队列长度为3

        // 填满队列
        demux.demultiplex(0, 0, vec![0x01]).unwrap();
        demux.demultiplex(0, 1, vec![0x02]).unwrap();
        demux.demultiplex(0, 2, vec![0x03]).unwrap();

        // 尝试再添加（应该失败）
        let result = demux.demultiplex(0, 3, vec![0x04]);
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_and_reset() {
        let mut demux = Demultiplexer::new(100);

        // 添加数据
        demux.demultiplex(0, 0, vec![0x01]).unwrap();
        demux.demultiplex(0, 1, vec![0x02]).unwrap();

        assert_eq!(demux.get_queue_length(0), 2);

        // 清空队列
        demux.clear_channel(0);
        assert_eq!(demux.get_queue_length(0), 0);

        // 重置通道（包括状态和序列号校验器）
        demux.demultiplex(0, 5, vec![0x03]).unwrap();
        demux.reset_channel(0);

        let state = demux.get_channel_state(0).unwrap();
        assert_eq!(state.frame_count, 0);
    }

    #[test]
    fn test_active_channels() {
        let mut demux = Demultiplexer::new(100);

        // 向不同通道发送数据
        demux.demultiplex(0, 0, vec![0x01]).unwrap();
        demux.demultiplex(2, 0, vec![0x02]).unwrap();
        demux.demultiplex(5, 0, vec![0x03]).unwrap();

        let active = demux.get_active_channels();
        assert_eq!(active.len(), 3);
        assert!(active.contains(&0));
        assert!(active.contains(&2));
        assert!(active.contains(&5));
    }
}
