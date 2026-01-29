//! 规范生成器模块
//!
//! 实现协议规范的生成功能

use std::collections::HashMap;

/// 规范生成器
pub struct SpecGenerator {
    templates: HashMap<String, String>,
}

impl SpecGenerator {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        // 添加默认模板
        templates.insert(
            "default".to_string(),
            "# Protocol Specification\n\nID: {{id}}\nName: {{name}}\nVersion: {{version}}\n"
                .to_string(),
        );

        Self { templates }
    }

    /// 生成协议规范
    pub fn generate(&self, template_name: &str, data: &HashMap<String, String>) -> String {
        match self.templates.get(template_name) {
            Some(template) => {
                let mut result = template.clone();
                for (key, value) in data {
                    let placeholder = format!("{{{{{}}}}}", key);
                    result = result.replace(&placeholder, value);
                }
                result
            }
            None => format!("Template '{}' not found", template_name),
        }
    }

    /// 注册新模板
    pub fn register_template(&mut self, name: String, template: String) {
        self.templates.insert(name, template);
    }
}
