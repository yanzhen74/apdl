//! 字段映射器实现
//!
//! 提供字段级别的映射功能，支持多种映射逻辑

use std::collections::HashMap;

/// 字段映射器
pub struct FieldMapper {
    /// 映射函数注册表
    pub mapping_functions: HashMap<String, Box<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync>>,
}

impl FieldMapper {
    /// 创建新的字段映射器
    pub fn new() -> Self {
        let mut mapper = Self {
            mapping_functions: HashMap::new(),
        };

        // 注册默认的映射函数
        mapper.register_default_functions();
        mapper
    }

    /// 注册默认的映射函数
    fn register_default_functions(&mut self) {
        // 恒等映射
        self.mapping_functions.insert(
            "identity".to_string(),
            Box::new(|input: &[u8]| input.to_vec()),
        );

        // 哈希模64映射（用于VCID）
        self.mapping_functions.insert(
            "hash_mod_64".to_string(),
            Box::new(|input: &[u8]| {
                let hash = Self::simple_hash(input);
                let result = hash % 64;
                vec![(result & 0xFF) as u8]
            }),
        );

        // 哈希模2048映射（用于APID）
        self.mapping_functions.insert(
            "hash_mod_2048".to_string(),
            Box::new(|input: &[u8]| {
                let hash = Self::simple_hash(input);
                let result = hash % 2048;
                vec![
                    ((result >> 8) & 0xFF) as u8, // 高字节
                    (result & 0xFF) as u8,        // 低字节
                ]
            }),
        );

        // 位移映射（用于子系统标志）
        self.mapping_functions.insert(
            "shift_right_8".to_string(),
            Box::new(|input: &[u8]| {
                if input.len() >= 2 {
                    vec![input[0]] // 使用高字节作为子系统标志
                } else {
                    vec![0x00]
                }
            }),
        );
    }

    /// 注册自定义映射函数
    pub fn register_mapping_function(
        &mut self,
        name: String,
        func: Box<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync>,
    ) {
        self.mapping_functions.insert(name, func);
    }

    /// 执行字段映射
    pub fn map_field(
        &self,
        source_value: &[u8],
        mapping_function_name: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if let Some(func) = self.mapping_functions.get(mapping_function_name) {
            Ok(func(source_value))
        } else {
            Err(format!("Unknown mapping function: {}", mapping_function_name).into())
        }
    }

    /// 批量映射字段
    pub fn batch_map_fields(
        &self,
        source_values: &[Vec<u8>],
        mapping_function_name: &str,
    ) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        for value in source_values {
            results.push(self.map_field(value, mapping_function_name)?);
        }
        Ok(results)
    }

    /// 简单的哈希函数
    fn simple_hash(data: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for FieldMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_mapper_creation() {
        let mapper = FieldMapper::new();
        assert!(!mapper.mapping_functions.is_empty());
    }

    #[test]
    fn test_identity_mapping() {
        let mapper = FieldMapper::new();
        let input = vec![0x12, 0x34];
        let result = mapper.map_field(&input, "identity").unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn test_hash_mod_64_mapping() {
        let mapper = FieldMapper::new();
        let input = vec![0x12, 0x34];
        let result = mapper.map_field(&input, "hash_mod_64").unwrap();
        assert_eq!(result.len(), 1); // 结果应为1字节
        assert!(result[0] < 64); // 结果应在0-63范围内
    }

    #[test]
    fn test_hash_mod_2048_mapping() {
        let mapper = FieldMapper::new();
        let input = vec![0x12, 0x34];
        let result = mapper.map_field(&input, "hash_mod_2048").unwrap();
        assert_eq!(result.len(), 2); // 结果应为2字节
    }

    #[test]
    fn test_shift_right_8_mapping() {
        let mapper = FieldMapper::new();
        let input = vec![0xAB, 0xCD];
        let result = mapper.map_field(&input, "shift_right_8").unwrap();
        assert_eq!(result, vec![0xAB]); // 应该返回高字节
    }

    #[test]
    fn test_unknown_mapping_function() {
        let mapper = FieldMapper::new();
        let input = vec![0x12, 0x34];
        let result = mapper.map_field(&input, "unknown_function");
        assert!(result.is_err());
    }
}
