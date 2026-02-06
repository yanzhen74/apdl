# APDL 项目调试指南

## 1. VS Code 调试配置

### 启动调试会话
1. 打开项目根目录
2. 按 `Ctrl+Shift+D` 进入调试视图
3. 选择合适的调试配置：
   - `Debug APDL-Poem Tests`: 调试所有测试
   - `Debug Specific Test`: 调试特定测试（如 `test_mpdu_comprehensive_scenario`）
   - `Debug Connector Engine Tests`: 调试连接器引擎相关测试
   - `Debug with Release Profile`: 发布模式下调试

### 设置断点
- 在代码行号左侧点击设置断点
- 或者使用快捷键 `F9`

### 调试控制
- `F5`: 继续执行
- `F10`: 单步跳过
- `F11`: 单步进入
- `Shift+F11`: 单步退出

## 2. 命令行调试

### 构建测试
```bash
cd code/apdl-poem
cargo build --test mpdu_comprehensive_test
```

### 运行特定测试
```bash
cargo test test_mpdu_comprehensive_scenario -- --nocapture
```

### 运行调试批处理脚本
```bash
debug_mpdu_test.bat
```

## 3. 内联调试技巧

### 使用 `dbg!` 宏
在代码中添加调试宏来输出变量值：
```rust
let result = some_function();
dbg!(&result);
```

### 使用 `println!` 宏
```rust
println!("Variable value: {:?}", variable);
```

## 4. 日志调试

项目支持 RUST_LOG 环境变量进行日志输出：
```bash
RUST_LOG=debug cargo test test_mpdu_comprehensive_scenario -- --nocapture
```

## 5. 常见调试场景

### 调试 MPDU 构建逻辑
1. 在 `connector_engine.rs` 的 `build_mpdu_packet` 函数中设置断点
2. 观察指针计算和数据填充逻辑

### 调试连接器逻辑
1. 在 `connect` 函数中设置断点
2. 检查字段映射和数据放置过程

### 调试轮询调度
1. 在 `build_packet` 函数中设置断点
2. 观察轮询索引的变化和队列选择逻辑

## 6. 调试配置文件说明

- `.vscode/launch.json`: VS Code 调试配置
- `.vscode/tasks.json`: VS Code 任务配置
- `debug_mpdu_test.bat`: Windows 批处理调试脚本