//! 依赖规则处理器
//!
//! 处理字段间的依赖关系规则

use apdl_core::ProtocolError;

use crate::standard_units::frame_assembler::core::FrameAssembler;

impl FrameAssembler {
    /// 应用依赖规则
    pub fn apply_dependency_rule(
        &mut self,
        dependent_field: &str,
        dependency_field: &str,
    ) -> Result<(), ProtocolError> {
        // 验证依赖关系是否存在
        if !self.field_index.contains_key(dependent_field)
            || !self.field_index.contains_key(dependency_field)
        {
            return Err(ProtocolError::FieldNotFound(format!(
                "Dependent or dependency field not found: {dependent_field} or {dependency_field}"
            )));
        }
        println!(
            "Applied dependency rule: {dependent_field} depends on {dependency_field}"
        );
        Ok(())
    }
}
