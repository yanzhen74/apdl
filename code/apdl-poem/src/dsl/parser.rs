//! DSL解析器实现
//!
//! 使用简单的字符串处理实现APDL DSL的解析，支持字段定义和协议结构描述

use apdl_core::{
    AlgorithmAst, ChecksumAlgorithm, Constraint, CoverDesc, LengthDesc, LengthUnit, ProtocolError,
    SemanticRule, SyntaxUnit, UnitType,
};
use std::collections::HashMap;

/// DSL解析器实现
pub struct DslParserImpl;

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
                    Err(e) => return Err(format!("Parse error on line '{}': {}", trimmed_line, e)),
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
                            "Semantic rule parse error on line '{}': {}",
                            trimmed_line, e
                        ))
                    }
                }
            }
        }

        Ok(rules)
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
                constraint = Some(Self::parse_constraint(&part[11..].trim())?);
            } else if part.starts_with("alg:") {
                alg = Some(Self::parse_algorithm(&part[4..].trim())?);
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
                let unit_type = Self::parse_unit_type(type_str)?;
                Ok((&input[end + 1..], unit_type))
            } else {
                let type_str = input[start_pos..].trim();
                let unit_type = Self::parse_unit_type(type_str)?;
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
                let length_desc = Self::parse_length_desc(length_str)?;
                Ok((&input[end + 1..], length_desc))
            } else {
                let length_str = input[start_pos..].trim();
                let length_desc = Self::parse_length_desc(length_str)?;
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
                let scope_desc = Self::parse_scope_desc(scope_str)?;
                Ok((&input[end + 1..], scope_desc))
            } else {
                let scope_str = input[start_pos..].trim();
                let scope_desc = Self::parse_scope_desc(scope_str)?;
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
                let cover_desc = Self::parse_cover_desc(cover_str)?;
                Ok((&input[end + 1..], cover_desc))
            } else {
                let cover_str = input[start_pos..].trim();
                let cover_desc = Self::parse_cover_desc(cover_str)?;
                Ok(("", cover_desc))
            }
        } else {
            Err("cover: not found".to_string())
        }
    }

    fn parse_unit_type(type_str: &str) -> Result<UnitType, String> {
        if type_str.starts_with("Uint") {
            let num_str = &type_str[4..];
            if let Ok(bits) = num_str.parse::<u8>() {
                Ok(UnitType::Uint(bits))
            } else {
                Err(format!("Invalid Uint type: {}", type_str))
            }
        } else if type_str.starts_with("Bit(") && type_str.ends_with(')') {
            let num_str = &type_str[4..type_str.len() - 1];
            if let Ok(bits) = num_str.parse::<u8>() {
                Ok(UnitType::Bit(bits))
            } else {
                Err(format!("Invalid Bit type: {}", type_str))
            }
        } else if type_str == "RawData" {
            Ok(UnitType::RawData)
        } else if type_str == "Ip6Addr" {
            Ok(UnitType::Ip6Addr)
        } else {
            Err(format!("Unknown type: {}", type_str))
        }
    }

    fn parse_length_desc(length_str: &str) -> Result<LengthDesc, String> {
        let length_str = length_str.trim();
        if length_str.ends_with("byte") {
            let num_str = length_str[..length_str.len() - 4].trim();
            if let Ok(size) = num_str.parse::<usize>() {
                Ok(LengthDesc {
                    size,
                    unit: LengthUnit::Byte,
                })
            } else {
                Err(format!("Invalid byte length: {}", length_str))
            }
        } else if length_str.ends_with("bit") {
            let num_str = length_str[..length_str.len() - 3].trim();
            if let Ok(size) = num_str.parse::<usize>() {
                Ok(LengthDesc {
                    size,
                    unit: LengthUnit::Bit,
                })
            } else {
                Err(format!("Invalid bit length: {}", length_str))
            }
        } else if length_str == "dynamic" {
            Ok(LengthDesc {
                size: 0,
                unit: LengthUnit::Dynamic,
            })
        } else if length_str.starts_with('(') && length_str.ends_with(')') {
            Ok(LengthDesc {
                size: 0,
                unit: LengthUnit::Expression(length_str.to_string()),
            })
        } else if let Ok(size) = length_str.parse::<usize>() {
            Ok(LengthDesc {
                size,
                unit: LengthUnit::Byte,
            }) // 默认单位
        } else {
            Ok(LengthDesc {
                size: 0,
                unit: LengthUnit::Expression(length_str.to_string()),
            })
        }
    }

    fn parse_scope_desc(scope_str: &str) -> Result<apdl_core::ScopeDesc, String> {
        let scope_str = scope_str.trim();
        if scope_str.starts_with("layer(") && scope_str.ends_with(')') {
            let layer_name = &scope_str[6..scope_str.len() - 1];
            Ok(apdl_core::ScopeDesc::Layer(layer_name.to_string()))
        } else if scope_str.starts_with("cross_layer(") && scope_str.ends_with(')') {
            let layers = &scope_str[12..scope_str.len() - 1];
            if let Some(pos) = layers.find("→") {
                let first = layers[..pos].trim();
                let second = layers[pos + 1..].trim();
                Ok(apdl_core::ScopeDesc::CrossLayer(
                    first.to_string(),
                    second.to_string(),
                ))
            } else {
                Err(format!("Invalid cross_layer format: {}", scope_str))
            }
        } else if scope_str.starts_with("global(") && scope_str.ends_with(')') {
            let scope_name = &scope_str[7..scope_str.len() - 1];
            Ok(apdl_core::ScopeDesc::Global(scope_name.to_string()))
        } else {
            Err(format!("Invalid scope format: {}", scope_str))
        }
    }

    fn parse_cover_desc(cover_str: &str) -> Result<CoverDesc, String> {
        let cover_str = cover_str.trim();
        if cover_str == "entire_field" {
            Ok(CoverDesc::EntireField)
        } else if cover_str.contains('[') && cover_str.contains(']') {
            // 解析 field[offset..offset] 格式
            if let Some(open_bracket) = cover_str.find('[') {
                if let Some(close_bracket) = cover_str.find(']') {
                    let field_part = &cover_str[..open_bracket];
                    let range_part = &cover_str[open_bracket + 1..close_bracket];

                    if let Some(double_dot) = range_part.find("..") {
                        let start_str = &range_part[..double_dot];
                        let end_str = &range_part[double_dot + 2..];

                        if let (Ok(start), Ok(end)) =
                            (start_str.parse::<usize>(), end_str.parse::<usize>())
                        {
                            return Ok(CoverDesc::Range(field_part.to_string(), start, end));
                        }
                    }
                }
            }
            Ok(CoverDesc::Expression(cover_str.to_string()))
        } else {
            Ok(CoverDesc::Expression(cover_str.to_string()))
        }
    }

    fn parse_constraint(constraint_str: &str) -> Result<Constraint, String> {
        let constraint_str = constraint_str.trim();
        if constraint_str.starts_with("fixed(") && constraint_str.ends_with(')') {
            let value_str = &constraint_str[6..constraint_str.len() - 1];
            // 尝试解析十进制或十六进制值
            let value = if value_str.starts_with("0x") || value_str.starts_with("0X") {
                u64::from_str_radix(&value_str[2..], 16)
                    .map_err(|_| format!("Invalid hex value: {}", value_str))?
            } else {
                value_str
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid decimal value: {}", value_str))?
            };
            Ok(Constraint::FixedValue(value))
        } else if constraint_str.starts_with("range(") && constraint_str.ends_with(')') {
            let range_str = &constraint_str[6..constraint_str.len() - 1];
            if let Some(double_dot_eq) = range_str.find("..=") {
                let start_str = &range_str[..double_dot_eq];
                let end_str = &range_str[double_dot_eq + 3..];

                // 解析起始值
                let start = if start_str.starts_with("0x") || start_str.starts_with("0X") {
                    u64::from_str_radix(&start_str[2..], 16)
                        .map_err(|_| format!("Invalid hex start value: {}", start_str))?
                } else {
                    start_str
                        .parse::<u64>()
                        .map_err(|_| format!("Invalid decimal start value: {}", start_str))?
                };

                // 解析结束值
                let end = if end_str.starts_with("0x") || end_str.starts_with("0X") {
                    u64::from_str_radix(&end_str[2..], 16)
                        .map_err(|_| format!("Invalid hex end value: {}", end_str))?
                } else {
                    end_str
                        .parse::<u64>()
                        .map_err(|_| format!("Invalid decimal end value: {}", end_str))?
                };

                Ok(Constraint::Range(start, end))
            } else {
                Err("Invalid range format, expected x..=y".to_string())
            }
        } else if constraint_str.starts_with("enum(") && constraint_str.ends_with(')') {
            let enum_str = &constraint_str[5..constraint_str.len() - 1];
            let mut enums = Vec::new();
            for pair_str in enum_str.split(',') {
                let pair_parts: Vec<&str> = pair_str.split('=').collect();
                if pair_parts.len() == 2 {
                    let value_str = pair_parts[1].trim();
                    let value = if value_str.starts_with("0x") || value_str.starts_with("0X") {
                        u64::from_str_radix(&value_str[2..], 16)
                            .map_err(|_| format!("Invalid hex enum value: {}", value_str))?
                    } else {
                        value_str
                            .parse::<u64>()
                            .map_err(|_| format!("Invalid decimal enum value: {}", value_str))?
                    };
                    enums.push((pair_parts[0].trim().to_string(), value));
                } else {
                    return Err("Invalid enum format".to_string());
                }
            }
            Ok(Constraint::Enum(enums))
        } else {
            Ok(Constraint::Custom(constraint_str.to_string()))
        }
    }

    fn parse_algorithm(alg_str: &str) -> Result<AlgorithmAst, String> {
        let alg_str = alg_str.trim();
        match alg_str {
            "crc16" => Ok(AlgorithmAst::Crc16),
            "crc32" => Ok(AlgorithmAst::Crc32),
            "crc15" => Ok(AlgorithmAst::Crc15), // CAN协议专用
            "xor_sum" => Ok(AlgorithmAst::XorSum),
            _ => Ok(AlgorithmAst::Custom(alg_str.to_string())),
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
            let mut last_pos = 0;
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
                last_pos = pos;
            }

            Err("Unmatched parenthesis in rule".to_string())
        } else {
            Err("No parameters found for rule".to_string())
        }
    }

    fn create_semantic_rule(rule_type: &str, params: &str) -> Result<SemanticRule, String> {
        match rule_type {
            "crc_range" | "checksum_range" => {
                // 解析范围，例如 "field1 to field2" 或 "first to last_before_fieldX"
                let params = params.trim();
                let parts: Vec<&str> = params.split(" to ").collect();
                if parts.len() == 2 {
                    Ok(SemanticRule::ChecksumRange {
                        algorithm: if rule_type == "crc_range" {
                            ChecksumAlgorithm::CRC16
                        } else {
                            ChecksumAlgorithm::XOR
                        },
                        start_field: parts[0].trim().to_string(),
                        end_field: parts[1].trim().to_string(),
                    })
                } else {
                    Err("Invalid checksum range format, expected 'field1 to field2'".to_string())
                }
            }
            "dependency" => {
                // 解析依赖关系，例如 "fieldA depends_on fieldB"
                let params = params.trim();
                let parts: Vec<&str> = params.split(" depends_on ").collect();
                if parts.len() == 2 {
                    Ok(SemanticRule::Dependency {
                        dependent_field: parts[0].trim().to_string(),
                        dependency_field: parts[1].trim().to_string(),
                    })
                } else {
                    Err(
                        "Invalid dependency format, expected 'fieldA depends_on fieldB'"
                            .to_string(),
                    )
                }
            }
            "conditional" => {
                // 解析条件关系，例如 "fieldC if fieldA.value == 0x01"
                let params = params.trim();
                Ok(SemanticRule::Conditional {
                    condition: params.to_string(),
                })
            }
            "order" => {
                // 解析字段顺序，例如 "fieldA before fieldB"
                let params = params.trim();
                let parts: Vec<&str> = params.split(" before ").collect();
                if parts.len() == 2 {
                    Ok(SemanticRule::Order {
                        first_field: parts[0].trim().to_string(),
                        second_field: parts[1].trim().to_string(),
                    })
                } else {
                    Err("Invalid order format, expected 'fieldA before fieldB'".to_string())
                }
            }
            "pointer" => {
                // 解析指针语义，例如 "pointer_field points_to target_field"
                let params = params.trim();
                let parts: Vec<&str> = params.split(" points_to ").collect();
                if parts.len() == 2 {
                    Ok(SemanticRule::Pointer {
                        pointer_field: parts[0].trim().to_string(),
                        target_field: parts[1].trim().to_string(),
                    })
                } else {
                    Err(
                        "Invalid pointer format, expected 'pointer_field points_to target_field'"
                            .to_string(),
                    )
                }
            }
            "algorithm" => {
                // 解析自定义算法，例如 "field_name uses custom_algorithm"
                let params = params.trim();
                let parts: Vec<&str> = params.split(" uses ").collect();
                if parts.len() == 2 {
                    Ok(SemanticRule::Algorithm {
                        field_name: parts[0].trim().to_string(),
                        algorithm: parts[1].trim().to_string(),
                    })
                } else {
                    Err(
                        "Invalid algorithm format, expected 'field_name uses custom_algorithm'"
                            .to_string(),
                    )
                }
            }
            "length_rule" => {
                // 解析长度规则，例如 "field_name equals expression"
                let params = params.trim();
                let parts: Vec<&str> = params.split(" equals ").collect();
                if parts.len() == 2 {
                    let field_name = parts[0].trim().to_string();
                    let expression = parts[1].trim().to_string();
                    Ok(SemanticRule::LengthRule {
                        field_name,
                        expression,
                    })
                } else {
                    Err(
                        "Invalid length rule format, expected 'field_name equals expression'"
                            .to_string(),
                    )
                }
            }
            "routing_dispatch" => {
                // 解析路由分发规则，例如 "field: vcid, apid; algorithm: hash_vcid_apid_to_route; desc: ..."
                let params = params.trim();
                let mut field_name = String::new();
                let mut algorithm = String::new();
                let mut description = String::new();

                // 简单解析，查找关键部分
                if params.contains("field:") && params.contains("algorithm:") {
                    // 提取字段列表
                    if let Some(field_start) = params.find("field:") {
                        if let Some(semi_pos) =
                            params[field_start..].find(';').map(|p| p + field_start)
                        {
                            let field_part = &params[field_start + 6..semi_pos].trim();
                            field_name = field_part.to_string();
                        }
                    }

                    if let Some(alg_start) = params.find("algorithm:") {
                        let remaining = &params[alg_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                            algorithm = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
                        } else {
                            algorithm = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                // 解析字段列表
                let fields: Vec<String> = field_name
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();

                Ok(SemanticRule::RoutingDispatch {
                    fields,
                    algorithm,
                    description,
                })
            }
            "sequence_control" => {
                // 解析序列控制规则
                let params = params.trim();
                let mut field_name = String::new();
                let mut trigger_condition = String::new();
                let mut algorithm = String::new();
                let mut description = String::new();

                if params.contains("field:")
                    && params.contains("trigger:")
                    && params.contains("algorithm:")
                {
                    if let Some(field_start) = params.find("field:") {
                        if let Some(semi_pos) =
                            params[field_start..].find(';').map(|p| p + field_start)
                        {
                            field_name = params[field_start + 6..semi_pos].trim().to_string();
                        }
                    }

                    if let Some(trigger_start) = params.find("trigger:") {
                        let remaining = &params[trigger_start + 8..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + trigger_start + 8) {
                            trigger_condition = remaining[..semi_pos - (trigger_start + 8)]
                                .trim()
                                .to_string();
                        } else {
                            trigger_condition = remaining.trim().to_string();
                        }
                    }

                    if let Some(alg_start) = params.find("algorithm:") {
                        let remaining = &params[alg_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                            algorithm = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
                        } else {
                            algorithm = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                Ok(SemanticRule::SequenceControl {
                    field_name,
                    trigger_condition,
                    algorithm,
                    description,
                })
            }
            "validation" => {
                // 解析校验规则
                let params = params.trim();
                let mut field_name = String::new();
                let mut algorithm = String::new();
                let mut range_start = String::new();
                let mut range_end = String::new();
                let mut description = String::new();

                if params.contains("field:")
                    && params.contains("algorithm:")
                    && params.contains("range:")
                {
                    if let Some(field_start) = params.find("field:") {
                        if let Some(semi_pos) =
                            params[field_start..].find(';').map(|p| p + field_start)
                        {
                            field_name = params[field_start + 6..semi_pos].trim().to_string();
                        }
                    }

                    if let Some(alg_start) = params.find("algorithm:") {
                        let remaining = &params[alg_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                            algorithm = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
                        } else {
                            algorithm = remaining.trim().to_string();
                        }
                    }

                    // 解析 from(...) to(...) 格式的范围
                    if let Some(from_start) = params.find("from(") {
                        let from_content_start = from_start + 5; // 跳过 "from("
                        if let Some(to_pos) = params[from_content_start..].find(") to(") {
                            let actual_to_pos = from_content_start + to_pos;
                            range_start =
                                params[from_content_start..actual_to_pos].trim().to_string();

                            // 查找结束括号的位置
                            let remaining = &params[actual_to_pos + 5..]; // 跳过 ") to("
                            if let Some(end_bracket_pos) = remaining.find(')') {
                                range_end = remaining[..end_bracket_pos].trim().to_string();
                            }
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                Ok(SemanticRule::Validation {
                    field_name,
                    algorithm,
                    range_start,
                    range_end,
                    description,
                })
            }
            "multiplexing" => {
                // 解析多路复用规则
                let params = params.trim();
                let mut field_name = String::new();
                let mut condition = String::new();
                let mut route_target = String::new();
                let mut description = String::new();

                if params.contains("field:")
                    && params.contains("condition:")
                    && params.contains("route_to:")
                {
                    if let Some(field_start) = params.find("field:") {
                        if let Some(semi_pos) =
                            params[field_start..].find(';').map(|p| p + field_start)
                        {
                            field_name = params[field_start + 6..semi_pos].trim().to_string();
                        }
                    }

                    if let Some(cond_start) = params.find("condition:") {
                        let remaining = &params[cond_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + cond_start + 10) {
                            condition =
                                remaining[..semi_pos - (cond_start + 10)].trim().to_string();
                        } else {
                            condition = remaining.trim().to_string();
                        }
                    }

                    if let Some(route_start) = params.find("route_to:") {
                        let remaining = &params[route_start + 9..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + route_start + 9) {
                            route_target =
                                remaining[..semi_pos - (route_start + 9)].trim().to_string();
                        } else {
                            route_target = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                Ok(SemanticRule::Multiplexing {
                    field_name,
                    condition,
                    route_target,
                    description,
                })
            }
            "priority_processing" => {
                // 解析优先级处理规则
                let params = params.trim();
                let mut field_name = String::new();
                let mut algorithm = String::new();
                let mut description = String::new();

                if params.contains("field:") && params.contains("algorithm:") {
                    if let Some(field_start) = params.find("field:") {
                        if let Some(semi_pos) =
                            params[field_start..].find(';').map(|p| p + field_start)
                        {
                            field_name = params[field_start + 6..semi_pos].trim().to_string();
                        }
                    }

                    if let Some(alg_start) = params.find("algorithm:") {
                        let remaining = &params[alg_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                            algorithm = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
                        } else {
                            algorithm = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                Ok(SemanticRule::PriorityProcessing {
                    field_name,
                    algorithm,
                    description,
                })
            }
            "synchronization" => {
                // 解析同步规则
                let params = params.trim();
                let mut field_name = String::new();
                let mut algorithm = String::new();
                let mut description = String::new();

                if params.contains("field:") && params.contains("algorithm:") {
                    if let Some(field_start) = params.find("field:") {
                        if let Some(semi_pos) =
                            params[field_start..].find(';').map(|p| p + field_start)
                        {
                            field_name = params[field_start + 6..semi_pos].trim().to_string();
                        }
                    }

                    if let Some(alg_start) = params.find("algorithm:") {
                        let remaining = &params[alg_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                            algorithm = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
                        } else {
                            algorithm = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                Ok(SemanticRule::Synchronization {
                    field_name,
                    algorithm,
                    description,
                })
            }
            "length_validation" => {
                // 解析长度验证规则
                let params = params.trim();
                let mut field_name = String::new();
                let mut condition = String::new();
                let mut description = String::new();

                if params.contains("field:") && params.contains("condition:") {
                    if let Some(field_start) = params.find("field:") {
                        if let Some(semi_pos) =
                            params[field_start..].find(';').map(|p| p + field_start)
                        {
                            field_name = params[field_start + 6..semi_pos].trim().to_string();
                        }
                    }

                    if let Some(cond_start) = params.find("condition:") {
                        let remaining = &params[cond_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + cond_start + 10) {
                            condition =
                                remaining[..semi_pos - (cond_start + 10)].trim().to_string();
                        } else {
                            condition = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                Ok(SemanticRule::LengthValidation {
                    field_name,
                    condition,
                    description,
                })
            }
            "state_machine" => {
                // 解析状态机规则
                let params = params.trim();
                let mut condition = String::new();
                let mut algorithm = String::new();
                let mut description = String::new();

                if params.contains("condition:") && params.contains("algorithm:") {
                    if let Some(cond_start) = params.find("condition:") {
                        if let Some(semi_pos) =
                            params[cond_start..].find(';').map(|p| p + cond_start)
                        {
                            condition = params[cond_start + 10..semi_pos].trim().to_string();
                        }
                    }

                    if let Some(alg_start) = params.find("algorithm:") {
                        let remaining = &params[alg_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                            algorithm = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
                        } else {
                            algorithm = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                Ok(SemanticRule::StateMachine {
                    condition,
                    algorithm,
                    description,
                })
            }
            "periodic_transmission" => {
                // 解析周期性传输规则
                let params = params.trim();
                let mut field_name = String::new();
                let mut condition = String::new();
                let mut algorithm = String::new();
                let mut description = String::new();

                if params.contains("field:")
                    && params.contains("condition:")
                    && params.contains("algorithm:")
                {
                    if let Some(field_start) = params.find("field:") {
                        if let Some(semi_pos) =
                            params[field_start..].find(';').map(|p| p + field_start)
                        {
                            field_name = params[field_start + 6..semi_pos].trim().to_string();
                        }
                    }

                    if let Some(cond_start) = params.find("condition:") {
                        let remaining = &params[cond_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + cond_start + 10) {
                            condition =
                                remaining[..semi_pos - (cond_start + 10)].trim().to_string();
                        } else {
                            condition = remaining.trim().to_string();
                        }
                    }

                    if let Some(alg_start) = params.find("algorithm:") {
                        let remaining = &params[alg_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                            algorithm = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
                        } else {
                            algorithm = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                Ok(SemanticRule::PeriodicTransmission {
                    field_name,
                    condition,
                    algorithm,
                    description,
                })
            }
            "message_filtering" => {
                // 解析消息过滤规则
                let params = params.trim();
                let mut condition = String::new();
                let mut action = String::new();
                let mut description = String::new();

                if params.contains("condition:") && params.contains("action:") {
                    if let Some(cond_start) = params.find("condition:") {
                        if let Some(semi_pos) =
                            params[cond_start..].find(';').map(|p| p + cond_start)
                        {
                            condition = params[cond_start + 10..semi_pos].trim().to_string();
                        }
                    }

                    if let Some(action_start) = params.find("action:") {
                        let remaining = &params[action_start + 7..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + action_start + 7) {
                            action = remaining[..semi_pos - (action_start + 7)]
                                .trim()
                                .to_string();
                        } else {
                            action = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                Ok(SemanticRule::MessageFiltering {
                    condition,
                    action,
                    description,
                })
            }
            "error_detection" => {
                // 解析错误检测规则
                let params = params.trim();
                let mut algorithm = String::new();
                let mut description = String::new();

                if params.contains("algorithm:") {
                    if let Some(alg_start) = params.find("algorithm:") {
                        let remaining = &params[alg_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                            algorithm = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
                        } else {
                            algorithm = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                } else {
                    algorithm = params.to_string();
                }

                Ok(SemanticRule::ErrorDetection {
                    algorithm,
                    description,
                })
            }
            "nested_sync" => {
                // 解析嵌套同步规则
                let params = params.trim();
                let mut field_name = String::new();
                let mut target = String::new();
                let mut algorithm = String::new();
                let mut description = String::new();

                if params.contains("field:")
                    && params.contains("target:")
                    && params.contains("algorithm:")
                {
                    if let Some(field_start) = params.find("field:") {
                        if let Some(semi_pos) =
                            params[field_start..].find(';').map(|p| p + field_start)
                        {
                            field_name = params[field_start + 6..semi_pos].trim().to_string();
                        }
                    }

                    if let Some(target_start) = params.find("target:") {
                        let remaining = &params[target_start + 7..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + target_start + 7) {
                            target = remaining[..semi_pos - (target_start + 7)]
                                .trim()
                                .to_string();
                        } else {
                            target = remaining.trim().to_string();
                        }
                    }

                    if let Some(alg_start) = params.find("algorithm:") {
                        let remaining = &params[alg_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                            algorithm = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
                        } else {
                            algorithm = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                Ok(SemanticRule::Pointer {
                    pointer_field: field_name,
                    target_field: target,
                })
            }
            "sequence_reset" => {
                // 解析序列重置规则
                let params = params.trim();
                let mut field_name = String::new();
                let mut condition = String::new();
                let mut action = String::new();
                let mut description = String::new();

                if params.contains("field:")
                    && params.contains("condition:")
                    && params.contains("action:")
                {
                    if let Some(field_start) = params.find("field:") {
                        if let Some(semi_pos) =
                            params[field_start..].find(';').map(|p| p + field_start)
                        {
                            field_name = params[field_start + 6..semi_pos].trim().to_string();
                        }
                    }

                    if let Some(cond_start) = params.find("condition:") {
                        let remaining = &params[cond_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + cond_start + 10) {
                            condition =
                                remaining[..semi_pos - (cond_start + 10)].trim().to_string();
                        } else {
                            condition = remaining.trim().to_string();
                        }
                    }

                    if let Some(action_start) = params.find("action:") {
                        let remaining = &params[action_start + 7..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + action_start + 7) {
                            action = remaining[..semi_pos - (action_start + 7)]
                                .trim()
                                .to_string();
                        } else {
                            action = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                Ok(SemanticRule::Conditional {
                    condition: format!("{} {} {}", field_name, condition, action),
                })
            }
            "timestamp_insertion" => {
                // 解析时间戳插入规则
                let params = params.trim();
                let mut condition = String::new();
                let mut field_name = String::new();
                let mut algorithm = String::new();
                let mut description = String::new();

                if params.contains("condition:")
                    && params.contains("field:")
                    && params.contains("algorithm:")
                {
                    if let Some(cond_start) = params.find("condition:") {
                        if let Some(semi_pos) =
                            params[cond_start..].find(';').map(|p| p + cond_start)
                        {
                            condition = params[cond_start + 10..semi_pos].trim().to_string();
                        }
                    }

                    if let Some(field_start) = params.find("field:") {
                        let remaining = &params[field_start + 6..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + field_start + 6) {
                            field_name =
                                remaining[..semi_pos - (field_start + 6)].trim().to_string();
                        } else {
                            field_name = remaining.trim().to_string();
                        }
                    }

                    if let Some(alg_start) = params.find("algorithm:") {
                        let remaining = &params[alg_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                            algorithm = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
                        } else {
                            algorithm = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                Ok(SemanticRule::Conditional {
                    condition: format!("{} on {} with {}", condition, field_name, algorithm),
                })
            }
            "flow_control" => {
                // 解析流量控制规则
                let params = params.trim();
                let mut field_name = String::new();
                let mut algorithm = String::new();
                let mut description = String::new();

                if params.contains("field:") && params.contains("algorithm:") {
                    if let Some(field_start) = params.find("field:") {
                        if let Some(semi_pos) =
                            params[field_start..].find(';').map(|p| p + field_start)
                        {
                            field_name = params[field_start + 6..semi_pos].trim().to_string();
                        } else {
                            // 尝试解析没有分号的情况
                            let remaining = &params[field_start + 6..];
                            if let Some(next_semicolon) = remaining.find(';') {
                                field_name = remaining[..next_semicolon].trim().to_string();
                            } else {
                                field_name = remaining.trim().to_string();
                            }
                        }
                    }

                    if let Some(alg_start) = params.find("algorithm:") {
                        let remaining = &params[alg_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                            algorithm = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
                        } else {
                            algorithm = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                Ok(SemanticRule::FlowControl {
                    field_name,
                    algorithm,
                    description,
                })
            }
            "time_synchronization" => {
                // 解析时间同步规则
                let params = params.trim();
                let mut field_name = String::new();
                let mut algorithm = String::new();
                let mut description = String::new();

                if params.contains("field:") && params.contains("algorithm:") {
                    if let Some(field_start) = params.find("field:") {
                        if let Some(semi_pos) =
                            params[field_start..].find(';').map(|p| p + field_start)
                        {
                            field_name = params[field_start + 6..semi_pos].trim().to_string();
                        } else {
                            let remaining = &params[field_start + 6..];
                            if let Some(next_semicolon) = remaining.find(';') {
                                field_name = remaining[..next_semicolon].trim().to_string();
                            } else {
                                field_name = remaining.trim().to_string();
                            }
                        }
                    }

                    if let Some(alg_start) = params.find("algorithm:") {
                        let remaining = &params[alg_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                            algorithm = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
                        } else {
                            algorithm = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                Ok(SemanticRule::TimeSynchronization {
                    field_name,
                    algorithm,
                    description,
                })
            }
            "address_resolution" => {
                // 解析地址解析规则
                let params = params.trim();
                let mut field_name = String::new();
                let mut algorithm = String::new();
                let mut description = String::new();

                if params.contains("field:") && params.contains("algorithm:") {
                    if let Some(field_start) = params.find("field:") {
                        if let Some(semi_pos) =
                            params[field_start..].find(';').map(|p| p + field_start)
                        {
                            field_name = params[field_start + 6..semi_pos].trim().to_string();
                        } else {
                            let remaining = &params[field_start + 6..];
                            if let Some(next_semicolon) = remaining.find(';') {
                                field_name = remaining[..next_semicolon].trim().to_string();
                            } else {
                                field_name = remaining.trim().to_string();
                            }
                        }
                    }

                    if let Some(alg_start) = params.find("algorithm:") {
                        let remaining = &params[alg_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                            algorithm = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
                        } else {
                            algorithm = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                Ok(SemanticRule::AddressResolution {
                    field_name,
                    algorithm,
                    description,
                })
            }
            "security" => {
                // 解析安全规则
                let params = params.trim();
                let mut field_name = String::new();
                let mut algorithm = String::new();
                let mut description = String::new();

                if params.contains("field:") && params.contains("algorithm:") {
                    if let Some(field_start) = params.find("field:") {
                        if let Some(semi_pos) =
                            params[field_start..].find(';').map(|p| p + field_start)
                        {
                            field_name = params[field_start + 6..semi_pos].trim().to_string();
                        } else {
                            let remaining = &params[field_start + 6..];
                            if let Some(next_semicolon) = remaining.find(';') {
                                field_name = remaining[..next_semicolon].trim().to_string();
                            } else {
                                field_name = remaining.trim().to_string();
                            }
                        }
                    }

                    if let Some(alg_start) = params.find("algorithm:") {
                        let remaining = &params[alg_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                            algorithm = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
                        } else {
                            algorithm = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                Ok(SemanticRule::Security {
                    field_name,
                    algorithm,
                    description,
                })
            }
            "redundancy" => {
                // 解析冗余规则
                let params = params.trim();
                let mut field_name = String::new();
                let mut algorithm = String::new();
                let mut description = String::new();

                if params.contains("field:") && params.contains("algorithm:") {
                    if let Some(field_start) = params.find("field:") {
                        if let Some(semi_pos) =
                            params[field_start..].find(';').map(|p| p + field_start)
                        {
                            field_name = params[field_start + 6..semi_pos].trim().to_string();
                        } else {
                            let remaining = &params[field_start + 6..];
                            if let Some(next_semicolon) = remaining.find(';') {
                                field_name = remaining[..next_semicolon].trim().to_string();
                            } else {
                                field_name = remaining.trim().to_string();
                            }
                        }
                    }

                    if let Some(alg_start) = params.find("algorithm:") {
                        let remaining = &params[alg_start + 10..];
                        if let Some(semi_pos) = remaining.find(';').map(|p| p + alg_start + 10) {
                            algorithm = remaining[..semi_pos - (alg_start + 10)].trim().to_string();
                        } else {
                            algorithm = remaining.trim().to_string();
                        }
                    }

                    if let Some(desc_start) = params.find("desc:") {
                        description = params[desc_start + 5..].trim().to_string();
                    }
                }

                Ok(SemanticRule::Redundancy {
                    field_name,
                    algorithm,
                    description,
                })
            }
            _ => Err(format!("Unknown rule type: {}", rule_type)),
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
}
