# APDL 项目上下文文档

## 项目概览

APDL (APDS Protocol Definition Language) 是一个面向航天领域的协议定义与仿真验证一体化平台。本项目提供标准化的协议定义、仿真验证和性能分析工具，满足航天领域复杂协议开发的需求。

## 当前状态

### 项目结构
- **工作区根目录**: `d:\user\yqd\project\apdl`
- **代码目录**: `d:\user\yqd\project\apdl\code`
- **核心模块**:
  - `apdl-core`: 核心抽象和数据结构
  - `apdl-poem`: 协议对象与实体映射（主要开发模块）
  - `apdl-iam`: 交互访问模块
  - `apdl-dpe`: 数据处理引擎
  - `apdl-lsk`: 仿真与验证工具包（新增接收引擎）
  - `apdl-pvpae`: 协议验证与性能分析引擎
  - `apdl-sak`: 安全认证工具包
  - `apdl-app`: 应用层

### 当前实现情况

#### 0. 软件化接收/解复接/拆包引擎 (新增 - 2026-02)

**阶段1：核心拆包能力** ✅
- **FrameDisassembler**: 帧拆包器
  - 支持bit级字段精确提取（与FrameAssembler对称）
  - 支持字段值校验（固定值、范围、枚举约束）
  - 支持CRC16和简单校验和验证
  - 支持字段bit位置自动计算
  - 单元测试覆盖率100%（14个测试用例全部通过）
  
- **ReceiveBuffer**: 接收缓存
  - 支持流式数据接收和缓存管理
  - 支持帧边界识别（基于同步字和长度字段）
  - 支持帧同步字搜索（固定标记、模式匹配）
  - 支持自动提取完整帧
  - 单元测试覆盖率100%（14个测试用例全部通过）
  
- **FrameSynchronizer**: 帧同步器
  - 支持CCSDS标准的固定同步字（如0xEB90、0x1ACFFC1D）
  - 支持带掩码的模式搜索
  - 支持滑动窗口高效搜索

**阶段2：解复接和异常处理** ✅
- **Demultiplexer**: 解复接器
  - 根据VCID/APID将复接流分离为多路独立数据流
  - 支持虚拟通道管理和状态跟踪
  - 集成序列号校验功能
  - 支持通道统计和丢帧率计算
  - 单元测试覆盖率100%（8个测试用例全部通过）

- **SequenceValidator**: 序列号校验器
  - 基于序列号连续性检测帧丢失
  - 支持CCSDS 14位序列号（模数0x4000）
  - 支持序列号回绕检测和重复帧检测
  - 多通道独立校验
  - 单元测试覆盖率100%（10个测试用例全部通过）

- **ReorderBuffer**: 乱序重排缓冲区
  - 基于序列号对PDU进行自动排序
  - 使用BTreeMap实现高效的有序存储
  - 支持滑动窗口管理和重排率统计
  - 单元测试覆盖率100%（7个测试用例全部通过）

- **集成测试**:
  - 端到端CCSDS Space Packet收发流程（2个测试用例）
  - 多通道解复接测试（5个测试用例）
  - 完整工作流测试（多通道+乱序+丢帧综合场景）
  - 总计：60个测试用例，覆盖率100%

#### 1. 基础 DSL 语法
- **字段定义**: 支持 `field`, `type`, `length`, `scope`, `cover` 等基础语法
- **类型系统**: 支持 `Uint8/16/32/64`, `Bit(n)`, `RawData` 等类型
- **长度描述**: 支持固定长度、动态长度、表达式长度
- **作用域**: 支持 `layer`, `cross_layer`, `global` 等作用域

#### 2. 语义规则系统
- **已实现规则**: 23 种语义规则，包括：
  - 校验和范围规则 (`checksum_range`)
  - 依赖关系规则 (`dependency`)
  - 条件规则 (`conditional`)
  - 顺序规则 (`order`)
  - 指针规则 (`pointer`)
  - 算法规则 (`algorithm`)
  - 长度规则 (`length_rule`)
  - 路由分发规则 (`routing_dispatch`)
  - 序列控制规则 (`sequence_control`)
  - 验证规则 (`validation`)
  - 多路复用规则 (`multiplexing`)
  - 优先级处理规则 (`priority_processing`)
  - 同步规则 (`synchronization`)
  - 长度验证规则 (`length_validation`)
  - 状态机规则 (`state_machine`)
  - 周期性传输规则 (`periodic_transmission`)
  - 消息过滤规则 (`message_filtering`)
  - 错误检测规则 (`error_detection`)
  - 嵌套同步规则 (`nested_sync`)
  - 序列重置规则 (`sequence_reset`)
  - 时间戳插入规则 (`timestamp_insertion`)
  - 流量控制规则 (`flow_control`)
  - 时间同步规则 (`time_synchronization`)
  - 地址解析规则 (`address_resolution`)
  - 安全规则 (`security`)
  - 冗余规则 (`redundancy`)
  - **字段映射规则** (`field_mapping`)

#### 3. 解析器架构
- **模块化设计**: 基于 nom 库的模块化解析器架构
- **语义规则解析器**: `semantic_rule_parsers` 模块
- **字段映射解析器**: `field_mapping_parser` 模块
- **DSL 解析器**: `dsl_parser` 模块

#### 4. 连接器与MPDU系统
- **连接器引擎**: `connector_engine` 模块，负责字段映射和数据放置
- **字段映射逻辑**: 支持多种映射策略
  - `identity`: 恒等映射（直接传递）
  - `hash_mod_64`/`hash_mod_2048`: 哈希取模映射
  - `mask_table`: 基于掩码表的字段映射 ✨ **新增**
  - 枚举映射：支持通配符的枚举值映射
- **掩码映射表**: 支持通过掩码进行字段值匹配和映射
  - 数据结构：`MaskMappingEntry` (mask, src_masked, dst)
  - JSON格式：支持十六进制字符串数组 `["0xFF", "0xF0"]`
  - DSL解析：支持 `mask_mapping_table` 字段定义
- **数据放置策略**: 
  - Direct: 直接放置，使用配置的 `target_field`
  - PointerBased: 基于指针的MPDU放置
  - StreamBased: 流式放置
- **MPDU管理器**: `mpdu_manager` 模块，实现CCSDS标准的多路协议数据单元
- **多路缓存队列**: 支持子包和父包模板的分类缓存管理
- **跨包分割处理**: 支持子包跨MPDU包分割和重组
- **填充码管理**: 符合CCSDS标准的填充码生成和处理
- **首导头指针**: 符合CCSDS标准的首导头指针机制
- **MPDU指针计算**: 正确计算首导头指针值，指示子包在父包数据区中的偏移位置

#### 4. Frame Assembler 系统
- **23 个规则处理器**: 每个语义规则对应一个处理器
- **TODO 标记系统**: 在所有规则处理器中添加了 TODO 注释
- **DSL 前缀支持**: 支持 `start:`, `field:`, `first:` 等前缀

#### 5. JSON反序列化增强
- **十六进制字符串支持**: `MaskMappingEntry` 支持字符串格式的十六进制数组
- **自定义反序列化器**: `deserialize_hex_array` 函数
- **多格式兼容**: 支持数字数组、十六进制字符串、十进制字符串

## 当前计划文档

### 已完成文档
- `APDL_LAYER_CONNECTOR_DESIGN.md`: 分层与连接器 DSL 设计文档
- `APDL_LAYER_CONNECTOR_DESIGN_UPDATED.md`: 基于当前实现现状的更新版设计文档
- `APDL_LAYER_CONNECTOR_DEV_PLAN.md`: 详细的开发计划
- `APDL_LAYER_CONNECTOR_DEV_CURRENT.md`: 基于当前实现现状的开发计划
- `CONNECTOR_ARCHITECTURE.md`: 连接器系统架构说明
- `LAYER_CONNECTOR_DSL_README.md`: 分层与连接器DSL用户文档

### 已实现功能
- **扩展数据结构**: 在 `apdl_core` 中添加了 `PackageDefinition`、`ConnectorDefinition`、`ProtocolStackDefinition` 等核心数据结构
- **掩码映射表**: 添加 `MaskMappingEntry` 数据结构，支持基于掩码的字段映射
- **包解析器**: 实现了 `package_parser`，支持包定义的解析
- **连接器解析器**: 实现了 `connector_parser`，支持连接器定义的解析
- **协议栈解析器**: 实现了 `protocol_stack_parser`，支持协议栈定义的解析
- **DSL 集成**: 将新的解析器集成到主 DSL 解析器中
- **基础解析功能**: 实现了对方括号数组、花括号对象、嵌套结构的解析支持
- **连接器系统**: 实现了DSL解析器与运行时引擎的协作架构
- **集成测试**: 创建了验证DSL解析器和运行时引擎协同工作的测试用例
- **数据放置策略**: 实现了多种数据放置策略（直接放入、指针基、数据流等）
- **字段映射功能**: 实现了完整的字段映射机制，包括掩码映射表支持
- **CCSDS标准测试**: 完成 CCSDS Space Packet → TM Frame 两层封装全流程测试

### 计划功能（待完善）
- **导头指针处理**: 支持数据区子包恢复机制
- **并列包处理**: 支持同一层内的并列包结构
- **高级映射逻辑**: 扩展连接器引擎支持更多映射类型

## 技术栈

- **编程语言**: Rust
- **解析器库**: nom 7.1
- **UI 框架**: egui/eframe (用于 apdl-iam)
- **模块化架构**: 清晰的依赖关系和接口定义
- **错误处理**: 自定义错误类型系统

## MVP 阶段成果

### 已实现功能
- CCSDS 协议标准语法单元实现
- DSL (Domain Specific Language) 设计与解析
- 核心架构设计
- 语义规则处理器系统
- 23 种语义规则处理器
- TODO 标记系统
- DSL 解析器增强
- 分层与连接器 DSL 系统

### 下一阶段目标
- 实现导头指针处理机制
- 完成数据区子包恢复功能
- 扩展连接器引擎支持更多映射类型
- 完善协议栈定义语法

## 项目特色

- **分层平等建模原则**: 每层都是语义完整、无隶属关系的自治单元
- **DSL 自治性**: 不依赖硬编码的帧实现文件，完全由 DSL 自动完成组帧逻辑
- **模块化设计**: 清晰的职责分离和接口定义
- **扩展性**: 易于添加新的语义规则和协议类型

## 开发规范

- **Git 提交规范**: 提交时必须回到项目根目录 (`d:\user\yqd\project\apdl`)
- **命令语法**: 在执行命令时，使用 PowerShell 语法
- **备份策略**: 在执行可能影响代码的重要操作前，创建备份
- **验证步骤**: 在提交前运行相关测试验证功能正常