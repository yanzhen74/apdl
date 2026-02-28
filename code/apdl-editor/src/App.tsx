import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Editor from "@monaco-editor/react";
import "./App.css";

interface ValidationError {
  path: string;
  message: string;
}

interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
}

function App() {
  const [editorValue, setEditorValue] = useState<string>("");
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);
  const [message, setMessage] = useState<string>("");

  // 加载协议定义
  async function handleLoad() {
    try {
      // TODO: 实现文件选择对话框
      const filePath = prompt("请输入协议定义文件路径:");
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
      const filePath = prompt("请输入保存路径:");
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
      
      <div style={{ marginBottom: "10px" }}>
        <button onClick={handleLoad} style={{ marginRight: "5px" }}>打开</button>
        <button onClick={handleSave} style={{ marginRight: "5px" }}>保存</button>
        <button onClick={handleValidate} style={{ marginRight: "5px" }}>验证</button>
        <button onClick={handleLoadExample}>加载示例</button>
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

      <div style={{ flex: 1, border: "1px solid #ccc", borderRadius: "4px" }}>
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
      </div>
    </div>
  );
}

export default App;
