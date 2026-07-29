import React from 'react';
import { Link, Radio } from 'lucide-react';
export default function IntegrationSpecialistWidget() {
    return (
        <div style={{
            background: 'rgba(255, 255, 255, 0.05)',
            backdropFilter: 'blur(12px)',
            borderRadius: '16px',
            border: '1px solid rgba(255, 255, 255, 0.1)',
            padding: '20px',
            color: '#fff',
            fontFamily: 'system-ui'
        }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '15px' }}>
                <Radio style={{ color: '#e08285' }} />
                <h3 style={{ margin: 0, fontSize: '1.2rem', letterSpacing: '1px' }}>11 EDGE GATEWAY</h3>
            </div>
            <p style={{ margin: '0 0 15px 0', fontSize: '0.9rem', color: '#ccc' }}>
                Active REST & WebSocket protocol harmonization throughput levels.
            </p>
            <div style={{ color: '#e08285', fontWeight: 'bold' }}>
                Gateway Status: <span style={{ color: '#fff' }}>9.4k Req/s</span>
            </div>
        </div>
    );
}