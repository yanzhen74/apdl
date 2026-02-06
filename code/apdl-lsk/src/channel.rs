//! 通信信道模块
//!
//! 实现仿真通信信道功能

use std::collections::VecDeque;

/// 通信信道类型
#[derive(Debug, Clone)]
pub enum ChannelType {
    PointToPoint,
    Broadcast,
    Multicast,
}

/// 通信信道结构
pub struct Channel {
    id: String,
    channel_type: ChannelType,
    buffer: VecDeque<Vec<u8>>,
    capacity: usize,
}

impl Channel {
    pub fn new(id: String, channel_type: ChannelType, capacity: usize) -> Self {
        Self {
            id,
            channel_type,
            buffer: VecDeque::new(),
            capacity,
        }
    }

    pub fn send(&mut self, data: Vec<u8>) -> Result<(), &'static str> {
        if self.buffer.len() >= self.capacity {
            return Err("Channel buffer full");
        }
        self.buffer.push_back(data);
        Ok(())
    }

    pub fn receive(&mut self) -> Option<Vec<u8>> {
        self.buffer.pop_front()
    }

    pub fn peek(&self) -> Option<&Vec<u8>> {
        self.buffer.front()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// 获取信道ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 获取信道类型
    pub fn channel_type(&self) -> &ChannelType {
        &self.channel_type
    }

    /// 获取信道容量
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
