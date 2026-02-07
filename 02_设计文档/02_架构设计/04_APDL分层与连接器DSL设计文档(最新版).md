# APDL 分层与连接器 DSL 设计文档（更新版）

## 概述

基于现有实现和《数据区子包恢复_DSL设计.md》文档，本设计文档描述了APDL的分层与连接器DSL。该DSL建立在现有基础DSL之上，实现多层协议结构和连接器功能。

## 现状分析

### 已实现功能
1. **基础DSL语法**：field, type, length, scope等基本语法
2. **23种语义规则**：校验和、依赖关系、控制规则等
3. **字段映射规则**：基础的field_mapping语义规则
4. **模块化解析器架构**：nom-based解析器框架

### 需要增强功能
1. **包定义语法**：完整的package定义能力
2. **连接器模式**：独立的包间映射机制
3. **导头指针处理**：数据区子包恢复机制
4. **多层协议栈**：层间关系定义

## DSL 语法设计（增强版）

### 1. 包定义语法

```
package <package_name> {
    name: "<package_display_name>";
    type: <package_type>;  // telemetry, command, encapsulating, etc.
    layers: [
        {
            name: "<layer_name>",
            units: [<syntax_unit_definitions>],
            rules: [<semantic_rule_definitions>]
        }
    ];
    desc: "<package_description>";
}
```

### 2. 连接器定义语法

```
connector <connector_name> {
    type: <connector_type>;  // field_mapping, header_pointer, etc.
    source_package: "<source_package_name>";
    target_package: "<target_package_name>";
    config: {
        mappings: [
            {
                source_field: "<source_field_name>",
                target_field: "<target_field_name>", 
                logic: "<mapping_logic>",
                default_value: "<default_value>"
            }
        ],
        header_pointers: {
            master_pointer: "<master_pointer_field>",
            secondary_pointers: ["<pointer1>", "<pointer2>"],
            descriptor_field: "<descriptor_field>"
        }
    };
    desc: "<connector_description>";
}
```

### 3. 协议栈定义语法

```
protocol_stack <stack_name> {
    packages: ["<package1>", "<package2>", ...];
    connectors: ["<connector1>", "<connector2>", ...];
    parallel_groups: [
        {
            name: "<group_name>",
            packages: ["<pkg1>", "<pkg2>"],
            algorithm: "<mux_algorithm>",
            priority: <priority_value>
        }
    ];
    desc: "<stack_description>";
}
```

## 与现有系统的集成

### 1. 扩展现有结构
- 扩展 `apdl_core::SyntaxUnit` 支持包定义
- 扩展 `apdl_core::SemanticRule` 支持连接器规则
- 新增 `PackageDefinition` 和 `ConnectorDefinition` 结构

### 2. 解析器扩展
- 扩展 `dsl_parser.rs` 支持 package 和 connector 语法
- 新增 `package_parser.rs` 处理包定义
- 新增 `connector_parser.rs` 处理连接器定义

## 完整示例

```
// 定义遥测包
package telemetry_packet {
    name: "telemetry_packet";
    type: "telemetry";
    layers: [
        {
            name: "telemetry_header",
            units: [
                field: tlm_version { type: Bit(3); length: 1; },
                field: tlm_type { type: Bit(5); length: 1; },
                field: tlm_source_id { type: Uint16; length: 2; },
                field: tlm_sequence { type: Uint32; length: 4; },
                field: tlm_length { type: Uint16; length: 2; },
                field: tlm_checksum { type: Uint16; length: 2; },
            ],
            rules: [
                rule: checksum_range(tlm_checksum from tlm_version to tlm_length);
                rule: dependency(tlm_sequence depends_on tlm_source_id);
            ]
        },
        {
            name: "telemetry_data", 
            units: [
                field: tlm_payload { type: RawData; length: "dynamic"; },
            ],
            rules: []
        }
    ];
    desc: "CCSDS Telemetry Packet Definition";
}

// 定义封装包
package encapsulating_packet {
    name: "encapsulating_packet"; 
    type: "encapsulating";
    layers: [
        {
            name: "primary_header",
            units: [
                field: version { type: Bit(3); length: 1; },
                field: type { type: Bit(1); length: 1; },
                field: sec_hdr_flg { type: Bit(1); length: 1; },
                field: apid { type: Bit(11); length: 2; },
                field: seq_flags { type: Bit(2); length: 2; },
                field: packet_seq_cnt { type: Bit(14); length: 2; },
                field: data_len { type: Uint16; length: 2; },
                
                // 主导头指针
                field: master_header_pointer { 
                    type: Uint16; 
                    length: 2; 
                    desc: "Pointer to first nested packet";
                },
                
                // 副导头指针数组
                field: secondary_header_pointers { 
                    type: Array(Uint16); 
                    length: "variable"; 
                    desc: "Pointers to subsequent nested packets";
                },
            ],
            rules: []
        },
        {
            name: "data_area",
            units: [
                field: nested_packet_descriptors { 
                    type: Array(nested_packet_descriptor); 
                    length: "variable"; 
                },
                field: data_field { 
                    type: RawData; 
                    length: "dynamic"; 
                    desc: "Data area containing nested packets";
                },
            ],
            rules: [
                rule: pointer_resolution(
                    field: master_header_pointer, 
                    target: data_field, 
                    method: "offset_from_start"
                );
                rule: boundary_detection(
                    descriptor: nested_packet_descriptors,
                    data_field: data_field, 
                    method: "offset_length_pairs"
                );
            ]
        }
    ];
    desc: "Encapsulating Packet with Header Pointers";
}

// 定义连接器（映射遥测包到封装包）
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
            },
            {
                source_field: "tlm_type", 
                target_field: "master_header_pointer",
                logic: "identity",
                default_value: "0"
            }
        ]
    };
    desc: "Map telemetry packet fields to encapsulating packet fields";
}

// 定义协议栈
protocol_stack ccsds_stack {
    packages: ["telemetry_packet", "encapsulating_packet"];
    connectors: ["telemetry_to_encapsulating"];
    parallel_groups: [
        {
            name: "tm_tc_mux",
            packages: ["telemetry_packet", "command_packet"], 
            algorithm: "time_division",
            priority: 5
        }
    ];
    desc: "CCSDS Protocol Stack with Telemetry Encapsulation";
}
```

## 技术实现架构

### 1. 新增数据结构
- `PackageDefinition` - 包定义结构
- `LayerDefinition` - 层定义结构  
- `ConnectorDefinition` - 连接器定义结构
- `ProtocolStackDefinition` - 协议栈定义结构

### 2. 新增解析器模块
- `package_parser.rs` - 包定义解析器
- `connector_parser.rs` - 连接器解析器
- `protocol_stack_parser.rs` - 协议栈解析器

### 3. 扩展现有模块
- 扩展 `dsl_parser.rs` 支持新语法
- 扩展 `semantic_rule_parsers` 支持新规则类型

## 开发计划

### Phase 1: 语法设计与数据结构 (Week 1)
1. 完善增强版DSL语法规范
2. 设计 `PackageDefinition` 结构
3. 设计 `ConnectorDefinition` 结构  
4. 设计 `ProtocolStackDefinition` 结构
5. 扩展 `apdl_core` 数据结构

### Phase 2: 解析器实现 (Week 2-3)
1. 实现 `package_parser.rs`
2. 实现 `connector_parser.rs` 
3. 实现 `protocol_stack_parser.rs`
4. 扩展主解析器支持新语法
5. 实现语法验证逻辑

### Phase 3: 处理器实现 (Week 4-5)
1. 实现包处理逻辑
2. 实现连接器处理逻辑
3. 实现协议栈处理逻辑
4. 实现层间数据流转机制
5. 实现导头指针处理机制

### Phase 4: 集成与测试 (Week 6)
1. 集成到现有系统
2. 实现端到端测试
3. 性能优化
4. 文档编写

## 与《数据区子包恢复_DSL设计.md》的对应关系

本设计完全涵盖了文档中描述的功能：
1. ✅ 独立字段映射机制（连接器模式）
2. ✅ 基于导头指针的子包处理
3. ✅ 包定义和层定义语法
4. ✅ 映射规则和指针处理规则

## 预期成果

1. 完整的分层与连接器 DSL 语法
2. 高效的解析器系统
3. 完整的处理器实现
4. 与现有系统的无缝集成
5. 全面的测试覆盖