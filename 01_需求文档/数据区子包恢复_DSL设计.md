# 数据区子包恢复DSL设计规范

## 1. 概述

本文档定义了APDL中支持数据区子包恢复功能的DSL语法规范，包括独立的字段映射机制（连接器模式）和基于导头指针的子包处理机制。

## 2. 独立字段映射机制DSL（连接器模式）

### 2.1 上下层包字段定义

```rust
// 下层包头部字段定义
field: lower_layer_source_id {
    type: Uint16;                     // 下层包源标识符
    length: 2;                        // 2字节长度
    desc: "下层包源标识符，用于确定目标VCID/APID";
    role: "lower_layer_identifier";
    constraint: required;
};

// 下层包类型字段定义
field: lower_layer_type {
    type: Uint8;                      // 下层包类型
    length: 1;                        // 1字节长度
    desc: "下层包类型标识，用于确定数据类型或子系统标志";
    role: "lower_layer_type_identifier";
    constraint: required;
};

// 上层包VCID字段定义
field: vcid {
    type: Bit(6);                      // 6位虚拟通道ID
    length: 1;                        // 1字节长度（高位6位）
    desc: "CCSDS虚拟通道标识符";
    role: "virtual_channel_id";
    constraint: range(0, 63);
};

// 上层包APID字段定义
field: apid {
    type: Bit(11);                     // 11位应用进程ID
    length: 2;                        // 2字节长度（高位11位）
    desc: "CCSDS应用进程标识符";
    role: "application_process_id";
    constraint: range(0, 2047);
};

// 上层包子系统标志字段定义
field: subsystem_flag {
    type: Bit(8);                      // 8位子系统标志
    length: 1;                        // 1字节长度
    desc: "子系统标志，用于标识数据来源子系统";
    role: "subsystem_identifier";
    constraint: range(0, 255);
};

// 上层包数据类型字段定义
field: data_type {
    type: Uint8;                      // 数据类型
    length: 1;                        // 1字节长度
    desc: "数据类型标识，用于标识数据类别";
    role: "data_type_identifier";
    constraint: required;
};
```

### 2.2 独立字段映射规则定义

```rust
// 独立的字段映射规则定义（连接器模式）
mapping: lower_to_upper_layer_mapping {
    type: field_mapping_connector;
    source_package: "lower_layer_package";      // 源包（下层包）
    target_package: "upper_layer_package";      // 目标包（上层包）
    mappings: [
        {
            source_field: "lower_layer_source_id",
            target_field: "vcid",
            mapping_logic: "hash(lower_layer_source_id) % 64",  // 使用下层包源标识符哈希映射到VCID范围
            default_value: 0
        },
        {
            source_field: "lower_layer_source_id",
            target_field: "apid",
            mapping_logic: "hash(lower_layer_source_id << 8 | lower_layer_type) % 2048",  // 使用源和类型组合哈希映射到APID范围
            default_value: 0
        },
        {
            source_field: "lower_layer_source_id",
            target_field: "subsystem_flag",
            mapping_logic: "lower_layer_source_id >> 8",  // 使用源标识符高字节作为子系统标志
            default_value: 0
        },
        {
            source_field: "lower_layer_type",
            target_field: "data_type",
            mapping_logic: "identity",  // 直接映射
            default_value: 0
        }
    ];
    desc: "下层包到上层包的字段映射连接器";
};
```

### 2.3 独立映射规则语法单元示例

```rust
// 下层包定义（被封装的包）
package: lower_layer_package {
    name: "telemetry_packet",
    layers: [
        {
            name: "telemetry_header",
            units: [
                field: tlm_version { type: Bit(3); length: 1; },
                field: tlm_type { type: Bit(5); length: 1; },        // 遥测包类型
                field: tlm_source_id { type: Uint16; length: 2; },   // 遥测源标识符
                field: tlm_sequence { type: Uint32; length: 4; },
                field: tlm_length { type: Uint16; length: 2; },
                field: tlm_checksum { type: Uint16; length: 2; },
            ];
        },
        {
            name: "telemetry_data",
            units: [
                field: tlm_payload { type: RawData; length: "dynamic"; },
            ];
        }
    ];
};

// 上层包定义（封装包）
package: upper_layer_package {
    name: "encapsulating_packet",
    layers: [
        {
            name: "primary_header",
            units: [
                field: version { type: Bit(3); length: 1; },
                field: type { type: Bit(1); length: 1; },
                field: sec_hdr_flg { type: Bit(1); length: 1; },
                
                field: vcid { type: Bit(6); length: 1; },                 // VCID将被映射填充
                field: apid { type: Bit(11); length: 2; },                // APID将被映射填充
                field: subsystem_flag { type: Bit(8); length: 1; },       // 子系统标志将被映射填充
                field: data_type { type: Uint8; length: 1; },             // 数据类型将被映射填充
                
                field: seq_flags { type: Bit(2); length: 2; },
                field: packet_seq_cnt { type: Bit(14); length: 2; },
                field: data_len { type: Uint16; length: 2; },
                
                field: data_field { type: RawData; length: "dynamic"; },   // 包含下层包数据
            ];
        }
    ];
};

// 独立的映射规则（连接器）
mapping: telemetry_to_encapsulating_mapping {
    type: field_mapping_connector;
    source_package: "telemetry_packet",     // 源包为遥测包
    target_package: "encapsulating_packet", // 目标包为封装包
    mappings: [
        {
            source_field: "tlm_source_id",     // 遥测包的源标识符
            target_field: "vcid",             // 映射到封装包的VCID
            mapping_logic: "hash(tlm_source_id) % 64",
            default_value: 0
        },
        {
            source_field: "tlm_source_id",     // 遥测包的源标识符
            target_field: "apid",             // 映射到封装包的APID
            mapping_logic: "hash(tlm_source_id << 8 | tlm_type) % 2048",
            default_value: 0
        },
        {
            source_field: "tlm_source_id",     // 遥测包的源标识符
            target_field: "subsystem_flag",   // 映射到封装包的子系统标志
            mapping_logic: "tlm_source_id >> 8",
            default_value: 0
        },
        {
            source_field: "tlm_type",          // 遥测包的类型
            target_field: "data_type",        // 映射到封装包的数据类型
            mapping_logic: "identity",
            default_value: 0
        }
    ];
    desc: "遥测包到封装包的字段映射连接器";
};
```

## 3. 基于导头指针的子包处理DSL

### 3.1 导头指针字段定义

```rust
// 主导头指针字段定义
field: master_header_pointer {
    type: Uint16;                    // 16位指针值
    length: 2;                       // 2字节长度
    desc: "指向第一个下层包在数据区中的偏移位置";
    role: "master_nested_packet_pointer";
    constraint: range(0, 65535);
};

// 副导头指针字段定义
field: secondary_header_pointers {
    type: Array(Uint16);             // 指针数组
    length: "variable";               // 可变长度
    desc: "指向后续下层包在数据区中的偏移位置数组";
    role: "secondary_nested_packet_pointers";
    constraint: max_items(255);
};

// 下层包描述符字段定义
field: nested_packet_descriptor {
    type: Struct({
        offset: Uint16,              // 下层包在数据区中的偏移
        length: Uint16,              // 下层包长度
        type: Bit(8),                // 下层包类型
        flags: Bit(8)                // 下层包标志位
    });
    length: 6;                       // 固定6字节长度
    desc: "下层包描述符，包含偏移、长度和类型信息";
    role: "nested_packet_metadata";
};
```

### 3.2 导头指针处理规则

```rust
// 主导头指针解析规则
rule: master_header_pointer_resolution {
    type: pointer_resolution;
    params: {
        pointer_field: "master_header_pointer",
        target_area: "data_field",    // 目标区域是数据字段
        resolution_method: "offset_from_start",  // 从数据区开始位置计算偏移
        output_field: "first_nested_packet_offset"   // 输出解析后的偏移值
    };
    desc: "解析主导头指针，确定第一个下层包的位置";
};

// 下层包边界识别规则
rule: nested_packet_boundary_detection {
    type: boundary_detection;
    params: {
        descriptor_field: "nested_packet_descriptor",
        data_field: "data_field",
        detection_method: "offset_length_pairs",  // 使用偏移-长度对检测边界
        output_format: "segment_array"           // 输出分段数组
    };
    desc: "基于下层包描述符识别数据区中的下层包边界";
};

// 数据流注入规则
rule: nested_packet_stream_injection {
    type: stream_injection;
    params: {
        source_stream: "incoming_nested_packets",     // 输入下层包流
        target_area: "data_field",               // 目标数据区
        injection_method: "sequential_fill",     // 顺序填充
        alignment: 8,                           // 8字节对齐
        boundary_markers: true                   // 添加边界标记
    };
    desc: "将具有完整帧定义的下层包注入到数据区";
};
```

### 3.3 完整的导头指针处理协议定义

```rust
// 完整的导头指针处理协议定义示例
package: header_pointer_processor {
    name: "encapsulating_packet_with_pointers",
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
                
                // 关键：主导头指针
                field: master_header_pointer { 
                    type: Uint16; 
                    length: 2; 
                    desc: "指向第一个下层包的指针";
                },
                
                // 可选：副导头指针数组
                field: secondary_header_pointers { 
                    type: Array(Uint16); 
                    length: "variable"; 
                    desc: "指向后续下层包的指针数组";
                },
            ];
        },
        {
            name: "data_area",
            units: [
                field: nested_packet_descriptors { 
                    type: Array(nested_packet_descriptor); 
                    length: "variable"; 
                    desc: "下层包描述符数组";
                },
                field: data_field { 
                    type: RawData; 
                    length: "dynamic"; 
                    desc: "包含具有完整帧定义的下层包的数据区";
                },
            ];
            rules: [
                rule: master_header_pointer_resolution {
                    type: pointer_resolution;
                    params: {
                        pointer_field: "master_header_pointer",
                        target_area: "data_field",
                        resolution_method: "offset_from_start",
                        output_field: "first_nested_packet_offset"
                    };
                },
                rule: nested_packet_boundary_detection {
                    type: boundary_detection;
                    params: {
                        descriptor_field: "nested_packet_descriptors",
                        data_field: "data_field",
                        detection_method: "offset_length_pairs",
                        output_format: "segment_array"
                    };
                },
                rule: nested_packet_stream_injection {
                    type: stream_injection;
                    params: {
                        source_stream: "incoming_nested_packets",
                        target_area: "data_field",
                        injection_method: "sequential_fill",
                        alignment: 8,
                        boundary_markers: true
                    };
                }
            ];
        }
    ];
};
```

## 4. 组合使用示例

```rust
// 组合独立字段映射和导头指针处理的完整协议定义
protocol: integrated_nested_packet_processor {
    packages: [
        // 下层包定义
        package: telemetry_packet {
            name: "telemetry_packet",
            layers: [
                {
                    name: "telemetry_header",
                    units: [
                        field: tlm_version { type: Bit(3); length: 1; },
                        field: tlm_type { type: Bit(5); length: 1; },        // 遥测包类型
                        field: tlm_source_id { type: Uint16; length: 2; },   // 遥测源标识符
                        field: tlm_sequence { type: Uint32; length: 4; },
                        field: tlm_length { type: Uint16; length: 2; },
                    ];
                },
                {
                    name: "telemetry_payload",
                    units: [
                        field: tlm_data { type: RawData; length: "dynamic"; },
                    ];
                }
            ];
        },
        
        // 上层包定义
        package: encapsulating_packet {
            name: "encapsulating_packet",
            layers: [
                {
                    name: "encapsulating_header",
                    units: [
                        field: version { type: Bit(3); length: 1; },
                        field: type { type: Bit(1); length: 1; },
                        field: sec_hdr_flg { type: Bit(1); length: 1; },
                        
                        // VCID和APID将通过映射规则填充
                        field: vcid { 
                            type: Bit(6); 
                            length: 1; 
                            desc: "根据下层包头部信息映射的VCID";
                        },
                        
                        field: apid { 
                            type: Bit(11); 
                            length: 2; 
                            desc: "根据下层包头部信息映射的APID";
                        },
                        
                        field: seq_flags { type: Bit(2); length: 2; },
                        field: packet_seq_cnt { type: Bit(14); length: 2; },
                        field: data_len { type: Uint16; length: 2; },
                        
                        // 主导头指针指向数据区中的第一个下层包
                        field: master_header_pointer { 
                            type: Uint16; 
                            length: 2; 
                            desc: "指向第一个下层包的指针";
                        },
                    ];
                },
                {
                    name: "encapsulating_data_area",
                    units: [
                        field: data_field { 
                            type: RawData; 
                            length: "dynamic"; 
                            desc: "包含通过导头指针定位的完整下层包数据";
                        },
                    ];
                }
            ];
        }
    ],
    
    // 独立的映射规则（连接器）
    mappings: [
        mapping: telemetry_to_encapsulating_mapping {
            type: field_mapping_connector;
            source_package: "telemetry_packet",     // 源包为遥测包
            target_package: "encapsulating_packet", // 目标包为封装包
            mappings: [
                {
                    source_field: "tlm_source_id",     // 遥测包的源标识符
                    target_field: "vcid",             // 映射到封装包的VCID
                    mapping_logic: "hash(tlm_source_id) % 64",
                    default_value: 0
                },
                {
                    source_field: "tlm_source_id",     // 遥测包的源标识符
                    target_field: "apid",             // 映射到封装包的APID
                    mapping_logic: "hash(tlm_source_id << 8 | tlm_type) % 2048",
                    default_value: 0
                }
            ];
            desc: "遥测包到封装包的字段映射连接器";
        }
    ],
    
    // 导头指针处理规则
    rules: [
        rule: master_header_pointer_resolution {
            type: pointer_resolution;
            params: {
                pointer_field: "master_header_pointer",
                target_area: "data_field",
                resolution_method: "offset_from_start",
                output_field: "first_nested_packet_location"
            };
        },
        rule: nested_packet_recovery {
            type: stream_recovery;
            params: {
                data_area: "data_field",
                recovery_method: "pointer_based",
                output_stream: "recovered_nested_packets"
            };
        }
    ]
};
```

## 5. DSL语法扩展

为了支持这些新功能，需要在DSL语法中增加以下关键字和结构：

1. **新增顶级语法元素**：
   - `package`: 定义独立的包结构
   - `mapping`: 定义独立的字段映射规则（连接器模式）

2. **新增映射类型**：
   - `field_mapping_connector`: 字段映射连接器类型

3. **新增数据类型**：
   - `Array(type)`：数组类型
   - `Struct({...})`：结构体类型

4. **新增字段属性**：
   - `role`：字段角色标识

5. **扩展规则参数**：
   - `mapping_logic`：映射逻辑
   - `resolution_method`：解析方法
   - `injection_method`：注入方法

6. **语义增强**：
   - 支持独立的包定义
   - 支持独立的映射规则定义
   - 支持包间的连接器模式