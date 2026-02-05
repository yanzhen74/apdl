//! MPDU（多路协议数据单元）管理器
//!
//! 负责实现CCSDS标准中的MPDU功能，包括多路复用、子包队列管理、
//! 首导头指针设置、填充码处理等功能

use crate::standard_units::frame_assembler::core::FrameAssembler;
use apdl_core::DataPlacementConfig;
use std::collections::{HashMap, VecDeque};

/// MPDU状态管理器
pub struct MpduManager {
    /// 子包缓存队列 - 按父包类型分类
    child_packet_queues: HashMap<String, VecDeque<Vec<u8>>>,
    /// 父包模板缓存队列
    parent_template_queues: HashMap<String, VecDeque<FrameAssembler>>,
    /// 当前正在构建的父包（用于处理跨包分割）
    current_partial_packets: HashMap<String, PartialMpduState>,
}

/// 部分MPDU包状态
#[derive(Clone)]
pub struct PartialMpduState {
    /// 当前父包数据
    pub data: Vec<u8>,
    /// 已使用的字节数
    pub used_bytes: usize,
    /// 总容量
    pub capacity: usize,
    /// 剩余的子包数据
    pub remaining_child_data: Vec<u8>,
}

impl MpduManager {
    /// 创建新的MPDU管理器
    pub fn new() -> Self {
        Self {
            child_packet_queues: HashMap::new(),
            parent_template_queues: HashMap::new(),
            current_partial_packets: HashMap::new(),
        }
    }

    /// 添加子包到指定类型的队列
    pub fn add_child_packet(&mut self, parent_type: &str, child_data: Vec<u8>) {
        self.child_packet_queues
            .entry(parent_type.to_string())
            .or_insert_with(VecDeque::new)
            .push_back(child_data);
    }

    /// 添加父包模板到队列
    pub fn add_parent_template(&mut self, parent_type: &str, template: FrameAssembler) {
        self.parent_template_queues
            .entry(parent_type.to_string())
            .or_insert_with(VecDeque::new)
            .push_back(template);
    }

    /// 从指定类型的队列中构建一个完整的MPDU包
    pub fn build_mpdu_packet(
        &mut self,
        parent_type: &str,
        mpdu_config: &DataPlacementConfig,
    ) -> Option<Vec<u8>> {
        // 获取父包模板
        let mut parent_assembler = self
            .parent_template_queues
            .get_mut(parent_type)?
            .pop_front()?;

        // 获取当前部分包状态（如果有）
        let mut partial_state = self
            .current_partial_packets
            .remove(parent_type)
            .unwrap_or_else(|| {
                PartialMpduState {
                    data: vec![],
                    used_bytes: 0,
                    capacity: 0, // 将在后面计算
                    remaining_child_data: vec![],
                }
            });

        // 计算MPDU容量（目标字段的大小）
        if partial_state.capacity == 0 {
            if let Ok(field_size) =
                parent_assembler.get_field_size_by_name(&mpdu_config.target_field)
            {
                partial_state.capacity = field_size;
            } else {
                // 如果无法获取字段大小，使用默认值
                partial_state.capacity = 1024; // 默认1KB
            }
        }

        // 先处理剩余的子包数据（如果有的话）
        let mut current_data = partial_state.data.clone();
        let mut used_bytes = partial_state.used_bytes;

        // 首导头指针位置初始化为0
        let mut pointer_pos = 0;

        // 如果有剩余的子包数据，先填充到当前包
        if !partial_state.remaining_child_data.is_empty() {
            let space_left = partial_state.capacity;
            let available = std::cmp::min(space_left, partial_state.remaining_child_data.len());

            current_data.extend_from_slice(&partial_state.remaining_child_data[..available]);
            used_bytes += available;

            // 更新剩余数据
            let remaining_after_fill = if available < partial_state.remaining_child_data.len() {
                partial_state.remaining_child_data[available..].to_vec()
            } else {
                vec![]
            };

            // 如果当前包已满，但还有剩余数据，保存状态供下次使用
            if used_bytes >= partial_state.capacity && !remaining_after_fill.is_empty() {
                let new_partial_state = PartialMpduState {
                    data: vec![],
                    used_bytes: 0,
                    capacity: partial_state.capacity,
                    remaining_child_data: remaining_after_fill,
                };
                self.current_partial_packets
                    .insert(parent_type.to_string(), new_partial_state);
            }
        }

        // indicate that the current packet is not full
        if used_bytes < partial_state.capacity {
            pointer_pos = 0xFFFF;
            if used_bytes > 0 {
                pointer_pos = 0x07FF;
            }
        }

        // 从队列中获取子包并填充
        let child_queue = self
            .child_packet_queues
            .entry(parent_type.to_string())
            .or_default();
        while used_bytes < partial_state.capacity && !child_queue.is_empty() {
            if let Some(child_data) = child_queue.pop_front() {
                if pointer_pos == 0xFFFF || pointer_pos == 0x07FF {
                    // 只要有子包，当前pos就是指针位置
                    pointer_pos = used_bytes as u16;
                }

                // 检查是否有足够空间放置当前子包
                let space_left = partial_state.capacity - used_bytes;

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
                    let new_partial_state = PartialMpduState {
                        data: vec![],
                        used_bytes: 0,
                        capacity: partial_state.capacity,
                        remaining_child_data: remaining_child,
                    };
                    self.current_partial_packets
                        .insert(parent_type.to_string(), new_partial_state);
                }
            }
        }

        // 如果当前包还没有填满，添加填充码
        if used_bytes < partial_state.capacity {
            let padding_size = partial_state.capacity - used_bytes;

            // 根据MPDU配置获取填充码，如果没有则使用默认填充码
            let padding_data = self.get_padding_bytes(mpdu_config, padding_size);
            current_data.extend_from_slice(&padding_data);
            // used_bytes += padding_size; // 这个变量不再需要更新
        }

        // 如果是空包
        if pointer_pos == 0xFFFF {
            pointer_pos = 0x07FE;
        }

        // 设置首导头指针
        self.set_mpdu_pointer(&mut parent_assembler, mpdu_config, pointer_pos);

        // 将MPDU数据设置到目标字段
        if parent_assembler
            .set_field_value(&mpdu_config.target_field, &current_data)
            .is_ok()
        {
            // 组装完整的帧并返回
            if let Ok(final_frame) = parent_assembler.assemble_frame() {
                return Some(final_frame);
            }
        }

        None
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
    ) {
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
            let _ = parent_assembler.set_field_value(pointer_field_name, &pointer_bytes);
        }
    }

    /// 获取指定类型子包队列的长度
    pub fn get_child_queue_length(&self, parent_type: &str) -> usize {
        self.child_packet_queues
            .get(parent_type)
            .map(|queue| queue.len())
            .unwrap_or(0)
    }

    /// 获取指定类型父包模板队列的长度
    pub fn get_parent_queue_length(&self, parent_type: &str) -> usize {
        self.parent_template_queues
            .get(parent_type)
            .map(|queue| queue.len())
            .unwrap_or(0)
    }
}

impl Default for MpduManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpdu_manager_creation() {
        let manager = MpduManager::new();
        assert_eq!(manager.child_packet_queues.len(), 0);
        assert_eq!(manager.parent_template_queues.len(), 0);
    }
}
