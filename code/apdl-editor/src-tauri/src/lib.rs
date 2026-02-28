use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use jsonschema::{Draft, JSONSchema};

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

// 加载协议定义文件
#[tauri::command]
fn load_protocol(file_path: String) -> Result<serde_json::Value, String> {
    fs::read_to_string(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))
        .and_then(|content| {
            serde_json::from_str(&content)
                .map_err(|e| format!("解析JSON失败: {}", e))
        })
}

// 保存协议定义文件
#[tauri::command]
fn save_protocol(file_path: String, content: serde_json::Value) -> Result<(), String> {
    let json_str = serde_json::to_string_pretty(&content)
        .map_err(|e| format!("序列化JSON失败: {}", e))?;
    
    fs::write(&file_path, json_str)
        .map_err(|e| format!("写入文件失败: {}", e))
}

// 验证协议定义是否符合Schema
#[tauri::command]
fn validate_protocol(protocol: serde_json::Value) -> Result<ValidationResult, String> {
    // 加载Schema文件 - 使用相对于src-tauri目录的路径
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let schema_path = manifest_dir.join("schema/apdl-protocol-schema-v1.json");
    
    let schema_content = fs::read_to_string(&schema_path)
        .map_err(|e| format!("读取Schema失败 (路径: {}): {}", schema_path.display(), e))?;
    
    let schema_json: serde_json::Value = serde_json::from_str(&schema_content)
        .map_err(|e| format!("解析Schema失败: {}", e))?;
    
    // 编译Schema
    let schema = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema_json)
        .map_err(|e| format!("编译Schema失败: {}", e))?;
    
    // 验证并立即收集错误
    let validation_result = match schema.validate(&protocol) {
        Ok(_) => ValidationResult {
            valid: true,
            errors: vec![],
        },
        Err(errors) => {
            let error_list: Vec<ValidationError> = errors
                .map(|e| ValidationError {
                    path: e.instance_path.to_string(),
                    message: e.to_string(),
                })
                .collect();
            
            ValidationResult {
                valid: false,
                errors: error_list,
            }
        }
    };
    
    Ok(validation_result)
}

// 获取Schema定义（供前端使用）
#[tauri::command]
fn get_schema() -> Result<serde_json::Value, String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let schema_path = manifest_dir.join("schema/apdl-protocol-schema-v1.json");
    
    fs::read_to_string(&schema_path)
        .map_err(|e| format!("读取Schema失败 (路径: {}): {}", schema_path.display(), e))
        .and_then(|content| {
            serde_json::from_str(&content)
                .map_err(|e| format!("解析Schema失败: {}", e))
        })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_protocol,
            save_protocol,
            validate_protocol,
            get_schema
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
