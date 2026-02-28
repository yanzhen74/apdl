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
  const relationships = protocol.frame_packet_relationships || [];
  const syntaxUnits = protocol.syntax_units || [];

  // 获取单元信息
  const getUnitInfo = (unitId: string) => {
    return syntaxUnits.find((u: any) => u.unit_id === unitId);
  };

  // 渲染帧/包字段表格（2行N列）
  const renderUnitTable = (unit: any) => {
    if (!unit || !unit.fields || unit.fields.length === 0) {
      return <div style={{ color: '#999', padding: '10px' }}>无字段定义</div>;
    }

    return (
      <div style={{ 
        display: 'inline-block',
        margin: '10px',
        border: '2px solid #1890ff',
        borderRadius: '8px',
        overflow: 'hidden',
        backgroundColor: 'white'
      }}>
        {/* 单元标题 */}
        <div style={{ 
          padding: '8px 12px',
          backgroundColor: '#1890ff',
          color: 'white',
          fontWeight: 'bold',
          fontSize: '13px'
        }}>
          {unit.unit_name} ({unit.unit_id})
          <span style={{ 
            marginLeft: '8px',
            fontSize: '11px',
            fontWeight: 'normal',
            opacity: 0.9
          }}>
            {unit.unit_type === 'frame' ? '帧' : '包'}
          </span>
        </div>
        
        {/* 字段表格 - 2行N列 */}
        <table style={{ 
          borderCollapse: 'collapse',
          fontSize: '11px'
        }}>
          <tbody>
            {/* 第一行：字段名称 */}
            <tr>
              {unit.fields.map((field: any, idx: number) => (
                <td key={idx} style={{
                  padding: '6px 10px',
                  border: '1px solid #e8e8e8',
                  borderTop: 'none',
                  textAlign: 'center',
                  fontWeight: 'bold',
                  backgroundColor: '#fafafa',
                  minWidth: '60px'
                }}>
                  {field.field_name}
                </td>
              ))}
            </tr>
            {/* 第二行：字段长度 */}
            <tr>
              {unit.fields.map((field: any, idx: number) => (
                <td key={idx} style={{
                  padding: '6px 10px',
                  border: '1px solid #e8e8e8',
                  borderTop: 'none',
                  textAlign: 'center',
                  color: '#666',
                  fontSize: '10px'
                }}>
                  {field.bit_length ? `${field.bit_length}bit` : ''}
                  {field.byte_length ? `${field.byte_length}B` : ''}
                  {!field.bit_length && !field.byte_length ? '-' : ''}
                </td>
              ))}
            </tr>
          </tbody>
        </table>
      </div>
    );
  };

  // 渲染关系连接（简化版，只显示关系）
  const renderRelation = (rel: any, index: number) => {
    const parentUnit = getUnitInfo(rel.parent_frame);
    const childUnit = getUnitInfo(rel.child_packet);

    return (
      <div key={index} style={{ 
        marginBottom: '20px',
        padding: '15px 20px',
        backgroundColor: 'white',
        borderRadius: '8px',
        border: '1px solid #e8e8e8',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '15px'
      }}>
        {/* 父帧名称 */}
        <span style={{ 
          padding: '8px 16px', 
          backgroundColor: '#1890ff',
          color: 'white',
          borderRadius: '4px',
          fontWeight: 'bold',
          fontSize: '14px'
        }}>
          {parentUnit?.unit_name || rel.parent_frame}
        </span>
        
        {/* 关系箭头和类型 */}
        <div style={{ 
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          color: '#1890ff'
        }}>
          <div style={{ fontSize: '20px' }}>→</div>
          <span style={{ 
            padding: '2px 8px',
            backgroundColor: '#f0f0f0',
            borderRadius: '4px',
            fontSize: '11px',
            color: '#666',
            marginTop: '2px'
          }}>
            {rel.relationship_type}
          </span>
          {rel.embedding_config?.container_field && (
            <span style={{ 
              fontSize: '10px',
              color: '#999',
              marginTop: '2px'
            }}>
              容器: {rel.embedding_config.container_field}
            </span>
          )}
        </div>
        
        {/* 子包名称 */}
        <span style={{ 
          padding: '8px 16px', 
          backgroundColor: '#52c41a',
          color: 'white',
          borderRadius: '4px',
          fontWeight: 'bold',
          fontSize: '14px'
        }}>
          {childUnit?.unit_name || rel.child_packet}
        </span>
      </div>
    );
  };

  return (
    <div>
      <h3>协议关系总图</h3>
      
      {/* 协议概览 */}
      <div style={{ 
        padding: '15px', 
        backgroundColor: '#e3f2fd',
        borderRadius: '8px',
        marginBottom: '20px'
      }}>
        <strong>协议: {protocol.protocol_meta?.protocol_name || '未命名'}</strong>
        <span style={{ marginLeft: '20px', color: '#666' }}>
          帧: {syntaxUnits.filter((u: any) => u.unit_type === 'frame').length} | 
          包: {syntaxUnits.filter((u: any) => u.unit_type === 'packet').length} | 
          关系: {relationships.length}
        </span>
      </div>

      {/* 所有帧/包概览 */}
      <div style={{ 
        padding: '20px', 
        backgroundColor: '#f8f9fa',
        borderRadius: '8px',
        marginBottom: '20px'
      }}>
        <h4 style={{ marginTop: 0, marginBottom: '15px' }}>帧/包概览</h4>
        <div style={{ 
          display: 'flex',
          flexWrap: 'wrap',
          justifyContent: 'center'
        }}>
          {syntaxUnits.map((unit: any, idx: number) => (
            <div key={idx}>
              {renderUnitTable(unit)}
            </div>
          ))}
        </div>
      </div>

      {/* 关系详情 */}
      <div style={{ 
        padding: '20px', 
        backgroundColor: '#f8f9fa',
        borderRadius: '8px'
      }}>
        <h4 style={{ marginTop: 0, marginBottom: '15px' }}>帧包关系</h4>
        {relationships.length === 0 ? (
          <p style={{ color: '#999', textAlign: 'center', padding: '30px' }}>
            暂无帧包关系定义
          </p>
        ) : (
          relationships.map((rel: any, index: number) => renderRelation(rel, index))
        )}
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
