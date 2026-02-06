//! 连接器引擎实现
//!
//! 负责执行字段映射规则，将源包的字段值映射到目标包的字段

use apdl_core::{DataPlacementConfig, DataPlacementStrategy, FieldMappingEntry};

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

/// 分路队列
struct MultiplexQueue {
    /// 子包队列
    child_packet_queue: VecDeque<ChildPacketData>,
    /// 父包组装器
    parent_assembler: FrameAssembler,
    /// 剩余的子包数据
    pub remaining_child_data: Vec<u8>,
    /// 包类型标识
    _packet_type: String,
}

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
        channel: &str,
        mappings: &[FieldMappingEntry],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut dispatch_flag = String::new();
        for mapping in mappings {
            // channel单独处理
            if mapping.source_field == "channel" {
                target_assembler
                    .set_field_value(&mapping.target_field, channel.as_bytes())
                    .map_err(|e| Box::new(e))?;
                dispatch_flag.push_str(channel);
                continue;
            }
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
                // 将目标字段的值添加到dispatch_flag
                let target_value = target_assembler
                    .get_field_value(&mapping.target_field)
                    .unwrap_or_else(|_| mapped_value.clone());
                dispatch_flag.push_str(&format!("{:?}", target_value));
            }
        }
        Ok(dispatch_flag)
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

    /// 构建MPDU包 - 从子包队列中取出数据填充到父包

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
                self.build_direct_packet(selected_dispatch_flag)
                    .map(|packet| (packet, selected_dispatch_flag.clone()))
            }
            DataPlacementStrategy::StreamBased => {
                // 流式放置策略：类似MPDU但不使用指针
                self.build_stream_packet(selected_dispatch_flag)
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

    /// 构建直接放置包
    fn build_direct_packet(&mut self, parent_type: &str) -> Option<Vec<u8>> {
        // 第一阶段：收集数据（持有可变引用）
        let (frame, should_remove) = {
            let current_queue = self.child_packet_queues.get_mut(parent_type)?;

            // 从队列中获取子包
            let mut child_data = current_queue.child_packet_queue.pop_front()?;

            // 组装子包
            let frame = child_data.assembler.assemble_frame().ok()?;

            // 检查是否应该移除队列：只有当child_packet_queue为空且没有剩余数据时
            let should_remove = current_queue.child_packet_queue.is_empty()
                && current_queue.remaining_child_data.is_empty();

            (frame, should_remove)
        }; // current_queue的可变引用在这里释放

        // 第二阶段：清理队列
        if should_remove {
            self.child_packet_queues.remove(parent_type);
        }

        Some(frame)
    }

    /// 构建流式放置包
    fn build_stream_packet(&mut self, parent_type: &str) -> Option<Vec<u8>> {
        // 第一阶段：收集数据（持有可变引用）
        let (result, should_remove) = {
            let current_queue = self.child_packet_queues.get_mut(parent_type)?;

            let mut result = Vec::new();

            // 取出所有可用的子包并连接它们的数据
            while let Some(mut child_data) = current_queue.child_packet_queue.pop_front() {
                if let Ok(frame) = child_data.assembler.assemble_frame() {
                    result.extend_from_slice(&frame);
                    // 限制结果大小以防止无限增长
                    if result.len() > 1024 {
                        break;
                    }
                }
            }

            // 检查是否应该移除队列：只有当child_packet_queue为空且没有剩余数据时
            let should_remove = current_queue.child_packet_queue.is_empty()
                && current_queue.remaining_child_data.is_empty();

            (result, should_remove)
        }; // current_queue的可变引用在这里释放

        // 第二阶段：清理队列
        if should_remove {
            self.child_packet_queues.remove(parent_type);
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// 从指定类型的队列中构建一个完整的MPDU包
    pub fn build_mpdu_packet(
        &mut self,
        dispatch_flag: &str,
        mpdu_config: &DataPlacementConfig,
    ) -> Result<Option<Vec<u8>>, String> {
        // 第一阶段：收集数据
        let (
            mut parent_assembler,
            capacity,
            mut current_data,
            mut used_bytes,
            mut pointer_pos,
            should_remove,
        ) = {
            // 先获取队列的可变引用，而不是直接remove
            let current_queue = self
                .child_packet_queues
                .get_mut(dispatch_flag)
                .ok_or("Queue not found")?;

            // 获取父包模板
            let mut parent_assembler = current_queue.parent_assembler.clone();

            // 计算MPDU容量（目标字段的大小）
            let capacity = parent_assembler
                .get_field_size_by_name(&mpdu_config.target_field)
                .map_err(|e| {
                    format!(
                        "Failed to get field size for '{}': {}",
                        mpdu_config.target_field, e
                    )
                })?;

            // 先处理剩余的子包数据（如果有的话）
            let mut current_data = vec![];
            let mut used_bytes = 0;

            // 首导头指针位置初始化为0
            let mut pointer_pos = 0;

            // 如果有剩余的子包数据，先填充到当前包
            if !current_queue.remaining_child_data.is_empty() {
                let space_left = capacity;
                let available = std::cmp::min(space_left, current_queue.remaining_child_data.len());

                current_data.extend_from_slice(&current_queue.remaining_child_data[..available]);
                used_bytes += available;

                // 更新剩余数据
                let remaining_after_fill = if available < current_queue.remaining_child_data.len() {
                    current_queue.remaining_child_data[available..].to_vec()
                } else {
                    vec![]
                };

                // 如果当前包已满，但还有剩余数据，保存状态供下次使用
                if used_bytes >= capacity && !remaining_after_fill.is_empty() {
                    current_queue.remaining_child_data = remaining_after_fill;
                } else {
                    current_queue.remaining_child_data = vec![];
                }
            }

            // indicate that the current packet is not full
            if used_bytes < capacity {
                pointer_pos = 0xFFFF;
                if used_bytes > 0 {
                    pointer_pos = 0x07FF;
                }
            }

            // 从队列中获取子包并填充
            while used_bytes < capacity && !current_queue.child_packet_queue.is_empty() {
                if let Some(mut child) = current_queue.child_packet_queue.pop_front() {
                    if let Ok(child_data) = child.assembler.assemble_frame() {
                        if pointer_pos == 0xFFFF || pointer_pos == 0x07FF {
                            // 只要有子包，当前pos就是指针位置
                            pointer_pos = used_bytes as u16;
                        }

                        // 检查是否有足够空间放置当前子包
                        let space_left = capacity - used_bytes;

                        if child_data.len() <= space_left {
                            // 整个子包可以放入当前MPDU
                            current_data.extend_from_slice(&child_data);
                            used_bytes += child_data.len();
                        } else {
                            // 子包太大，需要分割
                            let can_fit = &child_data[..space_left];
                            current_data.extend_from_slice(can_fit);
                            used_bytes += space_left;

                            // 保存剩余的子包数据供下次使用
                            let remaining_child = child_data[space_left..].to_vec();
                            current_queue.remaining_child_data = remaining_child;
                        }
                    }
                }
            }

            // 检查是否应该移除队列：只有当child_packet_queue为空且没有剩余数据时
            let should_remove = current_queue.child_packet_queue.is_empty()
                && current_queue.remaining_child_data.is_empty();

            (
                parent_assembler,
                capacity,
                current_data,
                used_bytes,
                pointer_pos,
                should_remove,
            )
        }; // current_queue的可变引用在这里释放

        // 第二阶段：处理填充和组装（不持有队列引用）
        // 如果当前包还没有填满，添加填充码
        if used_bytes < capacity {
            let padding_size = capacity - used_bytes;

            // 根据MPDU配置获取填充码，如果没有则使用默认填充码
            let padding_data = self.get_padding_bytes(mpdu_config, padding_size);
            current_data.extend_from_slice(&padding_data);
        }

        // 如果是空包
        if pointer_pos == 0xFFFF {
            pointer_pos = 0x07FE;
        }

        // 设置首导头指针
        self.set_mpdu_pointer(&mut parent_assembler, mpdu_config, pointer_pos)?;

        // 将MPDU数据设置到目标字段
        if parent_assembler
            .set_field_value(&mpdu_config.target_field, &current_data)
            .is_ok()
        {
            // 组装完整的帧并返回
            if let Ok(final_frame) = parent_assembler.assemble_frame() {
                // 更新队列中的parent_assembler以保持状态（如序列号）
                if let Some(queue) = self.child_packet_queues.get_mut(dispatch_flag) {
                    queue.parent_assembler = parent_assembler;
                }

                // 只有当child_packet_queue为空且没有剩余数据时，才移除队列
                if should_remove {
                    self.child_packet_queues.remove(dispatch_flag);
                }
                return Ok(Some(final_frame));
            }
        }

        Ok(None)
    }

    /// 获取填充码
    fn get_padding_bytes(&self, mpdu_config: &DataPlacementConfig, size: usize) -> Vec<u8> {
        // 检查配置中是否定义了填充码
        if let Some(padding_value) = mpdu_config
            .config_params
            .iter()
            .find(|(key, _)| key == "padding_value")
            .map(|(_, value)| value.as_str())
        {
            // 尝试解析为十六进制或十进制
            if let Ok(pad_byte) = u8::from_str_radix(padding_value.trim_start_matches("0x"), 16) {
                vec![pad_byte; size]
            } else if let Ok(pad_byte) = padding_value.parse::<u8>() {
                vec![pad_byte; size]
            } else {
                // 默认使用0xFF作为填充码
                vec![0xFF; size]
            }
        } else {
            // 默认使用0xFF作为填充码
            vec![0xFF; size]
        }
    }

    /// 设置MPDU首导头指针
    fn set_mpdu_pointer(
        &mut self,
        parent_assembler: &mut FrameAssembler,
        mpdu_config: &DataPlacementConfig,
        pointer_pos: u16,
    ) -> Result<(), String> {
        // 检查是否有指针字段配置
        if let Some(pointer_field_name) = mpdu_config
            .config_params
            .iter()
            .find(|(key, _)| key == "pointer_field")
            .map(|(_, value)| value.as_str())
        {
            // 根据CCSDS标准，首导头指针指向MPDU包区中第一个完整包的第一个字节位置
            // 首导头指针值等于第一个完整包在MPDU数据区中的偏移量

            let pointer_value = pointer_pos;

            let pointer_bytes = pointer_value.to_be_bytes().to_vec();

            // 设置指针字段值
            parent_assembler
                .set_field_value(pointer_field_name, &pointer_bytes)
                .map_err(|e| {
                    format!(
                        "Failed to set pointer field '{}': {}",
                        pointer_field_name, e
                    )
                })?;
        }
        Ok(())
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
