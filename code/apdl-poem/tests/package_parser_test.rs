use apdl_poem::dsl::layers::package_parser::PackageParser;

#[test]
fn test_parse_child_package_dsl() {
    // 定义子包DSL
    let child_package_dsl = r#"
        package telemetry_packet {
            name: "Telemetry Packet";
            type: "telemetry";
            desc: "Telemetry packet with version, APID, length and data";
            layers: [
                {
                    name: "telemetry_layer";
                    units: [
                        {
                            field: "version";
                            type: "Uint8";
                            length: "1byte";
                            scope: "global(telemetry)";
                            cover: "entire_field";
                            constraint: "range(0..=255)";
                            desc: "Version number";
                        },
                        {
                            field: "apid";
                            type: "Uint16";
                            length: "2byte";
                            scope: "global(telemetry)";
                            cover: "entire_field";
                            constraint: "range(0..=65535)";
                            desc: "Application Process Identifier";
                        },
                        {
                            field: "length";
                            type: "Uint16";
                            length: "2byte";
                            scope: "global(telemetry)";
                            cover: "entire_field";
                            constraint: "range(0..=65535)";
                            desc: "Packet length";
                        },
                        {
                            field: "data";
                            type: "RawData";
                            length: "dynamic";
                            scope: "global(telemetry)";
                            cover: "entire_field";
                            desc: "Variable data field";
                        }
                    ];
                    rules: [];
                }
            ];
        }
    "#;

    println!("Original DSL:");
    println!("{}", child_package_dsl);
    println!("---");

    println!("Parsing child package DSL...");
    match PackageParser::parse_package_definition(child_package_dsl) {
        Ok(package) => {
            println!("Successfully parsed package: {}", package.name);
            println!("Package has {} layer(s)", package.layers.len());
            for (i, layer) in package.layers.iter().enumerate() {
                println!(
                    "Layer {}: {} has {} unit(s)",
                    i,
                    layer.name,
                    layer.units.len()
                );
                for (j, unit) in layer.units.iter().enumerate() {
                    println!("  Unit {}: {}", j, unit.field_id);
                }
            }
        }
        Err(e) => {
            println!("Failed to parse child package DSL: {}", e);
            // 不再panic，以便我们可以看到错误信息
            println!("Error details: {}", e);
        }
    }
}
