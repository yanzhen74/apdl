//! Frame Assembler 核心结构和基础方法
//!
//! 包含 FrameAssembler 结构体定义和基础功能方法

use apdl_core::{
    LengthUnit, ProtocolError, SemanticRule,
    SyntaxUnit,
};
use std::collections::HashMap;

/// 协议帧组装器
pub struct FrameAssembler {
    pub fields: Vec<SyntaxUnit>,
    pub semantic_rules: Vec<SemanticRule>,
    pub field_index: HashMap<String, usize>,
    // 添加字段值存储
    pub field_values: HashMap<String, Vec<u8>>,
}

impl Default for FrameAssembler {
    fn default() -> Self {
        Self::new()
    }
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
                println!("Setting field {clean_field_name} to value: {value:?}");
                Ok(())
            } else {
                Err(ProtocolError::FieldNotFound(format!(
                    "Field not found: {clean_field_name}"
                )))
            }
        } else {
            Err(ProtocolError::FieldNotFound(format!(
                "Field not found: {clean_field_name}"
            )))
        }
    }

    /// 获取字段值
    pub fn get_field_value(&self, field_name: &str) -> Result<Vec<u8>, ProtocolError> {
        let clean_field_name = field_name.trim_start_matches("field: ").trim();
        self.field_values
            .get(clean_field_name)
            .cloned()
            .ok_or_else(|| {
                ProtocolError::FieldNotFound(format!("Field value not found: {clean_field_name}"))
            })
    }

    /// 获取字段字节
    fn get_field_bytes(&self, field_name: &str) -> Result<Vec<u8>, ProtocolError> {
        let clean_field_name = field_name.trim_start_matches("field: ").trim();
        if let Some(bytes) = self.field_values.get(clean_field_name) {
            Ok(bytes.clone())
        } else {
            // 如果字段值未设置，返回默认值
            if let Some(&index) = self.field_index.get(clean_field_name) {
                if let Some(field) = self.fields.get(index) {
                    let size = self.get_field_size(field)?;
                    Ok(vec![0; size])
                } else {
                    Err(ProtocolError::FieldNotFound(format!(
                        "Field definition not found: {clean_field_name}"
                    )))
                }
            } else {
                Err(ProtocolError::FieldNotFound(format!(
                    "Field not found: {clean_field_name}"
                )))
            }
        }
    }

    /// 获取字段大小
    pub fn get_field_size(&self, field: &SyntaxUnit) -> Result<usize, ProtocolError> {
        match field.length.unit {
            LengthUnit::Bit => Ok(field.length.size.div_ceil(8)), // 向上取整到字节
            LengthUnit::Byte => Ok(field.length.size),
            LengthUnit::Dynamic => {
                // 对于动态长度字段，尝试从已存储的值中获取大小
                if let Some(stored_value) = self.field_values.get(&field.field_id) {
                    Ok(stored_value.len())
                } else {
                    // 默认大小
                    Ok(1)
                }
            }
            LengthUnit::Expression(_) => {
                // 对于表达式长度，需要先计算表达式的值再确定大小
                // 这里我们先返回一个合理的默认值，实际大小会在长度规则处理阶段更新
                Ok(4) // 使用4字节作为表达式长度的默认大小
            }
        }
    }

    /// 获取字段大小（通过字段名）
    pub fn get_field_size_by_name(&self, field_name: &str) -> Result<usize, ProtocolError> {
        let clean_field_name = field_name.trim_start_matches("field: ").trim();
        if let Some(&index) = self.field_index.get(clean_field_name) {
            if let Some(field) = self.fields.get(index) {
                self.get_field_size(field)
            } else {
                Err(ProtocolError::FieldNotFound(format!(
                    "Field definition not found: {clean_field_name}"
                )))
            }
        } else {
            Err(ProtocolError::FieldNotFound(format!(
                "Field not found: {clean_field_name}"
            )))
        }
    }

    /// 获取字段位置
    pub fn get_field_position(&self, field_name: &str) -> Result<usize, ProtocolError> {
        let clean_field_name = field_name.trim_start_matches("field: ").trim();
        if let Some(&index) = self.field_index.get(clean_field_name) {
            self.calculate_field_offset(index)
        } else {
            Err(ProtocolError::FieldNotFound(format!(
                "Field not found: {clean_field_name}"
            )))
        }
    }

    /// 计算字段在帧中的偏移量
    pub fn calculate_field_offset(&self, field_index: usize) -> Result<usize, ProtocolError> {
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
    pub fn u64_to_bytes(&self, value: u64, size: usize) -> Vec<u8> {
        let mut bytes = Vec::new();
        for i in 0..size {
            bytes.push(((value >> (8 * (size - 1 - i))) & 0xFF) as u8);
        }
        bytes
    }

    /// 获取所有字段名称
    pub fn get_field_names(&self) -> Vec<&str> {
        self.fields
            .iter()
            .map(|field| field.field_id.as_str())
            .collect()
    }

    /// 验证组装器状态
    pub fn validate(&self) -> Result<bool, ProtocolError> {
        // 简单验证：检查是否有字段定义
        if self.fields.is_empty() {
            return Err(ProtocolError::InvalidFrameFormat(
                "No fields defined in FrameAssembler".to_string(),
            ));
        }
        Ok(true)
    }
}

impl FrameAssembler {
    /// 将字节数组转换为u64
    pub fn bytes_to_u64(&self, bytes: &[u8]) -> u64 {
        crate::standard_units::frame_assembler::utils::bytes_to_u64(bytes)
    }

    /// 判断是否为数据字段
    pub fn is_data_field(&self, field: &SyntaxUnit) -> bool {
        crate::standard_units::frame_assembler::utils::is_data_field(field)
    }

    /// 判断是否为头部字段
    pub fn is_header_field(&self, field: &SyntaxUnit) -> bool {
        crate::standard_units::frame_assembler::utils::is_header_field(field)
    }

    /// 通配符匹配实现
    pub fn wildcard_match(&self, text: &str, pattern: &str) -> bool {
        crate::standard_units::frame_assembler::utils::wildcard_match(text, pattern)
    }

    /// 计算数据的哈希值
    pub fn calculate_hash(&self, data: &[u8]) -> u64 {
        crate::standard_units::frame_assembler::utils::calculate_hash(data)
    }

    /// 计算CRC16校验和
    pub fn calculate_crc16(&self, data: &[u8]) -> u16 {
        crate::standard_units::frame_assembler::utils::calculate_crc16(data)
    }

    /// 计算简单校验和
    pub fn calculate_simple_checksum(&self, data: &[u8]) -> u16 {
        crate::standard_units::frame_assembler::utils::calculate_simple_checksum(data)
    }

    /// 计算XOR校验和
    pub fn calculate_xor(&self, data: &[u8]) -> u16 {
        crate::standard_units::frame_assembler::utils::calculate_xor(data)
    }

    /// 计算CRC15校验和 (CAN协议专用)
    pub fn calculate_crc15(&self, data: &[u8]) -> u16 {
        // CAN协议使用的CRC15算法
        let mut crc: u16 = 0x0000;
        for &byte in data {
            for i in 0..8 {
                let mut bit = (byte >> (7 - i)) & 0x01;
                if (crc & 0x4000) != 0 {
                    bit ^= 1;
                }
                crc <<= 1;
                if bit != 0 {
                    crc ^= 0x0599;
                }
            }
        }
        crc &= 0x7FFF; // 保留低15位
        crc
    }
}
