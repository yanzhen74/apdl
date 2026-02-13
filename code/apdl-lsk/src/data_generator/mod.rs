//! 数据模拟生成模块
//!
//! 基于DSL/JSON模型定义自动生成测试数据，支持规则化数据生成、
//! 参数约束、自定义数据导入等功能。
//!
//! # 示例
//! ```
//! use apdl_lsk::data_generator::{DataGenerator, GenerationStrategy};
//! use apdl_core::SyntaxUnit;
//!
//! // 从协议模型创建生成器
//! let syntax_units = vec![]; // SyntaxUnit定义
//! let mut generator = DataGenerator::new(&syntax_units);
//!
//! // 设置生成策略
//! generator.set_strategy(GenerationStrategy::Random);
//!
//! // 生成数据
//! let data = generator.generate_field("field_name");
//! ```

pub mod constraints;
pub mod core;
pub mod custom_import;
pub mod strategies;

pub use constraints::{ConstraintHandler, ConstraintValidator};
pub use core::DataGenerator;
pub use custom_import::DataImporter;
pub use strategies::{BoundaryValueStrategy, FixedStrategy, GenerationStrategy, RandomStrategy, SequentialStrategy};
