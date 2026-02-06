//! Direct策略包构建模块

use super::data_structures::MultiplexQueue;
use std::collections::HashMap;

/// 构建直接放置包
pub(super) fn build_direct_packet(
    child_packet_queues: &mut HashMap<String, MultiplexQueue>,
    parent_type: &str,
) -> Option<Vec<u8>> {
    // 第一阶段：收集数据（持有可变引用）
    let (mut parent_assembler, child_frame, should_remove) = {
        let current_queue = child_packet_queues.get_mut(parent_type)?;

        // 从队列中获取子包
        let mut child_data = current_queue.child_packet_queue.pop_front()?;

        // 组装子包
        let child_frame = child_data.assembler.assemble_frame().ok()?;

        // 克隆父包模板（需要在作用域外使用）
        let parent_assembler = current_queue.parent_assembler.clone();

        // 检查是否应该移除队列：只有当child_packet_queue为空且没有剩余数据时
        let should_remove = current_queue.child_packet_queue.is_empty()
            && current_queue.remaining_child_data.is_empty();

        (parent_assembler, child_frame, should_remove)
    }; // current_queue的可变引用在这里释放

    // 第二阶段：将子包数据填充到父包的data字段中
    // 获取父包data字段的当前内容
    if let Ok(data_field_value) = parent_assembler.get_field_value("data") {
        let data_field_size = data_field_value.len();

        // 创建新的data字段内容：将子包数据复制到data字段中
        let mut new_data = vec![0u8; data_field_size];
        let copy_len = child_frame.len().min(data_field_size);
        new_data[..copy_len].copy_from_slice(&child_frame[..copy_len]);

        // 更新父包的data字段
        if parent_assembler.set_field_value("data", &new_data).is_ok() {
            // 组装完整的父包
            if let Ok(parent_frame) = parent_assembler.assemble_frame() {
                // 将更新后的parent_assembler写回队列以保持状态
                if let Some(queue) = child_packet_queues.get_mut(parent_type) {
                    queue.parent_assembler = parent_assembler;
                }

                // 清理队列
                if should_remove {
                    child_packet_queues.remove(parent_type);
                }

                return Some(parent_frame);
            }
        }
    }

    // 如果失败，清理队列
    if should_remove {
        child_packet_queues.remove(parent_type);
    }

    None
}
