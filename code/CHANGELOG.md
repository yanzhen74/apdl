# 变更日志

## [未发布版本] - 2026-01-28

### 重大更新
- **语义规则处理器系统**: 新增 23 种语义规则处理器，包括校验和、依赖关系、控制规则、流控、冗余、过滤、多路复用等
- **TODO标记系统**: 在所有规则处理器中添加了 TODO 注释，标记需要在实际应用中实现的功能点
- **DSL解析器增强**: 改进 DSL 解析器以支持字段前缀语法（start:, field:, first:）
- **描述字符串处理**: 改进描述字符串引号处理机制

### 新增功能
- **Address Resolution Rule Handler**: 地址解析规则处理器 (7 个 TODO)
- **Error Detection Rule Handler**: 错误检测规则处理器 (1 个 TODO)
- **Flow Control Rule Handler**: 流量控制规则处理器 (6 个 TODO)
- **Message Filtering Rule Handler**: 消息过滤规则处理器 (2 个 TODO)
- **Multiplexing Rule Handler**: 多路复用规则处理器 (1 个 TODO)
- **Priority Processing Rule Handler**: 优先级处理规则处理器 (5 个 TODO)
- **Redundancy Rule Handler**: 冗余规则处理器 (3 个 TODO)

### DSL 语法增强
- 支持 `start:` 前缀解析（例如：`start: field1 to field2`）
- 支持 `field:` 前缀解析（例如：`field: fieldA depends_on fieldB`）
- 支持 `first:` 前缀解析（例如：`first: fieldA before fieldB`）
- 保持向后兼容性，旧语法仍然有效

### 修复
- 修复了 ChecksumRange、Dependency 和 Order 规则的 DSL 解析问题
- 修复了描述字符串引号处理问题
- 解决了多个编译警告问题

### 重构
- 优化了语义规则解析器的参数类型
- 改进了未使用变量的处理方式
- 整理了 Frame Assembler 模块结构

### 文档
- 添加 SEMANTIC_RULES_UPDATE.md 技术文档
- 更新 README.md 以反映新功能