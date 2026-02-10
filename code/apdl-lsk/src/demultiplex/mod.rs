//! 解复接模块
//!
//! 提供多路数据分离、虚拟通道管理和序列号校验功能

pub mod demultiplexer;
pub mod sequence_validator;
pub mod reorder_buffer;

pub use demultiplexer::{Demultiplexer, ChannelState};
pub use sequence_validator::{SequenceValidator, ValidationResult};
pub use reorder_buffer::ReorderBuffer;
