//! 顺序规则处理器
//!
//! 处理字段间的顺序关系规则

use apdl_core::ProtocolError;

use crate::standard_units::frame_assembler::core::FrameAssembler;

impl FrameAssembler {
    /// 应用顺序规则
    pub fn apply_order_rule(
        &mut self,
        first_field: &str,
        second_field: &str,
    ) -> Result<(), ProtocolError> {
        // 验证字段顺序是否正确
        let first_pos = self.get_field_position(first_field)?;
        let second_pos = self.get_field_position(second_field)?;

        if first_pos > second_pos {
            return Err(ProtocolError::InvalidFrameFormat(format!(
                "Field order violation: {} should come before {}",
                first_field, second_field
            )));
        }
        println!(
            "Applied order rule: {} before {}",
            first_field, second_field
        );
        Ok(())
    }
}
