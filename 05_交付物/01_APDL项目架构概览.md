# APDL项目架构概览

## 文档信息
- **版本**: 1.0
- **日期**: 2026年1月29日
- **项目**: APDL (APDS Protocol Definition Language)
- **状态**: MVP完成

## 1. 项目概述

APDL（APDS Protocol Definition Language）是一个面向航天领域的协议定义与仿真验证一体化平台。项目实现了字段级语法单元架构，通过DSL驱动的方式支持CCSDS、CAN等多种航天领域协议的定义、仿真和验证。

## 2. 架构演进

### 2.1 初始架构（已废弃）
- 帧级协议单元实现
- 特定协议硬编码实现
- 如ccsds_ap.rs等特定协议文件

### 2.2 当前架构（已实现）
- 字段级语法单元架构
- DSL驱动的协议定义
- 语义规则系统
- 平台与协议定义分离

## 3. 核心架构组件

### 3.1 DSL解析器 (DslParserImpl)
- **功能**: 解析协议定义语言
- **位置**: `apdl-poem/src/dsl/parser.rs`
- **职责**: 
  - 解析字段定义语法
  - 解析语义规则语法
  - 验证语法正确性

### 3.2 语法单元 (SyntaxUnit)
- **功能**: 描述协议字段
- **位置**: `apdl-core/src/protocol_meta/mod.rs`
- **职责**:
  - 定义字段属性（类型、长度、作用域等）
  - 存储字段约束条件
  - 提供字段元数据

### 3.3 语义规则 (SemanticRule)
- **功能**: 定义协议处理逻辑
- **位置**: `apdl-core/src/protocol_meta/mod.rs`
- **类型**:
  - 数据流向控制类（路由、多路复用、优先级等）
  - 数据完整性保障类（校验、验证、同步等）
  - 数据流状态管理类（序列控制、状态机、定时传输等）
  - 数据结构映射类（指针、顺序、长度等）
  - 业务逻辑处理类（依赖、算法、安全等）

### 3.4 帧组装器 (FrameAssembler)
- **功能**: 协议帧的组装和解析
- **位置**: `apdl-poem/src/standard_units/frame_assembler.rs`
- **职责**:
  - 根据语法单元组装协议帧
  - 应用语义规则
  - 执行协议解析

## 4. 语义规则系统

### 4.1 数据流向控制类
- **RoutingDispatch**: 路由分发规则
- **Multiplexing**: 多路复用规则
- **PriorityProcessing**: 优先级处理规则
- **MessageFiltering**: 消息过滤规则

### 4.2 数据完整性保障类
- **ChecksumRange**: 校验范围规则
- **Validation**: 验证规则
- **ErrorDetection**: 错误检测规则
- **Synchronization**: 同步规则
- **LengthValidation**: 长度验证规则

### 4.3 数据流状态管理类
- **SequenceControl**: 序列控制规则
- **PeriodicTransmission**: 周期传输规则
- **StateMachine**: 状态机规则
- **TimeSynchronization**: 时间同步规则

### 4.4 数据结构映射类
- **Pointer**: 指针规则
- **Order**: 顺序规则
- **LengthRule**: 长度规则
- **AddressResolution**: 地址解析规则

### 4.5 业务逻辑处理类
- **Dependency**: 依赖规则
- **Algorithm**: 算法规则
- **Conditional**: 条件规则
- **Security**: 安全规则
- **FlowControl**: 流量控制规则
- **Redundancy**: 冗余规则

## 5. DSL语法规范

### 5.1 字段定义语法
```
field: field_name; type: type_spec; length: length_spec; scope: scope_spec; cover: cover_spec; [optional_params]; desc: "description";
```

### 5.2 语义规则语法
```
rule: rule_type(parameters);
```

### 5.3 示例
```
field: sync_flag; type: Uint16; length: 2byte; scope: layer(physical); cover: entire_field; constraint: fixed(0xEB90); desc: "同步标志";

rule: checksum_range(start: sync_flag to data_field);
rule: routing_dispatch(field: vcid; algorithm: hash_vcid_to_route; desc: "根据虚拟信道进行分路");
```

## 6. 协议支持

### 6.1 CCSDS协议
- **标准**: CCSDS 132.0-B-3, 133.0-B-1, 232.0-B-3
- **特性**:
  - 同步标志字段 (0xEB90)
  - 虚拟信道和APID分路
  - 序列计数控制
  - CRC校验验证
  - 帧长度计算

### 6.2 CAN协议
- **标准**: ISO 11898-1, SAE J1939, CANopen
- **特性**:
  - 29位扩展标识符
  - 优先级仲裁
  - PGN路由
  - 错误检测

## 7. 项目结构

```
apdl/
├── 01_需求文档/           # 需求文档
│   └── APDL_需求规格说明书SRS.md
├── 02_设计文档/           # 设计文档
│   ├── 01_设计文档总览.md
│   └── 02_架构设计/
│       ├── 01_APDL_MVP实现状态总结.md
│       ├── 02_APDL语义规则系统设计说明.md
│       └── 03_APDL_DSL语法规范.md
├── 03_代码实现/           # 代码实现
│   └── code/
│       ├── apdl-core/     # 核心库
│       ├── apdl-poem/     # DSL解析器和协议处理
│       └── apdl-resources/ # 协议资源库
├── 04_实施计划/           # 实施计划
│   └── 01_APDL_MVP实施总结与后续计划.md
└── 05_交付物/            # 交付物
    └── 01_APDL项目架构概览.md
```

## 8. 技术栈

- **编程语言**: Rust
- **核心库**: nom (已简化为字符串处理), crc, tokio, anyhow
- **设计模式**: 策略模式, 配置驱动
- **架构风格**: 组件化, 模块化

## 9. 扩展性设计

### 9.1 DSL驱动扩展
- 通过DSL定义新协议，无需修改核心代码
- 支持新语义规则类型的添加
- 配置化的协议处理流程

### 9.2 统一接口设计
- 标准化的语法单元接口
- 统一的语义规则处理接口
- 模块化的组件设计

## 10. MVP验证结果

### 10.1 功能验证
- [x] DSL解析器功能正常
- [x] 22种语义规则全部支持
- [x] 协议组装/解析功能正常
- [x] CCSDS和CAN协议支持正常

### 10.2 性能验证
- [x] DSL解析性能满足要求
- [x] 内存使用合理
- [x] 执行效率达标

### 10.3 质量验证
- [x] 所有测试通过
- [x] 错误处理完善
- [x] 代码质量达标

## 11. 后续发展方向

### 11.1 短期优化
- 性能优化
- 用户体验改进
- 功能增强

### 11.2 中期扩展
- 规则组合框架
- 仿真验证引擎
- 图形化设计工具

### 11.3 长期愿景
- 智能化支持
- 生态建设
- 企业级特性

## 12. 总结

APDL项目MVP已成功实现，建立了以字段级语法单元和语义规则系统为核心的新架构。该架构具备良好的扩展性和灵活性，为航天领域协议定义与仿真验证提供了强大的支撑平台。