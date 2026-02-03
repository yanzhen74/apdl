# APDL 分层与连接器DSL开发详细总结

## 项目概述

本项目为APDL（APDS Protocol Definition Language）开发了一个分层与连接器DSL系统，实现了协议定义的分层平等建模原则。该系统允许定义包（Packages）、连接器（Connectors）和协议栈（Protocol Stacks），支持多层协议间的连接和数据传输。

## 核心架构设计

### 1. 分层平等建模原则
- 各协议层地位平等，无主次之分
- 通过连接器实现层间数据传输
- 支持并列包组（Parallel Package Groups）处理复接场景

### 2. 核心组件结构

#### 包（Package）
- 定义协议数据包的基本单位
- 包含多个协议层和字段定义
- 支持自定义类型和描述信息

#### 连接器（Connector）
- 定义层间数据传输机制
- 包含字段映射规则
- 支持头部指针配置

#### 协议栈（Protocol Stack）
- 组合多个包和连接器
- 定义协议处理流程
- 支持并列包组和优先级调度

## 技术实现细节

### 1. 数据结构定义

在 `apdl-core/src/protocol_meta/mod.rs` 中定义了核心数据结构：

```rust
// 包定义结构
pub struct PackageDefinition {
    pub name: String,
    pub display_name: String,
    pub package_type: String,
    pub layers: Vec<LayerDefinition>,
    pub description: String,
}

// 连接器定义结构
pub struct ConnectorDefinition {
    pub name: String,
    pub connector_type: String,
    pub source_package: String,
    pub target_package: String,
    pub config: ConnectorConfig,
    pub description: String,
}

// 协议栈定义结构
pub struct ProtocolStackDefinition {
    pub name: String,
    pub packages: Vec<String>,
    pub connectors: Vec<String>,
    pub parallel_groups: Vec<ParallelPackageGroup>,
    pub description: String,
}
```

### 2. 解析器实现

#### 包解析器（PackageParser）
- 位于 `apdl-poem/src/dsl/layers/package_parser.rs`
- 解析包定义DSL语法
- 处理嵌套结构和数组内容

#### 连接器解析器（ConnectorParser）
- 位于 `apdl-poem/src/dsl/layers/connector_parser.rs`
- 解析连接器定义和配置
- 处理字段映射和头部指针配置

#### 协议栈解析器（ProtocolStackParser）
- 位于 `apdl-poem/src/dsl/layers/protocol_stack_parser.rs`
- 解析协议栈组成和并列包组
- 处理复杂的嵌套对象结构

### 3. DSL语法示例

#### 包定义示例
```
package telemetry_packet {
    name: "Telemetry Packet";
    type: "telemetry";
    desc: "Standard telemetry packet definition";
    layers: [
        {
            name: "primary_header";
            units: [...];
            rules: [...];
        }
    ];
}
```

#### 连接器定义示例
```
connector telemetry_to_encapsulating {
    type: "field_mapping";
    source_package: "telemetry_packet";
    target_package: "encapsulating_packet";
    config: {
        mappings: [
            {
                source_field: "tlm_source_id",
                target_field: "apid",
                logic: "hash(tlm_source_id) % 2048",
                default_value: "0"
            }
        ];
        header_pointers: {
            master_pointer: "packet_length",
            secondary_pointers: ["first_header_ptr", "map_id"],
            descriptor_field: "data_field_desc"
        };
    };
    desc: "Map telemetry to encapsulating packet";
};
```

#### 协议栈定义示例
```
protocol_stack ccnds_stack {
    packages: ["telemetry_packet", "encapsulating_packet"];
    connectors: ["telemetry_to_encapsulating"];
    parallel_groups: [
        {
            name: "tm_tc_mux";
            packages: ["telemetry_packet", "command_packet"];
            algorithm: "time_division";
            priority: 5;
        }
    ];
    desc: "CCSDS TM/TC multiplexing stack";
};
```

## 关键技术挑战与解决方案

### 1. 嵌套结构解析
**挑战**: DSL语法支持深度嵌套的花括号和方括号结构
**解决方案**: 实现了带引号感知的括号计数算法，正确处理字符串中的括号

```rust
let mut brace_count = 0;
let mut in_string = false;
let mut escape_next = false;

for (i, c) in text.char_indices() {
    if escape_next {
        escape_next = false;
        continue;
    }
    
    match c {
        '\\' if in_string => escape_next = true,
        '"' => in_string = !in_string,
        '{' if !in_string => brace_count += 1,
        '}' if !in_string => {
            brace_count -= 1;
            if brace_count == 0 {
                end_pos = i + 1;
                break;
            }
        }
        _ => {}
    }
}
```

### 2. 分号处理
**挑战**: DSL语法中存在可选的分号结尾，需要灵活处理
**解决方案**: 在解析前统一移除行末分号

```rust
let line_no_semicolon = line.trim_end().trim_end_matches(';');
```

### 3. 数组内容提取
**挑战**: 跨多行的数组定义需要正确提取
**解决方案**: 实现了平衡括号检测算法

## 系统集成

### 1. 模块组织
- `apdl-poem/src/dsl/layers/mod.rs`: 模块声明
- `package_parser.rs`: 包解析器
- `connector_parser.rs`: 连接器解析器  
- `protocol_stack_parser.rs`: 协议栈解析器

### 2. 依赖关系
- 依赖 `apdl-core` 提供基础数据结构
- 与现有DSL解析器集成
- 保持向后兼容性

## 测试验证

### 1. 单元测试
- 每个解析器都有对应的单元测试
- 测试各种边界情况和错误处理
- 所有测试均已通过

### 2. 集成测试
- 验证解析器与系统的整体集成
- 测试DSL语法的完整解析流程
- 确保数据结构的正确构建

## 性能优化

### 1. 内存效率
- 使用字符串切片减少内存分配
- 避免不必要的字符串拷贝

### 2. 解析效率
- 优化括号匹配算法
- 减少重复的字符串操作

## 设计原则

### 1. 分层平等
- 各协议层独立定义，无层级依赖
- 连接器负责层间协调

### 2. 扩展性
- 模块化设计便于功能扩展
- DSL语法支持未来增强

### 3. 一致性
- 统一的API设计风格
- 一致的错误处理机制

## 应用场景

### 1. CCSDS协议栈
- 支持CCSDS标准的多层协议定义
- 处理TM/TC复接场景

### 2. CAN总线协议
- 支持CAN协议的多路复用
- 处理优先级调度

### 3. 通用协议适配
- 适用于各种网络协议栈
- 支持协议转换和映射

## 未来发展方向

### 1. 功能增强
- 更丰富的语义规则支持
- 高级协议分析功能

### 2. 性能提升
- 并行解析支持
- 更高效的内存管理

### 3. 生态扩展
- 与其他工具链集成
- 代码生成支持

## 结论

APDL分层与连接器DSL系统已成功实现，提供了强大的协议定义能力。该系统遵循分层平等建模原则，支持复杂的多层协议定义和连接，为协议开发提供了灵活、高效的解决方案。所有功能模块均已通过测试验证，系统稳定可靠。