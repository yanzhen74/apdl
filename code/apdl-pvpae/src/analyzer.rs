//! 性能分析器模块
//!
//! 实现协议性能统计分析功能

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub processing_time: Duration, // 处理时间
    pub throughput: f64,           // 吞吐量 (pps - 包/秒)
    pub latency: Duration,         // 延迟
    pub utilization: f64,          // 链路利用率 (%)
    pub error_rate: f64,           // 错误率 (%)
    pub packet_loss_rate: f64,     // 丢包率 (%)
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            processing_time: Duration::new(0, 0),
            throughput: 0.0,
            latency: Duration::new(0, 0),
            utilization: 0.0,
            error_rate: 0.0,
            packet_loss_rate: 0.0,
        }
    }
}

/// 性能分析器
pub struct PerformanceAnalyzer {
    metrics: HashMap<String, PerformanceMetrics>,
    start_time: Option<Instant>,
    total_processed: usize,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            start_time: None,
            total_processed: 0,
        }
    }

    /// 开始性能测量
    pub fn start_measurement(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// 记录处理事件
    pub fn record_processing_event(&mut self, event_name: &str, duration: Duration) {
        let metrics = self
            .metrics
            .entry(event_name.to_string())
            .or_insert_with(|| PerformanceMetrics::default());

        metrics.processing_time = duration;
        self.total_processed += 1;
    }

    /// 计算吞吐量
    pub fn calculate_throughput(&mut self, packet_count: usize) -> f64 {
        if let Some(start_time) = self.start_time {
            let elapsed = start_time.elapsed();
            if elapsed.as_secs_f64() > 0.0 {
                packet_count as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// 获取分析结果
    pub fn get_analysis_results(&self) -> &HashMap<String, PerformanceMetrics> {
        &self.metrics
    }

    /// 重置分析器
    pub fn reset(&mut self) {
        self.metrics.clear();
        self.start_time = None;
        self.total_processed = 0;
    }
}
