//! 协议栈解析器模块
//!
//! 处理协议栈定义的解析

use apdl_core::{ParallelPackageGroup, ProtocolStackDefinition};

/// 协议栈解析器
pub struct ProtocolStackParser;

impl ProtocolStackParser {
    /// 解析协议栈定义
    pub fn parse_protocol_stack_definition(
        dsl_text: &str,
    ) -> Result<ProtocolStackDefinition, String> {
        let dsl_text = dsl_text.trim();

        // 提取协议栈名
        let stack_def_start = "protocol_stack ";
        if !dsl_text.starts_with(stack_def_start) {
            return Err("Not a protocol stack definition".to_string());
        }

        let after_stack = &dsl_text[stack_def_start.len()..].trim_start();

        // 查找协议栈名结束位置（空格或左花括号）
        let mut stack_name_end = 0;
        for (i, c) in after_stack.chars().enumerate() {
            if c.is_whitespace() || c == '{' {
                stack_name_end = i;
                break;
            }
        }

        let stack_name = after_stack[..stack_name_end].trim();
        let remaining = &after_stack[stack_name_end..].trim_start();

        // 确保以左花括号开始
        if !remaining.starts_with('{') {
            return Err("Protocol stack definition must start with {".to_string());
        }

        // 找到匹配的右花括号
        let content = Self::extract_braced_content(remaining)?;

        // 解析协议栈定义内容
        let mut packages = Vec::new();
        let mut connectors = Vec::new();
        let mut parallel_groups = Vec::new();
        let mut description = String::new();

        // 使用模式匹配来查找各部分，而不是简单的逐行解析
        // 1. 查找 packages
        if let Some(start_pos) = content.find("packages:") {
            let from_packages = &content[start_pos..];
            if let Some(array_start) = from_packages.find('[') {
                let from_array_start = &from_packages[array_start..];
                let array_content = Self::extract_bracket_content_from_array(from_array_start)?;
                packages = Self::parse_string_array(array_content)?;
            }
        }

        // 2. 查找 connectors
        if let Some(start_pos) = content.find("connectors:") {
            let from_connectors = &content[start_pos..];
            if let Some(array_start) = from_connectors.find('[') {
                let from_array_start = &from_connectors[array_start..];
                let array_content = Self::extract_bracket_content_from_array(from_array_start)?;
                connectors = Self::parse_string_array(array_content)?;
            }
        }

        // 3. 查找 parallel_groups - 这是最复杂的一个，因为它包含对象
        if let Some(start_pos) = content.find("parallel_groups:") {
            let from_parallel_groups = &content[start_pos..];
            if let Some(array_start) = from_parallel_groups.find('[') {
                let from_array_start = &from_parallel_groups[array_start..];
                let array_content = Self::extract_bracket_content_from_array(from_array_start)?;
                parallel_groups = Self::parse_parallel_groups(array_content)?;
            }
        }

        // 4. 查找描述
        if let Some(start_pos) = content.find("desc:") {
            let from_desc = &content[start_pos..];
            if let Some(quote_start) = from_desc.find('"') {
                let after_first_quote = &from_desc[quote_start + 1..];
                if let Some(quote_end) = after_first_quote.find('"') {
                    description = after_first_quote[..quote_end].to_string();
                }
            }
        }

        Ok(ProtocolStackDefinition {
            name: stack_name.to_string(),
            packages,
            connectors,
            parallel_groups,
            description,
        })
    }

    /// 解析字符串数组
    fn parse_string_array(array_content: &str) -> Result<Vec<String>, String> {
        // array_content 是完整的数组字符串，可能包含或不包含方括号
        let trimmed_content = array_content.trim();

        // 检查是否包含方括号，如果包含则去掉
        let inner_content = if trimmed_content.starts_with('[') && trimmed_content.ends_with(']') {
            // 去除首尾的方括号
            &trimmed_content[1..trimmed_content.len() - 1].trim()
        } else {
            // 如果没有方括号，则直接使用原内容
            trimmed_content
        };

        // 按逗号分割并去除引号
        let mut result = Vec::new();
        if inner_content.is_empty() {
            // 如果内容为空，返回空数组
            return Ok(result);
        }

        for item in inner_content.split(',') {
            let item = item.trim();
            if item.starts_with('"') && item.ends_with('"') {
                let value = item[1..item.len() - 1].trim();
                result.push(value.to_string());
            }
        }

        Ok(result)
    }

    /// 解析并列包组列表
    fn parse_parallel_groups(groups_content: &str) -> Result<Vec<ParallelPackageGroup>, String> {
        let mut groups = Vec::new();

        // groups_content 已经是提取出的数组内容（不包含方括号）
        let groups_array_content = groups_content;

        // 按并列包组分割
        let group_defs = Self::split_group_definitions(groups_array_content);

        for group_def in group_defs {
            let group = Self::parse_single_group(group_def)?;
            groups.push(group);
        }

        Ok(groups)
    }

    /// 解析单个并列包组
    fn parse_single_group(group_content: &str) -> Result<ParallelPackageGroup, String> {
        let mut name = String::new();
        let mut packages = Vec::new();
        let mut algorithm = String::new();
        let mut priority = 0;

        for line in group_content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // 移除行末的分号后再进行匹配
            let line_no_semicolon = line.trim_end().trim_end_matches(';');

            if line_no_semicolon.starts_with("name:") {
                name = Self::extract_quoted_value(line)?;
            } else if line_no_semicolon.starts_with("packages:") {
                let packages_content = Self::extract_array_content(line)?;
                packages = Self::parse_string_array_in_content(&packages_content)?;
            } else if line_no_semicolon.starts_with("algorithm:") {
                algorithm = Self::extract_quoted_value(line)?;
            } else if line_no_semicolon.starts_with("priority:") {
                priority = Self::extract_number_value(line)?;
            }
        }

        Ok(ParallelPackageGroup {
            name: if name.is_empty() {
                "default_group".to_string()
            } else {
                name
            },
            packages,
            algorithm: if algorithm.is_empty() {
                "default".to_string()
            } else {
                algorithm
            },
            priority,
        })
    }

    /// 解析内容中的字符串数组
    fn parse_string_array_in_content(content: &str) -> Result<Vec<String>, String> {
        // content 已经是提取出的数组内容（不包含方括号）
        let array_content = content;

        let mut result = Vec::new();
        for item in array_content.split(',') {
            let item = item.trim();
            if item.starts_with('"') && item.ends_with('"') {
                let value = item[1..item.len() - 1].trim();
                result.push(value.to_string());
            }
        }

        Ok(result)
    }

    /// 提取数字值
    fn extract_number_value(line: &str) -> Result<u32, String> {
        let colon_pos = line.find(':').ok_or("Missing colon in line")?;

        let value_part = line[colon_pos + 1..].trim();
        let mut num_str = value_part.trim();

        // 移除末尾的分号
        num_str = num_str.trim_end_matches(';').trim();

        num_str
            .parse::<u32>()
            .map_err(|_| format!("Failed to parse number: {}", num_str))
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

    /// 从数组格式中提取方括号内容
    fn extract_bracket_content_from_array(array_str: &str) -> Result<&str, String> {
        // 跳过开始的 "["
        let start_pos = if array_str.trim_start().starts_with('[') {
            array_str.find('[').unwrap() + 1
        } else {
            return Err("Array must start with [".to_string());
        };

        let content = &array_str[start_pos..].trim_start();

        // 查找对应的右方括号，注意处理引号内的方括号
        let mut bracket_count = 1; // 开始时有一个左方括号
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
                '[' if !in_string => {
                    bracket_count += 1;
                }
                ']' if !in_string => {
                    bracket_count -= 1;
                    if bracket_count == 0 {
                        // 找到匹配的右方括号
                        end_pos = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        if bracket_count != 0 {
            return Err("Unmatched brackets in array".to_string());
        }

        Ok(&content[..end_pos])
    }

    /// 分割组定义
    fn split_group_definitions(content: &str) -> Vec<&str> {
        let mut defs = Vec::new();
        let mut brace_count = 0;
        let mut in_string = false;
        let mut start = 0;

        for (i, c) in content.char_indices() {
            match c {
                '"' => {
                    in_string = !in_string;
                }
                '{' if !in_string => {
                    if brace_count == 0 {
                        start = i; // 记录开始位置
                    }
                    brace_count += 1;
                }
                '}' if !in_string => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        defs.push(&content[start..i + 1]);
                    }
                }
                _ => {}
            }
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

    /// 提取数组内容
    fn extract_array_content(line: &str) -> Result<String, String> {
        let colon_pos = line.find(':').ok_or("Missing colon in line")?;

        let mut value_part = line[colon_pos + 1..].trim();

        // 先移除分号及之后的内容
        if let Some(pos) = value_part.find(';') {
            value_part = &value_part[..pos];
        }
        value_part = value_part.trim();

        Ok(value_part.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_protocol_stack() {
        let dsl = r#"
        protocol_stack test_stack {
            packages: ["telemetry_packet", "command_packet"];
            connectors: ["telemetry_to_encapsulating"];
            parallel_groups: [
                {
                    name: "tm_tc_mux";
                    packages: ["telemetry_packet", "command_packet"];
                    algorithm: "time_division";
                    priority: 5;
                }
            ];
            desc: "Test protocol stack";
        }
        "#;

        let result = ProtocolStackParser::parse_protocol_stack_definition(dsl);
        if result.is_err() {
            println!("Error parsing protocol stack: {:?}", result.as_ref().err());
        }
        assert!(
            result.is_ok(),
            "Failed to parse protocol stack: {:?}",
            result.as_ref().err()
        );

        let stack = result.unwrap();
        assert_eq!(stack.name, "test_stack");
        assert_eq!(stack.packages, vec!["telemetry_packet", "command_packet"]);
        assert_eq!(stack.connectors, vec!["telemetry_to_encapsulating"]);
        assert_eq!(stack.description, "Test protocol stack");
        assert_eq!(stack.parallel_groups.len(), 1);
        assert_eq!(stack.parallel_groups[0].name, "tm_tc_mux");
        assert_eq!(stack.parallel_groups[0].algorithm, "time_division");
        assert_eq!(stack.parallel_groups[0].priority, 5);
    }
}
