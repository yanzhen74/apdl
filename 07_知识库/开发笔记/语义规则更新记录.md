# 语义规则处理器更新文档

## 概述

本文档记录了对 APDL 语义规则处理器的更新，主要包括在各个规则处理器中添加 TODO 注释以及改进 DSL 解析器。

## 更新内容

### 1. 规则处理器 TODO 注释

在以下规则处理器中添加了 TODO 注释，标记需要在实际应用中实现的功能：

#### Address Resolution Rule Handler (7 个 TODO)
- `execute_address_resolution` - 地址解析逻辑
- `execute_arp_lookup` - ARP 查询逻辑
- `execute_dns_resolution` - DNS 解析逻辑
- `execute_static_mapping` - 静态地址映射表查询
- `execute_dynamic_mapping` - 动态地址映射表查询
- `execute_cache_lookup` - 地址解析缓存查询
- `execute_resolve_and_forward` - 解析地址再转发数据

#### Error Detection Rule Handler (1 个 TODO)
- `check_duplicate_frame` - 与历史帧哈希值比较

#### Flow Control Rule Handler (6 个 TODO)
- `execute_flow_control_algorithm` - 流量控制算法实现
- `execute_sliding_window_control` - 发送窗口和接收窗口管理
- `execute_rate_limiting_control` - 速率限制调节
- `execute_ack_based_control` - 确认情况调整传输行为
- `execute_buffer_management_control` - 发送和接收缓冲区管理
- `execute_congestion_control` - 网络状况调整传输参数

#### Message Filtering Rule Handler (2 个 TODO)
- `apply_content_filtering` - 检查历史消息缓存
- `extract_field_value_from_expression` - 根据协议结构进行解析

#### Multiplexing Rule Handler (1 个 TODO)
- `apply_multiplexing_condition` - 根据条件将数据路由到不同的处理路径

#### Priority Processing Rule Handler (5 个 TODO)
- `apply_priority_arbitration` - 根据优先级调整处理顺序
- `execute_high_priority_processing` - 提前处理高优先级数据
- `apply_round_robin_processing` - 根据轮次值进行循环调度
- `apply_fifo_priority_processing` - 维护队列确保先进先出
- `apply_weighted_rr_priority_processing` - 根据权重分配处理时间片

#### Redundancy Rule Handler (3 个 TODO)
- `apply_redundancy_protection` - 执行冗余处理逻辑
- `apply_failover_protection` - 管理主备切换逻辑
- `apply_load_balancing` - 执行负载均衡算法

### 2. DSL 解析器改进

#### Checksum Rules Parser
- 支持 `start:` 前缀解析（例如：`start: field1 to field2`）
- 改进 `parse_checksum_range` 函数以处理字段前缀

#### Dependency Rules Parser
- 支持 `field:` 前缀解析（例如：`field: fieldA depends_on fieldB`）
- 改进 `parse_dependency` 函数以处理字段前缀

#### Control Rules Parser
- 支持 `first:` 前缀解析（例如：`first: fieldA before fieldB`）
- 改进 `parse_order` 函数以处理字段前缀
- 改进多个控制规则解析器以处理字段前缀

#### 通用改进
- 统一处理字段名称前缀（`start:`, `field:`, `first:` 等）
- 改进描述字符串引号处理（移除两端引号）

## DSL 语法示例

### 校验和范围规则
```
rule: checksum_range(start: sync_flag to data_field) # 支持 start: 前缀
rule: checksum_range(sync_flag to data_field)        # 传统语法
```

### 依赖规则
```
rule: dependency(field: seq_count depends_on apid)   # 支持 field: 前缀
rule: dependency(seq_count depends_on apid)          # 传统语法
```

### 顺序规则
```
rule: order(first: sync_flag before data_field)      # 支持 first: 前缀
rule: order(sync_flag before data_field)             # 传统语法
```

## 实现细节

### 字段前缀处理
```rust
let field_name = if field_name.starts_with("start: ") {
    field_name[7..].trim()  // 跳过 "start: " 前缀
} else if field_name.starts_with("field: ") {
    field_name[6..].trim()  // 跳过 "field: " 前缀
} else if field_name.starts_with("first: ") {
    field_name[6..].trim()  // 跳过 "first: " 前缀
} else {
    field_name
};
```

### 描述字符串处理
```rust
description = description.trim_matches('"').to_string(); // 移除两端引号
```

## 注意事项

1. 所有 TODO 标记的实现需要考虑实际生产环境的性能和稳定性要求
2. DSL 解析器的字段前缀处理提高了语法的明确性
3. 描述字符串引号处理改进了语法的一致性
4. 保持向后兼容性，旧语法仍然有效