//! 协议元数据模块
//!
//! 定义协议相关的元数据结构

/// 协议层枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolLayer {
    Physical,
    DataLink,
    Network,
    Transport,
    Application,
    Custom(String),
}

/// 字段定义
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDefinition {
    pub name: String,
    pub field_type: FieldType,
    pub length: usize,   // 以字节为单位
    pub position: usize, // 在帧中的位置
    pub constraints: Vec<Constraint>,
}

/// 字段类型
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Bit(usize),   // 比特数
    Bytes(usize), // 字节数
    Variable,     // 可变长度
}

/// 约束条件
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    Range(u64, u64),          // 数值范围
    FixedValue(u64),          // 固定值
    Enum(Vec<(String, u64)>), // 枚举值
    Custom(String),           // 自定义约束表达式
}

/// 作用范围类型
#[derive(Debug, Clone, PartialEq)]
pub enum ScopeType {
    Layer(String),              // 层内局部
    CrossLayer(String, String), // 跨层穿透
    Global(String),             // 全局
}

/// 数据范围
#[derive(Debug, Clone, PartialEq)]
pub enum DataRange {
    Position(usize, usize), // 位置范围 (起始, 长度)
    Expression(String),     // 表达式描述
    Entire,                 // 整个数据
}

/// 单元元数据结构
#[derive(Debug, Clone, PartialEq)]
pub struct UnitMeta {
    pub id: String,                   // 单元唯一标识
    pub name: String,                 // 单元名称
    pub version: String,              // 版本号
    pub description: String,          // 描述
    pub standard: String,             // 遵循的标准 (如 "CCSDS 132.0-B-3")
    pub layer: ProtocolLayer,         // 协议层
    pub fields: Vec<FieldDefinition>, // 字段定义
    pub constraints: Vec<Constraint>, // 约束条件
    pub scope: ScopeType,             // 作用范围
    pub cover: DataRange,             // 数据覆盖范围,
    pub dsl_definition: String,       // DSL定义字符串
}

/// 单元类型
#[derive(Debug, Clone, PartialEq)]
pub enum UnitType {
    Uint(u8), // Uint8, Uint16, Uint32, etc.
    Bit(u8),  // Bit(1), Bit(2), etc.
    RawData,
    Ip6Addr,
}

/// 长度描述
#[derive(Debug, Clone, PartialEq)]
pub struct LengthDesc {
    pub size: usize,
    pub unit: LengthUnit,
}

/// 长度单位
#[derive(Debug, Clone, PartialEq)]
pub enum LengthUnit {
    Byte,
    Bit,
    Dynamic,
    Expression(String),
}

/// 作用范围描述
#[derive(Debug, Clone, PartialEq)]
pub enum ScopeDesc {
    Layer(String),              // layer(link)
    CrossLayer(String, String), // cross_layer(net→link)
    Global(String),             // global(end2end)
}

/// 覆盖描述
#[derive(Debug, Clone, PartialEq)]
pub enum CoverDesc {
    Range(String, usize, usize), // frame_header[0..1]
    Expression(String),          // $cover
    EntireField,                 // entire_field
}

/// 算法抽象语法树
#[derive(Debug, Clone, PartialEq)]
pub enum AlgorithmAst {
    Crc16,
    Crc32,
    Crc15, // CAN协议专用
    XorSum,
    Custom(String),
}

/// DSL解析错误
#[derive(Debug, Clone, PartialEq)]
pub enum DslParseError {
    ParseError(String),
    ValidationError(String),
}

/// DSL验证错误
#[derive(Debug, Clone, PartialEq)]
pub enum DslValidateError {
    ValidationError(String),
}

/// 语法单元结构
#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxUnit {
    pub field_id: String,
    pub unit_type: UnitType,
    pub length: LengthDesc,
    pub scope: ScopeDesc,
    pub cover: CoverDesc,
    pub constraint: Option<Constraint>,
    pub alg: Option<AlgorithmAst>,
    pub associate: Vec<String>,
    pub desc: String,
}

// 新增语义规则类型
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticRule {
    ChecksumRange {
        algorithm: ChecksumAlgorithm,
        start_field: String,
        end_field: String,
    },
    Dependency {
        dependent_field: String,
        dependency_field: String,
    },
    Conditional {
        condition: String,
    },
    Order {
        first_field: String,
        second_field: String,
    },
    Pointer {
        pointer_field: String,
        target_field: String,
    },
    Algorithm {
        field_name: String,
        algorithm: String,
    },
    LengthRule {
        field_name: String,
        expression: String,
    },
    // CCSDS协议特有语义规则
    RoutingDispatch {
        fields: Vec<String>,
        algorithm: String,
        description: String,
    },
    SequenceControl {
        field_name: String,
        trigger_condition: String,
        algorithm: String,
        description: String,
    },
    Validation {
        field_name: String,
        algorithm: String,
        range_start: String,
        range_end: String,
        description: String,
    },
    Synchronization {
        field_name: String,
        algorithm: String,
        description: String,
    },
    LengthValidation {
        field_name: String,
        condition: String,
        description: String,
    },
    // CAN协议特有语义规则
    Multiplexing {
        field_name: String,
        condition: String,
        route_target: String,
        description: String,
    },
    PriorityProcessing {
        field_name: String,
        algorithm: String,
        description: String,
    },
    StateMachine {
        condition: String,
        algorithm: String,
        description: String,
    },
    PeriodicTransmission {
        field_name: String,
        condition: String,
        algorithm: String,
        description: String,
    },
    MessageFiltering {
        condition: String,
        action: String,
        description: String,
    },
    ErrorDetection {
        algorithm: String,
        description: String,
    },
    // 其他通用语义规则
    FlowControl {
        field_name: String,
        algorithm: String,
        description: String,
    },
    TimeSynchronization {
        field_name: String,
        algorithm: String,
        description: String,
    },
    AddressResolution {
        field_name: String,
        algorithm: String,
        description: String,
    },
    Security {
        field_name: String,
        algorithm: String,
        description: String,
    },
    Redundancy {
        field_name: String,
        algorithm: String,
        description: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChecksumAlgorithm {
    CRC16,
    CRC32,
    CRC15,
    XOR,
}

/// DSL解析器接口
pub trait DslParser {
    fn parse_dsl(&self, dsl_text: &str) -> Result<SyntaxUnit, DslParseError>;
    fn validate_dsl(&self, dsl_text: &str) -> Result<(), DslValidateError>;
}
