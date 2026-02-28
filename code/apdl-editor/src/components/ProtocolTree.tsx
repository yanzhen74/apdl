import { useState } from 'react';

interface Field {
  field_name: string;
  field_type: string;
  description?: string;
}

interface SyntaxUnit {
  unit_id: string;
  unit_name: string;
  unit_type: 'frame' | 'packet' | 'segment';
  description?: string;
  fields?: Field[];
}

interface ProtocolData {
  protocol_name?: string;
  protocol_version?: string;
  description?: string;
  syntax_units?: SyntaxUnit[];
  frame_packet_relationships?: any[];
  field_mappings?: any[];
}

interface ProtocolTreeProps {
  protocol: ProtocolData | null;
  onSelectUnit?: (unit: SyntaxUnit) => void;
  onSelectField?: (unit: SyntaxUnit, field: Field) => void;
}

export function ProtocolTree({ protocol, onSelectUnit, onSelectField }: ProtocolTreeProps) {
  const [expandedUnits, setExpandedUnits] = useState<Set<string>>(new Set());

  if (!protocol) {
    return (
      <div style={{ padding: '20px', color: '#666' }}>
        请加载协议定义文件
      </div>
    );
  }

  const toggleUnit = (unitId: string) => {
    const newExpanded = new Set(expandedUnits);
    if (newExpanded.has(unitId)) {
      newExpanded.delete(unitId);
    } else {
      newExpanded.add(unitId);
    }
    setExpandedUnits(newExpanded);
  };

  const getUnitIcon = (unitType: string) => {
    switch (unitType) {
      case 'frame': return '📦';
      case 'packet': return '📋';
      case 'segment': return '📄';
      default: return '📄';
    }
  };

  const getFieldIcon = (fieldType: string) => {
    switch (fieldType) {
      case 'uint': return '🔢';
      case 'int': return '🔢';
      case 'bytes': return '📦';
      case 'bit_field': return '🔲';
      case 'container': return '📁';
      case 'array': return '📚';
      default: return '📄';
    }
  };

  return (
    <div style={{ 
      height: '100%', 
      overflow: 'auto',
      backgroundColor: '#f5f5f5',
      borderRight: '1px solid #ddd'
    }}>
      {/* 协议信息头部 */}
      <div style={{ 
        padding: '15px', 
        borderBottom: '1px solid #ddd',
        backgroundColor: '#fff'
      }}>
        <h3 style={{ margin: '0 0 5px 0', fontSize: '14px' }}>
          {protocol.protocol_name || '未命名协议'}
        </h3>
        <div style={{ fontSize: '12px', color: '#666' }}>
          版本: {protocol.protocol_version || '1.0.0'}
        </div>
        {protocol.description && (
          <div style={{ fontSize: '11px', color: '#999', marginTop: '5px' }}>
            {protocol.description}
          </div>
        )}
      </div>

      {/* 语法单元列表 */}
      <div style={{ padding: '10px' }}>
        <div style={{ 
          fontSize: '12px', 
          fontWeight: 'bold', 
          color: '#666',
          marginBottom: '10px',
          textTransform: 'uppercase'
        }}>
          语法单元 ({protocol.syntax_units?.length || 0})
        </div>

        {protocol.syntax_units?.map((unit) => (
          <div key={unit.unit_id} style={{ marginBottom: '5px' }}>
            {/* 单元标题 */}
            <div
              onClick={() => {
                toggleUnit(unit.unit_id);
                onSelectUnit?.(unit);
              }}
              style={{
                display: 'flex',
                alignItems: 'center',
                padding: '8px 10px',
                cursor: 'pointer',
                borderRadius: '4px',
                backgroundColor: '#fff',
                border: '1px solid #e0e0e0',
                fontSize: '13px'
              }}
            >
              <span style={{ marginRight: '8px' }}>
                {expandedUnits.has(unit.unit_id) ? '▼' : '▶'}
              </span>
              <span style={{ marginRight: '8px' }}>
                {getUnitIcon(unit.unit_type)}
              </span>
              <span style={{ flex: 1, fontWeight: 500 }}>
                {unit.unit_name}
              </span>
              <span style={{ 
                fontSize: '10px', 
                color: '#999',
                backgroundColor: '#f0f0f0',
                padding: '2px 6px',
                borderRadius: '3px'
              }}>
                {unit.unit_type}
              </span>
            </div>

            {/* 字段列表 */}
            {expandedUnits.has(unit.unit_id) && unit.fields && (
              <div style={{ marginLeft: '20px', marginTop: '5px' }}>
                {unit.fields.map((field, index) => (
                  <div
                    key={index}
                    onClick={() => onSelectField?.(unit, field)}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      padding: '6px 10px',
                      cursor: 'pointer',
                      borderRadius: '4px',
                      fontSize: '12px',
                      color: '#555',
                      borderLeft: '2px solid #e0e0e0',
                      marginLeft: '5px'
                    }}
                    onMouseEnter={(e) => {
                      e.currentTarget.style.backgroundColor = '#e8f4f8';
                      e.currentTarget.style.borderLeftColor = '#1890ff';
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.backgroundColor = 'transparent';
                      e.currentTarget.style.borderLeftColor = '#e0e0e0';
                    }}
                  >
                    <span style={{ marginRight: '6px' }}>
                      {getFieldIcon(field.field_type)}
                    </span>
                    <span style={{ flex: 1 }}>
                      {field.field_name}
                    </span>
                    <span style={{ 
                      fontSize: '10px', 
                      color: '#999',
                      backgroundColor: '#f5f5f5',
                      padding: '1px 4px',
                      borderRadius: '2px'
                    }}>
                      {field.field_type}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>

      {/* 帧包关系 */}
      {protocol.frame_packet_relationships && protocol.frame_packet_relationships.length > 0 && (
        <div style={{ padding: '10px', borderTop: '1px solid #ddd' }}>
          <div style={{ 
            fontSize: '12px', 
            fontWeight: 'bold', 
            color: '#666',
            marginBottom: '10px'
          }}>
            帧包关系 ({protocol.frame_packet_relationships.length})
          </div>
          {protocol.frame_packet_relationships.map((rel, index) => (
            <div
              key={index}
              style={{
                padding: '8px 10px',
                fontSize: '12px',
                backgroundColor: '#fff',
                borderRadius: '4px',
                marginBottom: '5px',
                border: '1px solid #e0e0e0'
              }}
            >
              <div style={{ fontWeight: 500 }}>
                {rel.parent_frame} → {rel.child_packet}
              </div>
              <div style={{ fontSize: '10px', color: '#999', marginTop: '2px' }}>
                {rel.relationship_type}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* 字段映射 */}
      {protocol.field_mappings && protocol.field_mappings.length > 0 && (
        <div style={{ padding: '10px', borderTop: '1px solid #ddd' }}>
          <div style={{ 
            fontSize: '12px', 
            fontWeight: 'bold', 
            color: '#666',
            marginBottom: '10px'
          }}>
            字段映射 ({protocol.field_mappings.length})
          </div>
          {protocol.field_mappings.map((mapping, index) => (
            <div
              key={index}
              style={{
                padding: '8px 10px',
                fontSize: '12px',
                backgroundColor: '#fff',
                borderRadius: '4px',
                marginBottom: '5px',
                border: '1px solid #e0e0e0'
              }}
            >
              <div style={{ fontWeight: 500 }}>
                {mapping.source_field?.field_name} → {mapping.target_field?.field_name}
              </div>
              <div style={{ fontSize: '10px', color: '#999', marginTop: '2px' }}>
                {mapping.mapping_logic}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
