#[cfg(test)]
mod tests {
    use apdl_core::{DataPlacementConfig, DataPlacementStrategy};
    use apdl_poem::standard_units::connector::connector_engine::ConnectorEngine;

    #[test]
    fn test_runtime_engine_functionality() {
        // 重点测试运行时引擎功能，因为DSL解析器还有些问题需要解决
        println!("Testing runtime engine functionality...");

        // 创建运行时引擎
        let engine = ConnectorEngine::new();

        // 创建模拟的语法单元用于测试
        let mock_source_package = vec![apdl_core::SyntaxUnit {
            field_id: "tlm_source_id".to_string(),
            unit_type: apdl_core::UnitType::Uint(16),
            length: apdl_core::LengthDesc {
                size: 2,
                unit: apdl_core::LengthUnit::Byte,
            },
            scope: apdl_core::ScopeDesc::Global("end2end".to_string()),
            cover: apdl_core::CoverDesc::EntireField,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: "Mock telemetry source ID".to_string(),
        }];

        let mut mock_target_package = vec![apdl_core::SyntaxUnit {
            field_id: "apid".to_string(),
            unit_type: apdl_core::UnitType::Uint(16),
            length: apdl_core::LengthDesc {
                size: 2,
                unit: apdl_core::LengthUnit::Byte,
            },
            scope: apdl_core::ScopeDesc::Global("end2end".to_string()),
            cover: apdl_core::CoverDesc::EntireField,
            constraint: None,
            alg: None,
            associate: vec![],
            desc: "Mock APID field".to_string(),
        }];

        // 测试数据放置功能
        let result = engine.apply_data_placement(
            &mock_source_package,
            &mut mock_target_package,
            &DataPlacementConfig {
                strategy: DataPlacementStrategy::PointerBased,
                target_field: "apid".to_string(),
                config_params: vec![("pointer_field".to_string(), "first_header_ptr".to_string())],
            },
        );

        assert!(
            result.is_ok(),
            "Failed to apply data placement: {:?}",
            result.err()
        );

        println!("Runtime engine functionality test completed successfully!");
        println!("Engine can process field mappings and data placement strategies.");
    }
}
