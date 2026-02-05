//! 连接器引擎实现
//!
//! 负责执行字段映射规则，将源包的字段值映射到目标包的字段

use apdl_core::{DataPlacementConfig, DataPlacementStrategy, FieldMappingEntry};

use crate::standard_units::connector::mpdu_manager::MpduManager;
use crate::standard_units::frame_assembler::core::FrameAssembler;

use std::collections::HashMap;
use std::collections::VecDeque;

/// 子包数据结构
struct ChildPacketData {
    /// 子包组装器
    assembler: FrameAssembler,
    /// 包类型标识
    _packet_type: String,
}

/// 连接器引擎
pub struct ConnectorEngine {
    /// MPDU管理器
    mpdu_manager: MpduManager,
    /// 子包缓存队列 - 按父包类型分类
    child_packet_queues: HashMap<String, VecDeque<ChildPacketData>>,
}

impl ConnectorEngine {
    /// 创建新的连接器引擎
    pub fn new() -> Self {
        Self {
            mpdu_manager: MpduManager::new(),
            child_packet_queues: HashMap::new(),
        }
    }

    /// 应用映射逻辑
    fn apply_mapping_logic(
        &self,
        source_value: &[u8],
        mapping_logic: &str,
        default_value: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match mapping_logic {
            "identity" => Ok(source_value.to_vec()),
            "hash_mod_64" => {
                // 简单的哈希实现
                let hash_value = self.simple_hash(source_value);
                let result = hash_value % 64;
                Ok(vec![(result & 0xFF) as u8])
            }
            "hash_mod_2048" => {
                // 用于APID的哈希实现
                let hash_value = self.simple_hash(source_value);
                let result = hash_value % 2048;
                Ok(vec![((result >> 8) & 0xFF) as u8, (result & 0xFF) as u8])
            }
            _ => {
                // 如果映射逻辑无法识别，使用默认值
                self.parse_default_value(default_value)
            }
        }
    }

    /// 简单的哈希函数
    fn simple_hash(&self, data: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    /// 解析默认值
    fn parse_default_value(
        &self,
        default_value: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if let Some(hex_str) = default_value.strip_prefix("0x") {
            let value = u64::from_str_radix(hex_str, 16)
                .map_err(|_| format!("Invalid hex value: {default_value}"))?;
            Ok(value.to_be_bytes().to_vec())
        } else {
            let value = default_value
                .parse::<u64>()
                .map_err(|_| format!("Invalid decimal value: {default_value}"))?;
            Ok(value.to_be_bytes().to_vec())
        }
    }

    /// 应用字段映射规则到FrameAssembler
    pub fn apply_field_mapping_rules(
        &self,
        source_assembler: &FrameAssembler,
        target_assembler: &mut FrameAssembler,
        mappings: &[FieldMappingEntry],
    ) -> Result<(), Box<dyn std::error::Error>> {
        for mapping in mappings {
            // 获取源字段值
            if let Ok(source_value) = source_assembler.get_field_value(&mapping.source_field) {
                // 应用映射逻辑
                let mapped_value = self.apply_mapping_logic(
                    &source_value,
                    &mapping.mapping_logic,
                    &mapping.default_value,
                )?;

                // 设置目标字段值
                target_assembler
                    .set_field_value(&mapping.target_field, &mapped_value)
                    .map_err(|e| Box::new(e))?;
                println!(
                    "Mapped {} to {} with value {:?} using logic {}",
                    mapping.source_field, mapping.target_field, source_value, mapping.mapping_logic
                );
            }
        }
        Ok(())
    }

    /// 执行完整的连接操作，包括字段映射和数据放置
    pub fn connect(
        &mut self,
        source_assembler: &mut FrameAssembler,
        target_assembler: &mut FrameAssembler,
        connector_config: &apdl_core::ConnectorConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. 应用字段映射
        self.apply_field_mapping_rules(
            source_assembler,
            target_assembler,
            &connector_config.mappings,
        )?;

        // 2. 应用数据放置策略
        if let Some(data_placement) = &connector_config.data_placement {
            println!(
                "Applying data placement strategy: {:?}",
                data_placement.strategy
            );

            match data_placement.strategy {
                DataPlacementStrategy::PointerBased => {
                    // 对于MPDU模式，使用MPDU管理器进行处理
                    // 将源包添加到相应的队列中
                    let source_frame =
                        source_assembler.assemble_frame().map_err(|e| Box::new(e))?;

                    // 使用目标包类型作为队列标识符
                    let parent_type = &data_placement.target_field; // 使用目标字段名作为父包类型标识
                    self.mpdu_manager
                        .add_child_packet(parent_type, source_frame.clone());

                    println!(
                        "Added source frame ({} bytes) to MPDU queue for type '{}'",
                        source_frame.len(),
                        parent_type
                    );

                    // 如果需要立即构建MPDU包（在某些场景下），可以从队列中构建
                    // 这里我们只是将子包加入队列，实际构建在build_mpdu_packet方法中进行
                }
                _ => {
                    // 其他策略的通用处理
                    let source_frame =
                        source_assembler.assemble_frame().map_err(|e| Box::new(e))?;

                    // 将源包数据嵌入到目标包的数据字段
                    target_assembler
                        .set_field_value(&data_placement.target_field, &source_frame)
                        .map_err(|e| Box::new(e))?;
                    println!(
                        "Embedded source frame ({} bytes) into target data field",
                        source_frame.len()
                    );
                }
            }
        }

        Ok(())
    }

    /// 构建MPDU包 - 从子包队列中取出数据填充到父包

    /// 添加子包到指定类型的队列
    pub fn add_child_packet(&mut self, parent_type: &str, assembler: FrameAssembler) {
        let child_data = ChildPacketData {
            assembler,
            _packet_type: parent_type.to_string(),
        };
        self.child_packet_queues
            .entry(parent_type.to_string())
            .or_insert_with(VecDeque::new)
            .push_back(child_data);
    }

    /// 获取指定类型子包队列的长度
    pub fn get_child_queue_length(&self, parent_type: &str) -> usize {
        self.mpdu_manager.get_child_queue_length(parent_type)
    }

    /// 获取指定类型父包模板队列的长度
    pub fn get_parent_queue_length(&self, parent_type: &str) -> usize {
        self.mpdu_manager.get_parent_queue_length(parent_type)
    }

    /// 构建包 - 统一接口，根据数据放置配置选择合适的构建策略
    pub fn build_packet(
        &mut self,
        parent_type: &str,
        placement_config: &DataPlacementConfig,
    ) -> Option<Vec<u8>> {
        match placement_config.strategy {
            DataPlacementStrategy::PointerBased => {
                // 使用MPDU策略构建包
                self.build_mpdu_packet_internal(parent_type, placement_config)
            }
            DataPlacementStrategy::Direct => {
                // 直接放置策略：从队列中取出子包直接作为结果
                self.build_direct_packet(parent_type)
            }
            DataPlacementStrategy::StreamBased => {
                // 流式放置策略：类似MPDU但不使用指针
                self.build_stream_packet(parent_type)
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
        match self
            .mpdu_manager
            .build_mpdu_packet(parent_type, mpdu_config)
        {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error building MPDU packet: {}", e);
                None
            }
        }
    }

    /// 构建直接放置包
    fn build_direct_packet(&mut self, parent_type: &str) -> Option<Vec<u8>> {
        // 从队列中获取子包并直接返回
        if let Some(mut queue) = self.child_packet_queues.remove(parent_type) {
            if let Some(mut child_data) = queue.pop_front() {
                if let Ok(frame) = child_data.assembler.assemble_frame() {
                    // 将队列放回去
                    self.child_packet_queues
                        .insert(parent_type.to_string(), queue);
                    return Some(frame);
                } else {
                    // 如果出错，仍要把队列放回去
                    self.child_packet_queues
                        .insert(parent_type.to_string(), queue);
                }
            } else {
                // 如果队列为空，仍要把队列放回去
                self.child_packet_queues
                    .insert(parent_type.to_string(), queue);
            }
        }
        None
    }

    /// 构建流式放置包
    fn build_stream_packet(&mut self, parent_type: &str) -> Option<Vec<u8>> {
        // 简单的流式放置：将多个子包连接成一个大的数据块
        let mut result = Vec::new();

        if let Some(mut queue) = self.child_packet_queues.remove(parent_type) {
            // 取出所有可用的子包并连接它们的数据
            while let Some(mut child_data) = queue.pop_front() {
                if let Ok(frame) = child_data.assembler.assemble_frame() {
                    result.extend_from_slice(&frame);
                    // 限制结果大小以防止无限增长
                    if result.len() > 1024 {
                        break;
                    }
                }
            }
            // 将队列放回去（即使可能已空）
            self.child_packet_queues
                .insert(parent_type.to_string(), queue);
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// 构建MPDU包 - 从子包队列中取出数据填充到父包
    pub fn build_mpdu_packet(
        &mut self,
        parent_type: &str,
        mpdu_config: &DataPlacementConfig,
    ) -> Option<Vec<u8>> {
        match self
            .mpdu_manager
            .build_mpdu_packet(parent_type, mpdu_config)
        {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error building MPDU packet: {}", e);
                None
            }
        }
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
        let engine = ConnectorEngine::new();
    }
}
