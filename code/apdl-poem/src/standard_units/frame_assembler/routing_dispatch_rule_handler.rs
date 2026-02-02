//! 路由分发规则处理器
//!
//! 处理路由分发相关的语义规则

use apdl_core::ProtocolError;

use crate::standard_units::frame_assembler::core::FrameAssembler;

impl FrameAssembler {
    /// 应用路由分发规则
    pub fn apply_routing_dispatch_rule(
        &mut self,
        fields: &[String],
        algorithm: &str,
        description: &str,
        _frame_data: &mut [u8],
    ) -> Result<(), ProtocolError> {
        println!("Applying routing dispatch rule: {description} with algorithm {algorithm}");

        // 根据字段值计算路由信息
        for field_name in fields {
            if let Ok(field_value) = self.get_field_value(field_name) {
                // 根据算法计算路由值
                let route_value = match algorithm {
                    "hash_sync_to_route" => self.hash_field_value(&field_value),
                    "hash_apid_to_route" => self.hash_field_value(&field_value),
                    "hash_vc_to_route" => self.hash_field_value(&field_value),
                    _ => self.hash_field_value(&field_value), // 默认使用哈希算法
                };

                println!("Field {field_name}: value={field_value:?}, route_value={route_value}");
            }
        }

        Ok(())
    }

    /// 计算字段值的哈希
    fn hash_field_value(&self, field_value: &[u8]) -> u64 {
        let mut hash: u64 = 5381;
        for &byte in field_value {
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u64);
        }
        hash
    }
}
