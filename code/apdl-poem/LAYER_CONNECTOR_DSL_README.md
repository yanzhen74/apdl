# APDL 分层与连接器DSL

## 概述

APDL（APDS Protocol Definition Language）分层与连接器DSL是一个用于定义协议层次结构和层间连接的领域特定语言。它实现了分层平等建模原则，允许各协议层地位平等，通过连接器实现层间数据传输。

## 设计原则

### 分层平等建模
- 各协议层地位平等，无主次之分
- 通过连接器实现层间数据传输
- 支持并列包组（Parallel Package Groups）处理复接场景

## 核心组件

### 1. 包（Package）
定义协议数据包的基本单位，包含多个协议层和字段定义。

### 2. 连接器（Connector）
定义层间数据传输机制，包含：
- 字段映射规则（field mapping rules）
- 数据放置策略（data placement strategies）

### 3. 协议栈（Protocol Stack）
组合多个包和连接器，定义协议处理流程，支持并列包组和优先级调度。

## 连接器功能详解

### 1. 字段映射（Field Mapping）
将源包的字段值映射到目标包的字段：
- 支持多种映射逻辑（identity, hash_mod_64, hash_mod_2048等）
- 支持默认值设置
- 支持枚举值映射

### 2. 数据放置策略（Data Placement Strategies）
定义如何将子包数据放入父包的数据区：
- **导头指针方式**：通过指针指向子包位置
- **直接放入方式**：当长度固定且匹配时直接嵌入
- **数据流方式**：按流数据放入固定长度的数据区
- **其他策略**：可根据协议需求扩展

## DSL语法

### 包定义语法
```
package <package_name> {
    name: "<Display Name>";
    type: "<Package Type>";
    desc: "<Description>";
    layers: [
        {
            name: "<Layer Name>";
            units: [...];  // 语法单元定义
            rules: [...];  // 语义规则定义
        }
    ];
};
```

### 连接器定义语法
```
connector <connector_name> {
    type: "<Connector Type>";          // 如 "field_mapping", "data_placement"
    source_package: "<Source Package>"; // 源包名称
    target_package: "<Target Package>"; // 目标包名称
    config: {
        mappings: [                    // 字段映射规则
            {
                source_field: "<Source Field Name>";
                target_field: "<Target Field Name>";
                logic: "<Mapping Logic>";      // 如 "hash(field) % 2048"
                default_value: "<Default Value>";
            }
        ];
        placement_strategy: {          // 数据放置策略
            strategy: "<Strategy Type>"; // "direct", "pointer_based", "stream_based"
            target_field: "<Target Field Name>"; // 在目标包中的放置位置
            config: {                  // 策略特定配置
                // 根据策略类型有所不同
            };
        };
    };
    desc: "<Description>";
};
```

### 协议栈定义语法
```
protocol_stack <stack_name> {
    packages: ["<Package1>", "<Package2>"];
    connectors: ["<Connector1>", "<Connector2>"];
    parallel_groups: [                // 并列包组（可选）
        {
            name: "<Group Name>";
            packages: ["<Package1>", "<Package2>"];
            algorithm: "<Algorithm>";  // 如 "time_division"
            priority: <Priority Number>;
        }
    ];
    desc: "<Description>";
};
```

## 功能特性

### 1. 嵌套结构支持
- 支持深度嵌套的花括号和方括号结构
- 正确处理字符串中的括号
- 平衡括号检测算法

### 2. 灵活的语法
- 支持可选的分号结尾
- 兼容多种格式风格
- 向后兼容性保证

### 3. 错误处理
- 详细的错误信息
- 上下文相关的错误提示
- 恢复性错误处理

## 使用示例

### CCSDS协议栈定义
```
package telemetry_packet {
    name: "Telemetry Packet";
    type: "telemetry";
    desc: "Standard CCSDS telemetry packet";
    layers: [
        {
            name: "primary_header";
            units: [...];
            rules: [...];
        }
    ];
};

package command_packet {
    name: "Command Packet";
    type: "command";
    desc: "Standard CCSDS command packet";
};

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
        placement_strategy: {
            strategy: "pointer_based";
            target_field: "data_field";
            config: {
                pointer_field: "first_header_ptr";
                map_id: "map_id_field";
            };
        };
    };
    desc: "Map telemetry source ID to APID and place data via pointers";
};

protocol_stack ccnds_stack {
    packages: ["telemetry_packet", "command_packet", "encapsulating_packet"];
    connectors: ["telemetry_to_encapsulating", "command_to_encapsulating"];
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

## 解析器API

### 包解析器
```rust
use apdl_poem::dsl::layers::package_parser::PackageParser;

let package = PackageParser::parse_package_definition(dsl_text)?;
```

### 连接器解析器
```rust
use apdl_poem::dsl::layers::connector_parser::ConnectorParser;

let connector = ConnectorParser::parse_connector_definition(dsl_text)?;
```

### 协议栈解析器
```rust
use apdl_poem::dsl::layers::protocol_stack_parser::ProtocolStackParser;

let stack = ProtocolStackParser::parse_protocol_stack_definition(dsl_text)?;
```

## 扩展性

该DSL设计具有良好的扩展性：
- 支持新的连接器类型
- 支持新的映射逻辑
- 支持自定义语义规则
- 支持协议栈级别的优化策略

## 性能特点

- 高效的字符串解析算法
- 最小化的内存分配
- 快速的错误恢复
- 流式的解析处理

## 应用场景

- CCSDS协议栈定义
- CAN总线协议适配
- 网络协议转换
- 数据包格式定义
- 协议仿真与测试