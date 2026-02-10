//! 接收缓存模块
//!
//! 提供流式数据接收、帧同步和边界识别功能

pub mod buffer;
pub mod sync;

pub use buffer::ReceiveBuffer;
pub use sync::{FrameSynchronizer, SyncMode};
