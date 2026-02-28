import { useState } from 'react';

interface VisualEditorProps {
  protocol: any;
  onProtocolChange: (protocol: any) => void;
}

type VisualTab = 'relation' | 'frame' | 'packet' | 'mapping';

export function VisualEditor({ protocol, onProtocolChange }: VisualEditorProps) {
  const [activeTab, setActiveTab] = useState<VisualTab>('relation');

  if (!protocol) {
    return (
      <div style={{ 
        height: '100%', 
        display: 'flex', 
        alignItems: 'center', 
        justifyContent: 'center',
        color: '#666'
      }}>
        请加载协议定义文件以使用可视化编辑器
      </div>
    );
  }

  const tabs: { id: VisualTab; label: string }[] = [
    { id: 'relation', label: '关系图' },
    { id: 'frame', label: '帧' },
    { id: 'packet', label: '包' },
    { id: 'mapping', label: '字段映射' },
  ];

  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Tab导航 */}
      <div style={{ 
        display: 'flex', 
        borderBottom: '1px solid #ddd',
        backgroundColor: '#f5f5f5'
      }}>
        {tabs.map(tab => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            style={{
              padding: '10px 20px',
              border: 'none',
              backgroundColor: activeTab === tab.id ? '#fff' : 'transparent',
              borderBottom: activeTab === tab.id ? '2px solid #1890ff' : 'none',
              cursor: 'pointer',
              fontSize: '14px',
              color: activeTab === tab.id ? '#1890ff' : '#666'
            }}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab内容区域 */}
      <div style={{ flex: 1, overflow: 'auto', padding: '20px' }}>
        {activeTab === 'relation' && (
          <RelationGraph protocol={protocol} />
        )}
        {activeTab === 'frame' && (
          <FrameEditor protocol={protocol} onChange={onProtocolChange} />
        )}
        {activeTab === 'packet' && (
          <PacketEditor protocol={protocol} onChange={onProtocolChange} />
        )}
        {activeTab === 'mapping' && (
          <MappingEditor protocol={protocol} onChange={onProtocolChange} />
        )}
      </div>
    </div>
  );
}

// 关系图组件
function RelationGraph({ protocol }: { protocol: any }) {
  return (
    <div>
      <h3>协议关系总图</h3>
      <div style={{ 
        padding: '20px', 
        backgroundColor: '#f8f9fa',
        borderRadius: '8px',
        minHeight: '300px'
      }}>
        <p>协议名称: {protocol.protocol_name}</p>
        <p>语法单元数: {protocol.syntax_units?.length || 0}</p>
        <p>帧包关系数: {protocol.frame_packet_relationships?.length || 0}</p>
        <p>字段映射数: {protocol.field_mappings?.length || 0}</p>
        
        {/* TODO: 实现图形化关系图 */}
        <div style={{ marginTop: '20px', color: '#999' }}>
          [关系图可视化区域 - 开发中]
        </div>
      </div>
    </div>
  );
}

// 帧编辑器组件
function FrameEditor({ protocol, onChange }: { protocol: any; onChange: (p: any) => void }) {
  const frames = protocol.syntax_units?.filter((u: any) => u.unit_type === 'frame') || [];
  
  return (
    <div>
      <h3>帧定义编辑</h3>
      {frames.length === 0 ? (
        <p>暂无帧定义</p>
      ) : (
        frames.map((frame: any) => (
          <div key={frame.unit_id} style={{ marginBottom: '20px' }}>
            <h4>{frame.unit_name} ({frame.unit_id})</h4>
            {/* TODO: 实现帧字段编辑表 */}
            <div style={{ 
              padding: '15px', 
              backgroundColor: '#f8f9fa',
              borderRadius: '8px',
              color: '#999'
            }}>
              [帧字段编辑表 - 开发中]
              <br />
              字段数: {frame.fields?.length || 0}
            </div>
          </div>
        ))
      )}
    </div>
  );
}

// 包编辑器组件
function PacketEditor({ protocol, onChange }: { protocol: any; onChange: (p: any) => void }) {
  const packets = protocol.syntax_units?.filter((u: any) => u.unit_type === 'packet') || [];
  
  return (
    <div>
      <h3>包定义编辑</h3>
      {packets.length === 0 ? (
        <p>暂无包定义</p>
      ) : (
        packets.map((packet: any) => (
          <div key={packet.unit_id} style={{ marginBottom: '20px' }}>
            <h4>{packet.unit_name} ({packet.unit_id})</h4>
            {/* TODO: 实现包字段编辑表 */}
            <div style={{ 
              padding: '15px', 
              backgroundColor: '#f8f9fa',
              borderRadius: '8px',
              color: '#999'
            }}>
              [包字段编辑表 - 开发中]
              <br />
              字段数: {packet.fields?.length || 0}
            </div>
          </div>
        ))
      )}
    </div>
  );
}

// 字段映射编辑器组件
function MappingEditor({ protocol, onChange }: { protocol: any; onChange: (p: any) => void }) {
  return (
    <div>
      <h3>字段映射编辑</h3>
      {protocol.frame_packet_relationships?.map((rel: any, index: number) => (
        <div key={index} style={{ marginBottom: '20px' }}>
          <h4>{rel.parent_frame} → {rel.child_packet}</h4>
          {/* TODO: 实现字段映射编辑表 */}
          <div style={{ 
            padding: '15px', 
            backgroundColor: '#f8f9fa',
            borderRadius: '8px',
            color: '#999'
          }}>
            [字段映射编辑表 - 开发中]
            <br />
            关系类型: {rel.relationship_type}
          </div>
        </div>
      ))}
    </div>
  );
}
