//! 协议仿真器模块
//!
//! 实现纯软件协议仿真功能

use apdl_core::{error::ProtocolError, ProtocolUnit};
use std::collections::HashMap;

/// 仿真器配置
#[derive(Debug, Clone)]
pub struct SimulatorConfig {
    pub error_rate: f64, // 误码率
    pub loss_rate: f64,  // 丢包率
    pub delay_ms: u64,   // 延迟（毫秒）
    pub jitter_ms: u64,  // 抖动（毫秒）
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            error_rate: 0.0,
            loss_rate: 0.0,
            delay_ms: 0,
            jitter_ms: 0,
        }
    }
}

/// 协议仿真器
pub struct ProtocolSimulator {
    units: Vec<Box<dyn ProtocolUnit>>,
    config: SimulatorConfig,
    stats: SimulationStats,
}

/// 仿真统计信息
#[derive(Debug, Clone, Default)]
pub struct SimulationStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub packets_lost: u64,
    pub errors_detected: u64,
    pub total_delay: u64,
}

impl ProtocolSimulator {
    pub fn new(config: SimulatorConfig) -> Self {
        Self {
            units: Vec::new(),
            config,
            stats: SimulationStats::default(),
        }
    }

    pub fn add_protocol_unit(&mut self, unit: Box<dyn ProtocolUnit>) {
        self.units.push(unit);
    }

    pub fn simulate_packet(&mut self, data: &[u8]) -> Result<Vec<u8>, ProtocolError> {
        // 模拟传输前的处理
        self.stats.packets_sent += 1;

        // 模拟丢包
        // 使用简单的伪随机数生成代替rand依赖
        let pseudo_random = ((std::time::SystemTime::now()
            .elapsed()
            .unwrap_or_default()
            .as_nanos() as u64)
            % 10000) as f64
            / 10000.0;
        if pseudo_random < self.config.loss_rate {
            self.stats.packets_lost += 1;
            return Err(ProtocolError::ParseError(
                "Packet lost in simulation".to_string(),
            ));
        }

        // 对数据进行协议层处理
        let mut processed_data = data.to_vec();
        for unit in &self.units {
            processed_data = unit.pack(&processed_data)?;
        }

        // 模拟传输延迟
        std::thread::sleep(std::time::Duration::from_millis(self.config.delay_ms));

        // 接收端处理
        let mut received_data = processed_data;
        for unit in self.units.iter().rev() {
            let (sdu, _remaining) = unit.unpack(&received_data)?;
            received_data = sdu;
        }

        self.stats.packets_received += 1;
        Ok(received_data)
    }

    pub fn get_stats(&self) -> &SimulationStats {
        &self.stats
    }

    pub fn reset_stats(&mut self) {
        self.stats = SimulationStats::default();
    }
}

// 为了编译暂时禁用rand依赖，使用简单模拟
// 在实际实现中，需要在Cargo.toml中添加rand依赖
