# APDL 协议定义资源库

## 概述

此目录包含APDL（APDS Protocol Definition Language）系统的协议定义资源文件。这些文件使用APDL DSL语法定义各种航天协议的结构，支持标准协议和自定义协议。

## 目录结构

```
resources/
├── standard/          # 标准协议定义
│   ├── ccsds_tm_frame.apdl      # CCSDS TM传输帧
│   ├── ccsds_tc_frame.apdl      # CCSDS TC传输帧
│   ├── ccsds_time_code.apdl     # CCSDS 时间码
│   ├── ccsds_aos_frame.apdl     # CCSDS AOS帧
│   └── ccsds_packet_structure.apdl # CCSDS 数据包结构
└── custom/            # 自定义协议定义
    └── custom_satellite_protocol.apdl # 自定义卫星协议
```

## DSL语法说明

APDL DSL语法支持以下元素：

### 字段定义
```
field: <字段名>; type: <类型>; length: <长度>; scope: <作用域>; cover: <覆盖范围>; [constraint: <约束>]; desc: "<描述>";
```

### 支持的类型
- `Uint8`, `Uint16`, `Uint32`, `Uint64` - 无符号整数类型
- `Bit(n)` - 位字段
- `RawData` - 原始数据
- `Ip6Addr` - IPv6地址

### 支持的约束
- `fixed(value)` - 固定值约束
- `range(start..=end)` - 范围约束
- `enum(...)` - 枚举约束

### 作用域类型
- `layer(physical)` - 物理层
- `layer(data_link)` - 数据链路层
- `layer(network)` - 网络层
- `layer(transport)` - 传输层
- `layer(application)` - 应用层

### 算法标识
- `alg: crc16` - CRC16校验
- `alg: crc32` - CRC32校验
- `alg: xor_sum` - XOR校验

## 使用方法

这些协议定义文件可被APDL系统加载和解析，用于：
1. 协议仿真
2. 数据包组装/解析
3. 协议验证
4. 性能分析

## 标准协议参考

- **CCSDS 132.0-B-3**: TM (Telemetry) 传输帧
- **CCSDS 232.0-B-3**: TC (Telecommand) 传输帧
- **CCSDS 301.0-B-4**: 时间码协议
- **CCSDS 732.0-B-4**: AOS (Advanced Orbiting Systems) 协议
- **CCSDS 133.0-B-2**: 数据包协议

## 自定义协议

用户可以在`custom/`目录下定义自己的协议，遵循相同的DSL语法规范。