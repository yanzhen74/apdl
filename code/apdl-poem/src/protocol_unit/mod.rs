//! 协议单元接口定义
//!
//! 定义协议单元的基本接口，支持字段级语法单元

use apdl_core::ProtocolUnit;

/// 协议单元管理器
pub struct ProtocolUnitManager {
    units: std::collections::HashMap<String, Box<dyn ProtocolUnit>>,
}

impl Default for ProtocolUnitManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolUnitManager {
    pub fn new() -> Self {
        Self {
            units: std::collections::HashMap::new(),
        }
    }

    pub fn register_unit(&mut self, id: String, unit: Box<dyn ProtocolUnit>) {
        self.units.insert(id, unit);
    }

    pub fn get_unit(&self, id: &str) -> Option<&dyn ProtocolUnit> {
        self.units.get(id).map(|boxed| &**boxed)
    }
}
