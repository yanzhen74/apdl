//! 包解析器模块
//!
//! 处理包定义的解析

use apdl_core::{LayerDefinition, PackageDefinition};

/// 包解析器
pub struct PackageParser;

impl PackageParser {
    /// 解析包定义
    pub fn parse_package_definition(dsl_text: &str) -> Result<PackageDefinition, String> {
        // 解析包定义的 DSL 文本
        // 格式: package <package_name> { ... }

        let dsl_text = dsl_text.trim();

        // 提取包名
        let package_def_start = "package ";
        if !dsl_text.starts_with(package_def_start) {
            return Err("Not a package definition".to_string());
        }

        let after_package = &dsl_text[package_def_start.len()..].trim_start();

        // 查找包名结束位置（空格或左花括号）
        let mut package_name_end = 0;
        for (i, c) in after_package.chars().enumerate() {
            if c.is_whitespace() || c == '{' {
                package_name_end = i;
                break;
            }
        }

        let package_name = after_package[..package_name_end].trim();
        let remaining = &after_package[package_name_end..].trim_start();

        // 确保以左花括号开始
        if !remaining.starts_with('{') {
            return Err("Package definition must start with {".to_string());
        }

        // 找到匹配的右花括号
        let content = Self::extract_braced_content(remaining)?;

        // 解析包定义内容
        let mut display_name = String::new();
        let mut package_type = String::new();
        let mut description = String::new();
        let mut layers_str = String::new();

        // 解析各个字段
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if line.starts_with("name:") {
                display_name = Self::extract_quoted_value(line)?;
            } else if line.starts_with("type:") {
                package_type = Self::extract_simple_value(line)?;
            } else if line.starts_with("desc:") {
                description = Self::extract_quoted_value(line)?;
            } else if line.starts_with("layers:") {
                // 提取整个layers数组内容
                layers_str = Self::extract_array_content(line)?;
            }
        }

        // 解析层定义
        let layers = if !layers_str.is_empty() {
            Self::parse_layers(&layers_str)?
        } else {
            Vec::new()
        };

        // 设置默认值
        let display_name = if display_name.is_empty() {
            package_name.to_string()
        } else {
            display_name
        };
        let package_type = if package_type.is_empty() {
            "generic".to_string()
        } else {
            package_type
        };

        Ok(PackageDefinition {
            name: package_name.to_string(),
            display_name,
            package_type,
            layers,
            description,
        })
    }

    /// 解析层定义列表
    fn parse_layers(layers_content: &str) -> Result<Vec<LayerDefinition>, String> {
        let mut layers = Vec::new();

        // 提取数组内容
        let layers_array_content = Self::extract_braced_content_from_array(layers_content)?;

        // 按层分割（寻找 }, 匹配来分隔不同的层）
        let layer_defs = Self::split_layer_definitions(layers_array_content);

        for layer_def in layer_defs {
            let layer = Self::parse_single_layer(layer_def)?;
            layers.push(layer);
        }

        Ok(layers)
    }

    /// 解析单个层定义
    fn parse_single_layer(layer_content: &str) -> Result<LayerDefinition, String> {
        let mut name = String::new();
        let mut units_str = String::new();
        let mut rules_str = String::new();

        // 解析层内容
        for line in layer_content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if line.starts_with("name:") {
                name = Self::extract_quoted_value(line)?;
            } else if line.starts_with("units:") {
                units_str = Self::extract_array_content(line)?;
            } else if line.starts_with("rules:") {
                rules_str = Self::extract_array_content(line)?;
            }
        }

        // 解析语法单元
        let units = if !units_str.is_empty() {
            // 这里需要调用现有的语法单元解析器来解析units数组
            // 为简化，暂时返回空向量，实际实现需要更复杂的解析
            Vec::new()
        } else {
            Vec::new()
        };

        // 解析语义规则
        let rules = if !rules_str.is_empty() {
            // 这里需要调用现有的语义规则解析器来解析rules数组
            // 为简化，暂时返回空向量，实际实现需要更复杂的解析
            Vec::new()
        } else {
            Vec::new()
        };

        Ok(LayerDefinition {
            name: if name.is_empty() {
                "default_layer".to_string()
            } else {
                name
            },
            units,
            rules,
        })
    }

    /// 提取花括号内容
    fn extract_braced_content(text: &str) -> Result<&str, String> {
        if !text.starts_with('{') {
            return Err("Content must start with {".to_string());
        }

        let mut brace_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut end_pos = 0;

        for (i, c) in text.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match c {
                '\\' if in_string => {
                    escape_next = true;
                }
                '"' => {
                    in_string = !in_string;
                }
                '{' if !in_string => {
                    brace_count += 1;
                }
                '}' if !in_string => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        end_pos = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if brace_count != 0 {
            return Err("Unmatched braces".to_string());
        }

        Ok(&text[1..end_pos - 1])
    }

    /// 从数组格式中提取花括号内容
    fn extract_braced_content_from_array(array_str: &str) -> Result<&str, String> {
        // 跳过开始的 "[\n" 或 "["
        let start_pos = if array_str.trim_start().starts_with('[') {
            array_str.find('[').unwrap() + 1
        } else {
            0
        };

        let content = &array_str[start_pos..].trim_start();

        let mut brace_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut end_pos = 0;

        for (i, c) in content.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match c {
                '\\' if in_string => {
                    escape_next = true;
                }
                '"' => {
                    in_string = !in_string;
                }
                '{' if !in_string => {
                    if brace_count == 0 {
                        // 找到第一个左花括号，从这里开始
                        continue; // 不包含第一个花括号
                    }
                    brace_count += 1;
                }
                '}' if !in_string => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        // 找到匹配的右花括号
                        end_pos = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        if brace_count != 0 {
            return Err("Unmatched braces in array".to_string());
        }

        Ok(&content[..end_pos])
    }

    /// 分割层定义
    fn split_layer_definitions(content: &str) -> Vec<&str> {
        let mut defs = Vec::new();
        let mut brace_count = 0;
        let mut in_string = false;
        let mut start = 0;
        let mut _last_comma = 0;

        for (i, c) in content.char_indices() {
            match c {
                '"' => {
                    in_string = !in_string;
                }
                '{' if !in_string => {
                    brace_count += 1;
                }
                '}' if !in_string => {
                    brace_count -= 1;
                }
                ',' if !in_string && brace_count == 0 => {
                    defs.push(content[start..i].trim());
                    start = i + 1;
                    _last_comma = i;
                }
                _ => {}
            }
        }

        // 添加最后一个定义
        if start < content.len() {
            defs.push(content[start..].trim());
        }

        defs
    }

    /// 提取引号值
    fn extract_quoted_value(line: &str) -> Result<String, String> {
        let colon_pos = line.find(':').ok_or("Missing colon in line")?;

        let value_part = line[colon_pos + 1..].trim();

        // 查找引号包围的值
        if let Some(start) = value_part.find('"') {
            if let Some(end) = value_part[start + 1..].find('"') {
                let quoted_content = &value_part[start + 1..start + 1 + end];
                // 检查引号后面是否有分号或其他内容
                let final_value = if start + 1 + end + 1 < value_part.len() {
                    // 引号后还有内容，可能是分号
                    let after_quote = &value_part[start + 1 + end + 1..];
                    if after_quote.find(';').is_some() {
                        // 在分号处分割
                        &quoted_content
                    } else {
                        // 没有分号，直接返回引号内容
                        quoted_content
                    }
                } else {
                    // 没有更多内容，直接返回引号内容
                    quoted_content
                };
                Ok(final_value.trim().to_string())
            } else {
                Err("Unmatched quote".to_string())
            }
        } else {
            Err("No quoted value found".to_string())
        }
    }

    /// 提取简单值（可能包含引号）
    fn extract_simple_value(line: &str) -> Result<String, String> {
        let colon_pos = line.find(':').ok_or("Missing colon in line")?;

        let mut value_part = line[colon_pos + 1..].trim();

        // 先移除分号及之后的内容
        if let Some(pos) = value_part.find(';') {
            value_part = &value_part[..pos];
        }
        value_part = value_part.trim();

        // 检查是否是带引号的值
        if value_part.starts_with('"') && value_part.ends_with('"') && value_part.len() > 1 {
            // 去掉首尾引号
            Ok(value_part[1..value_part.len() - 1].to_string())
        } else {
            Ok(value_part.to_string())
        }
    }

    /// 提取数组内容
    fn extract_array_content(line: &str) -> Result<String, String> {
        let colon_pos = line.find(':').ok_or("Missing colon in line")?;

        let value_part = line[colon_pos + 1..].trim();
        Ok(value_part.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_package() {
        let dsl = r#"
        package test_package {
            name: "Test Package";
            type: "telemetry";
            desc: "A test package definition";
        }
        "#;

        let result = PackageParser::parse_package_definition(dsl);
        assert!(result.is_ok());

        let pkg = result.unwrap();
        assert_eq!(pkg.name, "test_package");
        assert_eq!(pkg.display_name, "Test Package");
        assert_eq!(pkg.package_type, "telemetry");
        assert_eq!(pkg.description, "A test package definition");
    }
}
