//! 流量生成器模块
//!
//! 实现协议流量的模拟生成

/// 流量类型
#[derive(Debug, Clone)]
pub enum TrafficType {
    Constant, // 恒定速率
    Burst,    // 突发流量
    Random,   // 随机流量
    Periodic, // 周期性流量
}

/// 流量配置
#[derive(Debug, Clone)]
pub struct TrafficConfig {
    pub traffic_type: TrafficType,
    pub rate_kbps: f64,         // 速率（kbps）
    pub burst_size: usize,      // 突发大小（字节）
    pub packet_size_min: usize, // 最小包大小（字节）
    pub packet_size_max: usize, // 最大包大小（字节）
    pub interval_ms: u64,       // 发送间隔（毫秒）
}

impl Default for TrafficConfig {
    fn default() -> Self {
        Self {
            traffic_type: TrafficType::Constant,
            rate_kbps: 1.0,
            burst_size: 1000,
            packet_size_min: 64,
            packet_size_max: 1500,
            interval_ms: 100,
        }
    }
}

/// 流量生成器
pub struct TrafficGenerator {
    config: TrafficConfig,
    sequence_number: u32,
    last_generated: std::time::Instant,
}

impl TrafficGenerator {
    pub fn new(config: TrafficConfig) -> Self {
        Self {
            config,
            sequence_number: 0,
            last_generated: std::time::Instant::now(),
        }
    }

    /// 生成单个数据包
    pub fn generate_packet(&mut self) -> Vec<u8> {
        self.sequence_number += 1;

        // 根据配置生成包大小
        let packet_size = self.get_current_packet_size();

        // 创建数据包 - 包含序列号和随机数据
        let mut packet = Vec::with_capacity(packet_size);

        // 添加序列号
        packet.extend_from_slice(&self.sequence_number.to_le_bytes());

        // 添加随机数据
        let data_size = packet_size.saturating_sub(std::mem::size_of::<u32>());
        for i in 0..data_size {
            // 简单的伪随机数据生成
            packet.push(((i + self.sequence_number as usize) % 256) as u8);
        }

        packet
    }

    /// 生成一批数据包
    pub fn generate_batch(&mut self, count: usize) -> Vec<Vec<u8>> {
        let mut packets = Vec::with_capacity(count);
        for _ in 0..count {
            packets.push(self.generate_packet());
        }
        packets
    }

    /// 根据流量类型获取当前包大小
    fn get_current_packet_size(&self) -> usize {
        match self.config.traffic_type {
            TrafficType::Constant => {
                // 恒定大小，取平均值
                (self.config.packet_size_min + self.config.packet_size_max) / 2
            }
            TrafficType::Burst => {
                // 突发模式，偶尔大包
                if self.sequence_number % 10 == 0 {
                    self.config.burst_size.min(self.config.packet_size_max)
                } else {
                    self.config.packet_size_min
                }
            }
            TrafficType::Random => {
                // 随机大小
                let range = self.config.packet_size_max - self.config.packet_size_min;
                // 使用简单的伪随机算法
                let size = self.config.packet_size_min
                    + ((self.sequence_number as usize * 1103515245 + 12345) % (range + 1));
                size.clamp(self.config.packet_size_min, self.config.packet_size_max)
            }
            TrafficType::Periodic => {
                // 周期性模式
                if self.sequence_number % 5 == 0 {
                    self.config.packet_size_max
                } else {
                    self.config.packet_size_min
                }
            }
        }
    }

    /// 重置生成器状态
    pub fn reset(&mut self) {
        self.sequence_number = 0;
        self.last_generated = std::time::Instant::now();
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &TrafficConfig {
        &self.config
    }

    /// 更新配置
    pub fn set_config(&mut self, config: TrafficConfig) {
        self.config = config;
    }
}
