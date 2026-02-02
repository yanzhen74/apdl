//! 语义规则解析器模块
//!
//! 处理各种语义规则的解析

use apdl_core::SemanticRule;

pub mod checksum_rules;
pub mod control_rules;
pub mod dependency_rules;
pub mod multiplexing_rules;
pub mod routing_rules;
pub mod security_rules;
pub mod synchronization_rules;
pub mod validation_rules;


/// 语义规则解析器
pub struct SemanticRuleParsers;

impl SemanticRuleParsers {
    /// 解析校验和范围规则
    pub fn parse_checksum_range(params: &str, rule_type: &str) -> Result<SemanticRule, String> {
        checksum_rules::parse_checksum_range(params, rule_type)
    }

    /// 解析依赖关系规则
    pub fn parse_dependency(params: &str) -> Result<SemanticRule, String> {
        dependency_rules::parse_dependency(params)
    }

    /// 解析条件规则
    pub fn parse_conditional(params: &str) -> Result<SemanticRule, String> {
        control_rules::parse_conditional(params)
    }

    /// 解析顺序规则
    pub fn parse_order(params: &str) -> Result<SemanticRule, String> {
        control_rules::parse_order(params)
    }

    /// 解析指针规则
    pub fn parse_pointer(params: &str) -> Result<SemanticRule, String> {
        control_rules::parse_pointer(params)
    }

    /// 解析算法规则
    pub fn parse_algorithm(params: &str) -> Result<SemanticRule, String> {
        control_rules::parse_algorithm(params)
    }

    /// 解析长度规则
    pub fn parse_length_rule(params: &str) -> Result<SemanticRule, String> {
        control_rules::parse_length_rule(params)
    }

    /// 解析路由分发规则
    pub fn parse_routing_dispatch(params: &str) -> Result<SemanticRule, String> {
        routing_rules::parse_routing_dispatch(params)
    }

    /// 解析序列控制规则
    pub fn parse_sequence_control(params: &str) -> Result<SemanticRule, String> {
        control_rules::parse_sequence_control(params)
    }

    /// 解析校验规则
    pub fn parse_validation(params: &str) -> Result<SemanticRule, String> {
        validation_rules::parse_validation(params)
    }

    /// 解析多路复用规则
    pub fn parse_multiplexing(params: &str) -> Result<SemanticRule, String> {
        multiplexing_rules::parse_multiplexing(params)
    }

    /// 解析优先级处理规则
    pub fn parse_priority_processing(params: &str) -> Result<SemanticRule, String> {
        control_rules::parse_priority_processing(params)
    }

    /// 解析同步规则
    pub fn parse_synchronization(params: &str) -> Result<SemanticRule, String> {
        synchronization_rules::parse_synchronization(params)
    }

    /// 解析长度验证规则
    pub fn parse_length_validation(params: &str) -> Result<SemanticRule, String> {
        validation_rules::parse_length_validation(params)
    }

    /// 解析状态机规则
    pub fn parse_state_machine(params: &str) -> Result<SemanticRule, String> {
        control_rules::parse_state_machine(params)
    }

    /// 解析周期性传输规则
    pub fn parse_periodic_transmission(params: &str) -> Result<SemanticRule, String> {
        control_rules::parse_periodic_transmission(params)
    }

    /// 解析消息过滤规则
    pub fn parse_message_filtering(params: &str) -> Result<SemanticRule, String> {
        control_rules::parse_message_filtering(params)
    }

    /// 解析错误检测规则
    pub fn parse_error_detection(params: &str) -> Result<SemanticRule, String> {
        validation_rules::parse_error_detection(params)
    }

    /// 解析嵌套同步规则
    pub fn parse_nested_sync(params: &str) -> Result<SemanticRule, String> {
        synchronization_rules::parse_nested_sync(params)
    }

    /// 解析序列重置规则
    pub fn parse_sequence_reset(params: &str) -> Result<SemanticRule, String> {
        control_rules::parse_sequence_reset(params)
    }

    /// 解析时间戳插入规则
    pub fn parse_timestamp_insertion(params: &str) -> Result<SemanticRule, String> {
        control_rules::parse_timestamp_insertion(params)
    }

    /// 解析流量控制规则
    pub fn parse_flow_control(params: &str) -> Result<SemanticRule, String> {
        control_rules::parse_flow_control(params)
    }

    /// 解析时间同步规则
    pub fn parse_time_synchronization(params: &str) -> Result<SemanticRule, String> {
        synchronization_rules::parse_time_synchronization(params)
    }

    /// 解析地址解析规则
    pub fn parse_address_resolution(params: &str) -> Result<SemanticRule, String> {
        routing_rules::parse_address_resolution(params)
    }

    /// 解析安全规则
    pub fn parse_security(params: &str) -> Result<SemanticRule, String> {
        security_rules::parse_security(params)
    }

    /// 解析冗余规则
    pub fn parse_redundancy(params: &str) -> Result<SemanticRule, String> {
        security_rules::parse_redundancy(params)
    }
}
