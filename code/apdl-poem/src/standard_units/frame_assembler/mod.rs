//! Frame Assembler 模块
//!
//! 将 Frame Assembler 的功能拆分为多个子模块以提高可维护性

pub mod address_resolution_rule_handler;
pub mod checksum_rule_handler;
pub mod conditional_rule_handler;
pub mod core;
pub mod custom_algorithm_handler;
pub mod dependency_rule_handler;
pub mod error_detection_rule_handler;
pub mod field_mapping_rule_handler;
pub mod flow_control_rule_handler;
pub mod length_rule_handler;
pub mod length_validation_rule_handler;
pub mod message_filtering_rule_handler;
pub mod multiplexing_rule_handler;
pub mod order_rule_handler;
pub mod periodic_transmission_rule_handler;
pub mod pointer_rule_handler;
pub mod priority_processing_rule_handler;
pub mod redundancy_rule_handler;
pub mod routing_dispatch_rule_handler;
pub mod security_rule_handler;
pub mod sequence_control_rule_handler;
pub mod state_machine_rule_handler;
pub mod synchronization_rule_handler;
pub mod time_synchronization_rule_handler;
pub mod utils;
pub mod validation_rule_handler;

// 导出主要的结构和公共接口
pub use core::FrameAssembler;
