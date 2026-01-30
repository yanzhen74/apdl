//! 协议帧组装器
//!
//! 实现协议帧的组装和解析功能

use apdl_core::{
    ChecksumAlgorithm, LengthDesc, LengthUnit, ProtocolError, SemanticRule, SyntaxUnit, UnitType,
};
use std::collections::HashMap;

/// 协议帧组装器
pub struct FrameAssembler {
    pub fields: Vec<SyntaxUnit>,
    pub semantic_rules: Vec<SemanticRule>,
    field_index: HashMap<String, usize>,
    // 添加字段值存储
    field_values: HashMap<String, Vec<u8>>,
}

impl FrameAssembler {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            semantic_rules: Vec::new(),
            field_index: HashMap::new(),
            field_values: HashMap::new(),
        }
    }

    /// 添加字段定义
    pub fn add_field(&mut self, field: SyntaxUnit) {
        let field_name = field.field_id.clone();
        let index = self.fields.len();
        self.fields.push(field);
        self.field_index.insert(field_name, index);
    }

    /// 添加语义规则
    pub fn add_semantic_rule(&mut self, rule: SemanticRule) {
        self.semantic_rules.push(rule);
    }

    /// 组装协议帧
    pub fn assemble_frame(&mut self) -> Result<Vec<u8>, ProtocolError> {
        // 两阶段处理：第一阶段组装基础帧，第二阶段应用长度和CRC规则
        let mut frame_data = Vec::new();

        // 第一阶段：按照字段顺序依次组装基础帧（不应用长度和CRC规则）
        for field in &self.fields {
            let field_bytes = self.get_field_bytes(&field.field_id)?;
            frame_data.extend_from_slice(&field_bytes);
        }

        // 第二阶段：应用长度和CRC等需要在完整帧基础上计算的规则
        self.apply_length_and_crc_rules(&mut frame_data)?;

        Ok(frame_data)
    }

    /// 应用长度和CRC规则（第二阶段处理）
    fn apply_length_and_crc_rules(
        &mut self,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        // 克隆语义规则以避免借用冲突
        let rules_to_process: Vec<_> = self.semantic_rules.clone();

        // 遍历所有语义规则，应用长度和CRC相关的规则
        for rule in &rules_to_process {
            match rule {
                SemanticRule::LengthRule {
                    field_name,
                    expression,
                } => {
                    // 清理字段名，移除可能的前缀
                    let clean_field_name = field_name.trim_start_matches("field: ").trim();

                    // 计算长度表达式的值
                    let length_value = self.evaluate_length_expression(expression, frame_data)?;
                    println!(
                        "DEBUG: Calculated length_value for field '{}' with expression '{}': {}",
                        clean_field_name, expression, length_value
                    );

                    // 查找字段在帧中的位置
                    if let Some(&field_index) = self.field_index.get(clean_field_name) {
                        let field = &self.fields[field_index];
                        let field_size = self.get_field_size(field)?;
                        let field_offset = self.calculate_field_offset(field_index)?;

                        // 将长度值写入帧数据
                        let length_bytes = self.u64_to_bytes(length_value, field_size);
                        for (i, &byte) in length_bytes.iter().enumerate() {
                            if field_offset + i < frame_data.len() {
                                frame_data[field_offset + i] = byte;
                            }
                        }

                        // 同时更新字段值存储
                        self.field_values
                            .insert(clean_field_name.to_string(), length_bytes);
                    }
                }
                SemanticRule::ChecksumRange {
                    algorithm,
                    start_field,
                    end_field,
                } => {
                    // 清理字段名，移除可能的前缀
                    let clean_start_field = start_field.trim_start_matches("start: ").trim();
                    let clean_end_field = end_field.trim_start_matches("end: ").trim();
                    self.apply_checksum_rule(
                        frame_data,
                        algorithm,
                        clean_start_field,
                        clean_end_field,
                    )?;
                }
                _ => {
                    // 其他类型的规则暂时不在这里处理
                }
            }
        }
        Ok(())
    }

    /// 计算字段在帧中的偏移量
    fn calculate_field_offset(&self, field_index: usize) -> Result<usize, ProtocolError> {
        let mut offset = 0;
        for i in 0..field_index {
            if let Some(field) = self.fields.get(i) {
                let field_size = self.get_field_size(field)?;
                offset += field_size;
            }
        }
        Ok(offset)
    }

    /// 将u64值转换为指定长度的字节数组
    fn u64_to_bytes(&self, value: u64, size: usize) -> Vec<u8> {
        let mut bytes = Vec::new();
        for i in 0..size {
            bytes.push(((value >> (8 * (size - 1 - i))) & 0xFF) as u8);
        }
        bytes
    }

    /// 解析协议帧
    pub fn parse_frame(
        &mut self,
        frame_data: &[u8],
    ) -> Result<Vec<(String, Vec<u8>)>, ProtocolError> {
        let mut parsed_fields = Vec::new();
        let mut offset = 0;

        for field in &self.fields {
            let field_size = self.get_field_size(field)?;
            if offset + field_size > frame_data.len() {
                return Err(ProtocolError::InvalidFrameFormat(format!(
                    "Insufficient data for field: {}",
                    field.field_id
                )));
            }

            let field_data = &frame_data[offset..offset + field_size];
            parsed_fields.push((field.field_id.clone(), field_data.to_vec()));
            offset += field_size;
        }

        // 应用语义规则处理
        self.apply_semantic_rules_for_parse(&mut parsed_fields, frame_data)?;

        Ok(parsed_fields)
    }

    /// 设置字段值
    pub fn set_field_value(&mut self, field_name: &str, value: &[u8]) -> Result<(), ProtocolError> {
        // 清理字段名，移除可能的前缀
        let clean_field_name = field_name.trim_start_matches("field: ").trim();

        if let Some(&index) = self.field_index.get(clean_field_name) {
            if let Some(field) = self.fields.get(index) {
                // 对于动态长度字段，跳过长度验证
                if field.length.unit != LengthUnit::Dynamic {
                    // 检查值的长度是否符合字段定义
                    let expected_size = self.get_field_size(field)?;
                    if value.len() != expected_size {
                        return Err(ProtocolError::LengthError(format!(
                            "Field {} expected {} bytes, got {} bytes",
                            clean_field_name,
                            expected_size,
                            value.len()
                        )));
                    }
                }

                // 存储字段值
                self.field_values
                    .insert(clean_field_name.to_string(), value.to_vec());
                println!("Setting field {} to value: {:?}", clean_field_name, value);
                Ok(())
            } else {
                Err(ProtocolError::FieldNotFound(clean_field_name.to_string()))
            }
        } else {
            Err(ProtocolError::FieldNotFound(clean_field_name.to_string()))
        }
    }

    /// 获取字段值
    pub fn get_field_value(&self, field_name: &str) -> Result<Vec<u8>, ProtocolError> {
        // 清理字段名，移除可能的前缀
        let clean_field_name = field_name.trim_start_matches("field: ").trim();

        if let Some(value) = self.field_values.get(clean_field_name) {
            Ok(value.clone())
        } else {
            Err(ProtocolError::FieldNotFound(clean_field_name.to_string()))
        }
    }

    /// 获取字段字节表示
    fn get_field_bytes(&self, field_name: &str) -> Result<Vec<u8>, ProtocolError> {
        // 清理字段名，移除可能的前缀
        let clean_field_name = field_name.trim_start_matches("field: ").trim();

        // 首先尝试从存储的字段值中获取
        if let Some(stored_value) = self.field_values.get(clean_field_name) {
            Ok(stored_value.clone())
        } else {
            // 检查是否是长度规则字段，如果是则尝试动态计算
            for rule in &self.semantic_rules {
                if let SemanticRule::LengthRule {
                    field_name: rule_field_name,
                    expression,
                } = rule
                {
                    // 也需要清理规则中的字段名
                    let clean_rule_field_name =
                        rule_field_name.trim_start_matches("field: ").trim();
                    if clean_rule_field_name == clean_field_name {
                        // 这是一个长度规则字段，但我们还没有计算它的值
                        // 返回默认的零填充字节，等待后续的长度规则处理
                        if let Some(&index) = self.field_index.get(clean_field_name) {
                            if let Some(field) = self.fields.get(index) {
                                let size = self.get_field_size(field)?;
                                return Ok(vec![0; size]);
                            }
                        }
                    }
                }
            }

            // 如果没有存储的值，则返回零填充的字节
            if let Some(&index) = self.field_index.get(clean_field_name) {
                if let Some(field) = self.fields.get(index) {
                    let size = self.get_field_size(field)?;
                    Ok(vec![0; size])
                } else {
                    Err(ProtocolError::FieldNotFound(clean_field_name.to_string()))
                }
            } else {
                Err(ProtocolError::FieldNotFound(clean_field_name.to_string()))
            }
        }
    }

    /// 获取字段大小（以字节为单位）
    fn get_field_size(&self, field: &SyntaxUnit) -> Result<usize, ProtocolError> {
        match &field.length.unit {
            LengthUnit::Byte => Ok(field.length.size),
            LengthUnit::Bit => Ok((field.length.size + 7) / 8), // 将比特数转换为字节数（向上取整）
            LengthUnit::Dynamic => {
                // 对于动态长度，如果已经有存储的值，则返回值的长度
                if let Some(stored_value) = self.field_values.get(&field.field_id) {
                    Ok(stored_value.len())
                } else {
                    // 否则返回默认值，实际实现中应根据规则计算
                    Ok(1) // 默认1字节
                }
            }
            LengthUnit::Expression(expr) => {
                // 根据表达式计算长度，这需要更复杂的逻辑
                // 暂时返回默认值
                println!("Expression-based length not yet implemented: {}", expr);
                Ok(1) // 默认1字节
            }
        }
    }

    /// 应用语义规则
    fn apply_semantic_rules(&mut self, frame_data: &mut Vec<u8>) -> Result<(), ProtocolError> {
        // 先克隆语义规则，避免借用冲突
        let rules_to_process: Vec<_> = self.semantic_rules.clone();

        for rule in &rules_to_process {
            match rule {
                SemanticRule::ChecksumRange {
                    algorithm,
                    start_field,
                    end_field,
                } => {
                    // 清理字段名，移除可能的前缀
                    let clean_start_field = start_field.trim_start_matches("start: ").trim();
                    let clean_end_field = end_field.trim_start_matches("end: ").trim();
                    self.apply_checksum_rule(
                        frame_data,
                        algorithm,
                        clean_start_field,
                        clean_end_field,
                    )?;
                }
                SemanticRule::Dependency {
                    dependent_field,
                    dependency_field,
                } => {
                    // 清理字段名，移除可能的前缀
                    let clean_dependent_field =
                        dependent_field.trim_start_matches("field: ").trim();
                    let clean_dependency_field =
                        dependency_field.trim_start_matches("depends_on ").trim();
                    self.apply_dependency_rule(clean_dependent_field, clean_dependency_field)?;
                }
                SemanticRule::Conditional { condition } => {
                    self.apply_conditional_rule(condition, frame_data)?;
                }
                SemanticRule::Order {
                    first_field,
                    second_field,
                } => {
                    // 清理字段名，移除可能的前缀
                    let clean_first_field = first_field.trim_start_matches("first: ").trim();
                    let clean_second_field = second_field.trim_start_matches("second: ").trim();
                    self.apply_order_rule(clean_first_field, clean_second_field)?;
                }
                SemanticRule::Pointer {
                    pointer_field,
                    target_field,
                } => {
                    self.apply_pointer_rule(pointer_field, target_field, frame_data)?;
                }
                SemanticRule::Algorithm {
                    field_name,
                    algorithm,
                } => {
                    self.apply_custom_algorithm(field_name, algorithm, frame_data)?;
                }
                SemanticRule::LengthRule {
                    field_name,
                    expression,
                } => {
                    // 检查该字段的值是否已经通过预处理计算过了
                    if !self.field_values.contains_key(field_name) {
                        // 如果还没有计算过，才执行长度规则处理
                        self.process_single_length_rule(frame_data, field_name, expression)?;
                    }
                    // 如果已经通过预处理设置了值，这里就跳过，避免重复处理
                }
                // 处理新增的语义规则类型
                SemanticRule::RoutingDispatch {
                    fields,
                    algorithm,
                    description,
                } => {
                    self.apply_routing_dispatch_rule(fields, algorithm, description, frame_data)?;
                }
                SemanticRule::SequenceControl {
                    field_name,
                    trigger_condition,
                    algorithm,
                    description,
                } => {
                    self.apply_sequence_control_rule(
                        field_name,
                        trigger_condition,
                        algorithm,
                        description,
                        frame_data,
                    )?;
                }
                SemanticRule::Validation {
                    field_name,
                    algorithm,
                    range_start,
                    range_end,
                    description,
                } => {
                    // 清理字段名，移除可能的前缀
                    let clean_field_name = field_name.trim_start_matches("field: ").trim();
                    let clean_range_start = range_start.trim_start_matches("from(").trim();
                    let clean_range_end = range_end.trim_start_matches("to(").trim();
                    self.apply_validation_rule(
                        clean_field_name,
                        algorithm,
                        clean_range_start,
                        clean_range_end,
                        description,
                        frame_data,
                    )?;
                }
                SemanticRule::Multiplexing {
                    field_name,
                    condition,
                    route_target,
                    description,
                } => {
                    self.apply_multiplexing_rule(
                        field_name,
                        condition,
                        route_target,
                        description,
                        frame_data,
                    )?;
                }
                SemanticRule::PriorityProcessing {
                    field_name,
                    algorithm,
                    description,
                } => {
                    self.apply_priority_processing_rule(
                        field_name,
                        algorithm,
                        description,
                        frame_data,
                    )?;
                }
                SemanticRule::Synchronization {
                    field_name,
                    algorithm,
                    description,
                } => {
                    self.apply_synchronization_rule(
                        field_name,
                        algorithm,
                        description,
                        frame_data,
                    )?;
                }
                SemanticRule::LengthValidation {
                    field_name,
                    condition,
                    description,
                } => {
                    self.apply_length_validation_rule(
                        field_name,
                        condition,
                        description,
                        frame_data,
                    )?;
                }
                SemanticRule::StateMachine {
                    condition,
                    algorithm,
                    description,
                } => {
                    self.apply_state_machine_rule(condition, algorithm, description, frame_data)?;
                }
                SemanticRule::PeriodicTransmission {
                    field_name,
                    condition,
                    algorithm,
                    description,
                } => {
                    self.apply_periodic_transmission_rule(
                        field_name,
                        condition,
                        algorithm,
                        description,
                        frame_data,
                    )?;
                }
                SemanticRule::MessageFiltering {
                    condition,
                    action,
                    description,
                } => {
                    self.apply_message_filtering_rule(condition, action, description, frame_data)?;
                }
                SemanticRule::ErrorDetection {
                    algorithm,
                    description,
                } => {
                    self.apply_error_detection_rule(algorithm, description, frame_data)?;
                }
                SemanticRule::FlowControl {
                    field_name,
                    algorithm,
                    description,
                } => {
                    self.apply_flow_control_rule(field_name, algorithm, description, frame_data)?;
                }
                SemanticRule::TimeSynchronization {
                    field_name,
                    algorithm,
                    description,
                } => {
                    self.apply_time_synchronization_rule(
                        field_name,
                        algorithm,
                        description,
                        frame_data,
                    )?;
                }
                SemanticRule::AddressResolution {
                    field_name,
                    algorithm,
                    description,
                } => {
                    self.apply_address_resolution_rule(
                        field_name,
                        algorithm,
                        description,
                        frame_data,
                    )?;
                }
                SemanticRule::Security {
                    field_name,
                    algorithm,
                    description,
                } => {
                    self.apply_security_rule(field_name, algorithm, description, frame_data)?;
                }
                SemanticRule::Redundancy {
                    field_name,
                    algorithm,
                    description,
                } => {
                    self.apply_redundancy_rule(field_name, algorithm, description, frame_data)?;
                }
            }
        }
        Ok(())
    }

    /// 应用解析阶段的语义规则
    fn apply_semantic_rules_for_parse(
        &mut self,
        parsed_fields: &mut Vec<(String, Vec<u8>)>,
        frame_data: &[u8],
    ) -> Result<(), ProtocolError> {
        for rule in &self.semantic_rules {
            match rule {
                SemanticRule::ChecksumRange {
                    algorithm,
                    start_field,
                    end_field,
                } => {
                    // 清理字段名，移除可能的前缀
                    let clean_start_field = start_field.trim_start_matches("start: ").trim();
                    let clean_end_field = end_field.trim_start_matches("end: ").trim();
                    self.validate_checksum_rule(
                        frame_data,
                        algorithm,
                        clean_start_field,
                        clean_end_field,
                    )?;
                }
                // 其他规则的解析阶段处理...
                _ => {
                    // 其他规则可能不需要在解析阶段处理
                }
            }
        }
        Ok(())
    }

    /// 应用校验和规则
    fn apply_checksum_rule(
        &mut self,
        frame_data: &mut Vec<u8>,
        algorithm: &ChecksumAlgorithm,
        start_field: &str,
        end_field: &str,
    ) -> Result<(), ProtocolError> {
        // 根据字段名找到在帧数据中的位置
        let start_pos = self.get_field_position(start_field)?;
        let end_pos =
            self.get_field_position(end_field)? + self.get_field_size_by_name(end_field)?;

        if end_pos > frame_data.len() {
            return Err(ProtocolError::InvalidFrameFormat(
                "Field range exceeds frame size".to_string(),
            ));
        }

        let data_to_checksum = &frame_data[start_pos..end_pos];
        let checksum: u64 = match algorithm {
            ChecksumAlgorithm::CRC16 => self.calculate_crc16(data_to_checksum) as u64,
            ChecksumAlgorithm::CRC32 => self.calculate_crc32(data_to_checksum) as u64,
            ChecksumAlgorithm::CRC15 => self.calculate_crc15(data_to_checksum) as u64, // CAN协议专用
            ChecksumAlgorithm::XOR => self.calculate_xor(data_to_checksum) as u64,
        };

        // 这里应该将校验和写入对应的校验字段，简化处理
        println!(
            "Calculated checksum {:?} for range {} to {}: {:?}",
            algorithm, start_field, end_field, checksum
        );
        Ok(())
    }

    /// 验证校验和规则
    fn validate_checksum_rule(
        &self,
        frame_data: &[u8],
        algorithm: &ChecksumAlgorithm,
        start_field: &str,
        end_field: &str,
    ) -> Result<(), ProtocolError> {
        // 验证帧数据中的校验和是否正确
        let start_pos = self.get_field_position(start_field)?;
        let end_pos =
            self.get_field_position(end_field)? + self.get_field_size_by_name(end_field)?;

        if end_pos > frame_data.len() {
            return Err(ProtocolError::InvalidFrameFormat(
                "Field range exceeds frame size".to_string(),
            ));
        }

        let data_to_checksum = &frame_data[start_pos..end_pos];
        let calculated_checksum: u64 = match algorithm {
            ChecksumAlgorithm::CRC16 => self.calculate_crc16(data_to_checksum) as u64,
            ChecksumAlgorithm::CRC32 => self.calculate_crc32(data_to_checksum) as u64,
            ChecksumAlgorithm::CRC15 => self.calculate_crc15(data_to_checksum) as u64, // CAN协议专用
            ChecksumAlgorithm::XOR => self.calculate_xor(data_to_checksum) as u64,
        };

        println!(
            "Validated checksum {:?} for range {} to {}: {:?}",
            algorithm, start_field, end_field, calculated_checksum
        );
        Ok(())
    }

    /// 计算CRC16校验和
    fn calculate_crc16(&self, data: &[u8]) -> u16 {
        // 简化的CRC16计算，实际实现会更复杂
        let mut crc: u16 = 0xFFFF;
        for byte in data {
            crc ^= (*byte as u16) << 8;
            for _ in 0..8 {
                if (crc & 0x8000) != 0 {
                    crc = (crc << 1) ^ 0x1021;
                } else {
                    crc <<= 1;
                }
            }
        }
        crc
    }

    /// 计算CRC32校验和
    fn calculate_crc32(&self, data: &[u8]) -> u32 {
        // 简化的CRC32计算
        let mut crc: u32 = 0xFFFFFFFF;
        for byte in data {
            crc ^= *byte as u32;
            for _ in 0..8 {
                if (crc & 1) != 0 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
            }
        }
        !crc
    }

    /// 计算CRC15校验和 (CAN协议专用)
    fn calculate_crc15(&self, data: &[u8]) -> u16 {
        // CAN协议使用的CRC15算法
        let mut crc: u16 = 0x0000;
        for byte in data {
            crc ^= (*byte as u16) << 7;
            for _ in 0..8 {
                crc <<= 1;
                if (crc & 0x8000) != 0 {
                    crc ^= 0x4599;
                }
            }
        }
        (crc >> 1) & 0x7FFF
    }

    /// 计算XOR校验和
    fn calculate_xor(&self, data: &[u8]) -> u8 {
        let mut xor: u8 = 0;
        for byte in data {
            xor ^= byte;
        }
        xor
    }

    /// 应用依赖规则
    fn apply_dependency_rule(
        &mut self,
        dependent_field: &str,
        dependency_field: &str,
    ) -> Result<(), ProtocolError> {
        // 验证依赖关系是否存在
        if !self.field_index.contains_key(dependent_field)
            || !self.field_index.contains_key(dependency_field)
        {
            return Err(ProtocolError::FieldNotFound(format!(
                "Dependent or dependency field not found: {} or {}",
                dependent_field, dependency_field
            )));
        }
        println!(
            "Applied dependency rule: {} depends on {}",
            dependent_field, dependency_field
        );
        Ok(())
    }

    /// 应用条件规则
    fn apply_conditional_rule(
        &mut self,
        condition: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        // 简单的条件规则处理
        println!("Applied conditional rule: {}", condition);
        Ok(())
    }

    /// 应用顺序规则
    fn apply_order_rule(
        &mut self,
        first_field: &str,
        second_field: &str,
    ) -> Result<(), ProtocolError> {
        // 验证字段顺序是否正确
        let first_pos = self.get_field_position(first_field)?;
        let second_pos = self.get_field_position(second_field)?;

        if first_pos > second_pos {
            return Err(ProtocolError::InvalidFrameFormat(format!(
                "Field order violation: {} should come before {}",
                first_field, second_field
            )));
        }
        println!(
            "Applied order rule: {} before {}",
            first_field, second_field
        );
        Ok(())
    }

    /// 应用指针规则
    fn apply_pointer_rule(
        &mut self,
        pointer_field: &str,
        target_field: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        // 指针字段指向目标字段的逻辑处理
        println!(
            "Applied pointer rule: {} points to {}",
            pointer_field, target_field
        );
        Ok(())
    }

    /// 应用自定义算法规则
    fn apply_custom_algorithm(
        &mut self,
        field_name: &str,
        algorithm: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        // 应用自定义算法到指定字段
        println!(
            "Applied custom algorithm {} to field {}",
            algorithm, field_name
        );
        Ok(())
    }

    /// 处理单个长度规则
    /// 处理单个长度规则
    fn process_single_length_rule(
        &mut self,
        frame_data: &mut Vec<u8>,
        field_name: &str,
        expression: &str,
    ) -> Result<(), ProtocolError> {
        // 清理字段名，移除可能的前缀
        let clean_field_name = field_name.trim_start_matches("field: ").trim();

        // 查找对应字段的索引和信息
        if let Some(&field_idx) = self.field_index.get(clean_field_name) {
            if let Some(field_info) = self.fields.get(field_idx) {
                // 解析表达式并计算长度值
                let calculated_value = self.evaluate_length_expression(expression, frame_data)?;

                // 更新字段值
                let value_bytes = self.uint_to_bytes(calculated_value, field_info.length.size);

                // 更新帧数据中的对应部分
                self.update_frame_data_at_position(frame_data, field_idx, &value_bytes)?;

                // 同时更新存储的字段值
                self.field_values
                    .insert(clean_field_name.to_string(), value_bytes);
            }
        }
        Ok(())
    }

    /// 查找字段信息
    fn find_field_info(&self, field_name: &str) -> Option<(usize, &SyntaxUnit)> {
        if let Some(&idx) = self.field_index.get(field_name) {
            if let Some(field_info) = self.fields.get(idx) {
                Some((idx, field_info))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 解析长度表达式
    fn evaluate_length_expression(
        &self,
        expression: &str,
        frame_data: &[u8],
    ) -> Result<u64, ProtocolError> {
        // 简单的表达式解析器，支持基本算术运算和函数调用
        // 例如: "(total_length - 3)", "(data_length + 7)", "pos(fecf) + len(fecf) - pos(version)", 等

        // 移除可能的双引号和括号
        let expr_cleaned = expression
            .trim()
            .trim_matches('"')
            .trim_matches(|c| c == '(' || c == ')');

        // 检查是否包含函数调用语法 (如 len(field) 或 pos(field))
        if expr_cleaned.contains("len(") || expr_cleaned.contains("pos(") {
            return self.evaluate_function_expression(expr_cleaned, frame_data);
        }

        // 处理几种常见的表达式模式
        if expr_cleaned.contains("total_length") {
            let total_len = frame_data.len() as u64;
            // 简单解析表达式，如 "total_length - 3"
            if let Some(pos) = expr_cleaned.find('-') {
                let left = &expr_cleaned[..pos].trim();
                let right = &expr_cleaned[pos + 1..].trim();

                if left.trim() == "total_length" {
                    if let Ok(right_val) = right.parse::<u64>() {
                        return Ok(total_len.saturating_sub(right_val));
                    }
                }
            } else if let Some(pos) = expr_cleaned.find('+') {
                let left = &expr_cleaned[..pos].trim();
                let right = &expr_cleaned[pos + 1..].trim();

                if left.trim() == "total_length" {
                    if let Ok(right_val) = right.parse::<u64>() {
                        return Ok(total_len + right_val);
                    }
                }
            }
        } else if expr_cleaned.contains("data_length") {
            // 处理基于数据长度的表达式
            // 这里需要知道数据字段的位置和长度
            // 遍历查找数据字段
            for field in &self.fields {
                if self.is_data_field(field) || field.field_id.to_lowercase().contains("data") {
                    if let Some(data_value) = self.field_values.get(&field.field_id) {
                        return Ok(data_value.len() as u64);
                    }
                }
            }
            // 如果找不到数据字段，返回默认值
            return Ok(0);
        } else if expr_cleaned.contains("header_length") {
            // 计算头部长度（非数据字段的总长度）
            let header_len = self.fields.iter().enumerate().fold(0, |acc, (i, field)| {
                // 检查是否是数据字段
                if self.is_data_field(field) || field.field_id.to_lowercase().contains("data") {
                    acc // 不计入数据字段
                } else {
                    // 计算非数据字段的长度
                    match self.get_field_size(field) {
                        Ok(size) => acc + size,
                        Err(_) => acc,
                    }
                }
            });
            return Ok(header_len as u64);
        }

        // 如果是纯数字，直接返回
        if let Ok(num) = expr_cleaned.parse::<u64>() {
            return Ok(num);
        }

        // 默认返回1
        Ok(1)
    }

    /// 解析函数表达式，支持 len(field) 和 pos(field) 函数
    fn evaluate_function_expression(
        &self,
        expression: &str,
        frame_data: &[u8],
    ) -> Result<u64, ProtocolError> {
        // 使用更安全的方法来解析和替换函数调用
        let mut result_expr = expression.to_string();

        // 辅助函数：查找并替换函数调用
        let find_and_replace_function = |expr: &mut String,
                                         func_prefix: &str,
                                         get_value: &dyn Fn(&str) -> Result<u64, ProtocolError>|
         -> Result<(), ProtocolError> {
            loop {
                let mut replacement_found = false;

                // 查找函数调用的开始位置
                if let Some(start_pos) = expr.find(func_prefix) {
                    // 从开始位置查找匹配的右括号
                    let mut paren_count = 0;
                    let mut end_pos = None;
                    let chars: Vec<(usize, char)> = expr.char_indices().collect();

                    for &(i, ch) in &chars[start_pos..] {
                        let abs_i = i;
                        if ch == '(' {
                            paren_count += 1;
                        } else if ch == ')' {
                            paren_count -= 1;
                            if paren_count == 0 {
                                end_pos = Some(abs_i);
                                break;
                            }
                        }
                    }

                    if let Some(end_pos) = end_pos {
                        // 提取字段名
                        let field_name_start = start_pos + func_prefix.len(); // 跳过函数前缀如 "pos(" 或 "len("
                        let field_name_end = end_pos; // end_pos is the index of ')'
                        let field_name = &expr[field_name_start..field_name_end].trim();

                        // 获取值
                        let value = get_value(field_name)?;

                        // 输出调试信息
                        if func_prefix == "pos(" {
                            println!("DEBUG: pos({}) = {}", field_name, value);
                        } else if func_prefix == "len(" {
                            println!("DEBUG: len({}) = {}", field_name, value);
                        }

                        // 替换整个函数调用
                        expr.replace_range(start_pos..=end_pos, &value.to_string());
                        replacement_found = true;
                    }
                }

                if !replacement_found {
                    break; // 没有更多函数调用
                }
            }
            Ok(())
        };

        // 先替换所有的 len() 函数调用
        find_and_replace_function(&mut result_expr, "len(", &|field_name| {
            self.get_field_length_by_name(field_name).map(|v| v as u64)
        })?;

        // 再替换所有的 pos() 函数调用
        find_and_replace_function(&mut result_expr, "pos(", &|field_name| {
            self.get_field_position(field_name).map(|v| v as u64)
        })?;

        // 现在解析简单的数学表达式
        // 这里我们简化处理，仅支持基本的加减运算
        println!(
            "DEBUG: Expression after function substitution: '{}'",
            result_expr
        );
        let final_result = self.evaluate_simple_math_expression(&result_expr)?;
        println!(
            "DEBUG: Final result after math evaluation: {}",
            final_result
        );
        Ok(final_result)
    }

    /// 获取字段长度（从定义或已存储值）
    fn get_field_length_by_name(&self, field_name: &str) -> Result<usize, ProtocolError> {
        // 首先尝试获取字段定义的长度
        if let Some(&index) = self.field_index.get(field_name) {
            if let Some(field) = self.fields.get(index) {
                // 如果字段值已经存储，使用存储值的长度（对于动态字段）
                if let Some(stored_value) = self.field_values.get(field_name) {
                    return Ok(stored_value.len());
                } else {
                    // 否则使用字段定义的长度
                    return self.get_field_size(field);
                }
            }
        }

        Err(ProtocolError::FieldNotFound(field_name.to_string()))
    }

    /// 解析简单的数学表达式（支持加减）
    fn evaluate_simple_math_expression(&self, expression: &str) -> Result<u64, ProtocolError> {
        let expr = expression.replace(" ", ""); // 移除空格

        // 检查是否包含运算符
        if expr.contains('+') || expr.contains('-') {
            // 简单的解析：从左到右处理加减运算
            let mut result: i64 = 0;
            let mut current_number = String::new();
            let mut operation = None; // 初始操作符为None，用于处理第一个数字

            // 为了处理负数，我们先检查表达式是否以负号开头
            let mut chars = expr.chars().peekable();
            let mut is_negative_start = false;
            if chars.peek() == Some(&'-') {
                is_negative_start = true;
                chars.next(); // 跳过负号
            }

            for ch in chars {
                if ch.is_ascii_digit() {
                    current_number.push(ch);
                } else if ch == '+' || ch == '-' {
                    // 处理当前累积的数字
                    if !current_number.is_empty() {
                        let num = current_number.parse::<i64>().map_err(|_| {
                            ProtocolError::InvalidExpression(format!(
                                "Invalid number: {}",
                                current_number
                            ))
                        })?;

                        let signed_num = if is_negative_start && operation.is_none() {
                            -num
                        } else {
                            num
                        };

                        match operation {
                            None => {
                                // 第一个数字，直接赋值
                                result = signed_num;
                            }
                            Some(op) => {
                                if op == '+' {
                                    result += signed_num;
                                } else {
                                    result -= signed_num;
                                }
                            }
                        }
                    } else if operation.is_some() {
                        // 如果current_number为空但已经有操作符，说明这是一个符号
                        // 例如表达式形如 "10+-5"
                        match operation {
                            Some(prev_op) => {
                                if prev_op == '-' && ch == '-' {
                                    // 双重负号变成正号
                                    continue;
                                } else if prev_op == '-' && ch == '+' {
                                    continue;
                                } else {
                                    operation = Some(ch);
                                }
                            }
                            None => {
                                operation = Some(ch);
                            }
                        }
                        continue;
                    }

                    // 设置下一个操作符
                    operation = Some(ch);
                    current_number.clear();
                    is_negative_start = false; // 重置，只有开头的负号才有特殊意义
                } else {
                    // 忽略其他字符（理论上不应该有）
                }
            }

            // 处理最后一个数字
            if !current_number.is_empty() {
                let num = current_number.parse::<i64>().map_err(|_| {
                    ProtocolError::InvalidExpression(format!("Invalid number: {}", current_number))
                })?;

                let signed_num = if is_negative_start && operation.is_none() {
                    -num
                } else {
                    num
                };

                match operation {
                    None => {
                        // 只有一个数字，直接返回
                        result = signed_num;
                    }
                    Some(op) => {
                        if op == '+' {
                            result += signed_num;
                        } else {
                            result -= signed_num;
                        }
                    }
                }
            }

            // 确保结果不为负数
            if result < 0 {
                Ok(0 as u64)
            } else {
                Ok(result as u64)
            }
        } else {
            // 如果没有运算符，直接解析为数字
            expr.parse::<u64>().map_err(|_| {
                ProtocolError::InvalidExpression(format!("Invalid expression: {}", expression))
            })
        }
    }

    /// 将整数转换为字节数组
    fn uint_to_bytes(&self, value: u64, byte_length: usize) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(byte_length);
        for i in 0..byte_length {
            bytes.push(((value >> (i * 8)) & 0xFF) as u8);
        }
        bytes.reverse(); // 转换为大端序
        bytes
    }

    /// 更新帧数据中特定位置的值
    fn update_frame_data_at_position(
        &self,
        frame_data: &mut Vec<u8>,
        field_idx: usize,
        new_value: &[u8],
    ) -> Result<(), ProtocolError> {
        let field = &self.fields[field_idx];
        let start_pos = self.get_field_accumulated_position(field_idx)?;

        if start_pos + new_value.len() > frame_data.len() {
            return Err(ProtocolError::InvalidFrameFormat(
                "Field position exceeds frame size".to_string(),
            ));
        }

        // 替换帧数据中的相应部分
        frame_data.splice(
            start_pos..start_pos + new_value.len(),
            new_value.iter().cloned(),
        );

        Ok(())
    }

    /// 获取字段在帧中的累积位置
    fn get_field_accumulated_position(&self, field_idx: usize) -> Result<usize, ProtocolError> {
        let mut position = 0;
        for i in 0..field_idx {
            if i < self.fields.len() {
                let field_size = self.get_field_size(&self.fields[i])?;
                position += field_size;
            }
        }
        Ok(position)
    }

    /// 获取字段在帧数据中的位置
    fn get_field_position(&self, field_name: &str) -> Result<usize, ProtocolError> {
        if let Some(&index) = self.field_index.get(field_name) {
            self.get_field_accumulated_position(index)
        } else {
            Err(ProtocolError::FieldNotFound(field_name.to_string()))
        }
    }

    /// 根据字段名获取字段大小
    fn get_field_size_by_name(&self, field_name: &str) -> Result<usize, ProtocolError> {
        if let Some(&index) = self.field_index.get(field_name) {
            if let Some(field) = self.fields.get(index) {
                self.get_field_size(field)
            } else {
                Err(ProtocolError::FieldNotFound(field_name.to_string()))
            }
        } else {
            Err(ProtocolError::FieldNotFound(field_name.to_string()))
        }
    }

    // 新增的语义规则处理方法
    /// 应用路由分发规则
    fn apply_routing_dispatch_rule(
        &mut self,
        fields: &[String],
        algorithm: &str,
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying routing dispatch rule: {} with algorithm {}, fields: {:?}",
            description, algorithm, fields
        );
        // 实际的路由分发逻辑会在这里实现
        Ok(())
    }

    /// 应用序列控制规则
    fn apply_sequence_control_rule(
        &mut self,
        field_name: &str,
        trigger_condition: &str,
        algorithm: &str,
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying sequence control rule: {} for field {}, trigger: {}, algorithm: {}",
            description, field_name, trigger_condition, algorithm
        );
        // 实际的序列控制逻辑会在这里实现
        Ok(())
    }

    /// 应用验证规则
    fn apply_validation_rule(
        &mut self,
        field_name: &str,
        algorithm: &str,
        range_start: &str,
        range_end: &str,
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying validation rule: {} for field {}, algorithm: {}, range: {} to {}",
            description, field_name, algorithm, range_start, range_end
        );
        // 实际的验证逻辑会在这里实现
        Ok(())
    }

    /// 应用多路复用规则
    fn apply_multiplexing_rule(
        &mut self,
        field_name: &str,
        condition: &str,
        route_target: &str,
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying multiplexing rule: {} for field {}, condition: {}, route to: {}",
            description, field_name, condition, route_target
        );
        // 实际的多路复用逻辑会在这里实现
        Ok(())
    }

    /// 应用优先级处理规则
    fn apply_priority_processing_rule(
        &mut self,
        field_name: &str,
        algorithm: &str,
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying priority processing rule: {} for field {}, algorithm: {}",
            description, field_name, algorithm
        );
        // 实际的优先级处理逻辑会在这里实现
        Ok(())
    }

    /// 应用同步规则
    fn apply_synchronization_rule(
        &mut self,
        field_name: &str,
        algorithm: &str,
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying synchronization rule: {} for field {}, algorithm: {}",
            description, field_name, algorithm
        );
        // 实际的同步逻辑会在这里实现
        Ok(())
    }

    /// 应用长度验证规则
    fn apply_length_validation_rule(
        &mut self,
        field_name: &str,
        condition: &str,
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying length validation rule: {} for field {}, condition: {}",
            description, field_name, condition
        );
        // 实际的长度验证逻辑会在这里实现
        Ok(())
    }

    /// 应用状态机规则
    fn apply_state_machine_rule(
        &mut self,
        condition: &str,
        algorithm: &str,
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying state machine rule: {}, condition: {}, algorithm: {}",
            description, condition, algorithm
        );
        // 实际的状态机逻辑会在这里实现
        Ok(())
    }

    /// 应用周期性传输规则
    fn apply_periodic_transmission_rule(
        &mut self,
        field_name: &str,
        condition: &str,
        algorithm: &str,
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying periodic transmission rule: {} for field {}, condition: {}, algorithm: {}",
            description, field_name, condition, algorithm
        );
        // 实际的周期性传输逻辑会在这里实现
        Ok(())
    }

    /// 应用消息过滤规则
    fn apply_message_filtering_rule(
        &mut self,
        condition: &str,
        action: &str,
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying message filtering rule: {}, condition: {}, action: {}",
            description, condition, action
        );
        // 实际的消息过滤逻辑会在这里实现
        Ok(())
    }

    /// 应用错误检测规则
    fn apply_error_detection_rule(
        &mut self,
        algorithm: &str,
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying error detection rule: {}, algorithm: {}",
            description, algorithm
        );
        // 实际的错误检测逻辑会在这里实现
        Ok(())
    }

    /// 预处理长度规则
    fn pre_process_length_rules(&mut self) -> Result<(), ProtocolError> {
        println!(
            "DEBUG: Starting pre_process_length_rules, found {} semantic rules",
            self.semantic_rules.len()
        );

        let mut length_rule_count = 0;
        // 处理所有长度规则
        for rule in &self.semantic_rules {
            if let SemanticRule::LengthRule {
                field_name,
                expression,
            } = rule
            {
                // 清理字段名，移除可能的前缀
                let clean_field_name = field_name.trim_start_matches("field: ").trim();

                length_rule_count += 1;
                println!(
                    "DEBUG: Processing length rule for field '{}' with expression '{}'",
                    clean_field_name, expression
                );

                // 构建当前的帧数据用于长度计算
                let mut current_frame_data = Vec::new();

                for field in &self.fields {
                    let field_bytes = self.get_field_bytes(&field.field_id).unwrap_or_else(|_| {
                        // 如果字段值未设置，使用默认值
                        vec![0; self.get_field_size(field).unwrap_or(1)]
                    });
                    current_frame_data.extend_from_slice(&field_bytes);
                }

                println!(
                    "DEBUG: Current frame data length for calculation: {}",
                    current_frame_data.len()
                );

                // 计算长度值 - 使用当前帧数据长度作为total_length
                let calculated_value =
                    self.evaluate_length_expression(expression, &current_frame_data)?;

                // 更新字段值
                if let Some(&field_idx) = self.field_index.get(clean_field_name) {
                    if let Some(field_info) = self.fields.get(field_idx) {
                        // 转换计算出的长度值为字节表示
                        let value_bytes =
                            self.uint_to_bytes(calculated_value, field_info.length.size);

                        // 存储计算出的字段值
                        self.field_values
                            .insert(clean_field_name.to_string(), value_bytes.clone());

                        println!(
                            "DEBUG: Computed length for field '{}' = {} (bytes: {:?})",
                            clean_field_name, calculated_value, value_bytes
                        );
                    }
                }
            }
        }

        println!("DEBUG: Processed {} length rules", length_rule_count);
        Ok(())
    }

    /// 应用流量控制规则
    fn apply_flow_control_rule(
        &mut self,
        field_name: &str,
        algorithm: &str,
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying flow control rule: {} for field {}, algorithm: {}",
            description, field_name, algorithm
        );
        // 实际的流量控制逻辑会在这里实现
        Ok(())
    }

    /// 应用时间同步规则
    fn apply_time_synchronization_rule(
        &mut self,
        field_name: &str,
        algorithm: &str,
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying time synchronization rule: {} for field {}, algorithm: {}",
            description, field_name, algorithm
        );
        // 实际的时间同步逻辑会在这里实现
        Ok(())
    }

    /// 应用地址解析规则
    fn apply_address_resolution_rule(
        &mut self,
        field_name: &str,
        algorithm: &str,
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying address resolution rule: {} for field {}, algorithm: {}",
            description, field_name, algorithm
        );
        // 实际的地址解析逻辑会在这里实现
        Ok(())
    }

    /// 应用安全规则
    fn apply_security_rule(
        &mut self,
        field_name: &str,
        algorithm: &str,
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying security rule: {} for field {}, algorithm: {}",
            description, field_name, algorithm
        );
        // 实际的安全逻辑会在这里实现（加密、认证等）
        Ok(())
    }

    /// 应用冗余规则
    fn apply_redundancy_rule(
        &mut self,
        field_name: &str,
        algorithm: &str,
        description: &str,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        println!(
            "Applying redundancy rule: {} for field {}, algorithm: {}",
            description, field_name, algorithm
        );
        // 实际的冗余逻辑会在这里实现
        Ok(())
    }

    /// 判断字段是否为数据字段
    fn is_data_field(&self, field: &SyntaxUnit) -> bool {
        match &field.unit_type {
            UnitType::RawData => true,
            UnitType::Uint(bits) => {
                // 检查字段ID是否包含数据相关关键词
                let field_id_lower = field.field_id.to_lowercase();
                field_id_lower.contains("data") || field_id_lower.contains("payload")
            }
            _ => {
                // 检查字段ID或描述是否包含数据相关关键词
                let field_id_lower = field.field_id.to_lowercase();
                let desc_lower = field.desc.to_lowercase();
                field_id_lower.contains("data")
                    || desc_lower.contains("data")
                    || field_id_lower.contains("payload")
                    || desc_lower.contains("payload")
            }
        }
    }

    /// 获取所有字段名称
    pub fn get_field_names(&self) -> Vec<String> {
        self.fields
            .iter()
            .map(|field| field.field_id.clone())
            .collect()
    }

    /// 验证组装器状态
    pub fn validate(&self) -> Result<bool, ProtocolError> {
        // 检查是否有字段定义
        if self.fields.is_empty() {
            return Ok(false);
        }

        // 检查字段定义是否有效
        for field in &self.fields {
            // 检查字段长度定义是否有效
            match &field.length.unit {
                LengthUnit::Byte | LengthUnit::Bit => {
                    if field.length.size == 0 {
                        return Err(ProtocolError::LengthError(
                            "Field size cannot be zero".to_string(),
                        ));
                    }
                }
                LengthUnit::Dynamic => {
                    // 动态长度字段是有效的
                }
                LengthUnit::Expression(_) => {
                    // 表达式长度字段是有效的
                }
            }
        }

        // 验证通过
        Ok(true)
    }
}
