import { useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import Editor from "@monaco-editor/react";
import { ProtocolTree } from "./components/ProtocolTree";
import { VisualEditor } from "./components/VisualEditor";
import "./App.css";

interface ValidationError {
  path: string;
  message: string;
}

interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
}

type ViewMode = "json" | "visual";

function App() {
  const [editorValue, setEditorValue] = useState<string>("");
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);
  const [message, setMessage] = useState<string>("");
  const [selectedUnit, setSelectedUnit] = useState<any>(null);
  const [selectedField, setSelectedField] = useState<any>(null);
  const [viewMode, setViewMode] = useState<ViewMode>("json");

  // 解析协议数据供树形组件使用
  const protocolData = useMemo(() => {
    try {
      return editorValue ? JSON.parse(editorValue) : null;
    } catch {
      return null;
    }
  }, [editorValue]);

  // 加载协议定义
  async function handleLoad() {
    try {
      const filePath = await open({
        multiple: false,
        directory: false,
        filters: [
          { name: "JSON", extensions: ["json"] },
          { name: "All Files", extensions: ["*"] }
        ]
      });
      
      if (!filePath) return;

      const content = await invoke<any>("load_protocol", { filePath });
      setEditorValue(JSON.stringify(content, null, 2));
      setMessage(`成功加载: ${filePath}`);
    } catch (error) {
      setMessage(`加载失败: ${error}`);
    }
  }

  // 保存协议定义
  async function handleSave() {
    try {
      const filePath = await save({
        filters: [
          { name: "JSON", extensions: ["json"] },
          { name: "All Files", extensions: ["*"] }
        ]
      });
      
      if (!filePath) return;

      const content = JSON.parse(editorValue);
      await invoke("save_protocol", { filePath, content });
      setMessage(`成功保存: ${filePath}`);
    } catch (error) {
      setMessage(`保存失败: ${error}`);
    }
  }

  // 验证协议定义
  async function handleValidate() {
    try {
      // 检查编辑器内容
      if (!editorValue || editorValue.trim() === '') {
        setMessage("❌ 编辑器内容为空，请先加载或输入协议定义");
        return;
      }
      
      console.log("Editor content:", editorValue.substring(0, 100) + "...");
      
      const protocol = JSON.parse(editorValue);
      console.log("Parsed protocol:", protocol);
      
      const result = await invoke<ValidationResult>("validate_protocol", { protocol });
      console.log("Validation result:", result);
      
      setValidationResult(result);
      
      if (result.valid) {
        setMessage("✅ 验证通过！");
      } else {
        setMessage(`❌ 验证失败，发现 ${result.errors.length} 个错误`);
      }
    } catch (error) {
      console.error("Validation error:", error);
      setMessage(`验证失败: ${error}`);
    }
  }

  // 加载示例
  async function handleLoadExample() {
    try {
      const examplePath = "schema/examples/ccsds_tm_frame.json";
      const content = await invoke<any>("load_protocol", { filePath: examplePath });
      setEditorValue(JSON.stringify(content, null, 2));
      setMessage(`成功加载示例: ${examplePath}`);
    } catch (error) {
      setMessage(`加载示例失败: ${error}`);
    }
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100vh", padding: "10px" }}>
      <h1 style={{ margin: "0 0 10px 0" }}>APDL 协议编辑器</h1>
      
      <div style={{ marginBottom: "10px", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div>
          <button onClick={handleLoad} style={{ marginRight: "5px" }}>打开</button>
          <button onClick={handleSave} style={{ marginRight: "5px" }}>保存</button>
          <button onClick={handleValidate} style={{ marginRight: "5px" }}>验证</button>
          <button onClick={handleLoadExample}>加载示例</button>
        </div>
        <div>
          <button 
            onClick={() => setViewMode("json")}
            style={{ 
              marginRight: "5px",
              backgroundColor: viewMode === "json" ? "#1890ff" : "#fff",
              color: viewMode === "json" ? "#fff" : "#333",
              border: "1px solid #d9d9d9",
              padding: "5px 15px",
              cursor: "pointer"
            }}
          >
            JSON源码
          </button>
          <button 
            onClick={() => setViewMode("visual")}
            style={{ 
              backgroundColor: viewMode === "visual" ? "#1890ff" : "#fff",
              color: viewMode === "visual" ? "#fff" : "#333",
              border: "1px solid #d9d9d9",
              padding: "5px 15px",
              cursor: "pointer"
            }}
          >
            可视化编辑
          </button>
        </div>
      </div>

      {message && (
        <div style={{ 
          padding: "10px", 
          marginBottom: "10px", 
          backgroundColor: message.includes("失败") || message.includes("❌") ? "#ffebee" : "#e8f5e9",
          borderRadius: "4px" 
        }}>
          {message}
        </div>
      )}

      {validationResult && !validationResult.valid && (
        <div style={{ 
          padding: "10px", 
          marginBottom: "10px", 
          backgroundColor: "#fff3e0",
          borderRadius: "4px",
          maxHeight: "150px",
          overflow: "auto"
        }}>
          <h3 style={{ margin: "0 0 10px 0" }}>验证错误:</h3>
          {validationResult.errors.map((err, idx) => (
            <div key={idx} style={{ marginBottom: "5px" }}>
              <strong>{err.path || "根路径"}</strong>: {err.message}
            </div>
          ))}
        </div>
      )}

      <div style={{ flex: 1, display: "flex", overflow: "hidden" }}>
        {/* 左侧树形面板 */}
        <div style={{ width: "300px", minWidth: "250px", height: "100%" }}>
          <ProtocolTree
            protocol={protocolData}
            onSelectUnit={(unit) => {
              setSelectedUnit(unit);
              setSelectedField(null);
            }}
            onSelectField={(unit, field) => {
              setSelectedUnit(unit);
              setSelectedField(field);
            }}
          />
        </div>

        {/* 右侧区域：编辑器 + 详情面板 */}
        <div style={{ flex: 1, marginLeft: "10px", display: "flex", flexDirection: "column" }}>
          {/* 编辑器区域 - 根据viewMode切换 */}
          <div style={{ flex: 1, border: "1px solid #ccc", borderRadius: "4px", marginBottom: "10px", overflow: "hidden" }}>
            {viewMode === "json" ? (
              <Editor
                height="100%"
                defaultLanguage="json"
                value={editorValue}
                onChange={(value) => setEditorValue(value || "")}
                theme="vs-dark"
                options={{
                  minimap: { enabled: true },
                  fontSize: 14,
                  formatOnPaste: true,
                  formatOnType: true,
                }}
              />
            ) : (
              <VisualEditor
                protocol={protocolData}
                onProtocolChange={(newProtocol: any) => {
                  setEditorValue(JSON.stringify(newProtocol, null, 2));
                }}
              />
            )}
          </div>

          {/* 详情面板 - 仅在JSON视图显示 */}
          {viewMode === "json" && (selectedUnit || selectedField) && (
            <div style={{ 
              height: "200px", 
              border: "1px solid #ccc", 
              borderRadius: "4px",
              padding: "15px",
              backgroundColor: "#f8f9fa",
              overflow: "auto"
            }}>
              <h3 style={{ margin: "0 0 10px 0", fontSize: "14px", color: "#333" }}>
                {selectedField ? `字段: ${selectedField.field_name}` : `单元: ${selectedUnit?.unit_name}`}
              </h3>
              
              {selectedField ? (
                <div style={{ fontSize: "13px" }}>
                  <div style={{ marginBottom: "8px" }}>
                    <strong>类型:</strong> {selectedField.field_type}
                  </div>
                  {selectedField.bit_length && (
                    <div style={{ marginBottom: "8px" }}>
                      <strong>位长度:</strong> {selectedField.bit_length}
                    </div>
                  )}
                  {selectedField.byte_length && (
                    <div style={{ marginBottom: "8px" }}>
                      <strong>字节长度:</strong> {selectedField.byte_length}
                    </div>
                  )}
                  {selectedField.description && (
                    <div style={{ marginBottom: "8px" }}>
                      <strong>描述:</strong> {selectedField.description}
                    </div>
                  )}
                  {selectedField.default_value && (
                    <div style={{ marginBottom: "8px" }}>
                      <strong>默认值:</strong> {selectedField.default_value}
                    </div>
                  )}
                  <div style={{ marginTop: "10px", padding: "8px", backgroundColor: "#e3f2fd", borderRadius: "4px" }}>
                    <strong>所属单元:</strong> {selectedUnit?.unit_name}
                  </div>
                </div>
              ) : (
                <div style={{ fontSize: "13px" }}>
                  <div style={{ marginBottom: "8px" }}>
                    <strong>ID:</strong> {selectedUnit?.unit_id}
                  </div>
                  <div style={{ marginBottom: "8px" }}>
                    <strong>类型:</strong> {selectedUnit?.unit_type}
                  </div>
                  {selectedUnit?.description && (
                    <div style={{ marginBottom: "8px" }}>
                      <strong>描述:</strong> {selectedUnit?.description}
                    </div>
                  )}
                  {selectedUnit?.fields && (
                    <div style={{ marginBottom: "8px" }}>
                      <strong>字段数:</strong> {selectedUnit.fields.length}
                    </div>
                  )}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default App;
