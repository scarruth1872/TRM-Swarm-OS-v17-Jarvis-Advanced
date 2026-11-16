import React from 'react';
import { LayoutDashboard, Sparkles } from 'lucide-react';
export default function UIUXDesignerWidget() {
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
                <LayoutDashboard style={{ color: '#bf55ec' }} />
                <h3 style={{ margin: 0, fontSize: '1.2rem', letterSpacing: '1px' }}>07 GLASSMORPHIC DIALS</h3>
            </div>
            <p style={{ margin: '0 0 15px 0', fontSize: '0.9rem', color: '#ccc' }}>
                Real-time query performance visualization using custom tailored color palettes.
            </p>
            <div style={{ color: '#bf55ec', fontWeight: 'bold' }}>
                RENDER CYCLE: 8ms
            </div>
        </div>
    );
}