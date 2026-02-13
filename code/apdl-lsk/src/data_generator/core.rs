//! 数据生成器核心实现
//!
//! 提供基于协议模型定义的数据生成功能

use apdl_core::{Constraint, LengthUnit, SyntaxUnit, UnitType};
use std::collections::HashMap;

use super::constraints::ConstraintHandler;
use super::strategies::{
    BoundaryValueStrategy, GenerationStrategy, RandomStrategy, SequentialStrategy,
};

/// 数据生成器
///
/// 基于协议模型定义（SyntaxUnit列表）自动生成测试数据
///
/// # 示例
/// ```
/// use apdl_lsk::data_generator::{DataGenerator, GenerationStrategy};
/// use apdl_core::SyntaxUnit;
///
/// let syntax_units = vec![]; // 协议模型定义
/// let mut generator = DataGenerator::new(&syntax_units);
/// generator.set_strategy(GenerationStrategy::Random);
///
/// let data = generator.generate_field("field_name");
/// ```
pub struct DataGenerator {
    /// 协议模型定义（字段名 -> SyntaxUnit）
    model: HashMap<String, SyntaxUnit>,
    /// 字段顺序（保持定义时的顺序）
    field_order: Vec<String>,
    /// 当前生成策略
    strategy: GenerationStrategy,
    /// 随机策略实例
    random_strategy: RandomStrategy,
    /// 顺序策略实例（每个字段独立计数）
    sequential_strategies: HashMap<String, SequentialStrategy>,
    /// 边界值策略实例
    boundary_strategy: BoundaryValueStrategy,
    /// 固定值策略实例
    fixed_value: Option<Vec<u8>>,
}

impl DataGenerator {
    /// 创建新的数据生成器
    ///
    /// # 参数
    /// - `syntax_units`: 协议模型定义（SyntaxUnit列表）
    ///
    /// # 示例
    /// ```
    /// use apdl_lsk::data_generator::DataGenerator;
    /// use apdl_core::SyntaxUnit;
    ///
    /// let syntax_units: Vec<SyntaxUnit> = vec![];
    /// let generator = DataGenerator::new(&syntax_units);
    /// ```
    pub fn new(syntax_units: &[SyntaxUnit]) -> Self {
        let mut model = HashMap::new();
        let mut field_order = Vec::new();

        for unit in syntax_units {
            let field_name = unit.field_id.clone();
            model.insert(field_name.clone(), unit.clone());
            field_order.push(field_name);
        }

        Self {
            model,
            field_order,
            strategy: GenerationStrategy::Random,
            random_strategy: RandomStrategy::new(),
            sequential_strategies: HashMap::new(),
            boundary_strategy: BoundaryValueStrategy::new(),
            fixed_value: None,
        }
    }

    /// 使用指定种子创建生成器（用于可重复测试）
    ///
    /// # 参数
    /// - `syntax_units`: 协议模型定义
    /// - `seed`: 随机数种子
    pub fn with_seed(syntax_units: &[SyntaxUnit], seed: u64) -> Self {
        let mut generator = Self::new(syntax_units);
        generator.random_strategy = RandomStrategy::with_seed(seed);
        generator
    }

    /// 设置生成策略
    ///
    /// # 参数
    /// - `strategy`: 生成策略
    pub fn set_strategy(&mut self, strategy: GenerationStrategy) {
        self.strategy = strategy.clone();
        
        // 如果是固定值策略，保存固定值
        if let GenerationStrategy::Fixed(value) = &strategy {
            self.fixed_value = Some(value.clone());
        }
        
        // 重置各策略状态
        self.sequential_strategies.clear();
        self.boundary_strategy.reset();
    }

    /// 生成单个字段的值
    ///
    /// # 参数
    /// - `field_name`: 字段名称
    ///
    /// # 返回
    /// - `Some(Vec<u8>)`: 生成的字段值
    /// - `None`: 字段不存在
    ///
    /// # 示例
    /// ```
    /// use apdl_lsk::data_generator::DataGenerator;
    ///
    /// let syntax_units = vec![];
    /// let mut generator = DataGenerator::new(&syntax_units);
    /// if let Some(data) = generator.generate_field("version") {
    ///     // 使用生成的数据
    /// }
    /// ```
    pub fn generate_field(&mut self, field_name: &str) -> Option<Vec<u8>> {
        let unit = self.model.get(field_name)?.clone();
        Some(self.generate_for_unit(&unit))
    }

    /// 根据SyntaxUnit生成数据
    fn generate_for_unit(&mut self, unit: &SyntaxUnit) -> Vec<u8> {
        // 获取字段长度
        let length = self.calculate_length(unit);
        let field_name = &unit.field_id;
        
        // 检查是否有约束
        let constraints = self.extract_constraints(unit);
        
        // 根据策略生成基础值
        let base_value = match &self.strategy {
            GenerationStrategy::Random => {
                self.generate_random_value(&unit.unit_type, length)
            }
            GenerationStrategy::Sequential => {
                self.generate_sequential_value(field_name, &unit.unit_type, length)
            }
            GenerationStrategy::Fixed(_) => {
                self.generate_fixed_value(length)
            }
            GenerationStrategy::BoundaryValues => {
                self.generate_boundary_value(&unit.unit_type, length)
            }
        };
        
        // 应用约束
        if !constraints.is_empty() {
            self.apply_constraints_to_bytes(&base_value, &constraints)
        } else {
            base_value
        }
    }

    /// 计算字段长度（字节）
    fn calculate_length(&self, unit: &SyntaxUnit) -> usize {
        match unit.length.unit {
            LengthUnit::Byte => unit.length.size,
            LengthUnit::Bit => (unit.length.size + 7) / 8, // 向上取整
            LengthUnit::Dynamic | LengthUnit::Expression(_) => {
                // 动态长度默认使用一个合理值
                // 实际使用时可能需要根据上下文确定
                16
            }
        }
    }

    /// 从SyntaxUnit提取约束
    fn extract_constraints(&self, _unit: &SyntaxUnit) -> Vec<Constraint> {
        // 注意：当前SyntaxUnit结构中没有直接的constraints字段
        // 这里假设约束可能存储在其他地方，或者通过其他方式传递
        // 实际实现时可能需要修改数据结构
        Vec::new()
    }

    /// 生成随机值
    fn generate_random_value(&mut self, unit_type: &UnitType, length: usize) -> Vec<u8> {
        match unit_type {
            UnitType::Uint(bits) => {
                let value = self.random_strategy.generate_bits(*bits as usize);
                self.u64_to_bytes(value, length)
            }
            UnitType::Bit(bits) => {
                let value = self.random_strategy.generate_bits(*bits as usize);
                self.u64_to_bytes(value, length)
            }
            UnitType::RawData | UnitType::Ip6Addr => {
                self.random_strategy.generate_bytes(length)
            }
        }
    }

    /// 生成顺序值
    fn generate_sequential_value(
        &mut self,
        field_name: &str,
        unit_type: &UnitType,
        length: usize,
    ) -> Vec<u8> {
        // 获取或创建该字段的顺序策略
        let strategy = self
            .sequential_strategies
            .entry(field_name.to_string())
            .or_insert_with(SequentialStrategy::new);

        match unit_type {
            UnitType::Uint(bits) | UnitType::Bit(bits) => {
                let value = strategy.next() & ((1u64 << (*bits as usize)) - 1);
                self.u64_to_bytes(value, length)
            }
            UnitType::RawData | UnitType::Ip6Addr => strategy.generate_bytes(length),
        }
    }

    /// 生成固定值
    fn generate_fixed_value(&self, length: usize) -> Vec<u8> {
        match &self.fixed_value {
            Some(value) => {
                if value.len() >= length {
                    value[..length].to_vec()
                } else {
                    let mut result = value.clone();
                    result.resize(length, 0);
                    result
                }
            }
            None => vec![0; length],
        }
    }

    /// 生成边界值
    fn generate_boundary_value(&mut self, unit_type: &UnitType, length: usize) -> Vec<u8> {
        match unit_type {
            UnitType::Uint(bits) | UnitType::Bit(bits) => {
                let bits = *bits as usize;
                let mut strategy = BoundaryValueStrategy::for_bits(bits.min(64));
                let value = strategy.next();
                self.u64_to_bytes(value, length)
            }
            UnitType::RawData | UnitType::Ip6Addr => {
                self.boundary_strategy.generate_bytes(length)
            }
        }
    }

    /// 将u64转换为指定长度的字节数组（大端序）
    fn u64_to_bytes(&self, value: u64, length: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(length);
        for i in (0..length).rev() {
            result.push(((value >> (i * 8)) & 0xFF) as u8);
        }
        result
    }

    /// 应用约束到字节数据
    fn apply_constraints_to_bytes(&self, data: &[u8], constraints: &[Constraint]) -> Vec<u8> {
        // 将字节转换为u64（假设最多8字节）
        let mut value: u64 = 0;
        for &byte in data.iter().take(8) {
            value = (value << 8) | (byte as u64);
        }

        // 应用约束
        let constrained_value = ConstraintHandler::apply_constraints(constraints, value);

        // 转换回字节
        self.u64_to_bytes(constrained_value, data.len())
    }

    /// 生成完整帧（所有字段按顺序组合）
    ///
    /// # 返回
    /// 包含所有字段值的完整帧数据
    pub fn generate_frame(&mut self) -> Vec<u8> {
        let field_order: Vec<String> = self.field_order.clone();
        let mut frame = Vec::new();
        
        for field_name in field_order {
            if let Some(data) = self.generate_field(&field_name) {
                frame.extend(data);
            }
        }
        
        frame
    }

    /// 批量生成多个帧
    ///
    /// # 参数
    /// - `count`: 生成数量
    ///
    /// # 返回
    /// 帧数据列表
    pub fn generate_batch(&mut self, count: usize) -> Vec<Vec<u8>> {
        (0..count).map(|_| self.generate_frame()).collect()
    }

    /// 重置生成器状态
    pub fn reset(&mut self) {
        self.sequential_strategies.clear();
        self.boundary_strategy.reset();
        // 随机策略不重置，保持随机性
    }

    /// 获取模型中的所有字段名
    pub fn get_field_names(&self) -> Vec<&String> {
        self.field_order.iter().collect()
    }

    /// 检查字段是否存在
    pub fn has_field(&self, field_name: &str) -> bool {
        self.model.contains_key(field_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apdl_core::{CoverDesc, LengthDesc, ScopeDesc};

    fn create_test_syntax_unit(field_id: &str, unit_type: UnitType, size: usize) -> SyntaxUnit {
        SyntaxUnit {
            field_id: field_id.to_string(),
            unit_type,
            length: LengthDesc {
                size,
                unit: LengthUnit::Byte,
            },
            scope: ScopeDesc::Global("test".to_string()),
            cover: CoverDesc::EntireField,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: "Test field".to_string(),
        }
    }

    #[test]
    fn test_new_generator() {
        let units = vec![
            create_test_syntax_unit("field1", UnitType::Uint(8), 1),
            create_test_syntax_unit("field2", UnitType::Uint(16), 2),
        ];
        
        let generator = DataGenerator::new(&units);
        
        assert!(generator.has_field("field1"));
        assert!(generator.has_field("field2"));
        assert!(!generator.has_field("field3"));
        
        let names = generator.get_field_names();
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_generate_field_random() {
        let units = vec![
            create_test_syntax_unit("version", UnitType::Uint(8), 1),
        ];
        
        let mut generator = DataGenerator::new(&units);
        generator.set_strategy(GenerationStrategy::Random);
        
        let data = generator.generate_field("version").unwrap();
        assert_eq!(data.len(), 1);
    }

    #[test]
    fn test_generate_field_sequential() {
        let units = vec![
            create_test_syntax_unit("seq", UnitType::Uint(8), 1),
        ];
        
        let mut generator = DataGenerator::new(&units);
        generator.set_strategy(GenerationStrategy::Sequential);
        
        let data1 = generator.generate_field("seq").unwrap();
        let data2 = generator.generate_field("seq").unwrap();
        
        // 顺序递增
        assert_eq!(data1[0], 0);
        assert_eq!(data2[0], 1);
    }

    #[test]
    fn test_generate_field_fixed() {
        let units = vec![
            create_test_syntax_unit("fixed_field", UnitType::RawData, 4),
        ];
        
        let mut generator = DataGenerator::new(&units);
        generator.set_strategy(GenerationStrategy::Fixed(vec![0xDE, 0xAD, 0xBE, 0xEF]));
        
        let data = generator.generate_field("fixed_field").unwrap();
        assert_eq!(data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_generate_frame() {
        let units = vec![
            create_test_syntax_unit("header", UnitType::Uint(8), 1),
            create_test_syntax_unit("data", UnitType::Uint(16), 2),
        ];
        
        let mut generator = DataGenerator::new(&units);
        generator.set_strategy(GenerationStrategy::Sequential);
        
        let frame = generator.generate_frame();
        assert_eq!(frame.len(), 3); // 1 + 2 bytes
    }

    #[test]
    fn test_generate_batch() {
        let units = vec![
            create_test_syntax_unit("field", UnitType::Uint(8), 1),
        ];
        
        let mut generator = DataGenerator::new(&units);
        let frames = generator.generate_batch(5);
        
        assert_eq!(frames.len(), 5);
    }

    #[test]
    fn test_reset() {
        let units = vec![
            create_test_syntax_unit("seq", UnitType::Uint(8), 1),
        ];
        
        let mut generator = DataGenerator::new(&units);
        generator.set_strategy(GenerationStrategy::Sequential);
        
        let _ = generator.generate_field("seq");
        generator.reset();
        let data = generator.generate_field("seq").unwrap();
        
        // 重置后从0开始
        assert_eq!(data[0], 0);
    }

    #[test]
    fn test_with_seed() {
        let units = vec![
            create_test_syntax_unit("rand", UnitType::Uint(32), 4),
        ];
        
        let mut gen1 = DataGenerator::with_seed(&units, 12345);
        let mut gen2 = DataGenerator::with_seed(&units, 12345);
        
        let data1 = gen1.generate_field("rand").unwrap();
        let data2 = gen2.generate_field("rand").unwrap();
        
        // 相同种子应该生成相同数据
        assert_eq!(data1, data2);
    }
}
