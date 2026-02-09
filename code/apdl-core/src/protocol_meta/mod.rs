//! 协议元数据模块
//!
//! 定义协议相关的元数据结构

use serde::{Deserialize, Serialize};

/// 协议层枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProtocolLayer {
    Physical,
    DataLink,
    Network,
    Transport,
    Application,
    Custom(String),
}

/// 字段定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,
    pub field_type: FieldType,
    pub length: usize,   // 以字节为单位
    pub position: usize, // 在帧中的位置
    pub constraints: Vec<Constraint>,
}

/// 字段类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Constraint {
    Range(u64, u64),          // 数值范围
    FixedValue(u64),          // 固定值
    Enum(Vec<(String, u64)>), // 枚举值
    Custom(String),           // 自定义约束表达式
}

/// 作用范围类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScopeType {
    Layer(String),              // 层内局部
    CrossLayer(String, String), // 跨层穿透
    Global(String),             // 全局
}

/// 数据范围
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataRange {
    Position(usize, usize), // 位置范围 (起始, 长度)
    Expression(String),     // 表达式描述
    Entire,                 // 整个数据
}

/// 单元元数据结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnitType {
    Uint(u8), // Uint8, Uint16, Uint32, etc.
    Bit(u8),  // Bit(1), Bit(2), etc.
    RawData,
    Ip6Addr,
}

/// 长度描述
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LengthDesc {
    pub size: usize,
    pub unit: LengthUnit,
}

/// 长度单位
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LengthUnit {
    Byte,
    Bit,
    Dynamic,
    Expression(String),
}

/// 作用范围描述
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScopeDesc {
    Layer(String),              // layer(link)
    CrossLayer(String, String), // cross_layer(net→link)
    Global(String),             // global(end2end)
}

/// 覆盖描述
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoverDesc {
    Range(String, usize, usize), // frame_header[0..1]
    Expression(String),          // $cover
    EntireField,                 // entire_field
}

/// 算法抽象语法树
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlgorithmAst {
    Crc16,
    Crc32,
    Crc15, // CAN协议专用
    XorSum,
    Custom(String),
}

/// 枚举映射条目
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumMappingEntry {
    pub source_enum: String, // 源枚举值，可以包含通配符
    pub target_enum: String, // 目标枚举值
}

/// 掩码映射表条目
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaskMappingEntry {
    #[serde(deserialize_with = "deserialize_hex_array")]
    pub mask: Vec<u8>, // 掩码，如 [0xFF, 0xF0] 或 ["0xFF", "0xF0"]
    #[serde(deserialize_with = "deserialize_hex_array")]
    pub src_masked: Vec<u8>, // 源值应用掩码后的期望值，如 [0x04, 0x80]
    #[serde(deserialize_with = "deserialize_hex_array")]
    pub dst: Vec<u8>, // 目标映射值，如 [0x35]
}

/// 自定义反序列化：支持数字数组或十六进制字符串数组
fn deserialize_hex_array<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct HexArrayVisitor;

    impl<'de> Visitor<'de> for HexArrayVisitor {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an array of numbers or hex strings")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(value) = seq.next_element::<serde_json::Value>()? {
                match value {
                    serde_json::Value::Number(n) => {
                        if let Some(num) = n.as_u64() {
                            vec.push(num as u8);
                        } else {
                            return Err(de::Error::custom("number out of range"));
                        }
                    }
                    serde_json::Value::String(s) => {
                        let s = s.trim();
                        let byte_val = if s.starts_with("0x") || s.starts_with("0X") {
                            u8::from_str_radix(&s[2..], 16).map_err(|_| {
                                de::Error::custom(format!("invalid hex string: {s}"))
                            })?
                        } else {
                            s.parse::<u8>().map_err(|_| {
                                de::Error::custom(format!("invalid number string: {s}"))
                            })?
                        };
                        vec.push(byte_val);
                    }
                    _ => return Err(de::Error::custom("expected number or string")),
                }
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_seq(HexArrayVisitor)
}

/// 字段映射条目
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldMappingEntry {
    pub source_field: String,
    pub target_field: String,
    pub mapping_logic: String, // 映射逻辑，如 "hash(x) % 64" 或 "mask_table"
    pub default_value: String, // 默认值
    pub enum_mappings: Option<Vec<EnumMappingEntry>>, // 可选的枚举映射列表
    pub mask_mapping_table: Option<Vec<MaskMappingEntry>>, // 可选的掩码映射表
}

/// DSL解析错误
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DslParseError {
    ParseError(String),
    ValidationError(String),
}

/// DSL验证错误
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DslValidateError {
    ValidationError(String),
}

/// 语法单元结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    // 连接器模式语义规则
    FieldMapping {
        source_package: String,
        target_package: String,
        mappings: Vec<FieldMappingEntry>,
        description: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChecksumAlgorithm {
    CRC16,
    CRC32,
    CRC15,
    XOR,
}

/// 层定义结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerDefinition {
    pub name: String,
    pub units: Vec<SyntaxUnit>,
    pub rules: Vec<SemanticRule>,
}

/// 包定义结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageDefinition {
    pub name: String,
    pub display_name: String,
    pub package_type: String, // telemetry, command, encapsulating, etc.
    pub layers: Vec<LayerDefinition>,
    pub description: String,
}

/// 数据放置策略类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataPlacementStrategy {
    Direct,         // 直接放入方式
    PointerBased,   // 导头指针方式
    StreamBased,    // 数据流方式
    Custom(String), // 自定义策略
}

/// 数据放置配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataPlacementConfig {
    pub strategy: DataPlacementStrategy,
    pub target_field: String,                 // 在目标包中的放置位置
    pub config_params: Vec<(String, String)>, // 策略特定配置参数
}

/// 连接器配置结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectorConfig {
    pub mappings: Vec<FieldMappingEntry>,
    pub header_pointers: Option<HeaderPointerConfig>,
    pub data_placement: Option<DataPlacementConfig>, // 新增数据放置配置
}

/// 导头指针配置结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeaderPointerConfig {
    pub master_pointer: String,
    pub secondary_pointers: Vec<String>,
    pub descriptor_field: String,
}

/// 连接器定义结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectorDefinition {
    pub name: String,
    pub connector_type: String, // field_mapping, header_pointer, etc.
    pub source_package: String,
    pub target_package: String,
    pub config: ConnectorConfig,
    pub description: String,
}

/// 并列包组结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParallelPackageGroup {
    pub name: String,
    pub packages: Vec<String>,
    pub algorithm: String,
    pub priority: u32,
}

/// 协议栈定义结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolStackDefinition {
    pub name: String,
    pub packages: Vec<String>,
    pub connectors: Vec<String>,
    pub parallel_groups: Vec<ParallelPackageGroup>,
    pub description: String,
}

/// DSL解析器接口
pub trait DslParser {
    fn parse_dsl(&self, dsl_text: &str) -> Result<SyntaxUnit, DslParseError>;
    fn validate_dsl(&self, dsl_text: &str) -> Result<(), DslValidateError>;
}

impl PackageDefinition {
    pub fn new(
        name: String,
        display_name: String,
        package_type: String,
        description: String,
    ) -> Self {
        Self {
            name,
            display_name,
            package_type,
            layers: Vec::new(),
            description,
        }
    }
}

impl ConnectorDefinition {
    pub fn new(
        name: String,
        connector_type: String,
        source_package: String,
        target_package: String,
        description: String,
    ) -> Self {
        Self {
            name,
            connector_type,
            source_package,
            target_package,
            config: ConnectorConfig {
                mappings: Vec::new(),
                header_pointers: None,
                data_placement: None, // 新增数据放置配置
            },
            description,
        }
    }
}

impl ProtocolStackDefinition {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            packages: Vec::new(),
            connectors: Vec::new(),
            parallel_groups: Vec::new(),
            description,
        }
    }
}
