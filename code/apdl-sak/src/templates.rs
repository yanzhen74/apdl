//! 模板引擎模块
//!
//! 实现协议规范模板的管理和渲染功能

use std::collections::HashMap;

/// 模板引擎
pub struct TemplateEngine {
    templates: HashMap<String, String>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        // 添加CCSDS标准模板
        templates.insert(
            "ccsds_standard".to_string(),
            include_str!("../templates/ccsds_standard.template").to_string(),
        );

        // 添加默认模板
        templates.insert(
            "default".to_string(),
            "# {{title}}\n\n{{content}}".to_string(),
        );

        Self { templates }
    }

    /// 渲染模板
    pub fn render(&self, template_name: &str, context: &HashMap<String, String>) -> String {
        match self.templates.get(template_name) {
            Some(template) => {
                let mut result = template.clone();
                for (key, value) in context {
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

    /// 检查模板是否存在
    pub fn has_template(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }
}

// 如果模板文件不存在，创建一个默认的
pub fn ensure_default_templates() {
    // 这里可以确保必要的模板文件存在
}
