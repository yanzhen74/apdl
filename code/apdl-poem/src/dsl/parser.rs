//! DSL解析器实现
//!
//! 使用简单的字符串处理实现APDL DSL的解析，支持字段定义和协议结构描述

use apdl_core::{CoverDesc, LengthDesc, SemanticRule, SyntaxUnit, UnitType};
use serde_json;

// 导入其他模块的函数
use crate::dsl::field_mapping_parser::FieldMappingParser;
use crate::dsl::layers::{
    connector_parser::ConnectorParser, package_parser::PackageParser,
    protocol_stack_parser::ProtocolStackParser,
};
use crate::dsl::parser_utils::*;
use crate::dsl::semantic_rule_parsers::SemanticRuleParsers;

/// DSL解析器实现
pub struct DslParserImpl;

impl Default for DslParserImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl DslParserImpl {
    pub fn new() -> Self {
        Self
    }

    /// 解析语法单元定义
    pub fn parse_syntax_unit(&self, input: &str) -> Result<SyntaxUnit, String> {
        Self::parse_syntax_unit_internal(input)
    }

    /// 解析多个语法单元定义（协议结构）
    pub fn parse_protocol_structure(&self, input: &str) -> Result<Vec<SyntaxUnit>, String> {
        let mut units = Vec::new();

        // 按行分割输入，过滤掉注释和空行，逐行解析
        for line in input.lines() {
            let trimmed_line = line.trim();
            // 跳过注释行（以//开头）和空行
            if !trimmed_line.is_empty()
                && !trimmed_line.starts_with("//")
                && !trimmed_line.starts_with("rule:")
            {
                match Self::parse_syntax_unit_internal(trimmed_line) {
                    Ok(unit) => {
                        units.push(unit);
                    }
                    Err(e) => return Err(format!("Parse error on line '{trimmed_line}': {e}")),
                }
            }
        }

        Ok(units)
    }

    /// 解析协议语义规则
    pub fn parse_semantic_rules(&self, input: &str) -> Result<Vec<SemanticRule>, String> {
        let mut rules = Vec::new();

        for line in input.lines() {
            let trimmed_line = line.trim();
            if !trimmed_line.is_empty()
                && !trimmed_line.starts_with("//")
                && trimmed_line.starts_with("rule:")
            {
                match Self::parse_semantic_rule_internal(trimmed_line) {
                    Ok(rule) => {
                        rules.push(rule);
                    }
                    Err(e) => {
                        return Err(format!(
                            "Semantic rule parse error on line '{trimmed_line}': {e}"
                        ))
                    }
                }
            }
        }

        Ok(rules)
    }

    /// 解析包定义
    /// 解析包定义（优先使用JSON格式）
    ///
    /// 如果输入是有效的JSON格式，则使用JSON解析器
    /// 否则回退到DSL解析器（向后兼容）
    pub fn parse_package_definitions(
        &self,
        input: &str,
    ) -> Result<Vec<apdl_core::PackageDefinition>, String> {
        // 首先检查输入是否为JSON格式
        if input.trim_start().starts_with('{') {
            // 尝试将整个输入作为JSON解析
            match serde_json::from_str::<apdl_core::PackageDefinition>(input.trim()) {
                Ok(pkg_def) => return Ok(vec![pkg_def]),
                Err(_) => {
                    // 如果单个包解析失败，尝试解析为包数组
                    match serde_json::from_str::<Vec<apdl_core::PackageDefinition>>(input.trim()) {
                        Ok(pkg_defs) => return Ok(pkg_defs),
                        Err(_) => {
                            // JSON解析失败，回退到DSL解析
                        }
                    }
                }
            }
        }

        // 如果不是JSON格式或JSON解析失败，使用传统的DSL解析
        let mut packages = Vec::new();

        // 查找包定义
        let mut lines = input.lines().peekable();

        while let Some(line) = lines.next() {
            let trimmed_line = line.trim();
            if !trimmed_line.is_empty() && !trimmed_line.starts_with("//") {
                if trimmed_line.starts_with("package ") {
                    // 找到包定义的开始，收集直到找到匹配的右花括号
                    let mut package_def = String::from(trimmed_line);
                    let mut brace_count = 0;

                    // 计算当前行的左花括号数量
                    for c in trimmed_line.chars() {
                        if c == '{' {
                            brace_count += 1;
                        } else if c == '}' {
                            brace_count -= 1;
                        }
                    }

                    // 继续收集行直到括号平衡
                    while brace_count > 0 {
                        if let Some(next_line) = lines.next() {
                            let next_trimmed = next_line.trim();
                            package_def.push_str(" ");
                            package_def.push_str(next_trimmed);

                            for c in next_trimmed.chars() {
                                if c == '{' {
                                    brace_count += 1;
                                } else if c == '}' {
                                    brace_count -= 1;
                                }
                            }
                        } else {
                            return Err("Unmatched braces in package definition".to_string());
                        }
                    }

                    // 解析包定义
                    match PackageParser::parse_package_definition(&package_def) {
                        Ok(pkg) => packages.push(pkg),
                        Err(e) => return Err(format!("Package parse error: {e}")),
                    }
                }
            }
        }

        Ok(packages)
    }

    /// 解析连接器定义（优先使用JSON格式）
    ///
    /// 如果输入是有效的JSON格式，则使用JSON解析器
    /// 否则回退到DSL解析器（向后兼容）
    pub fn parse_connector_definitions(
        &self,
        input: &str,
    ) -> Result<Vec<apdl_core::ConnectorDefinition>, String> {
        // 首先检查输入是否为JSON格式
        if input.trim_start().starts_with('{') {
            // 尝试将整个输入作为JSON解析
            match serde_json::from_str::<apdl_core::ConnectorDefinition>(input.trim()) {
                Ok(conn_def) => return Ok(vec![conn_def]),
                Err(_) => {
                    // 如果单个连接器解析失败，尝试解析为连接器数组
                    match serde_json::from_str::<Vec<apdl_core::ConnectorDefinition>>(input.trim())
                    {
                        Ok(conn_defs) => return Ok(conn_defs),
                        Err(_) => {
                            // JSON解析失败，回退到DSL解析
                        }
                    }
                }
            }
        }

        // 如果不是JSON格式或JSON解析失败，使用传统的DSL解析
        let mut connectors = Vec::new();

        // 查找连接器定义
        let mut lines = input.lines().peekable();

        while let Some(line) = lines.next() {
            let trimmed_line = line.trim();
            if !trimmed_line.is_empty() && !trimmed_line.starts_with("//") {
                if trimmed_line.starts_with("connector ") {
                    // 找到连接器定义的开始，收集直到找到匹配的右花括号
                    let mut connector_def = String::from(trimmed_line);
                    let mut brace_count = 0;

                    // 计算当前行的左花括号数量
                    for c in trimmed_line.chars() {
                        if c == '{' {
                            brace_count += 1;
                        } else if c == '}' {
                            brace_count -= 1;
                        }
                    }

                    // 继续收集行直到括号平衡
                    while brace_count > 0 {
                        if let Some(next_line) = lines.next() {
                            let next_trimmed = next_line.trim();
                            connector_def.push_str(" ");
                            connector_def.push_str(next_trimmed);

                            for c in next_trimmed.chars() {
                                if c == '{' {
                                    brace_count += 1;
                                } else if c == '}' {
                                    brace_count -= 1;
                                }
                            }
                        } else {
                            return Err("Unmatched braces in connector definition".to_string());
                        }
                    }

                    // 解析连接器定义
                    match ConnectorParser::parse_connector_definition(&connector_def) {
                        Ok(conn) => connectors.push(conn),
                        Err(e) => return Err(format!("Connector parse error: {e}")),
                    }
                }
            }
        }

        Ok(connectors)
    }

    /// 解析协议栈定义（优先使用JSON格式）
    ///
    /// 如果输入是有效的JSON格式，则使用JSON解析器
    /// 否则回退到DSL解析器（向后兼容）
    pub fn parse_protocol_stack_definitions(
        &self,
        input: &str,
    ) -> Result<Vec<apdl_core::ProtocolStackDefinition>, String> {
        // 首先检查输入是否为JSON格式
        if input.trim_start().starts_with('{') {
            // 尝试将整个输入作为JSON解析
            match serde_json::from_str::<apdl_core::ProtocolStackDefinition>(input.trim()) {
                Ok(stack_def) => return Ok(vec![stack_def]),
                Err(_) => {
                    // 如果单个协议栈解析失败，尝试解析为协议栈数组
                    match serde_json::from_str::<Vec<apdl_core::ProtocolStackDefinition>>(
                        input.trim(),
                    ) {
                        Ok(stack_defs) => return Ok(stack_defs),
                        Err(_) => {
                            // JSON解析失败，回退到DSL解析
                        }
                    }
                }
            }
        }

        // 如果不是JSON格式或JSON解析失败，使用传统的DSL解析
        let mut stacks = Vec::new();

        // 查找协议栈定义
        let mut lines = input.lines().peekable();

        while let Some(line) = lines.next() {
            let trimmed_line = line.trim();
            if !trimmed_line.is_empty() && !trimmed_line.starts_with("//") {
                if trimmed_line.starts_with("protocol_stack ") {
                    // 找到协议栈定义的开始，收集直到找到匹配的右花括号
                    let mut stack_def = String::from(trimmed_line);
                    let mut brace_count = 0;

                    // 计算当前行的左花括号数量
                    for c in trimmed_line.chars() {
                        if c == '{' {
                            brace_count += 1;
                        } else if c == '}' {
                            brace_count -= 1;
                        }
                    }

                    // 继续收集行直到括号平衡
                    while brace_count > 0 {
                        if let Some(next_line) = lines.next() {
                            let next_trimmed = next_line.trim();
                            stack_def.push_str(" ");
                            stack_def.push_str(next_trimmed);

                            for c in next_trimmed.chars() {
                                if c == '{' {
                                    brace_count += 1;
                                } else if c == '}' {
                                    brace_count -= 1;
                                }
                            }
                        } else {
                            return Err("Unmatched braces in protocol stack definition".to_string());
                        }
                    }

                    // 解析协议栈定义
                    match ProtocolStackParser::parse_protocol_stack_definition(&stack_def) {
                        Ok(stack) => stacks.push(stack),
                        Err(e) => return Err(format!("Protocol stack parse error: {e}")),
                    }
                }
            }
        }

        Ok(stacks)
    }

    fn parse_syntax_unit_internal(input: &str) -> Result<SyntaxUnit, String> {
        let input = input.trim();

        // 解析field
        let (input, field_id) = Self::extract_field(input)?;

        // 解析type
        let (input, unit_type) = Self::extract_type(input)?;

        // 解析length
        let (input, length) = Self::extract_length(input)?;

        // 解析scope
        let (input, scope) = Self::extract_scope(input)?;

        // 解析cover
        let (input, cover) = Self::extract_cover(input)?;

        // 解析可选部分
        let mut constraint = None;
        let mut alg = None;
        let mut associate = Vec::new();
        let mut desc = String::new();

        let remaining = input;
        for part in remaining.split(';') {
            let part = part.trim();
            if part.starts_with("constraint:") {
                constraint = Some(parse_constraint(part[11..].trim())?);
            } else if part.starts_with("alg:") {
                alg = Some(parse_algorithm(part[4..].trim())?);
            } else if part.starts_with("associate:") {
                associate = part[10..]
                    .trim()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
            } else if part.starts_with("desc:") {
                desc = part[5..].trim().trim_matches('"').to_string();
            }
        }

        Ok(SyntaxUnit {
            field_id,
            unit_type,
            length,
            scope,
            cover,
            constraint,
            alg,
            associate,
            desc,
        })
    }

    fn extract_field(input: &str) -> Result<(&str, String), String> {
        if let Some(start) = input.find("field:") {
            let start_pos = start + 6;
            if let Some(end) = input[start_pos..].find(';').map(|i| start_pos + i) {
                let field = input[start_pos..end].trim();
                Ok((&input[end + 1..], field.to_string()))
            } else {
                Ok(("", input[start_pos..].trim().to_string()))
            }
        } else {
            Err("field: not found".to_string())
        }
    }

    fn extract_type(input: &str) -> Result<(&str, UnitType), String> {
        if let Some(start) = input.find("type:") {
            let start_pos = start + 5;
            if let Some(end) = input[start_pos..].find(';').map(|i| start_pos + i) {
                let type_str = input[start_pos..end].trim();
                let unit_type = parse_unit_type(type_str)?;
                Ok((&input[end + 1..], unit_type))
            } else {
                let type_str = input[start_pos..].trim();
                let unit_type = parse_unit_type(type_str)?;
                Ok(("", unit_type))
            }
        } else {
            Err("type: not found".to_string())
        }
    }

    fn extract_length(input: &str) -> Result<(&str, LengthDesc), String> {
        if let Some(start) = input.find("length:") {
            let start_pos = start + 7;
            if let Some(end) = input[start_pos..].find(';').map(|i| start_pos + i) {
                let length_str = input[start_pos..end].trim();
                let length_desc = parse_length_desc(length_str)?;
                Ok((&input[end + 1..], length_desc))
            } else {
                let length_str = input[start_pos..].trim();
                let length_desc = parse_length_desc(length_str)?;
                Ok(("", length_desc))
            }
        } else {
            Err("length: not found".to_string())
        }
    }

    fn extract_scope(input: &str) -> Result<(&str, apdl_core::ScopeDesc), String> {
        if let Some(start) = input.find("scope:") {
            let start_pos = start + 6;
            if let Some(end) = input[start_pos..].find(';').map(|i| start_pos + i) {
                let scope_str = input[start_pos..end].trim();
                let scope_desc = parse_scope_desc(scope_str)?;
                Ok((&input[end + 1..], scope_desc))
            } else {
                let scope_str = input[start_pos..].trim();
                let scope_desc = parse_scope_desc(scope_str)?;
                Ok(("", scope_desc))
            }
        } else {
            Err("scope: not found".to_string())
        }
    }

    fn extract_cover(input: &str) -> Result<(&str, CoverDesc), String> {
        if let Some(start) = input.find("cover:") {
            let start_pos = start + 6;
            if let Some(end) = input[start_pos..].find(';').map(|i| start_pos + i) {
                let cover_str = input[start_pos..end].trim();
                let cover_desc = parse_cover_desc(cover_str)?;
                Ok((&input[end + 1..], cover_desc))
            } else {
                let cover_str = input[start_pos..].trim();
                let cover_desc = parse_cover_desc(cover_str)?;
                Ok(("", cover_desc))
            }
        } else {
            Err("cover: not found".to_string())
        }
    }

    // 解析语义规则的内部实现
    fn parse_semantic_rule_internal(input: &str) -> Result<SemanticRule, String> {
        let input = input.trim();

        // 提取 "rule:type(" 部分
        if !input.starts_with("rule:") {
            return Err("Not a rule definition".to_string());
        }

        let after_rule = &input[5..].trim_start();

        // 查找第一个'('的位置
        if let Some(paren_pos) = after_rule.find('(') {
            let rule_type = after_rule[..paren_pos].trim();
            let params_str = &after_rule[paren_pos + 1..];

            // 查找匹配的')'
            let mut paren_count = 1;
            let mut in_quote = false;
            let mut quote_char = '"';

            for (pos, c) in params_str.char_indices() {
                match c {
                    '"' | '\'' => {
                        if !in_quote {
                            in_quote = true;
                            quote_char = c;
                        } else if c == quote_char {
                            in_quote = false;
                        }
                    }
                    '(' if !in_quote => {
                        paren_count += 1;
                    }
                    ')' if !in_quote => {
                        paren_count -= 1;
                        if paren_count == 0 {
                            // 找到了匹配的右括号
                            let params = &params_str[..pos].trim();
                            return Self::create_semantic_rule(rule_type, params);
                        }
                    }
                    _ => {}
                }
            }

            Err("Unmatched parenthesis in rule".to_string())
        } else {
            Err("No parameters found for rule".to_string())
        }
    }

    fn create_semantic_rule(rule_type: &str, params: &str) -> Result<SemanticRule, String> {
        match rule_type {
            "field_mapping" => {
                // 使用新的字段映射解析器
                FieldMappingParser::parse_field_mapping_rule(params)
            }
            "crc_range" | "checksum_range" => {
                SemanticRuleParsers::parse_checksum_range(params, rule_type)
            }
            "dependency" => SemanticRuleParsers::parse_dependency(params),
            "conditional" => SemanticRuleParsers::parse_conditional(params),
            "order" => SemanticRuleParsers::parse_order(params),
            "pointer" => SemanticRuleParsers::parse_pointer(params),
            "algorithm" => SemanticRuleParsers::parse_algorithm(params),
            "length_rule" => SemanticRuleParsers::parse_length_rule(params),
            "routing_dispatch" => SemanticRuleParsers::parse_routing_dispatch(params),
            "sequence_control" => SemanticRuleParsers::parse_sequence_control(params),
            "validation" => SemanticRuleParsers::parse_validation(params),
            "multiplexing" => SemanticRuleParsers::parse_multiplexing(params),
            "priority_processing" => SemanticRuleParsers::parse_priority_processing(params),
            "synchronization" => SemanticRuleParsers::parse_synchronization(params),
            "length_validation" => SemanticRuleParsers::parse_length_validation(params),
            "state_machine" => SemanticRuleParsers::parse_state_machine(params),
            "periodic_transmission" => SemanticRuleParsers::parse_periodic_transmission(params),
            "message_filtering" => SemanticRuleParsers::parse_message_filtering(params),
            "error_detection" => SemanticRuleParsers::parse_error_detection(params),
            "nested_sync" => SemanticRuleParsers::parse_nested_sync(params),
            "sequence_reset" => SemanticRuleParsers::parse_sequence_reset(params),
            "timestamp_insertion" => SemanticRuleParsers::parse_timestamp_insertion(params),
            "flow_control" => SemanticRuleParsers::parse_flow_control(params),
            "time_synchronization" => SemanticRuleParsers::parse_time_synchronization(params),
            "address_resolution" => SemanticRuleParsers::parse_address_resolution(params),
            "security" => SemanticRuleParsers::parse_security(params),
            "redundancy" => SemanticRuleParsers::parse_redundancy(params),
            _ => Err(format!("Unknown rule type: {rule_type}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_field() {
        let parser = DslParserImpl::new();
        let dsl = r#"field: llc_sync_marker; type: Uint16; length: 2byte; scope: layer(link); cover: entire_field; constraint: fixed(0xEB90); desc: "CCSDS sync marker""#;

        let result = parser.parse_syntax_unit(dsl);
        assert!(result.is_ok());

        let unit = result.unwrap();
        assert_eq!(unit.field_id, "llc_sync_marker");
        assert_eq!(unit.desc, "CCSDS sync marker");
    }

    #[test]
    fn test_parse_complex_field() {
        let parser = DslParserImpl::new();
        let dsl = r#"field: seq_num_field; type: Bit(8); length: 1byte; scope: layer(data_link); cover: entire_field; constraint: range(0..=255); desc: "Sequence number field""#;

        let result = parser.parse_syntax_unit(dsl);
        assert!(result.is_ok());

        let unit = result.unwrap();
        assert_eq!(unit.field_id, "seq_num_field");
        assert_eq!(unit.desc, "Sequence number field");
    }

    #[test]
    fn test_parse_protocol_structure() {
        let parser = DslParserImpl::new();
        let dsl = r#"
        field: sync_flag; type: Uint16; length: 2byte; scope: layer(physical); cover: entire_field; constraint: fixed(60528); desc: "Sync flag";
        field: version; type: Uint8; length: 1byte; scope: layer(data_link); cover: entire_field; constraint: range(0..=7); desc: "Version field";
        field: data; type: RawData; length: dynamic; scope: layer(application); cover: entire_field; desc: "Data field"
        "#;

        let result = parser.parse_protocol_structure(dsl);
        assert!(result.is_ok());

        let units = result.unwrap();
        assert_eq!(units.len(), 3);
        assert_eq!(units[0].field_id, "sync_flag");
        assert_eq!(units[1].field_id, "version");
        assert_eq!(units[2].field_id, "data");
    }

    #[test]
    fn test_parse_package_definitions_with_json() {
        let json_input = r#"{
            "name": "test_package",
            "display_name": "Test Package",
            "package_type": "telemetry",
            "description": "A test package",
            "layers": []
        }"#;

        let parser = DslParserImpl;
        let result = parser.parse_package_definitions(json_input);

        assert!(result.is_ok());
        let packages = result.unwrap();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name, "test_package");
        assert_eq!(packages[0].display_name, "Test Package");
    }

    #[test]
    fn test_parse_protocol_stack_definitions_with_json() {
        let json_input = r#"{
            "name": "test_stack",
            "packages": [],
            "connectors": [],
            "parallel_groups": [],
            "description": "A test protocol stack"
        }"#;

        let parser = DslParserImpl;
        let result = parser.parse_protocol_stack_definitions(json_input);

        assert!(result.is_ok(), "JSON parsing failed: {:?}", result.err());
        let stacks = result.unwrap();
        assert_eq!(stacks.len(), 1, "Expected 1 stack, got {}", stacks.len());
        assert_eq!(stacks[0].name, "test_stack");
    }

    #[test]
    fn test_parse_connector_definitions_with_json() {
        let json_input = r#"{
            "name": "test_connector",
            "connector_type": "field_mapping",
            "source_package": "source_pkg",
            "target_package": "target_pkg",
            "config": {
                "mappings": []
            },
            "description": "A test connector"
        }"#;

        let parser = DslParserImpl;
        let result = parser.parse_connector_definitions(json_input);

        assert!(result.is_ok());
        let connectors = result.unwrap();
        assert_eq!(connectors.len(), 1);
        assert_eq!(connectors[0].name, "test_connector");
        assert_eq!(connectors[0].connector_type, "field_mapping");
    }
}
