//! 约束处理模块
//!
//! 处理字段约束（范围、固定值、枚举），确保生成的数据符合约束条件

use apdl_core::Constraint;

/// 约束处理器
pub struct ConstraintHandler;

impl ConstraintHandler {
    /// 根据约束生成符合要求的值
    ///
    /// # 参数
    /// - `constraints`: 约束条件列表
    /// - `default_value`: 默认生成值（无约束时使用）
    ///
    /// # 返回
    /// 符合约束的值
    pub fn apply_constraints(constraints: &[Constraint], default_value: u64) -> u64 {
        if constraints.is_empty() {
            return default_value;
        }

        // 优先处理固定值约束
        for constraint in constraints {
            if let Constraint::FixedValue(value) = constraint {
                return *value;
            }
        }

        // 处理范围约束
        for constraint in constraints {
            if let Constraint::Range(min, max) = constraint {
                // 将默认值限制在范围内
                return default_value.clamp(*min, *max);
            }
        }

        // 处理枚举约束
        for constraint in constraints {
            if let Constraint::Enum(values) = constraint {
                if !values.is_empty() {
                    // 返回第一个枚举值
                    return values[0].1;
                }
            }
        }

        default_value
    }

    /// 获取范围约束的边界
    ///
    /// # 返回
    /// - `Some((min, max))`: 如果存在范围约束
    /// - `None`: 如果不存在范围约束
    pub fn get_range(constraints: &[Constraint]) -> Option<(u64, u64)> {
        for constraint in constraints {
            if let Constraint::Range(min, max) = constraint {
                return Some((*min, *max));
            }
        }
        None
    }

    /// 获取固定值约束
    ///
    /// # 返回
    /// - `Some(value)`: 如果存在固定值约束
    /// - `None`: 如果不存在固定值约束
    pub fn get_fixed_value(constraints: &[Constraint]) -> Option<u64> {
        for constraint in constraints {
            if let Constraint::FixedValue(value) = constraint {
                return Some(*value);
            }
        }
        None
    }

    /// 获取枚举值列表
    ///
    /// # 返回
    /// - `Some(values)`: 如果存在枚举约束
    /// - `None`: 如果不存在枚举约束
    pub fn get_enum_values(constraints: &[Constraint]) -> Option<Vec<u64>> {
        for constraint in constraints {
            if let Constraint::Enum(entries) = constraint {
                let values: Vec<u64> = entries.iter().map(|(_, v)| *v).collect();
                return Some(values);
            }
        }
        None
    }
}

/// 约束验证器
pub struct ConstraintValidator;

impl ConstraintValidator {
    /// 验证值是否符合约束
    ///
    /// # 参数
    /// - `value`: 要验证的值
    /// - `constraints`: 约束条件列表
    ///
    /// # 返回
    /// - `true`: 值符合所有约束
    /// - `false`: 值不符合至少一个约束
    pub fn validate(value: u64, constraints: &[Constraint]) -> bool {
        for constraint in constraints {
            if !Self::validate_single(value, constraint) {
                return false;
            }
        }
        true
    }

    /// 验证单个约束
    fn validate_single(value: u64, constraint: &Constraint) -> bool {
        match constraint {
            Constraint::Range(min, max) => value >= *min && value <= *max,
            Constraint::FixedValue(expected) => value == *expected,
            Constraint::Enum(entries) => entries.iter().any(|(_, v)| *v == value),
            Constraint::Custom(_) => {
                // 自定义约束暂不验证，返回true
                true
            }
        }
    }

    /// 获取约束描述字符串
    pub fn describe_constraint(constraint: &Constraint) -> String {
        match constraint {
            Constraint::Range(min, max) => format!("范围 [{}..={}]", min, max),
            Constraint::FixedValue(value) => format!("固定值 {}", value),
            Constraint::Enum(entries) => {
                let values: Vec<String> = entries.iter().map(|(n, v)| format!("{}={}", n, v)).collect();
                format!("枚举 [{}]", values.join(", "))
            }
            Constraint::Custom(expr) => format!("自定义: {}", expr),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_range_constraint() {
        let constraints = vec![Constraint::Range(0, 100)];
        
        // 默认值在范围内
        assert_eq!(ConstraintHandler::apply_constraints(&constraints, 50), 50);
        
        // 默认值超出范围，应该被限制
        assert_eq!(ConstraintHandler::apply_constraints(&constraints, 150), 100);
        assert_eq!(ConstraintHandler::apply_constraints(&constraints, 0), 0);
    }

    #[test]
    fn test_apply_fixed_value_constraint() {
        let constraints = vec![Constraint::FixedValue(42)];
        
        // 固定值约束优先
        assert_eq!(ConstraintHandler::apply_constraints(&constraints, 100), 42);
        assert_eq!(ConstraintHandler::apply_constraints(&constraints, 0), 42);
    }

    #[test]
    fn test_apply_enum_constraint() {
        let constraints = vec![Constraint::Enum(vec![
            ("A".to_string(), 1),
            ("B".to_string(), 2),
            ("C".to_string(), 3),
        ])];
        
        // 返回第一个枚举值
        assert_eq!(ConstraintHandler::apply_constraints(&constraints, 100), 1);
    }

    #[test]
    fn test_constraint_priority() {
        // 固定值约束应该优先于范围约束
        let constraints = vec![
            Constraint::Range(0, 10),
            Constraint::FixedValue(5),
        ];
        
        assert_eq!(ConstraintHandler::apply_constraints(&constraints, 100), 5);
    }

    #[test]
    fn test_validate_range() {
        let constraints = vec![Constraint::Range(10, 100)];
        
        assert!(ConstraintValidator::validate(50, &constraints));
        assert!(ConstraintValidator::validate(10, &constraints));
        assert!(ConstraintValidator::validate(100, &constraints));
        assert!(!ConstraintValidator::validate(5, &constraints));
        assert!(!ConstraintValidator::validate(101, &constraints));
    }

    #[test]
    fn test_validate_fixed_value() {
        let constraints = vec![Constraint::FixedValue(42)];
        
        assert!(ConstraintValidator::validate(42, &constraints));
        assert!(!ConstraintValidator::validate(41, &constraints));
        assert!(!ConstraintValidator::validate(43, &constraints));
    }

    #[test]
    fn test_validate_enum() {
        let constraints = vec![Constraint::Enum(vec![
            ("Red".to_string(), 1),
            ("Green".to_string(), 2),
            ("Blue".to_string(), 3),
        ])];
        
        assert!(ConstraintValidator::validate(1, &constraints));
        assert!(ConstraintValidator::validate(2, &constraints));
        assert!(ConstraintValidator::validate(3, &constraints));
        assert!(!ConstraintValidator::validate(4, &constraints));
    }

    #[test]
    fn test_get_range() {
        let constraints = vec![Constraint::Range(10, 100)];
        
        assert_eq!(ConstraintHandler::get_range(&constraints), Some((10, 100)));
    }

    #[test]
    fn test_get_fixed_value() {
        let constraints = vec![Constraint::FixedValue(42)];
        
        assert_eq!(ConstraintHandler::get_fixed_value(&constraints), Some(42));
    }

    #[test]
    fn test_get_enum_values() {
        let constraints = vec![Constraint::Enum(vec![
            ("A".to_string(), 1),
            ("B".to_string(), 2),
        ])];
        
        assert_eq!(ConstraintHandler::get_enum_values(&constraints), Some(vec![1, 2]));
    }

    #[test]
    fn test_describe_constraint() {
        let range = Constraint::Range(0, 100);
        assert_eq!(ConstraintValidator::describe_constraint(&range), "范围 [0..=100]");

        let fixed = Constraint::FixedValue(42);
        assert_eq!(ConstraintValidator::describe_constraint(&fixed), "固定值 42");

        let enum_constraint = Constraint::Enum(vec![
            ("A".to_string(), 1),
            ("B".to_string(), 2),
        ]);
        assert_eq!(ConstraintValidator::describe_constraint(&enum_constraint), "枚举 [A=1, B=2]");
    }
}
