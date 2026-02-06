//! 连接器数据结构定义

use crate::standard_units::frame_assembler::core::FrameAssembler;
use std::collections::VecDeque;

/// 子包数据结构
pub(super) struct ChildPacketData {
    /// 子包组装器
    pub assembler: FrameAssembler,
    /// 包类型标识
    pub _packet_type: String,
}

/// 分路队列
pub(super) struct MultiplexQueue {
    /// 子包队列
    pub child_packet_queue: VecDeque<ChildPacketData>,
    /// 父包组装器
    pub parent_assembler: FrameAssembler,
    /// 剩余的子包数据
    pub remaining_child_data: Vec<u8>,
    /// 包类型标识
    pub _packet_type: String,
}
