//! 长度规则处理器
//!
//! 处理与长度相关的语义规则，包括长度表达式计算和函数表达式解析


use crate::standard_units::frame_assembler::core::FrameAssembler;
use apdl_core::{ProtocolError, SemanticRule};

impl FrameAssembler {
    /// 应用长度和CRC规则（第二阶段处理）
    pub fn apply_length_and_crc_rules(
        &mut self,
        frame_data: &mut Vec<u8>,
    ) -> Result<(), ProtocolError> {
        // 克隆语义规则以避免借用冲突
        let rules_to_process: Vec<_> = self.semantic_rules.clone();

        // 分离长度规则和校验和规则，确保长度规则先处理
        let mut length_rules = Vec::new();
        let mut checksum_rules = Vec::new();
        let mut other_rules = Vec::new();

        for rule in &rules_to_process {
            match rule {
                SemanticRule::LengthRule { .. } => {
                    length_rules.push(rule);
                }
                SemanticRule::ChecksumRange { .. } => {
                    checksum_rules.push(rule);
                }
                _ => {
                    other_rules.push(rule);
                }
            }
        }

        // 首先处理所有长度规则
        for rule in &length_rules {
            if let SemanticRule::LengthRule {
                field_name,
                expression,
            } = rule
            {
                // 清理字段名，移除可能的前缀
                let clean_field_name = field_name.trim_start_matches("field: ").trim();

                // 计算长度表达式的值
                let length_value = self.evaluate_length_expression(expression, frame_data)?;
                println!(
                    "DEBUG: Calculated length_value for field '{clean_field_name}' with expression '{expression}': {length_value}"
                );

                // 查找字段在帧中的位置
                if let Some(&field_index) = self.field_index.get(clean_field_name) {
                    let field = &self.fields[field_index];
                    let field_size = self.get_field_size(field)?;
                    let field_offset = self.calculate_field_offset(field_index)?;

                    // 将长度值写入帧数据
                    let length_bytes = self.u64_to_bytes(length_value, field_size);
                    for (i, &byte) in length_bytes.iter().enumerate() {
                        if field_offset + i < frame_data.len() {
                            frame_data[field_offset + i] = byte;
                        }
                    }

                    // 同时更新字段值存储
                    self.field_values
                        .insert(clean_field_name.to_string(), length_bytes);
                }
            }
        }

        // 然后处理所有校验和规则（此时所有长度字段已更新）
        for rule in &checksum_rules {
            if let SemanticRule::ChecksumRange {
                algorithm,
                start_field,
                end_field,
            } = rule
            {
                // 清理字段名，移除可能的前缀
                let clean_start_field = start_field.trim_start_matches("start: ").trim();
                let clean_end_field = end_field.trim_start_matches("end: ").trim();
                self.apply_checksum_rule(
                    frame_data,
                    algorithm,
                    clean_start_field,
                    clean_end_field,
                )?;
            }
        }

        // 最后处理其他规则
        for _rule in &other_rules {
            // 其他规则的处理逻辑
        }

        Ok(())
    }

    /// 解析长度表达式
    pub fn evaluate_length_expression(
        &self,
        expression: &str,
        frame_data: &[u8],
    ) -> Result<u64, ProtocolError> {
        // 简单的表达式解析器，支持基本算术运算和函数调用
        // 例如: "(total_length - 3)", "(data_length + 7)", "pos(fecf) + len(fecf) - pos(version)", 等

        // 移除可能的双引号和括号
        println!(
            "DEBUG: evaluate_length_expression - Original expression: '{expression:?}'"
        );
        // 首先移除最外层的引号（处理转义引号）
        let mut expr_cleaned = expression.trim().to_string();

        // 检查是否以引号开始和结束，并移除它们
        if expr_cleaned.starts_with('"') && expr_cleaned.ends_with('"') && expr_cleaned.len() > 1 {
            expr_cleaned = expr_cleaned[1..expr_cleaned.len() - 1].to_string();
        }

        // 再移除最外层的括号
        expr_cleaned = expr_cleaned.trim().to_string();
        if expr_cleaned.starts_with('(') && expr_cleaned.ends_with(')') && expr_cleaned.len() > 1 {
            expr_cleaned = expr_cleaned[1..expr_cleaned.len() - 1].to_string();
        }

        println!(
            "DEBUG: evaluate_length_expression - After cleaning: '{expr_cleaned:?}'"
        );

        // 检查是否包含 min 或 max 函数
        if expr_cleaned.starts_with("min(") && expr_cleaned.ends_with(')') {
            let args_str = &expr_cleaned[4..expr_cleaned.len() - 1]; // 移除 "min(" 和 ")"
            let args: Vec<&str> = args_str.split(',').collect();
            if args.len() == 2 {
                let left = self.evaluate_length_argument(args[0].trim())?;
                let right = self.evaluate_length_argument(args[1].trim())?;
                return Ok(left.min(right));
            } else {
                return Err(ProtocolError::InvalidExpression(
                    "Min function requires exactly two arguments".to_string(),
                ));
            }
        }

        if expr_cleaned.starts_with("max(") && expr_cleaned.ends_with(')') {
            let args_str = &expr_cleaned[4..expr_cleaned.len() - 1]; // 移除 "max(" 和 ")"
            let args: Vec<&str> = args_str.split(',').collect();
            if args.len() == 2 {
                let left = self.evaluate_length_argument(args[0].trim())?;
                let right = self.evaluate_length_argument(args[1].trim())?;
                return Ok(left.max(right));
            } else {
                return Err(ProtocolError::InvalidExpression(
                    "Max function requires exactly two arguments".to_string(),
                ));
            }
        }

        // 处理特殊情况：如果表达式是形如 "fieldname_length" 的格式，将其视为 len(fieldname)
        let final_expr = if expr_cleaned.ends_with("_length") {
            // 查找字段名（去掉"_length"后缀）
            if let Some(field_name) = expr_cleaned.strip_suffix("_length") {
                // 尝试获取该字段的长度
                if let Ok(size) = self.get_field_size_by_name(field_name) {
                    return Ok(size as u64);
                }
            }
            expr_cleaned.clone()
        } else {
            expr_cleaned.clone()
        };

        // 如果表达式包含引号，则直接尝试解析为数字
        if final_expr.starts_with('"') && final_expr.ends_with('"') {
            let inner = &final_expr[1..final_expr.len() - 1];
            return match inner.parse::<u64>() {
                Ok(value) => Ok(value),
                Err(_) => Err(ProtocolError::InvalidExpression(format!(
                    "Unable to parse quoted expression: {expression}"
                ))),
            };
        }

        // 检查是否包含函数调用语法 (如 len(field) 或 pos(field))
        if expr_cleaned.contains("len(") || expr_cleaned.contains("pos(") {
            return self.evaluate_function_expression(&expr_cleaned, frame_data);
        }

        // 处理几种常见的表达式模式
        if expr_cleaned.contains("total_length") {
            let total_len = frame_data.len() as u64;
            // 简单解析表达式，如 "total_length - 3"
            if let Some(pos) = expr_cleaned.find('-') {
                let left = &expr_cleaned[..pos].trim();
                let right = &expr_cleaned[pos + 1..].trim();

                if left.trim() == "total_length" {
                    if let Ok(right_val) = right.parse::<u64>() {
                        return Ok(total_len.saturating_sub(right_val));
                    }
                }
            } else if let Some(pos) = expr_cleaned.find('+') {
                let left = &expr_cleaned[..pos].trim();
                let right = &expr_cleaned[pos + 1..].trim();

                if left.trim() == "total_length" {
                    if let Ok(right_val) = right.parse::<u64>() {
                        return Ok(total_len + right_val);
                    }
                }
            }
        } else if expr_cleaned.contains("data_length") {
            // 处理基于数据长度的表达式
            // 这里需要知道数据字段的位置和长度
            // 遍历查找数据字段
            for field in &self.fields {
                if self.is_data_field(field) || field.field_id.to_lowercase().contains("data") {
                    // 实现基于数据长度的计算
                    // 这里简化处理，实际实现可能更复杂
                    return Ok(self.get_field_size(field)? as u64);
                }
            }
        }

        // 默认情况下，尝试解析为数字
        match expr_cleaned.parse::<u64>() {
            Ok(value) => Ok(value),
            Err(_) => Err(ProtocolError::InvalidExpression(format!(
                "Unable to parse expression: {expression}"
            ))),
        }
    }

    /// 评估函数表达式（如 len(field) 和 pos(field)）
    pub fn evaluate_function_expression(
        &self,
        expression: &str,
        _frame_data: &[u8],
    ) -> Result<u64, ProtocolError> {
        // 移除可能的引号和括号
        let expr_cleaned = expression
            .trim()
            .trim_matches('"')
            .trim_matches(|c| c == '(' || c == ')');
        let mut result = expr_cleaned.to_string();

        println!("DEBUG: Original expression: '{expression:?}'");
        println!("DEBUG: Cleaned expression: '{expr_cleaned:?}'");

        // 检查表达式是否可能缺少右括号（平衡性检查）
        // 如果原始表达式以右括号结尾，但在清理过程中丢失了，我们尝试恢复它
        if expression.trim().ends_with(')') && !result.trim().ends_with(')') {
            // 计算左右括号数量，判断是否不平衡
            let left_parens = result.matches('(').count();
            let right_parens = result.matches(')').count();

            if left_parens > right_parens {
                // 尝试在末尾补充缺失的右括号
                let missing_parens = left_parens - right_parens;
                for _ in 0..missing_parens {
                    result.push(')');
                }
                println!(
                    "DEBUG: Restored missing parentheses, new result: '{result:?}'"
                );
            }
        }

        // 使用两阶段处理来避免替换冲突
        // 注意：在这里我们使用相同的字符串进行查找和替换，确保一致性
        let mut temp_replacements = Vec::new();

        // 查找所有 len() 函数调用
        let len_regex = regex::Regex::new(r"len\([^)]+\)").unwrap();
        for mat in len_regex.find_iter(&result) {
            // 使用result而不是expression
            let matched = mat.as_str();
            let field_name_start = matched.find('(').map(|i| i + 1).unwrap_or(0);
            let field_name_end = matched.rfind(')').unwrap_or(matched.len());
            if field_name_start >= field_name_end {
                continue; // 跳过无效的匹配
            }
            let field_name = &matched[field_name_start..field_name_end].trim();

            println!(
                "DEBUG: Found len function: {matched:?}, field_name: {field_name:?}"
            );

            if let Ok(size) = self.get_field_size_by_name(field_name) {
                temp_replacements.push((matched.to_string(), size.to_string()));
                println!("DEBUG: Adding replacement: {matched:?} -> {size:?}");
            }
        }

        // 查找所有 pos() 函数调用
        let pos_regex = regex::Regex::new(r"pos\([^)]+\)").unwrap();
        for mat in pos_regex.find_iter(&result) {
            // 使用result而不是expression
            let matched = mat.as_str();
            let field_name_start = matched.find('(').map(|i| i + 1).unwrap_or(0);
            let field_name_end = matched.rfind(')').unwrap_or(matched.len());
            if field_name_start >= field_name_end {
                continue; // 跳过无效的匹配
            }
            let field_name = &matched[field_name_start..field_name_end].trim();

            println!(
                "DEBUG: Found pos function: {matched:?}, field_name: {field_name:?}"
            );

            if let Ok(position) = self.get_field_position(field_name) {
                temp_replacements.push((matched.to_string(), position.to_string()));
                println!("DEBUG: Adding replacement: {matched:?} -> {position:?}");
            }
        }

        // 按照在原字符串中的位置倒序排列，以避免替换时的索引偏移
        temp_replacements.sort_by(|a, b| {
            result
                .find(&a.0)
                .unwrap_or(0)
                .cmp(&result.find(&b.0).unwrap_or(0))
                .reverse()
        });

        println!("DEBUG: Temp replacements: {temp_replacements:?}");

        // 应用替换
        for (old, new) in temp_replacements {
            println!(
                "DEBUG: Replacing '{old:?}' with '{new:?}' in '{result:?}'"
            );
            result = result.replacen(&old, &new, 1);
            println!("DEBUG: After replacement: '{result:?}'");
        }

        println!(
            "DEBUG: Expression after function substitution: '{result:?}'"
        );

        // 移除可能的外部引号
        let result_without_quotes = result.trim().trim_matches('"').to_string();

        // 简单的数学表达式求值
        // 这里简化处理，实际可能需要更复杂的表达式解析器
        // 支持 +, -, *, / 等基本运算和 min/max 函数
        let final_result = self.evaluate_math_expression(&result_without_quotes)?;
        println!(
            "DEBUG: Final result after math evaluation: {final_result:?}"
        );

        Ok(final_result)
    }

    /// 评估数学表达式
    fn evaluate_math_expression(&self, expr: &str) -> Result<u64, ProtocolError> {
        // 移除空格
        let cleaned = expr.replace(" ", "");

        // 检查是否包含 min 函数
        if cleaned.starts_with("min(") && cleaned.ends_with(')') {
            let args_str = &cleaned[4..cleaned.len() - 1]; // 移除 "min(" 和 ")"
            let args: Vec<&str> = args_str.split(',').collect();
            if args.len() == 2 {
                let left = args[0].trim().parse::<u64>().map_err(|_| {
                    ProtocolError::InvalidExpression(format!(
                        "Invalid number in min function: {}",
                        args[0]
                    ))
                })?;
                let right = args[1].trim().parse::<u64>().map_err(|_| {
                    ProtocolError::InvalidExpression(format!(
                        "Invalid number in min function: {}",
                        args[1]
                    ))
                })?;
                return Ok(left.min(right));
            } else {
                return Err(ProtocolError::InvalidExpression(
                    "Min function requires exactly two arguments".to_string(),
                ));
            }
        }

        // 检查是否包含 max 函数
        if cleaned.starts_with("max(") && cleaned.ends_with(')') {
            let args_str = &cleaned[4..cleaned.len() - 1]; // 移除 "max(" 和 ")"
            let args: Vec<&str> = args_str.split(',').collect();
            if args.len() == 2 {
                let left = args[0].trim().parse::<u64>().map_err(|_| {
                    ProtocolError::InvalidExpression(format!(
                        "Invalid number in max function: {}",
                        args[0]
                    ))
                })?;
                let right = args[1].trim().parse::<u64>().map_err(|_| {
                    ProtocolError::InvalidExpression(format!(
                        "Invalid number in max function: {}",
                        args[1]
                    ))
                })?;
                return Ok(left.max(right));
            } else {
                return Err(ProtocolError::InvalidExpression(
                    "Max function requires exactly two arguments".to_string(),
                ));
            }
        }

        // 检查是否包含运算符
        if cleaned.contains('+')
            || cleaned.contains('-')
            || cleaned.contains('*')
            || cleaned.contains('/')
        {
            return self.evaluate_arithmetic_expression(&cleaned);
        }

        // 如果没有运算符，尝试直接解析为数字
        cleaned
            .parse::<u64>()
            .map_err(|_| ProtocolError::InvalidExpression(format!("Invalid expression: {expr}")))
    }

    /// 评估算术表达式（支持 +, -, *, / 运算符）
    fn evaluate_arithmetic_expression(&self, expr: &str) -> Result<u64, ProtocolError> {
        let s = expr.trim();

        // 从左到右解析和计算表达式
        // 实现一个简单的表达式计算器

        // 首先处理乘除法
        let mut tokens: Vec<String> = Vec::new();
        let mut current_token = String::new();
        let chars = s.chars().peekable();

        // 分词
        for ch in chars {
            match ch {
                '+' | '-' | '*' | '/' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                    tokens.push(ch.to_string());
                }
                c if c.is_whitespace() => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }
        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        // 先处理乘除法
        let mut i = 0;
        while i < tokens.len() {
            if tokens[i] == "*" || tokens[i] == "/" {
                if i > 0 && i < tokens.len() - 1 {
                    let left = tokens[i - 1].parse::<u64>().map_err(|_| {
                        ProtocolError::InvalidExpression(format!(
                            "Invalid number: {}",
                            tokens[i - 1]
                        ))
                    })?;
                    let right = tokens[i + 1].parse::<u64>().map_err(|_| {
                        ProtocolError::InvalidExpression(format!(
                            "Invalid number: {}",
                            tokens[i + 1]
                        ))
                    })?;

                    let result = if tokens[i] == "*" {
                        left * right
                    } else {
                        if right == 0 {
                            return Err(ProtocolError::InvalidExpression(
                                "Division by zero".to_string(),
                            ));
                        }
                        left / right
                    };

                    // 替换三个token为结果
                    tokens.splice(i - 1..=i + 1, vec![result.to_string()]);
                    i = i.saturating_sub(1); // 回退一步检查
                } else {
                    return Err(ProtocolError::InvalidExpression(
                        "Invalid expression syntax".to_string(),
                    ));
                }
            } else {
                i += 1;
            }
        }

        // 再处理加减法
        let mut i = 0;
        while i < tokens.len() {
            if tokens[i] == "+" || tokens[i] == "-" {
                if i > 0 && i < tokens.len() - 1 {
                    let left = tokens[i - 1].parse::<u64>().map_err(|_| {
                        ProtocolError::InvalidExpression(format!(
                            "Invalid number: {}",
                            tokens[i - 1]
                        ))
                    })?;
                    let right = tokens[i + 1].parse::<u64>().map_err(|_| {
                        ProtocolError::InvalidExpression(format!(
                            "Invalid number: {}",
                            tokens[i + 1]
                        ))
                    })?;

                    let result = if tokens[i] == "+" {
                        left + right
                    } else {
                        if left < right {
                            return Err(ProtocolError::InvalidExpression(
                                "Underflow in subtraction".to_string(),
                            ));
                        }
                        left - right
                    };

                    // 替换三个token为结果
                    tokens.splice(i - 1..=i + 1, vec![result.to_string()]);
                    i = i.saturating_sub(1); // 回退一步检查
                } else {
                    return Err(ProtocolError::InvalidExpression(
                        "Invalid expression syntax".to_string(),
                    ));
                }
            } else {
                i += 1;
            }
        }

        if tokens.len() != 1 {
            return Err(ProtocolError::InvalidExpression(
                "Invalid expression syntax".to_string(),
            ));
        }

        tokens[0]
            .parse::<u64>()
            .map_err(|_| ProtocolError::InvalidExpression(format!("Invalid expression: {expr}")))
    }

    /// 评估长度参数（可以是数字、字段长度或函数）
    fn evaluate_length_argument(&self, arg: &str) -> Result<u64, ProtocolError> {
        let trimmed_arg = arg.trim();

        // 尝试解析为数字
        if let Ok(num) = trimmed_arg.parse::<u64>() {
            return Ok(num);
        }

        // 检查是否是函数调用（len或pos）
        if trimmed_arg.starts_with("len(") && trimmed_arg.ends_with(')') {
            let field_name = &trimmed_arg[4..trimmed_arg.len() - 1].trim();
            // 清理可能的前缀
            let clean_field_name = field_name
                .trim_start_matches("field: ")
                .trim_start_matches("field:")
                .trim();
            if let Some(&index) = self.field_index.get(clean_field_name) {
                if let Some(field) = self.fields.get(index) {
                    if let Some(value) = self.field_values.get(&field.field_id) {
                        return Ok(value.len() as u64);
                    }
                }
            }
            // 如果没在field_values中找到，尝试获取字段的定义大小
            return self
                .get_field_size_by_name(clean_field_name)
                .map(|size| size as u64);
        } else if trimmed_arg.starts_with("pos(") && trimmed_arg.ends_with(')') {
            let field_name = &trimmed_arg[4..trimmed_arg.len() - 1].trim();
            // 清理可能的前缀
            let clean_field_name = field_name
                .trim_start_matches("field: ")
                .trim_start_matches("field:")
                .trim();
            return self
                .get_field_position(clean_field_name)
                .map(|pos| pos as u64);
        }

        // 检查是否是形如 "fieldname_length" 的格式
        if trimmed_arg.ends_with("_length") {
            if let Some(field_name) = trimmed_arg.strip_suffix("_length") {
                // 尝试多种清理方式来找到字段
                let clean_field_name = field_name.trim();

                // 直接查找字段
                if let Ok(size) = self.get_field_size_by_name(clean_field_name) {
                    return Ok(size as u64);
                }

                // 清理常见前缀
                let prefixes = [
                    "field: ",
                    "field:",
                    "start: ",
                    "start:",
                    "end: ",
                    "end:",
                    "depends_on: ",
                    "depends_on:",
                ];
                for prefix in &prefixes {
                    if let Some(name_without_prefix) = clean_field_name.strip_prefix(prefix) {
                        let name = name_without_prefix.trim();
                        if let Ok(size) = self.get_field_size_by_name(name) {
                            return Ok(size as u64);
                        }
                    }
                }

                // 如果都没找到，尝试在field_values中查找已设置的字段
                if let Some(&index) = self.field_index.get(clean_field_name) {
                    if let Some(field) = self.fields.get(index) {
                        if let Some(value) = self.field_values.get(&field.field_id) {
                            return Ok(value.len() as u64);
                        }
                    }
                }

                // 尝试清理前缀后在field_values中查找
                for prefix in &prefixes {
                    if let Some(name_without_prefix) = clean_field_name.strip_prefix(prefix) {
                        let name = name_without_prefix.trim();
                        if let Some(&index) = self.field_index.get(name) {
                            if let Some(field) = self.fields.get(index) {
                                if let Some(value) = self.field_values.get(&field.field_id) {
                                    return Ok(value.len() as u64);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 如果都不是，返回错误
        Err(ProtocolError::InvalidExpression(format!(
            "Invalid length argument: {arg}"
        )))
    }
}
