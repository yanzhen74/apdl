//! 协议规范导出器
//!
//! 提供多种格式的协议规范导出功能

use std::collections::HashMap;

/// 导出格式枚举
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Markdown,
    Json,
}

/// 协议规范导出器
#[derive(Default)]
pub struct ProtocolExporter {
    _formats: HashMap<String, Box<dyn ExportFormatHandler>>,
}

impl ProtocolExporter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn export(
        &self,
        format: &ExportFormat,
        content: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match format {
            ExportFormat::Markdown => Ok(MarkdownExporter.export(content)),
            ExportFormat::Json => Ok(JsonExporter.export(content)),
        }
    }
}

/// 导出格式处理器trait
pub trait ExportFormatHandler {
    fn export(&self, content: &str) -> String;
}

/// Markdown导出器
pub struct MarkdownExporter;

impl ExportFormatHandler for MarkdownExporter {
    fn export(&self, content: &str) -> String {
        format!("# Protocol Specification\n\n{content}")
    }
}

/// JSON导出器
pub struct JsonExporter;

impl ExportFormatHandler for JsonExporter {
    fn export(&self, content: &str) -> String {
        format!(
            "{{\"specification\": \"{}\"}}",
            content.replace("\"", "\\\"")
        )
    }
}
