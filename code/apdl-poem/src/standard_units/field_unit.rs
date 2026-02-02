//! 字段级语法单元实现
//!
//! 实现最小粒度的语法单元——字段，支持通过DSL定义和组装

use apdl_core::error::ProtocolError;
use apdl_core::{
    Constraint, DataRange, FieldDefinition, FieldType, ProtocolLayer, ProtocolUnit, ScopeType,
    UnitMeta,
};
use std::collections::HashMap;

/// 字段级语法单元实现
/// 这是最小粒度的语法单元，代表协议中的单个字段
pub struct FieldUnit {
    pub meta: UnitMeta, // 修改为pub以允许外部访问
    params: HashMap<String, String>,
    field_value: Vec<u8>,               // 字段的实际值
    field_constraints: Vec<Constraint>, // 字段约束
}

impl FieldUnit {
    /// 创建新的字段单元
    pub fn new(field_def: FieldDefinition) -> Self {
        let meta = UnitMeta {
            id: format!("FIELD_{}", field_def.name.to_uppercase().replace(" ", "_")),
            name: field_def.name.clone(),
            version: "1.0".to_string(),
            description: format!("Field unit for {}", field_def.name),
            standard: "Generic".to_string(),
            layer: ProtocolLayer::Application, // 可根据需要更改
            fields: vec![field_def.clone()],   // 包含自身定义
            constraints: field_def.constraints.clone(),
            scope: ScopeType::Layer("generic".to_string()),
            cover: DataRange::Entire,
            dsl_definition: "".to_string(), // 将在初始化后填充
        };

        let mut params = HashMap::new();
        params.insert(
            "field_type".to_string(),
            format!("{:?}", field_def.field_type),
        );

        Self {
            meta,
            params,
            field_value: vec![],
            field_constraints: field_def.constraints.clone(),
        }
    }

    /// 设置字段值
    pub fn set_value(&mut self, value: &[u8]) -> Result<(), ProtocolError> {
        // 验证约束
        self.validate_value(value)?;

        // 根据字段类型截断或填充值
        self.field_value = match self.meta.fields[0].field_type {
            FieldType::Bytes(size) => {
                let mut val = value.to_vec();
                if val.len() > size {
                    val.truncate(size);
                } else if val.len() < size {
                    val.resize(size, 0); // 用0填充
                }
                val
            }
            FieldType::Uint8 => {
                if value.is_empty() {
                    return Err(ProtocolError::ParseError(
                        "Insufficient data for Uint8".to_string(),
                    ));
                }
                vec![value[0]]
            }
            FieldType::Uint16 => {
                let mut val = value.to_vec();
                if val.len() < 2 {
                    val.resize(2, 0);
                }
                val[..2].to_vec()
            }
            FieldType::Uint32 => {
                let mut val = value.to_vec();
                if val.len() < 4 {
                    val.resize(4, 0);
                }
                val[..4].to_vec()
            }
            FieldType::Uint64 => {
                let mut val = value.to_vec();
                if val.len() < 8 {
                    val.resize(8, 0);
                }
                val[..8].to_vec()
            }
            FieldType::Bit(_) => {
                // 对于位字段，直接存储
                value.to_vec()
            }
            FieldType::Variable => {
                // 可变长度字段，直接存储
                value.to_vec()
            }
        };

        Ok(())
    }

    /// 获取字段值
    pub fn get_value(&self) -> &[u8] {
        &self.field_value
    }

    /// 验证字段值是否符合约束
    fn validate_value(&self, value: &[u8]) -> Result<(), ProtocolError> {
        for constraint in &self.field_constraints {
            match constraint {
                Constraint::Range(min, max) => {
                    // 将字节转换为数值进行比较
                    let num_value = bytes_to_u64(value);
                    if num_value < *min || num_value > *max {
                        return Err(ProtocolError::ValidationError(format!(
                            "Value {num_value} out of range [{min}, {max}]"
                        )));
                    }
                }
                Constraint::FixedValue(expected) => {
                    let actual = bytes_to_u64(value);
                    if actual != *expected {
                        return Err(ProtocolError::ValidationError(format!(
                            "Expected fixed value {expected}, got {actual}"
                        )));
                    }
                }
                Constraint::Enum(enum_values) => {
                    let actual = bytes_to_u64(value);
                    if !enum_values.iter().any(|(_, val)| *val == actual) {
                        return Err(ProtocolError::ValidationError(format!(
                            "Value {actual} not in allowed enum values"
                        )));
                    }
                }
                Constraint::Custom(_) => {
                    // 自定义约束，暂时跳过
                }
            }
        }
        Ok(())
    }
}

/// 将字节转换为u64（大端序）
fn bytes_to_u64(bytes: &[u8]) -> u64 {
    let mut result = 0u64;
    for (i, &byte) in bytes.iter().enumerate() {
        if i >= 8 {
            break;
        } // 防止溢出
        result |= (byte as u64) << (8 * (bytes.len() - 1 - i));
    }
    result
}

impl ProtocolUnit for FieldUnit {
    fn get_meta(&self) -> &UnitMeta {
        &self.meta
    }

    fn pack(&self, sdu: &[u8]) -> Result<Vec<u8>, ProtocolError> {
        // 如果提供了SDU，将其作为字段值
        if !sdu.is_empty() {
            let mut temp_unit = self.clone();
            temp_unit.set_value(sdu)?;
            Ok(temp_unit.field_value.clone())
        } else {
            // 否则返回当前字段值
            Ok(self.field_value.clone())
        }
    }

    fn unpack<'a>(&self, pdu: &'a [u8]) -> Result<(Vec<u8>, &'a [u8]), ProtocolError> {
        let field_size = match self.meta.fields[0].field_type {
            FieldType::Bytes(size) => size,
            FieldType::Uint8 => 1,
            FieldType::Uint16 => 2,
            FieldType::Uint32 => 4,
            FieldType::Uint64 => 8,
            FieldType::Bit(_) => {
                // 对于位字段，获取长度信息
                1 // 简化处理
            }
            FieldType::Variable => pdu.len(), // 可变长度使用全部数据
        };

        if pdu.len() < field_size {
            return Err(ProtocolError::ParseError(
                "PDU too short for field extraction".to_string(),
            ));
        }

        let field_data = pdu[..field_size].to_vec();
        let remaining = &pdu[field_size..];

        // 验证提取的字段数据
        self.validate_value(&field_data)?;

        Ok((field_data, remaining))
    }

    fn validate(&self) -> Result<(), ProtocolError> {
        // 验证当前字段值是否符合约束
        self.validate_value(&self.field_value)
    }

    fn get_params(&self) -> &HashMap<String, String> {
        &self.params
    }

    fn set_param(&mut self, key: &str, value: &str) -> Result<(), ProtocolError> {
        self.params.insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn get_unit_type(&self) -> &str {
        "FIELD_UNIT"
    }
}

impl Clone for FieldUnit {
    fn clone(&self) -> Self {
        Self {
            meta: self.meta.clone(),
            params: self.params.clone(),
            field_value: self.field_value.clone(),
            field_constraints: self.field_constraints.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_unit_creation() {
        let field_def = FieldDefinition {
            name: "Test Field".to_string(),
            field_type: FieldType::Uint16,
            length: 2,
            position: 0,
            constraints: vec![Constraint::Range(0, 100)],
        };

        let field_unit = FieldUnit::new(field_def);
        assert_eq!(field_unit.get_meta().name, "Test Field");
        assert!(field_unit.validate().is_ok());
    }

    #[test]
    fn test_field_pack_unpack() {
        let field_def = FieldDefinition {
            name: "Test Field".to_string(),
            field_type: FieldType::Uint16,
            length: 2,
            position: 0,
            constraints: vec![Constraint::Range(0, 1000)],
        };

        let mut field_unit = FieldUnit::new(field_def);

        // 设置字段值
        field_unit.set_value(&[0x01, 0x2C]).unwrap(); // 0x012C = 300

        // 打包
        let packed = field_unit.pack(&[]).unwrap();
        assert_eq!(packed, vec![0x01, 0x2C]);

        // 解包
        let (unpacked, remaining) = field_unit.unpack(&[0x01, 0x2C, 0xAA, 0xBB]).unwrap();
        assert_eq!(unpacked, vec![0x01, 0x2C]);
        assert_eq!(remaining, &[0xAA, 0xBB]);
    }

    #[test]
    fn test_field_constraints() {
        let field_def = FieldDefinition {
            name: "Constrained Field".to_string(),
            field_type: FieldType::Uint8,
            length: 1,
            position: 0,
            constraints: vec![Constraint::Range(10, 20)],
        };

        let mut field_unit = FieldUnit::new(field_def);

        // 有效的值
        assert!(field_unit.set_value(&[15]).is_ok());

        // 无效的值（超出范围）
        assert!(field_unit.set_value(&[5]).is_err());
        assert!(field_unit.set_value(&[25]).is_err());
    }
}
