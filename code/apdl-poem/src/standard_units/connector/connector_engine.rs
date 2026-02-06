//! 连接器引擎实现
//!
//! 负责执行字段映射规则，将源包的字段值映射到目标包的字段

use super::{
    data_structures::{ChildPacketData, MultiplexQueue},
    field_mapping, packet_builder_direct, packet_builder_mpdu, packet_builder_stream,
};
use crate::standard_units::frame_assembler::core::FrameAssembler;
use apdl_core::{DataPlacementConfig, DataPlacementStrategy};
use std::collections::{HashMap, VecDeque};

/// 连接器引擎
pub struct ConnectorEngine {
    /// 子包缓存队列 - 按分发标志分类
    child_packet_queues: HashMap<String, MultiplexQueue>,
    /// 轮询索引，用于在多个队列间轮询
    round_robin_index: std::sync::atomic::AtomicUsize,
}

impl ConnectorEngine {
    /// 创建新的连接器引擎
    pub fn new() -> Self {
        Self {
            child_packet_queues: HashMap::new(),
            round_robin_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// 应用字段映射规则到FrameAssembler
    pub fn apply_field_mapping_rules(
        &self,
        source_assembler: &FrameAssembler,
        target_assembler: &mut FrameAssembler,
        channel: &str,
        mappings: &[apdl_core::FieldMappingEntry],
    ) -> Result<String, Box<dyn std::error::Error>> {
        field_mapping::apply_field_mapping_rules(
            source_assembler,
            target_assembler,
            channel,
            mappings,
        )
    }

    /// 执行完整的连接操作，包括字段映射和数据放置
    pub fn connect(
        &mut self,
        source_assembler: &mut FrameAssembler,
        target_assembler: &mut FrameAssembler,
        channel: &str,
        connector_config: &apdl_core::ConnectorConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. 应用字段映射
        let dispatch_flag = self.apply_field_mapping_rules(
            source_assembler,
            target_assembler,
            channel,
            &connector_config.mappings,
        )?;

        // 2. 根据dispatch_flag进行分路
        if !dispatch_flag.is_empty() {
            println!("Dispatch flag: {}", dispatch_flag);
        }
        let child_data = ChildPacketData {
            assembler: source_assembler.clone(),
            _packet_type: dispatch_flag.clone(),
        };
        let child_packet_queue = self
            .child_packet_queues
            .entry(dispatch_flag.clone())
            .or_insert_with(|| MultiplexQueue {
                child_packet_queue: VecDeque::new(),
                parent_assembler: target_assembler.clone(),
                remaining_child_data: Vec::new(),
                _packet_type: dispatch_flag.clone(),
            });
        child_packet_queue.child_packet_queue.push_back(child_data);

        Ok(())
    }

    /// 添加子包到指定类型的队列
    pub fn add_child_packet(&mut self, parent_type: &str, assembler: FrameAssembler) {
        let child_data = ChildPacketData {
            assembler,
            _packet_type: parent_type.to_string(),
        };
        let queue_item = self
            .child_packet_queues
            .entry(parent_type.to_string())
            .or_insert_with(|| MultiplexQueue {
                child_packet_queue: VecDeque::new(),
                parent_assembler: FrameAssembler::new(),
                remaining_child_data: Vec::new(),
                _packet_type: parent_type.to_string(),
            });
        queue_item.child_packet_queue.push_back(child_data);
    }

    /// 构建包 - 统一接口，根据数据放置配置选择合适的构建策略
    /// 支持轮询调度，返回(包数据, dispatch_flag)
    pub fn build_packet(
        &mut self,
        placement_config: &DataPlacementConfig,
    ) -> Option<(Vec<u8>, String)> {
        // 获取所有可用的dispatch_flag
        let dispatch_flags: Vec<String> = self.child_packet_queues.keys().cloned().collect();
        if dispatch_flags.is_empty() {
            return None;
        }

        // 使用轮询索引选择下一个队列
        let index = self
            .round_robin_index
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % dispatch_flags.len();
        let selected_dispatch_flag = &dispatch_flags[index];

        match placement_config.strategy {
            DataPlacementStrategy::PointerBased => {
                // 使用MPDU策略构建包
                self.build_mpdu_packet_internal(selected_dispatch_flag, placement_config)
                    .map(|packet| (packet, selected_dispatch_flag.clone()))
            }
            DataPlacementStrategy::Direct => {
                // 直接放置策略：从队列中取出子包直接作为结果
                packet_builder_direct::build_direct_packet(
                    &mut self.child_packet_queues,
                    selected_dispatch_flag,
                )
                .map(|packet| (packet, selected_dispatch_flag.clone()))
            }
            DataPlacementStrategy::StreamBased => {
                // 流式放置策略：类似MPDU但不使用指针
                packet_builder_stream::build_stream_packet(
                    &mut self.child_packet_queues,
                    selected_dispatch_flag,
                )
                .map(|packet| (packet, selected_dispatch_flag.clone()))
            }
            DataPlacementStrategy::Custom(_) => {
                // 自定义策略，暂时返回None
                None
            }
        }
    }

    /// 构建MPDU包 - 从子包队列中取出数据填充到父包（内部方法）
    fn build_mpdu_packet_internal(
        &mut self,
        parent_type: &str,
        mpdu_config: &DataPlacementConfig,
    ) -> Option<Vec<u8>> {
        match self.build_mpdu_packet(parent_type, mpdu_config) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error building MPDU packet: {}", e);
                None
            }
        }
    }

    /// 从指定类型的队列中构建一个完整的MPDU包
    pub fn build_mpdu_packet(
        &mut self,
        dispatch_flag: &str,
        mpdu_config: &DataPlacementConfig,
    ) -> Result<Option<Vec<u8>>, String> {
        packet_builder_mpdu::build_mpdu_packet(
            &mut self.child_packet_queues,
            dispatch_flag,
            mpdu_config,
        )
    }
}

impl Default for ConnectorEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connector_engine_creation() {
        let _engine = ConnectorEngine::new();
    }
}
