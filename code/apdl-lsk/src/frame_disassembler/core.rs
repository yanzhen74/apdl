//! FrameDisassembler核心实现
//!
//! 与FrameAssembler对称的拆包器，负责从二进制帧数据中提取字段

use apdl_core::{LengthUnit, ProtocolError, SemanticRule, SyntaxUnit, UnitType};
use std::collections::HashMap;

use super::bit_extractor::extract_bit_field;

/// 帧拆包器
///
/// 与FrameAssembler对称，用于从二进制帧数据中提取字段值
#[derive(Clone)]
pub struct FrameDisassembler {
    /// 字段定义列表
    pub fields: Vec<SyntaxUnit>,
    /// 语义规则
    pub semantic_rules: Vec<SemanticRule>,
    /// 字段名到索引的映射
    pub field_index: HashMap<String, usize>,
}

impl Default for FrameDisassembler {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameDisassembler {
    /// 创建新的帧拆包器
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            semantic_rules: Vec::new(),
            field_index: HashMap::new(),
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

    /// 解析帧数据，提取所有字段
    ///
    /// # 参数
    /// - `frame_data`: 原始帧数据
    ///
    /// # 返回
    /// - `Ok(HashMap<String, Vec<u8>>)`: 字段名到字段值的映射
    /// - `Err(ProtocolError)`: 解析错误
    pub fn disassemble_frame(
        &self,
        frame_data: &[u8],
    ) -> Result<HashMap<String, Vec<u8>>, ProtocolError> {
        let mut fields = HashMap::new();
        let mut bit_offset = 0usize; // 当前bit偏移

        for field in &self.fields {
            let field_name = &field.field_id;

            // 根据字段类型提取值
            let value = match field.unit_type {
                UnitType::Bit(bits) => {
                    // 提取bit字段
                    let bit_value = extract_bit_field(frame_data, bit_offset, bits as usize)?;
                    bit_offset += bits as usize;

                    // 将bit值转换为字节数组
                    self.u64_to_bytes(bit_value, (bits as usize).div_ceil(8))
                }
                UnitType::Uint(bits) => {
                    // 字节对齐的整数字段
                    let byte_offset = bit_offset.div_ceil(8);
                    let byte_size = (bits as usize) / 8;

                    if byte_offset + byte_size > frame_data.len() {
                        return Err(ProtocolError::InvalidFrameFormat(format!(
                            "Field {field_name} exceeds frame boundary"
                        )));
                    }

                    let value = frame_data[byte_offset..byte_offset + byte_size].to_vec();
                    bit_offset = (byte_offset + byte_size) * 8;
                    value
                }
                UnitType::RawData => {
                    // 动态长度数据字段，提取剩余所有数据
                    let byte_offset = bit_offset.div_ceil(8);
                    let value = frame_data[byte_offset..].to_vec();
                    bit_offset = frame_data.len() * 8;
                    value
                }
                UnitType::Ip6Addr => {
                    // IPv6地址，固定16字节
                    let byte_offset = bit_offset.div_ceil(8);
                    if byte_offset + 16 > frame_data.len() {
                        return Err(ProtocolError::InvalidFrameFormat(
                            "IPv6 address field exceeds frame boundary".to_string(),
                        ));
                    }
                    let value = frame_data[byte_offset..byte_offset + 16].to_vec();
                    bit_offset = (byte_offset + 16) * 8;
                    value
                }
            };

            fields.insert(field_name.clone(), value);
        }

        Ok(fields)
    }

    /// 提取单个字段值（支持bit字段）
    ///
    /// # 参数
    /// - `frame_data`: 原始帧数据
    /// - `field_name`: 字段名
    ///
    /// # 返回
    /// - `Ok(Vec<u8>)`: 字段值
    /// - `Err(ProtocolError)`: 字段不存在或提取失败
    pub fn extract_field_value(
        &self,
        frame_data: &[u8],
        field_name: &str,
    ) -> Result<Vec<u8>, ProtocolError> {
        // 先解析整个帧
        let fields = self.disassemble_frame(frame_data)?;

        // 获取目标字段
        fields
            .get(field_name)
            .cloned()
            .ok_or_else(|| ProtocolError::FieldNotFound(format!("Field not found: {field_name}")))
    }

    /// 获取字段的bit级位置
    ///
    /// # 参数
    /// - `field_name`: 字段名
    ///
    /// # 返回
    /// - `Ok((bit_offset, bit_length))`: 字段的bit偏移和bit长度
    /// - `Err(ProtocolError)`: 字段不存在
    pub fn get_field_bit_position(
        &self,
        field_name: &str,
    ) -> Result<(usize, usize), ProtocolError> {
        let Some(&field_index) = self.field_index.get(field_name) else {
            return Err(ProtocolError::FieldNotFound(format!(
                "Field not found: {field_name}"
            )));
        };

        let Some(field) = self.fields.get(field_index) else {
            return Err(ProtocolError::FieldNotFound(format!(
                "Field definition not found for index: {field_index}"
            )));
        };

        // 计算该字段之前所有字段占用的总bit数
        let mut total_bits_before = 0usize;
        for i in 0..field_index {
            if let Some(prev_field) = self.fields.get(i) {
                let field_bits = self.get_field_bit_length(prev_field)?;
                total_bits_before += field_bits;
            }
        }

        // 获取当前字段的bit长度
        let field_bit_length = self.get_field_bit_length(field)?;

        Ok((total_bits_before, field_bit_length))
    }

    /// 获取字段的bit长度
    fn get_field_bit_length(&self, field: &SyntaxUnit) -> Result<usize, ProtocolError> {
        match field.length.unit {
            LengthUnit::Bit => {
                if let UnitType::Bit(bits) = field.unit_type {
                    Ok(bits as usize)
                } else {
                    Ok(field.length.size)
                }
            }
            LengthUnit::Byte => Ok(field.length.size * 8),
            LengthUnit::Dynamic => {
                // 动态长度字段，默认返回0（需要根据实际数据确定）
                Ok(0)
            }
            LengthUnit::Expression(_) => Ok(0),
        }
    }

    /// 将u64值转换为字节数组
    fn u64_to_bytes(&self, value: u64, size: usize) -> Vec<u8> {
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

    /// 打印解析结果（调试用）
    pub fn print_disassemble_result(&self, fields: &HashMap<String, Vec<u8>>) {
        println!("\n=== 帧拆包结果 ===");
        for (field_name, value) in fields {
            print!("{field_name}: ");
            for byte in value {
                print!("{byte:02X} ");
            }
            println!();
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apdl_core::{Constraint, CoverDesc, LengthDesc, ScopeDesc};

    #[test]
    fn test_disassemble_simple_frame() {
        // 创建简单的帧定义：version(1 byte) + data(4 bytes)
        let version_field = SyntaxUnit {
            field_id: "version".to_string(),
            unit_type: UnitType::Uint(8),
            length: LengthDesc {
                size: 1,
                unit: LengthUnit::Byte,
            },
            scope: ScopeDesc::Global("test".to_string()),
            cover: CoverDesc::EntireField,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: "Version".to_string(),
        };

        let data_field = SyntaxUnit {
            field_id: "data".to_string(),
            unit_type: UnitType::Uint(32),
            length: LengthDesc {
                size: 4,
                unit: LengthUnit::Byte,
            },
            scope: ScopeDesc::Global("test".to_string()),
            cover: CoverDesc::EntireField,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: "Data".to_string(),
        };

        let mut disassembler = FrameDisassembler::new();
        disassembler.add_field(version_field);
        disassembler.add_field(data_field);

        // 测试帧数据
        let frame_data = vec![0x01, 0xDE, 0xAD, 0xBE, 0xEF];

        let fields = disassembler
            .disassemble_frame(&frame_data)
            .expect("Failed to disassemble");

        assert_eq!(fields.get("version").unwrap(), &vec![0x01]);
        assert_eq!(fields.get("data").unwrap(), &vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_disassemble_bit_fields() {
        // 测试bit字段拆包：version(3bit) + type(1bit) + flag(1bit) + apid(11bit)
        let version_field = SyntaxUnit {
            field_id: "version".to_string(),
            unit_type: UnitType::Bit(3),
            length: LengthDesc {
                size: 3,
                unit: LengthUnit::Bit,
            },
            scope: ScopeDesc::Global("test".to_string()),
            cover: CoverDesc::EntireField,
            constraint: Some(Constraint::FixedValue(0)),
            alg: None,
            associate: vec![],
            desc: "Version".to_string(),
        };

        let type_field = SyntaxUnit {
            field_id: "type".to_string(),
            unit_type: UnitType::Bit(1),
            length: LengthDesc {
                size: 1,
                unit: LengthUnit::Bit,
            },
            scope: ScopeDesc::Global("test".to_string()),
            cover: CoverDesc::EntireField,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: "Type".to_string(),
        };

        let flag_field = SyntaxUnit {
            field_id: "flag".to_string(),
            unit_type: UnitType::Bit(1),
            length: LengthDesc {
                size: 1,
                unit: LengthUnit::Bit,
            },
            scope: ScopeDesc::Global("test".to_string()),
            cover: CoverDesc::EntireField,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: "Flag".to_string(),
        };

        let apid_field = SyntaxUnit {
            field_id: "apid".to_string(),
            unit_type: UnitType::Bit(11),
            length: LengthDesc {
                size: 11,
                unit: LengthUnit::Bit,
            },
            scope: ScopeDesc::Global("test".to_string()),
            cover: CoverDesc::EntireField,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: "APID".to_string(),
        };

        let mut disassembler = FrameDisassembler::new();
        disassembler.add_field(version_field);
        disassembler.add_field(type_field);
        disassembler.add_field(flag_field);
        disassembler.add_field(apid_field);

        // 测试帧数据：0x0A45 = 0000_1010_0100_0101
        // version(000) + type(0) + flag(1) + apid(01001000101=0x245)
        let frame_data = vec![0x0A, 0x45];

        let fields = disassembler
            .disassemble_frame(&frame_data)
            .expect("Failed to disassemble");

        // 验证提取的字段值
        let version = fields.get("version").unwrap();
        assert_eq!(version[0], 0x00, "Version should be 0");

        let pkt_type = fields.get("type").unwrap();
        assert_eq!(pkt_type[0], 0x00, "Type should be 0");

        let flag = fields.get("flag").unwrap();
        assert_eq!(flag[0], 0x01, "Flag should be 1");

        let apid = fields.get("apid").unwrap();
        let apid_value = ((apid[0] as u16) << 8) | (apid[1] as u16);
        assert_eq!(apid_value, 0x0245, "APID should be 0x0245");
    }
}
