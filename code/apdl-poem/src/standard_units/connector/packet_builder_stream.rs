//! Stream策略包构建模块

use super::data_structures::MultiplexQueue;
use std::collections::HashMap;

/// 构建流式放置包
pub(super) fn build_stream_packet(
    child_packet_queues: &mut HashMap<String, MultiplexQueue>,
    parent_type: &str,
) -> Option<Vec<u8>> {
    // 第一阶段：收集数据（持有可变引用）
    let (result, should_remove) = {
        let current_queue = child_packet_queues.get_mut(parent_type)?;

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
        child_packet_queues.remove(parent_type);
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}
