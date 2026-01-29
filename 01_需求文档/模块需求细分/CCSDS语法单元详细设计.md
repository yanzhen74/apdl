# CCSDS协议语法单元详细设计

## 1. 设计概述

### 1.1 目标
本设计文档详细描述了APDL系统中CCSDS协议语法单元的实现方案，确保协议单元能够正确实现CCSDS标准，并支持灵活的组合和扩展。

### 1.2 设计原则
- 标准兼容性：严格遵循CCSDS标准规范
- 模块化：每个语法单元独立实现
- 可扩展：支持自定义语法单元
- 互操作：单元间接口统一

## 2. 语法单元架构设计

### 2.1 核心接口定义

```rust
/// 语法单元基础接口 - 实现平台与语法单元的分离
pub trait ProtocolUnit: Send + Sync {
    /// 获取单元元数据
    fn get_meta(&self) -> &UnitMeta;
    
    /// 打包：将SDU（服务数据单元）封装为PDU（协议数据单元）
    fn pack(&self, sdu: &[u8]) -> Result<Vec<u8>, ProtocolError>;
    
    /// 拆包：将PDU解析为SDU
    fn unpack(&self, pdu: &[u8]) -> Result<(Vec<u8>, &[u8]), ProtocolError>;
    
    /// 单元合法性校验
    fn validate(&self) -> Result<(), ProtocolError>;
    
    /// 获取单元配置参数
    fn get_params(&self) -> &HashMap<String, String>;
    
    /// 设置单元配置参数
    fn set_param(&mut self, key: &str, value: &str) -> Result<(), ProtocolError>;
}
```

### 2.2 元数据结构

```rust
/// 单元元数据结构
#[derive(Debug, Clone)]
pub struct UnitMeta {
    pub id: String,                    // 单元唯一标识
    pub name: String,                  // 单元名称
    pub version: String,               // 版本号
    pub description: String,           // 描述
    pub standard: String,              // 遵循的标准 (如 "CCSDS 132.0-B-3")
    pub layer: ProtocolLayer,          // 协议层
    pub fields: Vec<FieldDefinition>,  // 字段定义
    pub constraints: Vec<Constraint>,  // 约束条件
    pub scope: ScopeType,              // 作用范围
    pub cover: DataRange,              // 数据覆盖范围,
    pub dsl_definition: String,        // DSL定义字符串
    pub json_description: Value,       // JSON格式的协议描述
}
```

## 3. DSL规范与JSON协议描述设计

### 3.1 语法单元DSL通用规范

#### 3.1.1 基础结构
```
field: [唯一 ID]; // 必选，格式：协议_类型_功能
type: [数据类型]; // 必选，如 Uint16/Bit(2)/RawData/Ip6Addr
length: [长度]; // 必选，如 2byte/dynamic/calc(公式)
scope: [作用范围]; // 必选，如 layer(link)/cross_layer(net→link)/global(end2end)
cover: [数据覆盖]; // 必选，如 frame_header[0..1]/$cover
[可选字段]; // constraint/alg/associate/desc/role 等
```

#### 3.1.2 数据类型与约束规则
| 数据类型 | 约束规则示例 |
|----------|-------------|
| UintN | fixed(0xEB90)/range(0..=1023) |
| Bit(N) | enum(00=single,01=first) |
| RawData | dynamic，关联其他单元 |
| Ip6Addr | scps_ip_format |

#### 3.1.3 DSL解析器设计
```rust
pub struct DslParser {
    pub dsl_content: String,
}

impl DslParser {
    pub fn parse_syntax_unit(&self) -> Result<SyntaxUnit, ParseError> {
        // 解析field
        let field_id = self.parse_field()?;
        
        // 解析type
        let data_type = self.parse_type()?;
        
        // 解析length
        let length = self.parse_length()?;
        
        // 解析scope
        let scope = self.parse_scope()?;
        
        // 解析cover
        let cover = self.parse_cover()?;
        
        // 解析可选字段
        let constraint = self.parse_constraint()?;
        let algorithm = self.parse_algorithm()?;
        let associate = self.parse_associate()?;
        let description = self.parse_description()?;
        let role = self.parse_role()?;
        
        Ok(SyntaxUnit {
            field_id,
            data_type,
            length,
            scope,
            cover,
            constraint,
            algorithm,
            associate,
            description,
            role,
        })
    }
}
```

### 3.2 CCSDS分包遥测协议JSON描述

#### 3.2.1 JSON协议描述结构
```json
{
  "protocol_meta": {
    "protocol_id": "CCSD_TM_PACKET_001",
    "protocol_name": "CCSDS分包遥测协议",
    "version": "V1.0",
    "base_standard": "CCSDS 132.0-B-2,CCSDS 143.0-B-1",
    "max_frame_len": 4096,
    "max_subpkg_count": 8
  },
  "syntax_units": [
    {
      "field_id": "SYNC_MARKER_001",
      "dsl_def": "field: SYNC_MARKER_001; type: Uint16; length: 2byte; scope: layer(ccsd_tm_frame_header); cover: ccsd_tm_frame_header[0..1]; constraint: fixed(0xEB90); desc: \"CCSDS 132.0-B-2 4.2.1\"; role: mandatory;"
    },
    {
      "field_id": "PRIMARY_PTR_001",
      "dsl_def": "field: PRIMARY_PTR_001; type: Uint16; length: 2byte; scope: layer(ccsd_tm_data_area); cover: ccsd_tm_data_area[0..1]; constraint: range(0..=4096); associate: TM_SUBPKG_001; desc: \"首个子包长度\"; role: primary_ptr;"
    }
  ],
  "frame_structure": {
    "frame_header": {
      "unit_list": [
        {"field_id": "SYNC_MARKER_001", "byte_offset": 0},
        {"field_id": "SEQ_FLAG_001", "byte_offset": 2},
        {"field_id": "SEQ_NUM_001", "byte_offset": 4}
      ],
      "total_len": 6
    },
    "frame_data_area": {
      "total_len_expr": "protocol_meta.max_frame_len - frame_header.total_len - frame_tail.total_len",
      "subpkg_nesting": {
        "subpkg_proto_id": "CCSD_TM_SUBPKG_001",
        "subpkg_structure": {
          "subpkg_header": {"unit_list": [{"field_id": "TM_SUBPKG_ID_001", "byte_offset": 0}], "total_len": 2},
          "subpkg_data": {"len_expr": "PRIMARY_PTR_001.value - 4", "data_type": "RawData"},
          "subpkg_tail": {"unit_list": [{"field_id": "TM_SUBPKG_CRC_001", "byte_offset": 0}], "total_len": 2}
        },
        "subpkg_location": [
          {"subpkg_id": "TM_SUBPKG_001", "start_expr": "frame_data_area.byte_offset + 2", "len_expr": "PRIMARY_PTR_001.value"}
        ]
      }
    },
    "frame_tail": {
      "unit_list": [{"field_id": "FRAME_CRC_001", "byte_offset": 0}],
      "total_len": 2
    }
  },
  "pack_unpack_spec": {
    "pack_order": [
      {"step":1, "operation":"assemble_frame_header", "target":"frame_header", "detail":"按 SYNC→FLAG→NUM 顺序组装"},
      {"step":2, "operation":"assemble_subpkg", "target":"frame_data_area", "detail":"计算子包长度→写入主指针→组装子包"},
      {"step":3, "operation":"calculate_crc", "target":"frame_tail", "detail":"计算帧头+数据区 CRC→写入 FRAME_CRC_001"},
      {"step":4, "operation":"merge_frame", "target":"ccsd_tm_frame", "detail":"拼接帧头→数据区→帧尾"}
    ],
    "unpack_order": [
      {"step":1, "operation":"extract_header", "target":"frame_header", "detail":"验证同步码→解析序列信息"},
      {"step":2, "operation":"validate_crc", "target":"frame_tail", "detail":"比对CRC→错误则丢弃"},
      {"step":3, "operation":"extract_subpkg", "target":"frame_data_area", "detail":"按主指针提取子包→验证子包 CRC"},
      {"step":4, "operation":"output", "target":"unpacked_data", "detail":"输出序列信息+有效子包"}
    ]
  }
}
```

#### 3.2.2 JSON协议描述解析器
```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProtocolDescription {
    pub protocol_meta: ProtocolMeta,
    pub syntax_units: Vec<SyntaxUnitDesc>,
    pub frame_structure: FrameStructure,
    pub pack_unpack_spec: PackUnpackSpec,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProtocolMeta {
    pub protocol_id: String,
    pub protocol_name: String,
    pub version: String,
    pub base_standard: String,
    pub max_frame_len: u32,
    pub max_subpkg_count: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyntaxUnitDesc {
    pub field_id: String,
    pub dsl_def: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FrameStructure {
    pub frame_header: FrameSection,
    pub frame_data_area: FrameDataArea,
    pub frame_tail: FrameSection,
}

pub struct JsonProtocolParser {
    pub json_content: String,
}

impl JsonProtocolParser {
    pub fn parse_protocol_description(&self) -> Result<ProtocolDescription, ParseError> {
        let protocol_desc: ProtocolDescription = serde_json::from_str(&self.json_content)?;
        Ok(protocol_desc)
    }
    
    pub fn convert_to_syntax_units(&self) -> Result<Vec<SyntaxUnit>, ParseError> {
        let protocol_desc = self.parse_protocol_description()?;
        let mut syntax_units = Vec::new();
        
        for unit_desc in protocol_desc.syntax_units {
            let dsl_parser = DslParser {
                dsl_content: unit_desc.dsl_def,
            };
            let syntax_unit = dsl_parser.parse_syntax_unit()?;
            syntax_units.push(syntax_unit);
        }
        
        Ok(syntax_units)
    }
}
```

## 4. Rust开发适配与模块接口设计

### 4.1 DSL引擎接口设计

#### 4.1.1 DSL解析器接口
```rust
pub trait DslParser {
    fn parse_dsl(&self, dsl_text: &str) -> Result<SyntaxUnit, DslParseError>;
    fn validate_dsl(&self, dsl_text: &str) -> Result<(), DslValidateError>;
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum UnitType {
    Uint(u8),      // Uint8, Uint16, Uint32, etc.
    Bit(u8),       // Bit(1), Bit(2), etc.
    RawData,
    Ip6Addr,
}

#[derive(Debug, Clone)]
pub struct LengthDesc {
    pub size: usize,
    pub unit: LengthUnit,
}

#[derive(Debug, Clone)]
pub enum LengthUnit {
    Byte,
    Bit,
    Dynamic,
    Expression(String),
}

#[derive(Debug, Clone)]
pub enum ScopeDesc {
    Layer(String),              // layer(link)
    CrossLayer(String, String), // cross_layer(net→link)
    Global(String),             // global(end2end)
}

#[derive(Debug, Clone)]
pub enum CoverDesc {
    Range(String, usize, usize), // frame_header[0..1]
    Expression(String),          // $cover
    EntireField,                 // entire_field
}

#[derive(Debug, Clone)]
pub enum Constraint {
    Fixed(u64),
    Range(u64, u64),
    Enum(Vec<(String, u64)>),
    Custom(String),
}
```

#### 4.1.2 DSL解析器实现
```rust
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0},
    combinator::{map_res, opt},
    multi::many0,
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
};

pub struct NomDslParser;

impl DslParser for NomDslParser {
    fn parse_dsl(&self, dsl_text: &str) -> Result<SyntaxUnit, DslParseError> {
        let (_, parsed_unit) = Self::parse_syntax_unit(dsl_text)
            .map_err(|e| DslParseError::ParseError(format!("{:?}", e)))?;
        Ok(parsed_unit)
    }

    fn validate_dsl(&self, dsl_text: &str) -> Result<(), DslValidateError> {
        match Self::parse_syntax_unit(dsl_text) {
            Ok(_) => Ok(()),
            Err(e) => Err(DslValidateError::ValidationError(format!("{:?}", e))),
        }
    }
}

impl NomDslParser {
    fn parse_syntax_unit(input: &str) -> IResult<&str, SyntaxUnit> {
        let (input, _) = multispace0(input)?;
        let (input, field_id) = Self::parse_field(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = char(';')(input)?;
        let (input, _) = multispace0(input)?;
        
        let (input, unit_type) = Self::parse_type(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = char(';')(input)?;
        let (input, _) = multispace0(input)?;
        
        let (input, length) = Self::parse_length(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = char(';')(input)?;
        let (input, _) = multispace0(input)?;
        
        let (input, scope) = Self::parse_scope(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = char(';')(input)?;
        let (input, _) = multispace0(input)?;
        
        let (input, cover) = Self::parse_cover(input)?;
        
        // 解析可选字段
        let (input, optional_fields) = many0(|i| {
            let (i, _) = multispace0(i)?;
            let (i, _) = char(';')(i)?;
            let (i, _) = multispace0(i)?;
            Self::parse_optional_field(i)
        })(input)?;
        
        // 构建语法单元
        let mut unit = SyntaxUnit {
            field_id,
            unit_type,
            length,
            scope,
            cover,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: String::new(),
        };
        
        // 应用可选字段
        for field in optional_fields {
            match field {
                OptionalField::Constraint(c) => unit.constraint = Some(c),
                OptionalField::Algorithm(a) => unit.alg = Some(a),
                OptionalField::Associate(a) => unit.associate = a,
                OptionalField::Description(d) => unit.desc = d,
            }
        };
        
        Ok((input, unit))
    }
    
    fn parse_field(input: &str) -> IResult<&str, String> {
        let (input, _) = tag("field:")(input)?;
        let (input, _) = multispace0(input)?;
        let (input, field_name) = take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)?;
        Ok((input, field_name.to_string()))
    }
    
    // 其他解析函数...
}
```

### 4.2 协议组装平台接口设计

#### 4.2.1 协议组装器接口
```rust
pub trait ProtocolAssembler {
    fn add_syntax_unit(&mut self, unit: SyntaxUnit);
    fn define_layer(&mut self, layer: ProtocolLayer);
    fn generate_spec(&self) -> Result<String, AssembleError>;
}

#[derive(Debug, Clone)]
pub struct ProtocolLayer {
    pub layer_id: String,
    pub unit_ids: Vec<String>,
    pub lower_layer: Option<String>,
    pub upper_layer: Option<String>,
    pub params: HashMap<String, String>,
}

#[derive(Debug)]
pub enum AssembleError {
    InvalidLayer(String),
    DuplicateUnit(String),
    CircularDependency(String),
    MissingDependency(String),
}
```

#### 4.2.2 协议组装器实现
```rust
use std::collections::HashMap;

pub struct DefaultProtocolAssembler {
    layers: Vec<ProtocolLayer>,
    units: Vec<SyntaxUnit>,
    layer_dependencies: HashMap<String, Vec<String>>,
}

impl DefaultProtocolAssembler {
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            units: Vec::new(),
            layer_dependencies: HashMap::new(),
        }
    }
}

impl ProtocolAssembler for DefaultProtocolAssembler {
    fn add_syntax_unit(&mut self, unit: SyntaxUnit) {
        self.units.push(unit);
    }

    fn define_layer(&mut self, layer: ProtocolLayer) {
        self.layers.push(layer);
    }

    fn generate_spec(&self) -> Result<String, AssembleError> {
        // 验证协议栈的有效性
        self.validate_protocol_stack()?;
        
        // 生成协议规范
        let mut spec = String::new();
        spec.push_str("# Generated Protocol Specification\n\n");
        
        // 添加层定义
        for layer in &self.layers {
            spec.push_str(&format!("## Layer: {}\n", layer.layer_id));
            spec.push_str(&format!("Units: {:?}\n", layer.unit_ids));
            spec.push_str("\n");
        }
        
        // 添加单元定义
        for unit in &self.units {
            spec.push_str(&format!("## Unit: {}\n", unit.field_id));
            spec.push_str(&format!("Type: {:?}\n", unit.unit_type));
            spec.push_str(&format!("Scope: {:?}\n", unit.scope));
            spec.push_str(&format!("Cover: {:?}\n", unit.cover));
            spec.push_str("\n");
        }
        
        Ok(spec)
    }
}

impl DefaultProtocolAssembler {
    fn validate_protocol_stack(&self) -> Result<(), AssembleError> {
        // 检查循环依赖
        if self.has_circular_dependency() {
            return Err(AssembleError::CircularDependency(
                "Circular dependency detected in protocol stack".to_string()
            ));
        }
        
        // 检查缺失依赖
        for layer in &self.layers {
            if let Some(ref lower) = layer.lower_layer {
                if !self.layer_exists(lower) {
                    return Err(AssembleError::MissingDependency(
                        format!("Layer {} depends on non-existent layer {}", layer.layer_id, lower)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    fn has_circular_dependency(&self) -> bool {
        // 简单的循环依赖检测
        // 在实际实现中可能需要更复杂的图算法
        false
    }
    
    fn layer_exists(&self, layer_id: &str) -> bool {
        self.layers.iter().any(|l| l.layer_id == layer_id)
    }
}
```

### 4.3 Rust依赖库选择与适配

#### 4.3.1 DSL解析库（nom）
```toml
[dependencies]
nom = "7.1"
```

```rust
// 使用nom进行DSL解析
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0},
    combinator::{map_res, opt},
    multi::many0,
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
};
```

#### 4.3.2 多格式文档解析库
```toml
[dependencies]
docx-rs = "0.4"
calamine = "0.21"
pdf-extract = "0.7"
```

```rust
// Word文档解析
use docx_rs::*;

// Excel文档解析
use calamine::{Reader, Xlsx};

// PDF文档解析
use pdf_extract::*;
```

#### 4.3.3 CRC计算库
```toml
[dependencies]
crc = "3.0"
```

```rust
use crc::{Crc, CRC_16_CCITT_FALSE, CRC_32_ISCSI};

pub fn calculate_ccsds_crc(data: &[u8]) -> u16 {
    let crc = Crc::<u16>::new(&CRC_16_CCITT_FALSE);
    crc.checksum(data)
}
```

#### 4.3.4 异步链路仿真库
```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
bytes = "1.0"
```

```rust
use tokio::net::{TcpStream, UdpSocket};
use bytes::BytesMut;

pub async fn simulate_link_transmission(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // 异步链路仿真实现
    Ok(data.to_vec())
}
```

#### 4.3.5 自定义脚本执行库
```toml
[dependencies]
rhai = { version = "1.0", features = ["sync"] }
```

```rust
use rhai::{Engine, EvalAltResult};

pub fn execute_custom_algorithm(script: &str, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let engine = Engine::new();
    let result = engine.eval::<i64>(script)?;
    Ok(result.to_le_bytes().to_vec())
}
```

#### 4.3.6 错误处理库
```toml
[dependencies]
anyhow = "1.0"
thiserror = "1.0"
```

```rust
use anyhow::{Result, Context};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Unknown parameter: {0}")]
    UnknownParameter(String),
    #[error("Not implemented")]
    NotImplemented,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

## 5. 语法单元核心分类

### 5.1 帧结构基础单元（控制帧/数据帧通用骨架）

#### 5.1.1 LLC帧同步序列单元
- **功能**：帧边界检测，提供同步标记
- **字段约束**：固定值 0xEB90（2字节）
- **DSL描述**：
```
field: llc_sync_marker, 
type: Uint16, 
length: 2byte, 
constraint: fixed(0xEB90), 
desc: "CCSDS 132.0-B-2 4.2.1"
```

#### 5.1.2 LLC帧类型标识单元
- **功能**：区分数据帧与控制帧
- **字段约束**：1比特，0=数据帧，1=控制帧
- **DSL描述**：
```
field: llc_frame_type, 
type: Bit(1), 
length: 1bit, 
constraint: enum(0=data,1=control), 
desc: "CCSDS 132.0-B-2 4.2.2.1"
```

#### 5.1.3 LLC帧长度字段单元
- **功能**：标识帧的实际长度
- **字段约束**：16比特，0~1023，实际长度=字段值+1
- **DSL描述**：
```
field: llc_frame_len, 
type: Uint16, 
length: 2byte, 
constraint: range(0..=1023), 
calc: actual_len=value+1, 
desc: "CCSDS 132.0-B-2 4.2.2.3"
```

### 5.2 信道与地址单元（数据路由定位）

#### 5.2.1 虚拟信道标识单元
- **功能**：标识数据传输的虚拟信道
- **字段约束**：6比特，0~63，其中0~3为预留信道
- **DSL描述**：
```
field: net_vc_id, 
type: Bit(6), 
length: 6bit, 
constraint: range(0..=63), 
reserve: 0~3, 
desc: "CCSDS 142.0-B-1 5.3.1"
```

#### 5.2.2 目标地址单元
- **功能**：指定数据包的目标地址
- **字段约束**：32比特，IPv6压缩格式，符合SCPS-IP规则
- **DSL描述**：
```
field: net_dst_addr, 
type: Ip6Addr, 
length: 4byte, 
constraint: scps_ip_format, 
desc: "CCSDS 142.0-B-1 6.2.2"
```

### 5.3 数据完整性单元（错误检测/重传）

#### 5.3.1 CRC-16校验单元
- **功能**：检测数据传输错误
- **字段约束**：16比特，多项式0x1021，覆盖帧控制→用户数据
- **DSL描述**：
```
field: llc_crc16, 
type: Uint16, 
length: 2byte, 
constraint: crc(poly=0x1021, cover=frame_ctrl..user_data), 
desc: "CCSDS 132.0-B-2 4.2.4"
```

#### 5.3.2 帧序号单元
- **功能**：检测丢帧情况
- **字段约束**：8比特，0~255，递增循环
- **DSL描述**：
```
field: llc_frame_seq, 
type: Uint8, 
length: 1byte, 
constraint: increment(step=1, wrap=255), 
desc: "CCSDS 132.0-B-2 4.2.2.2"
```

### 5.4 数据控制单元（分段/复接逻辑）

#### 5.4.1 分段标识单元
- **功能**：标识数据包的分段类型，用于子包重组
- **字段约束**：2比特，00=首段，01=中段，10=尾段，11=单段
- **DSL描述**：
```
field: sp_seg_flag, 
type: Bit(2), 
length: 2bit, 
constraint: enum(00=first,01=middle,10=last,11=single), 
desc: "CCSDS 143.0-B-1 4.2.1"
```

#### 5.4.2 数据长度指示单元
- **功能**：定位数据边界
- **字段约束**：16比特，0~4095
- **DSL描述**：
```
field: dli_len, 
type: Uint16, 
length: 2byte, 
constraint: range(0..=4095), 
desc: "CCSDS 143.0-B-1 4.2.3"
```

### 5.5 用户自定义扩展单元（协议扩展）

#### 5.5.1 设备状态码单元
- **功能**：扩展设备状态信息
- **字段约束**：8比特，0=正常，1=告警，2=故障，复用链路层扩展字段
- **DSL描述**：
```
field: user_dev_status, 
type: Uint8, 
length: 1byte, 
constraint: enum(0=normal,1=alert,2=fault), 
extend: yes, 
desc: "CCSDS 132.0-B-2 4.2.5 用户扩展"
```

#### 5.5.2 自定义时间戳单元
- **功能**：添加时间标记到遥测数据
- **字段约束**：32比特，UNIX秒级时间戳，新增于用户数据区头部
- **DSL描述**：
```
field: user_timestamp, 
type: Uint32, 
length: 4byte, 
constraint: unix_time(sec), 
extend: yes, 
desc: "遥测数据时间标记"
```

## 6. 帧结构关键单元深化设计

### 6.1 帧数据区嵌套单元（导头指针）

#### 6.1.1 主导头指针单元
- **功能**：定位帧数据区的首个子包
- **字段约束**：16比特，范围0到帧数据区总长度，关联子包数据区
- **作用范围**：帧数据区层
- **DSL描述**：
```
field: subpkg_primary_ptr,
type: Uint16,
length: 2byte,
scope: layer(frame_data),
cover: frame_data[0..1],
constraint: range(0..=frame_data.total_len),
associate: subpkg_data_area,
desc: "帧数据区首个子包定位，标识子包长度"
```

#### 6.1.2 副导头指针单元
- **功能**：定位帧数据区的后续子包
- **字段约束**：16比特，范围从主指针值之后到帧数据区末尾，关联主指针和第二个子包数据区
- **作用范围**：帧数据区层
- **DSL描述**：
```
field: subpkg_secondary_ptr,
type: Uint16,
length: 2byte,
scope: layer(frame_data),
cover: frame_data[2+primary_ptr.value .. 4+primary_ptr.value],
constraint: range(0..=frame_data.total_len - (4+primary_ptr.value)),
associate: subpkg_primary_ptr, subpkg_data_area_2,
desc: "帧数据区后续子包定位，标识下子包偏移"
```

### 6.2 帧序列管理单元（标志+序列号）

#### 6.2.1 帧序列标志单元
- **功能**：区分帧的角色（单帧/首帧/中间帧/尾帧）
- **字段约束**：2比特，枚举值00=单帧，01=首帧，10=中间帧，11=尾帧，关联帧序列号
- **作用范围**：帧头部层
- **DSL描述**：
```
field: frame_seq_flag,
type: Bit(2),
length: 2bit,
scope: layer(frame_header),
cover: frame_header[2..3],
constraint: enum(00=single,01=first,10=middle,11=last),
associate: frame_seq_num,
desc: "多帧重组标识，区分单帧/首帧/中间帧/尾帧"
```

#### 6.2.2 帧序列号单元
- **功能**：标识帧的顺序，用于丢帧检测和重复过滤
- **字段约束**：8比特，递增步长1，循环值255，关联帧序列标志，接收端检查规则
- **作用范围**：帧头部层
- **DSL描述**：
```
field: frame_seq_num,
type: Uint8,
length: 1byte,
scope: layer(frame_header),
cover: frame_header[4..5],
constraint: increment(step=1, wrap=255),
associate: frame_seq_flag,
check_rule: receiver_check(prev_num+1==current_num),
desc: "帧顺序标识，丢帧检测/重复过滤"
```

### 6.3 分路单元（数据分发）

#### 6.3.1 基础分路标识单元
- **功能**：提供基础的数据分路依据
- **字段约束**：6比特，枚举值0=TM主信道，1=TM备份信道，2=TC主信道，3=TC备份信道，必需字段
- **作用范围**：跨层（网络层→链路层）
- **DSL描述**：
```
field: basic_routing_vc_id,
type: Bit(6),
length: 6bit,
scope: cross_layer(net→link),
cover: net_header[0..5],
constraint: enum(0=tm_main,1=tm_backup,2=tc_main,3=tc_backup),
routing_role: mandatory,
desc: "CCSDS虚拟信道ID，基础分路依据"
```

#### 6.3.2 扩展分路标识单元
- **功能**：提供精细化的业务类型分路
- **字段约束**：8比特，枚举值0x01=温度，0x02=电压，0x03=电流，可选字段，与基础分路标识组合
- **作用范围**：应用层
- **DSL描述**：
```
field: ext_routing_service_type,
type: Uint8,
length: 1byte,
scope: layer(app),
cover: app_header[2..3],
constraint: enum(0x01=temp,0x02=voltage,0x03=current),
routing_role: optional,
routing_combine: basic_routing_vc_id + self,
desc: "业务类型标识，精细化分路"
```

## 7. CAN总线协议语法单元扩展

### 7.1 总线仲裁类单元

#### 7.1.1 CAN仲裁场单元
- **功能**：总线仲裁，确定消息优先级
- **字段约束**：11/29位ID，RTR位标识数据帧/远程帧
- **作用范围**：CAN仲裁层
- **DSL描述**：
```
field: can_arbitration_field,
type: Bit(11)/Bit(29),
length: 11bit/29bit,
scope: layer(can_arbitration),
constraint: enum(RTR=0=data_frame, RTR=1=remote_frame),
desc: "CAN总线仲裁场，显性电平优先"
```

#### 7.1.2 CAN仲裁退避单元
- **功能**：仲裁失败后等待总线空闲并重新发送
- **字段约束**：8比特，范围0-10次
- **作用范围**：CAN控制层
- **DSL描述**：
```
field: can_arbitration_backoff,
type: Uint8,
length: 1byte,
scope: layer(can_control),
constraint: range(0..=10),
desc: "CAN仲裁失败退避次数，最大10次"
```

### 7.2 帧控制与错误处理类单元

#### 7.2.1 DLC字段单元
- **功能**：数据长度码，标识数据字段长度
- **字段约束**：4比特，0~15，DLC=9~15对应8字节（传统CAN）或64字节（CAN FD）
- **作用范围**：CAN控制层
- **DSL描述**：
```
field: can_dlc,
type: Bit(4),
length: 4bit,
scope: layer(can_control),
constraint: enum(0=0B, 1=1B, .., 8=8B, 9=8B/64B),
desc: "CAN数据长度码，支持传统CAN和CAN FD"
```

#### 7.2.2 位填充单元
- **功能**：连续5个相同位后插入相反位，用于接收端同步
- **字段约束**：1比特，触发条件为连续5个相同位
- **作用范围**：CAN物理层
- **DSL描述**：
```
field: can_bit_stuffing,
type: Bit(1),
length: 1bit,
scope: layer(can_physical),
constraint: trigger=5_same_bits,
desc: "CAN总线同步，填充位自动插入/删除"
```

#### 7.2.3 ACK应答单元
- **功能**：接收端确认机制
- **字段约束**：2比特，ACK位（发送端1→接收端0覆盖）+1位界定符
- **作用范围**：CAN帧尾层
- **DSL描述**：
```
field: can_ack,
type: Bit(2),
length: 2bit,
scope: layer(can_frame_tail),
constraint: ack_bit=0(valid), ack_delimiter=1,
desc: "CAN帧接收确认"
```

## 8. 语法单元作用范围分类

### 8.1 层内局部单元（单一层+部分字段）

#### 8.1.1 帧序号单元（层内局部）
- **作用范围**：链路层，当前帧
- **覆盖范围**：当前帧
- **DSL描述**：
```
field: llc_frame_seq,
type: Uint8,
length: 1byte,
scope: layer(link),
cover: current_frame,
desc: "链路层帧序号，作用于当前帧"
```

#### 8.1.2 链路层帧长度单元（层内局部）
- **作用范围**：链路层，帧长度字段后的数据
- **覆盖范围**：帧长度字段到帧尾部，总长度=值+2
- **DSL描述**：
```
field: llc_frame_len,
type: Uint16,
length: 2byte,
scope: layer(link),
cover: frame_len_field..frame_tail,
calc: total_len=value+2,
desc: "链路层帧长度，作用于长度字段之后的数据"
```

### 8.2 层内全量单元（单一层+全量数据）

#### 8.2.1 链路层CRC-16单元（层内全量）
- **作用范围**：链路层，帧控制到用户数据（不含自身）
- **覆盖范围**：帧控制到用户数据，排除自身
- **DSL描述**：
```
field: link_crc16,
type: Uint16,
length: 2byte,
scope: layer(link),
cover: frame_ctrl..user_data,
exclude: self,
alg: {
    type: crc16,
    params: {poly:0x1021, init:0xFFFF, ref_in:false, ref_out:false, xor_out:0x0000},
    data_range: $cover
},
desc: "链路层CRC-16，覆盖帧控制到用户数据（不含自身）"
```

#### 8.2.2 网络层总长度单元（层内全量）
- **作用范围**：网络层，完整分段包（含头+数据+CRC）
- **覆盖范围**：完整分段包
- **DSL描述**：
```
field: net_total_len,
type: Uint16,
length: 2byte,
scope: layer(net),
cover: entire_segment,
calc: total_len=header_len+data_len+crc_len,
desc: "网络层总长度，覆盖完整分段包"
```

### 8.3 跨层穿透单元（相邻层+层间数据）

#### 8.3.1 SDU长度指示单元（跨层穿透）
- **作用范围**：应用层→网络层，应用层ADU数据
- **覆盖范围**：应用层ADU数据
- **DSL描述**：
```
field: sdu_len_indicator,
type: Uint16,
length: 2byte,
scope: cross_layer(app->net),
cover: app_adu,
desc: "SDU长度指示，跨应用层到网络层，作用于应用层ADU数据"
```

#### 8.3.2 跨层包长同步单元（跨层穿透）
- **作用范围**：网络层→链路层，网络层分段包
- **覆盖范围**：网络层分段包
- **DSL描述**：
```
field: cross_layer_pkt_len_sync,
type: Uint16,
length: 2byte,
scope: cross_layer(net->link),
cover: net_segment,
calc: link_frame_len=net_segment_len+link_header_len,
desc: "跨层包长同步，从网络层到链路层，作用于网络层分段包"
```

### 8.4 全链路全局单元（全链路+端到端数据）

#### 8.4.1 全局数据ID单元（全链路全局）
- **作用范围**：全链路，遥测任务所有帧
- **覆盖范围**：遥测任务数据
- **DSL描述**：
```
field: global_data_id,
type: Uint32,
length: 4byte,
scope: global(end2end),
cover: telemetry_task_data,
desc: "全局数据ID，作用于遥测任务的所有帧"
```

#### 8.4.2 端到端CRC单元（全链路全局）
- **作用范围**：全链路，完整文件数据
- **覆盖范围**：完整文件，排除各层CRC
- **DSL描述**：
```
field: end2end_crc,
type: Uint32,
length: 4byte,
scope: global(end2end),
cover: entire_file,
exclude: layer_crcs,
desc: "端到端CRC，覆盖完整文件数据，排除各层CRC"
```

## 9. 算法类语法单元设计

### 9.1 DSL算法描述核心结构
```
field: <field_name>,
type: <data_type>,
length: <size>,
scope: <scope_type>,
cover: <data_coverage>,
alg: {
    type: [算法类型，如 crc16/crc32/custom/xor_sum], // 必选
    params: {[参数键值对，如 poly=0x1021, init=0xFFFF]}, // 可选
    data_range: [引用 cover 字段，如$cover], // 必选
    logic: [自定义逻辑，类 Rust 伪代码] // 仅 custom 类型需填
},
desc: <description>
```

### 9.2 标准算法实例 - CRC-16
```
field: link_crc16,
type: Uint16,
length: 2byte,
scope: layer(link),
cover: frame_ctrl..user_data,
exclude: self,
alg: {
    type: crc16,
    params: {poly:0x1021, init:0xFFFF, ref_in:false, ref_out:false, xor_out:0x0000},
    data_range: $cover
},
desc: "CCSDS 132.0-B-2 4.2.4 链路层 CRC-16"
```

### 9.3 自定义算法实例 - 字节和取模
```
field: custom_sum_mod,
type: Uint8,
length: 1byte,
scope: cross_layer(app→net),
cover: app_adu,
alg: {
    type: custom,
    params: {mod_value:256, init_sum:0},
    data_range: $cover,
    logic: |
        let mut sum = params.init_sum;
        for byte in data_range.iter() {
            sum += byte as u32;
        }
        let result = (sum % params.mod_value) as u8;
        return result;
    |
},
desc: "用户自定义字节和取模校验"
```

### 9.4 Rust解析与执行逻辑
```rust
// 1. 算法抽象语法树
#[derive(Debug, Clone)]
pub enum AlgorithmType {
    Crc16, Crc32, XorSum, Custom(String)
}

#[derive(Debug, Clone)]
pub struct AlgorithmAst {
    alg_type: AlgorithmType,
    params: HashMap<String, String>,
    data_range: DataRange,
}

// 2. CRC-16 执行器
pub fn execute_crc16(ast: &AlgorithmAst, data: &[u8]) -> Result<u16, AlgorithmError> {
    let poly = ast.params.get("poly").unwrap_or(&"0x1021".to_string()).parse::<u16>()?;
    let init = ast.params.get("init").unwrap_or(&"0xFFFF".to_string()).parse::<u16>()?;
    let mut crc = crc::Crc::<u16>::new(&crc::Algorithm {
        width:16, 
        poly, 
        init, 
        ref_in:false, 
        ref_out:false, 
        xor_out:0x0000, 
        check:0x0000,
        residue:0x0000
    });
    crc.update(data);
    Ok(crc.finalize())
}

// 3. 统一执行入口
pub fn execute_algorithm(ast: &AlgorithmAst, protocol_data: &[u8]) -> 
    Result<Vec<u8>, AlgorithmError> {
    let target_data = extract_data_range(protocol_data, &ast.data_range)?;
    let result = match &ast.alg_type {
        AlgorithmType::Crc16 => execute_crc16(ast, &target_data)?.to_be_bytes().to_vec(),
        AlgorithmType::Custom(logic) => execute_custom_logic(ast, &target_data, logic)?,
        _ => unimplemented!()
    };
    Ok(result)
}
```

## 10. 语法单元组合设计

### 10.1 协议栈管理器
```rust
pub struct ProtocolStack {
    units: Vec<Arc<dyn ProtocolUnit>>, // 语法单元列表
    name: String,                      // 协议栈名称
    description: String,               // 描述
}
```

### 10.2 组合策略
1. **垂直组合**: 上层协议封装在下层协议中
   - 例如：Space Packet → TM Transfer Frame
   
2. **水平组合**: 同层协议的并行处理
   - 例如：多路TM虚拟信道复接

3. **混合组合**: 复杂的协议栈结构
   - 例如：多层协议的嵌套和复接

4. **跨协议组合**: 不同协议栈的混合使用
   - 例如：CCSDS协议与CAN协议的桥接

## 11. DSL语法设计

### 11.1 协议定义语法
```
ProtocolDef "PROTOCOL_NAME" {
    Version: "VERSION_STRING",
    Desc: "DESCRIPTION",
    
    Layers {
        Layer LAYER_NAME [Standard(CCSDS_STANDARD)] {
            Params: {
                PARAM_NAME: PARAM_VALUE,
                ...
            },
            Fields {
                FIELD_NAME: TYPE [CONSTRAINTS],
                ...
            }
        }
    }
    
    Relation {
        LAYER1 -> LAYER2: "RELATION_TYPE"
    }
    
    RegisterUnit to UNIT_LIBRARY;
}
```

### 11.2 示例：CCSDS-CAN桥接协议栈
```
ProtocolDef "CCSDS_CAN_BRIDGE" {
    Version: "1.0",
    Desc: "CCSDS与CAN协议桥接栈：CCSDS协议数据转换为CAN帧传输",
    
    Layers {
        Layer SP [Standard(CCSDS_133_0_B_3)] {
            Params: {
                APID: 100,
                Type: Telemetry,
                SecondaryHeader: false
            },
            Fields {
                subpkg_primary_ptr: Uint16 [range(0..=frame_data.total_len)],
                subpkg_secondary_ptr: Uint16 [range(0..=frame_data.total_len - offset)],
                frame_seq_flag: Bit(2) [enum(00=single,01=first,10=middle,11=last)],
                frame_seq_num: Uint8 [increment(step=1, wrap=255)],
                basic_routing_vc_id: Bit(6) [enum(0=tm_main,1=tm_backup,2=tc_main,3=tc_backup)],
                ext_routing_service_type: Uint8 [enum(0x01=temp,0x02=voltage,0x03=current)]
            }
        }
        
        Layer TM [Standard(CCSDS_132_0_B_3)] {
            Params: {
                VC_ID: 5,
                SC_ID: 123,
                FrameLength: 1024,
                ErrorControl: BCH
            }
        }
        
        Layer CAN [Standard(CAN_2.0B)] {
            Params: {
                ArbitrationID: 0x123,
                FrameType: Data,
                DLC: 8
            }
        }
    }
    
    Relation {
        SP -> TM: "Encapsulation",
        TM -> CAN: "BridgeConversion"
    }
    
    RegisterUnit to StandardUnitLib;
}
```

## 12. 错误处理设计

### 12.1 错误类型定义
```rust
#[derive(Debug)]
pub enum ProtocolError {
    InvalidFormat(String),
    ValidationError(String),
    UnknownParameter(String),
    NotImplemented,
    IoError(std::io::Error),
}
```

### 12.2 错误处理策略
- 输入验证：在处理前验证参数和数据
- 早期失败：发现问题立即返回错误
- 详细信息：提供足够的错误上下文信息
- 可恢复性：设计可恢复的错误处理机制

## 13. 性能优化考虑

### 13.1 内存管理
- 预分配缓冲区减少内存分配
- 重用中间数据结构
- 使用零拷贝技术

### 13.2 计算优化
- 批量处理数据
- 并行处理多路数据流
- 优化关键算法

### 13.3 并发设计
- 使用Rust的并发原语
- 避免共享状态
- 利用异步处理

## 14. 测试策略

### 14.1 单元测试
- 每个语法单元独立测试
- 边界条件测试
- 错误路径测试

### 14.2 集成测试
- 语法单元组合测试
- 端到端协议栈测试
- 性能基准测试

### 14.3 标准合规性测试
- 与CCSDS标准一致性验证
- 与参考实现对比测试
- 交叉验证测试

## 15. 扩展性设计

### 15.1 插件架构
- 支持第三方语法单元
- 动态加载机制
- 版本兼容性

### 15.2 配置管理
- 运行时配置更新
- 参数验证机制
- 配置持久化

## 16. 安全性考虑

### 16.1 输入验证
- 严格的输入数据验证
- 防止缓冲区溢出
- 类型安全保证

### 16.2 内存安全
- 利用Rust所有权系统
- 避免未定义行为
- 防止内存泄漏

通过以上详细设计，我们确保了CCSDS协议语法单元的实现既符合标准要求，又具备良好的扩展性和可维护性，同时支持CAN协议的扩展、帧结构关键单元的深化设计、DSL规范与JSON协议描述的集成，以及Rust开发适配与模块接口的实现，从而保证了项目需求的合理性。