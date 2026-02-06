//! MPDU (PointerBased)策略包构建模块

use super::data_structures::MultiplexQueue;
use crate::standard_units::frame_assembler::core::FrameAssembler;
use apdl_core::DataPlacementConfig;
use std::collections::HashMap;

/// 从指定类型的队列中构建一个完整的MPDU包
pub(super) fn build_mpdu_packet(
    child_packet_queues: &mut HashMap<String, MultiplexQueue>,
    dispatch_flag: &str,
    mpdu_config: &DataPlacementConfig,
) -> Result<Option<Vec<u8>>, String> {
    // 第一阶段：收集数据
    let (
        mut parent_assembler,
        capacity,
        mut current_data,
        used_bytes,
        mut pointer_pos,
        should_remove,
    ) = {
        // 先获取队列的可变引用，而不是直接remove
        let current_queue = child_packet_queues
            .get_mut(dispatch_flag)
            .ok_or("Queue not found")?;

        // 获取父包模板
        let parent_assembler = current_queue.parent_assembler.clone();

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
        let padding_data = get_padding_bytes(mpdu_config, padding_size);
        current_data.extend_from_slice(&padding_data);
    }

    // 如果是空包
    if pointer_pos == 0xFFFF {
        pointer_pos = 0x07FE;
    }

    // 设置首导头指针
    set_mpdu_pointer(&mut parent_assembler, mpdu_config, pointer_pos)?;

    // 将MPDU数据设置到目标字段
    if parent_assembler
        .set_field_value(&mpdu_config.target_field, &current_data)
        .is_ok()
    {
        // 组装完整的帧并返回
        // 注意：assemble_frame会自动触发所有语义规则（包括SequenceControl）
        if let Ok(final_frame) = parent_assembler.assemble_frame() {
            // 更新队列中的parent_assembler以保持状态（如序列号）
            if let Some(queue) = child_packet_queues.get_mut(dispatch_flag) {
                queue.parent_assembler = parent_assembler;
            }

            // 只有当child_packet_queue为空且没有剩余数据时，才移除队列
            if should_remove {
                child_packet_queues.remove(dispatch_flag);
            }
            return Ok(Some(final_frame));
        }
    }

    Ok(None)
}

/// 获取填充码
fn get_padding_bytes(mpdu_config: &DataPlacementConfig, size: usize) -> Vec<u8> {
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
            .map_err(|e| format!("Failed to set pointer field '{pointer_field_name}': {e}"))?;
    }
    Ok(())
}
