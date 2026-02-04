//! 连接器引擎实现
//!
//! 负责执行字段映射规则，将源包的字段值映射到目标包的字段

use apdl_core::{
    DataPlacementConfig, DataPlacementStrategy, FieldMappingEntry, SemanticRule, SyntaxUnit,
};

use crate::standard_units::connector::mpdu_manager::MpduManager;
use crate::standard_units::frame_assembler::core::FrameAssembler;

/// 连接器引擎
pub struct ConnectorEngine {
    /// 映射规则集合
    mapping_rules: Vec<apdl_core::SemanticRule>,
    /// MPDU管理器
    mpdu_manager: MpduManager,
}

impl ConnectorEngine {
    /// 创建新的连接器引擎
    pub fn new() -> Self {
        Self {
            mapping_rules: Vec::new(),
            mpdu_manager: MpduManager::new(),
        }
    }

    /// 添加映射规则
    pub fn add_mapping_rule(&mut self, rule: SemanticRule) {
        if let SemanticRule::FieldMapping { .. } = rule {
            self.mapping_rules.push(rule);
        }
    }

    /// 应用映射规则
    pub fn apply_mapping_rules(
        &self,
        source_package: &[SyntaxUnit],
        target_package: &mut [SyntaxUnit],
    ) -> Result<(), Box<dyn std::error::Error>> {
        for rule in &self.mapping_rules {
            if let SemanticRule::FieldMapping {
                source_package: _source_pkg_name,
                target_package: _target_pkg_name,
                mappings,
                description: _,
            } = rule
            {
                // 检查包名称是否匹配（简化实现，实际中可能需要更复杂的匹配逻辑）
                self.apply_single_mapping(source_package, target_package, mappings)?;
            }
        }
        Ok(())
    }

    /// 应用单个映射规则
    fn apply_single_mapping(
        &self,
        source_package: &[SyntaxUnit],
        target_package: &mut [SyntaxUnit],
        mappings: &[FieldMappingEntry],
    ) -> Result<(), Box<dyn std::error::Error>> {
        for mapping_entry in mappings {
            let source_field_name = &mapping_entry.source_field;
            let target_field_name = &mapping_entry.target_field;
            let mapping_logic = &mapping_entry.mapping_logic;

            // 在源包中查找源字段
            if let Some(source_field) = source_package
                .iter()
                .find(|f| f.field_id == *source_field_name)
            {
                // 在目标包中查找目标字段
                if let Some(target_idx) = target_package
                    .iter()
                    .position(|f| f.field_id == *target_field_name)
                {
                    // 获取源字段的值（这里简化为假设值是从某个地方来的）
                    let source_value = self.get_field_value(source_field)?;

                    // 应用映射逻辑
                    let mapped_value = self.apply_mapping_logic(
                        &source_value,
                        mapping_logic,
                        &mapping_entry.default_value,
                    )?;

                    // 设置目标字段的值（通过更新target_package中的相应字段）
                    self.set_field_value(&mut target_package[target_idx], &mapped_value)?;
                }
            }
        }
        Ok(())
    }

    /// 执行数据放置策略
    pub fn apply_data_placement(
        &self,
        source_package: &[SyntaxUnit],
        target_package: &mut [SyntaxUnit],
        placement_config: &DataPlacementConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &placement_config.strategy {
            DataPlacementStrategy::Direct => {
                self.apply_direct_placement(source_package, target_package, placement_config)?;
            }
            DataPlacementStrategy::PointerBased => {
                self.apply_pointer_based_placement(
                    source_package,
                    target_package,
                    placement_config,
                )?;
            }
            DataPlacementStrategy::StreamBased => {
                self.apply_stream_based_placement(
                    source_package,
                    target_package,
                    placement_config,
                )?;
            }
            DataPlacementStrategy::Custom(strategy_name) => {
                self.apply_custom_placement(
                    source_package,
                    target_package,
                    placement_config,
                    strategy_name,
                )?;
            }
        }
        Ok(())
    }

    /// 直接放置策略
    fn apply_direct_placement(
        &self,
        source_package: &[SyntaxUnit],
        target_package: &mut [SyntaxUnit],
        placement_config: &DataPlacementConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 将源包数据直接放置到目标包的指定字段
        let target_field_name = &placement_config.target_field;

        if let Some(target_idx) = target_package
            .iter()
            .position(|f| f.field_id == *target_field_name)
        {
            // 这里简化实现，实际上需要将整个源包数据复制到目标字段
            let source_data = self.extract_package_data(source_package)?;
            self.set_field_value(&mut target_package[target_idx], &source_data)?;
        }

        Ok(())
    }

    /// 指针基于放置策略 - MPDU（多路协议数据单元）方式
    /// 根据CCSDS标准实现MPDU的首导头指针机制
    /// MPDU由MPDU导头（2字节）和MPDU包区（可变长度）组成
    /// 首导头指针指向MPDU包区中第一个完整包的第一个字节位置
    fn apply_pointer_based_placement(
        &self,
        source_package: &[SyntaxUnit],
        target_package: &mut [SyntaxUnit],
        placement_config: &DataPlacementConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 获取目标字段名
        let target_field_name = &placement_config.target_field;

        // 获取源包数据（将要放入MPDU包区的内容）
        let source_data = self.extract_package_data(source_package)?;

        if let Some(target_idx) = target_package
            .iter()
            .position(|f| f.field_id == *target_field_name)
        {
            // 检查目标字段是否已有数据
            let existing_data_len = 0; // 在当前实现中，我们先假定目标字段是空的

            // 计算新包的起始位置
            let new_packet_start_pos = existing_data_len;

            // 将源包数据追加到目标字段
            // 在实际的FrameAssembler实现中，这会将数据追加到现有的数据中
            // 这里我们只是模拟这种行为

            // 如果配置中指定了指针字段，则更新该字段的值
            if let Some(pointer_field_name) = placement_config
                .config_params
                .iter()
                .find(|(key, _)| key == "pointer_field")
                .map(|(_, value)| value.as_str())
            {
                // 查找指针字段索引
                if let Some(pointer_idx) = target_package
                    .iter()
                    .position(|f| f.field_id == pointer_field_name)
                {
                    // 根据CCSDS标准，首导头指针指向MPDU包区中第一个完整包的第一个字节位置
                    // 在流式数据处理中，第一个包总是位于偏移0
                    let first_packet_pointer = 0;

                    // 将指针值转换为2字节（CCSDS标准中指针为2字节）
                    let pointer_bytes = (first_packet_pointer as u16).to_be_bytes().to_vec();

                    self.set_field_value(&mut target_package[pointer_idx], &pointer_bytes)?;
                    println!(
                        "Set MPDU pointer field '{}' to offset: {} (points to first packet)",
                        pointer_field_name, first_packet_pointer
                    );
                }
            }

            // 实际上，这里需要FrameAssembler来处理数据追加
            // 但现在我们只是记录操作
            println!(
                "Would apply MPDU pointer-based placement: placed {} bytes into field '{}' (new packet at pos {})",
                source_data.len(),
                target_field_name,
                new_packet_start_pos
            );
        }

        Ok(())
    }

    /// 数据流放置策略
    fn apply_stream_based_placement(
        &self,
        source_package: &[SyntaxUnit],
        target_package: &mut [SyntaxUnit],
        placement_config: &DataPlacementConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 将源包数据按流形式放置到目标包
        let target_field_name = &placement_config.target_field;

        if let Some(target_idx) = target_package
            .iter()
            .position(|f| f.field_id == *target_field_name)
        {
            let stream_data = self.extract_package_data_as_stream(source_package)?;
            self.set_field_value(&mut target_package[target_idx], &stream_data)?;
        }

        Ok(())
    }

    /// 自定义放置策略
    fn apply_custom_placement(
        &self,
        source_package: &[SyntaxUnit],
        target_package: &mut [SyntaxUnit],
        placement_config: &DataPlacementConfig,
        strategy_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 根据自定义策略名称应用特定逻辑
        println!("Applying custom placement strategy: {}", strategy_name);

        let target_field_name = &placement_config.target_field;

        if let Some(target_idx) = target_package
            .iter()
            .position(|f| f.field_id == *target_field_name)
        {
            let custom_data = self.process_custom_placement(source_package, strategy_name)?;
            self.set_field_value(&mut target_package[target_idx], &custom_data)?;
        }

        Ok(())
    }

    /// 提取包数据
    fn extract_package_data(
        &self,
        package: &[SyntaxUnit],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // 简化的数据提取实现
        // 在实际实现中，这会根据协议格式将所有字段数据拼接起来
        let mut result = Vec::new();
        for field in package {
            let field_data = self.get_field_value(field)?;
            result.extend_from_slice(&field_data);
        }
        Ok(result)
    }

    /// 提取包数据作为数据流
    fn extract_package_data_as_stream(
        &self,
        package: &[SyntaxUnit],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // 与普通提取类似，但可能应用额外的流处理逻辑
        self.extract_package_data(package)
    }

    /// 生成指针值
    fn generate_pointer_value(
        &self,
        package: &[SyntaxUnit],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // 生成指向数据的指针值（这里简化为哈希值）
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let data = self.extract_package_data(package)?;
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let hash_value = hasher.finish();

        Ok(hash_value.to_be_bytes().to_vec())
    }

    /// 处理自定义放置策略
    fn process_custom_placement(
        &self,
        package: &[SyntaxUnit],
        strategy: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // 根据策略名称执行特定处理
        match strategy {
            "complex_routing" => {
                // 复杂路由策略
                self.extract_package_data(package)
            }
            _ => {
                // 默认为直接提取数据
                self.extract_package_data(package)
            }
        }
    }

    /// 获取字段值（简化实现）
    fn get_field_value(&self, _field: &SyntaxUnit) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // 在实际实现中，这里会从实际的数据中获取字段值
        // 这里返回一个示例值
        Ok(vec![0x01, 0x02]) // 示例值
    }

    /// 应用映射逻辑
    fn apply_mapping_logic(
        &self,
        source_value: &[u8],
        mapping_logic: &str,
        default_value: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match mapping_logic {
            "identity" => Ok(source_value.to_vec()),
            "hash_mod_64" => {
                // 简单的哈希实现
                let hash_value = self.simple_hash(source_value);
                let result = hash_value % 64;
                Ok(vec![(result & 0xFF) as u8])
            }
            "hash_mod_2048" => {
                // 用于APID的哈希实现
                let hash_value = self.simple_hash(source_value);
                let result = hash_value % 2048;
                Ok(vec![((result >> 8) & 0xFF) as u8, (result & 0xFF) as u8])
            }
            _ => {
                // 如果映射逻辑无法识别，使用默认值
                self.parse_default_value(default_value)
            }
        }
    }

    /// 简单的哈希函数
    fn simple_hash(&self, data: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    /// 解析默认值
    fn parse_default_value(
        &self,
        default_value: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if let Some(hex_str) = default_value.strip_prefix("0x") {
            let value = u64::from_str_radix(hex_str, 16)
                .map_err(|_| format!("Invalid hex value: {default_value}"))?;
            Ok(value.to_be_bytes().to_vec())
        } else {
            let value = default_value
                .parse::<u64>()
                .map_err(|_| format!("Invalid decimal value: {default_value}"))?;
            Ok(value.to_be_bytes().to_vec())
        }
    }

    /// 设置字段值
    fn set_field_value(
        &self,
        field: &mut SyntaxUnit,
        value: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 在实际实现中，这里会更新字段的值
        // 由于SyntaxUnit是不可变的，我们需要一个不同的方法来更新值
        // 这里只是示意
        println!("Setting field {} to value {:?}", field.field_id, value);
        Ok(())
    }

    /// 执行完整的连接操作，包括字段映射和数据放置
    pub fn connect(
        &mut self,
        source_assembler: &mut FrameAssembler,
        target_assembler: &mut FrameAssembler,
        connector_config: &apdl_core::ConnectorConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. 应用字段映射
        for mapping in &connector_config.mappings {
            // 获取源字段值
            if let Ok(source_value) = source_assembler.get_field_value(&mapping.source_field) {
                // 设置目标字段值
                target_assembler
                    .set_field_value(&mapping.target_field, &source_value)
                    .map_err(|e| Box::new(e))?;
                println!(
                    "Mapped {} to {} with value {:?}",
                    mapping.source_field, mapping.target_field, source_value
                );
            }
        }

        // 2. 应用数据放置策略
        if let Some(data_placement) = &connector_config.data_placement {
            println!(
                "Applying data placement strategy: {:?}",
                data_placement.strategy
            );

            match data_placement.strategy {
                DataPlacementStrategy::PointerBased => {
                    // 对于MPDU模式，使用MPDU管理器进行处理
                    // 将源包添加到相应的队列中
                    let source_frame =
                        source_assembler.assemble_frame().map_err(|e| Box::new(e))?;

                    // 使用目标包类型作为队列标识符
                    let parent_type = &data_placement.target_field; // 使用目标字段名作为父包类型标识
                    self.mpdu_manager
                        .add_child_packet(parent_type, source_frame.clone());

                    println!(
                        "Added source frame ({} bytes) to MPDU queue for type '{}'",
                        source_frame.len(),
                        parent_type
                    );

                    // 如果需要立即构建MPDU包（在某些场景下），可以从队列中构建
                    // 这里我们只是将子包加入队列，实际构建在build_mpdu_packet方法中进行
                }
                _ => {
                    // 其他策略的通用处理
                    let source_frame =
                        source_assembler.assemble_frame().map_err(|e| Box::new(e))?;

                    // 将源包数据嵌入到目标包的数据字段
                    target_assembler
                        .set_field_value(&data_placement.target_field, &source_frame)
                        .map_err(|e| Box::new(e))?;
                    println!(
                        "Embedded source frame ({} bytes) into target data field",
                        source_frame.len()
                    );
                }
            }
        }

        Ok(())
    }

    /// 添加父包模板到MPDU管理器
    pub fn add_parent_template(&mut self, parent_type: &str, template: FrameAssembler) {
        self.mpdu_manager.add_parent_template(parent_type, template);
    }

    /// 构建MPDU包 - 从子包队列中取出数据填充到父包
    pub fn build_mpdu_packet(
        &mut self,
        parent_type: &str,
        mpdu_config: &DataPlacementConfig,
    ) -> Option<Vec<u8>> {
        self.mpdu_manager
            .build_mpdu_packet(parent_type, mpdu_config)
    }

    /// 获取指定类型子包队列的长度
    pub fn get_child_queue_length(&self, parent_type: &str) -> usize {
        self.mpdu_manager.get_child_queue_length(parent_type)
    }

    /// 获取指定类型父包模板队列的长度
    pub fn get_parent_queue_length(&self, parent_type: &str) -> usize {
        self.mpdu_manager.get_parent_queue_length(parent_type)
    }

    /// MPDU（多路协议数据单元）处理 - 生成CCSDS标准的首导头指针
    /// 支持真正的流式存储功能，其中连续的子包可以被添加到父包中
    pub fn generate_mpdu_with_pointers(
        &self,
        source_assemblers: &mut [FrameAssembler],
        target_assembler: &mut FrameAssembler,
        mpdu_config: &DataPlacementConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. 组装所有源包（形成MPDU包区的内容）
        let mut mpdu_data = Vec::new();
        let mut packet_offsets = Vec::new();
        let mut current_offset = 0;

        for source_asm in source_assemblers {
            let packet_data = source_asm.assemble_frame().map_err(|e| Box::new(e))?;
            packet_offsets.push(current_offset);
            mpdu_data.extend_from_slice(&packet_data);
            current_offset += packet_data.len();
        }

        // 2. 设置MPDU数据到目标包（作为MPDU包区）
        target_assembler
            .set_field_value(&mpdu_config.target_field, &mpdu_data)
            .map_err(|e| Box::new(e))?;

        // 3. 如果配置了指针字段，则设置CCSDS标准的首导头指针
        // 根据CCSDS标准，首导头指针指向MPDU包区中第一个完整包的第一个字节位置
        if let Some(pointer_field_name) = mpdu_config
            .config_params
            .iter()
            .find(|(key, _)| key == "pointer_field")
            .map(|(_, value)| value.as_str())
        {
            if !packet_offsets.is_empty() {
                // 设置第一个完整包的偏移量（CCSDS标准首导头指针）
                // 在CCSDS标准中，首导头指针指向第一个完整包的位置
                let first_packet_offset = packet_offsets[0] as u16; // 总是0
                let pointer_bytes = first_packet_offset.to_be_bytes().to_vec(); // 2字节指针字段
                target_assembler
                    .set_field_value(pointer_field_name, &pointer_bytes)
                    .map_err(|e| Box::new(e))?;
                println!(
                    "Set CCSDS MPDU first header pointer to offset: {} (first packet)",
                    first_packet_offset
                );
            }
        }

        // 4. 如果配置了多个指针字段，可以设置多个子包的指针（用于复杂MPDU场景）
        if let Some(packet_pointers_field_name) = mpdu_config
            .config_params
            .iter()
            .find(|(key, _)| key == "packet_pointers_field")
            .map(|(_, value)| value.as_str())
        {
            // 将所有子包偏移量打包到一个指针字段中（用于接收端重组）
            let mut all_pointers = Vec::new();
            for &offset in &packet_offsets {
                let ptr_bytes = (offset as u16).to_be_bytes().to_vec(); // 2字节指针
                all_pointers.extend_from_slice(&ptr_bytes);
            }
            target_assembler
                .set_field_value(packet_pointers_field_name, &all_pointers)
                .map_err(|e| Box::new(e))?;
            println!(
                "Set CCSDS MPDU packet pointers for {} packets",
                packet_offsets.len()
            );
        }

        Ok(())
    }
}

impl Default for ConnectorEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connector_engine_creation() {
        let engine = ConnectorEngine::new();
        assert_eq!(engine.mapping_rules.len(), 0);
    }
}
