//! 测试辅助模块
//!
//! 提供便捷的测试数据生成API，简化测试用例中的数据生成

use super::{DataGenerator, DataImporter, GenerationStrategy};
use apdl_core::{Constraint, CoverDesc, LengthDesc, LengthUnit, ScopeDesc, SyntaxUnit, UnitType};

/// 测试数据生成器
///
/// 为测试用例提供便捷的数据生成API
///
/// # 示例
/// ```
/// use apdl_lsk::data_generator::TestDataGenerator;
///
/// let mut gen = TestDataGenerator::new();
///
/// // 生成随机字节
/// let random_bytes = gen.random_bytes(16);
///
/// // 生成顺序字节
/// let sequential_bytes = gen.sequential_bytes(16);
///
/// // 从十六进制导入
/// let hex_data = gen.from_hex("DEADBEEF").unwrap();
/// ```
pub struct TestDataGenerator {
    generator: DataGenerator,
}

impl TestDataGenerator {
    /// 创建新的测试数据生成器
    pub fn new() -> Self {
        let syntax_units = Self::create_default_syntax_units();
        Self {
            generator: DataGenerator::new(&syntax_units),
        }
    }

    /// 使用指定种子创建（可重复测试）
    pub fn with_seed(seed: u64) -> Self {
        let syntax_units = Self::create_default_syntax_units();
        Self {
            generator: DataGenerator::with_seed(&syntax_units, seed),
        }
    }

    /// 创建默认的语法单元列表
    fn create_default_syntax_units() -> Vec<SyntaxUnit> {
        vec![
            SyntaxUnit {
                field_id: "data_field".to_string(),
                unit_type: UnitType::RawData,
                length: LengthDesc {
                    size: 1024,
                    unit: LengthUnit::Byte,
                },
                scope: ScopeDesc::Global("test".to_string()),
                cover: CoverDesc::EntireField,
                constraint: None,
                alg: None,
                associate: vec![],
                desc: "Generic data field".to_string(),
            },
            SyntaxUnit {
                field_id: "sync_flag".to_string(),
                unit_type: UnitType::Uint(16),
                length: LengthDesc {
                    size: 2,
                    unit: LengthUnit::Byte,
                },
                scope: ScopeDesc::Global("test".to_string()),
                cover: CoverDesc::EntireField,
                constraint: Some(Constraint::FixedValue(0xEB90)),
                alg: None,
                associate: vec![],
                desc: "Sync flag".to_string(),
            },
            SyntaxUnit {
                field_id: "version".to_string(),
                unit_type: UnitType::Uint(8),
                length: LengthDesc {
                    size: 1,
                    unit: LengthUnit::Byte,
                },
                scope: ScopeDesc::Global("test".to_string()),
                cover: CoverDesc::EntireField,
                constraint: Some(Constraint::Range(0, 3)),
                alg: None,
                associate: vec![],
                desc: "Version field".to_string(),
            },
            SyntaxUnit {
                field_id: "payload".to_string(),
                unit_type: UnitType::RawData,
                length: LengthDesc {
                    size: 256,
                    unit: LengthUnit::Byte,
                },
                scope: ScopeDesc::Global("test".to_string()),
                cover: CoverDesc::EntireField,
                constraint: None,
                alg: None,
                associate: vec![],
                desc: "Payload data".to_string(),
            },
        ]
    }

    /// 生成指定长度的随机字节
    pub fn random_bytes(&mut self, length: usize) -> Vec<u8> {
        self.generator.set_strategy(GenerationStrategy::Random);
        // 使用内部随机策略直接生成
        use super::RandomStrategy;
        let mut strategy = RandomStrategy::new();
        strategy.generate_bytes(length)
    }

    /// 生成指定长度的顺序字节
    pub fn sequential_bytes(&mut self, length: usize) -> Vec<u8> {
        self.generator.set_strategy(GenerationStrategy::Sequential);
        use super::SequentialStrategy;
        let mut strategy = SequentialStrategy::new();
        strategy.generate_bytes(length)
    }

    /// 生成固定值字节
    pub fn fixed_bytes(&self, value: &[u8], length: usize) -> Vec<u8> {
        let mut result = value.to_vec();
        result.resize(length, 0);
        result
    }

    /// 生成边界值字节
    pub fn boundary_bytes(&mut self, length: usize) -> Vec<u8> {
        use super::BoundaryValueStrategy;
        let mut strategy = BoundaryValueStrategy::new();
        strategy.generate_bytes(length)
    }

    /// 从十六进制字符串导入数据
    pub fn from_hex(&self, hex_str: &str) -> Result<Vec<u8>, String> {
        DataImporter::import_from_hex(hex_str)
            .map_err(|e| format!("Failed to import hex: {}", e))
    }

    /// 从文本导入数据
    pub fn from_text(&self, text: &str) -> Vec<u8> {
        DataImporter::import_from_text(text)
    }

    /// 从Base64导入数据
    pub fn from_base64(&self, base64_str: &str) -> Result<Vec<u8>, String> {
        DataImporter::import_from_base64(base64_str)
            .map_err(|e| format!("Failed to import base64: {}", e))
    }

    /// 调整数据长度（截断或填充）
    pub fn adjust_length(&self, data: &[u8], target_length: usize) -> Vec<u8> {
        DataImporter::adjust_length(data, target_length, 0)
    }

    /// 生成经典的"死牛肉"模式数据
    pub fn deadbeef_pattern(&self, repetitions: usize) -> Vec<u8> {
        let pattern = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let mut result = Vec::with_capacity(pattern.len() * repetitions);
        for _ in 0..repetitions {
            result.extend_from_slice(&pattern);
        }
        result
    }

    /// 生成经典的"咖啡宝贝"模式数据
    pub fn cafebabe_pattern(&self, repetitions: usize) -> Vec<u8> {
        let pattern = vec![0xCA, 0xFE, 0xBA, 0xBE];
        let mut result = Vec::with_capacity(pattern.len() * repetitions);
        for _ in 0..repetitions {
            result.extend_from_slice(&pattern);
        }
        result
    }

    /// 生成测试用的混合模式数据
    pub fn mixed_pattern(&self) -> Vec<u8> {
        vec![
            0xDE, 0xAD, 0xBE, 0xEF, // 死牛肉
            0xCA, 0xFE, 0xBA, 0xBE, // 咖啡宝贝
            0x12, 0x34, 0x56, 0x78, // 递增
            0x9A, 0xBC, 0xDE, 0xF0, // 递减
        ]
    }

    /// 生成CCSDS TM帧格式的测试数据
    pub fn ccsds_tm_payload(&mut self, length: usize) -> Vec<u8> {
        // 使用随机数据作为TM帧载荷
        self.random_bytes(length)
    }

    /// 生成CCSDS Space Packet格式的测试数据
    pub fn ccsds_space_packet_payload(&mut self, length: usize) -> Vec<u8> {
        // 使用顺序数据作为Space Packet载荷，便于调试
        self.sequential_bytes(length)
    }
}

impl Default for TestDataGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// 预定义的测试数据模式
pub mod patterns {
    /// 经典的死牛肉模式
    pub const DEADBEEF: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];

    /// 咖啡宝贝模式
    pub const CAFEBABE: [u8; 4] = [0xCA, 0xFE, 0xBA, 0xBE];

    /// 全零模式
    pub const ZEROS: [u8; 4] = [0x00, 0x00, 0x00, 0x00];

    /// 全一模式
    pub const ONES: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

    /// 递增模式
    pub const INCREMENTAL: [u8; 4] = [0x00, 0x01, 0x02, 0x03];

    /// 递减模式
    pub const DECREMENTAL: [u8; 4] = [0x03, 0x02, 0x01, 0x00];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_bytes() {
        let mut gen = TestDataGenerator::new();
        let data = gen.random_bytes(16);

        assert_eq!(data.len(), 16);
        // 验证生成了数据（不检查具体内容，因为是随机的）
        // 多次生成验证长度一致
        for _ in 0..5 {
            let d = gen.random_bytes(16);
            assert_eq!(d.len(), 16);
        }
    }

    #[test]
    fn test_sequential_bytes() {
        let mut gen = TestDataGenerator::new();
        let data = gen.sequential_bytes(8);

        assert_eq!(data.len(), 8);
        assert_eq!(data, vec![0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_fixed_bytes() {
        let gen = TestDataGenerator::new();
        let data = gen.fixed_bytes(&[0xDE, 0xAD], 4);

        assert_eq!(data, vec![0xDE, 0xAD, 0x00, 0x00]);
    }

    #[test]
    fn test_from_hex() {
        let gen = TestDataGenerator::new();
        let data = gen.from_hex("DEADBEEF").unwrap();

        assert_eq!(data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_deadbeef_pattern() {
        let gen = TestDataGenerator::new();
        let data = gen.deadbeef_pattern(2);

        assert_eq!(data, vec![0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_mixed_pattern() {
        let gen = TestDataGenerator::new();
        let data = gen.mixed_pattern();

        assert_eq!(data.len(), 16);
        assert_eq!(&data[0..4], &[0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(&data[4..8], &[0xCA, 0xFE, 0xBA, 0xBE]);
    }

    #[test]
    fn test_with_seed() {
        let mut gen1 = TestDataGenerator::with_seed(12345);
        let mut gen2 = TestDataGenerator::with_seed(12345);

        let data1 = gen1.random_bytes(16);
        let data2 = gen2.random_bytes(16);

        // 相同种子应该生成相同数据
        assert_eq!(data1, data2);
    }
}
