//! 分层拆包数据结构

use std::collections::HashMap;

/// 单层数据
#[derive(Debug, Clone)]
pub struct LayerData {
    /// 层名称
    pub layer_name: String,
    /// 层索引（0=最外层）
    pub layer_index: usize,
    /// 该层提取的字段（字段名 -> 字段值）
    pub fields: HashMap<String, Vec<u8>>,
    /// 净荷字段名（如果有）
    pub payload_field: Option<String>,
    /// 净荷数据（如果有）
    pub payload_data: Option<Vec<u8>>,
}

impl LayerData {
    /// 创建新的层数据
    pub fn new(layer_name: String, layer_index: usize) -> Self {
        Self {
            layer_name,
            layer_index,
            fields: HashMap::new(),
            payload_field: None,
            payload_data: None,
        }
    }

    /// 添加字段
    pub fn add_field(&mut self, field_name: String, value: Vec<u8>) {
        self.fields.insert(field_name, value);
    }

    /// 设置净荷数据
    pub fn set_payload(&mut self, payload_field: String, payload_data: Vec<u8>) {
        self.payload_field = Some(payload_field);
        self.payload_data = Some(payload_data);
    }

    /// 获取字段值
    pub fn get_field(&self, field_name: &str) -> Option<&Vec<u8>> {
        self.fields.get(field_name)
    }

    /// 打印层信息（调试用）
    pub fn print(&self) {
        println!("\n=== 层 {} - {} ===", self.layer_index, self.layer_name);
        println!("字段数: {}", self.fields.len());
        for (name, value) in &self.fields {
            print!("  {}: ", name);
            for byte in value {
                print!("{:02X} ", byte);
            }
            println!();
        }
        if let Some(ref payload_field) = self.payload_field {
            if let Some(ref payload) = self.payload_data {
                println!("净荷字段: {}", payload_field);
                println!("净荷大小: {} 字节", payload.len());
            }
        }
    }
}

/// 校验错误
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// 错误所在层
    pub layer_index: usize,
    /// 错误所在字段
    pub field_name: String,
    /// 错误描述
    pub error_message: String,
}

impl ValidationError {
    /// 创建新的校验错误
    pub fn new(layer_index: usize, field_name: String, error_message: String) -> Self {
        Self {
            layer_index,
            field_name,
            error_message,
        }
    }
}

/// 完整的拆包结果
#[derive(Debug, Clone)]
pub struct DisassembleResult {
    /// 各层数据（从外到内）
    pub layers: Vec<LayerData>,
    /// 最终应用数据
    pub application_data: Vec<u8>,
    /// 校验错误列表
    pub errors: Vec<ValidationError>,
}

impl DisassembleResult {
    /// 创建新的拆包结果
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            application_data: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// 添加层数据
    pub fn add_layer(&mut self, layer: LayerData) {
        self.layers.push(layer);
    }

    /// 设置应用数据
    pub fn set_application_data(&mut self, data: Vec<u8>) {
        self.application_data = data;
    }

    /// 添加错误
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// 检查是否有错误
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// 获取指定层的数据
    pub fn get_layer(&self, layer_index: usize) -> Option<&LayerData> {
        self.layers.get(layer_index)
    }

    /// 获取层数
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// 打印完整结果
    pub fn print(&self) {
        println!("\n╔══════════════════════════════════════════╗");
        println!("║        分层拆包结果                      ║");
        println!("╚══════════════════════════════════════════╝");

        // 打印各层
        for layer in &self.layers {
            layer.print();
        }

        // 打印应用数据
        if !self.application_data.is_empty() {
            println!("\n=== 应用数据 ===");
            println!("大小: {} 字节", self.application_data.len());
            print!("数据: ");
            for (i, byte) in self.application_data.iter().enumerate() {
                if i > 0 && i % 16 == 0 {
                    print!("\n      ");
                }
                print!("{:02X} ", byte);
            }
            println!();
        }

        // 打印错误
        if !self.errors.is_empty() {
            println!("\n=== 校验错误 ({}) ===", self.errors.len());
            for error in &self.errors {
                println!(
                    "  [层{}] {}: {}",
                    error.layer_index, error.field_name, error.error_message
                );
            }
        }

        println!();
    }
}

impl Default for DisassembleResult {
    fn default() -> Self {
        Self::new()
    }
}
