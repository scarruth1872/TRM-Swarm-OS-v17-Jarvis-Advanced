import React from 'react';
import { Code, Terminal } from 'lucide-react';
export default function LeadDeveloperWidget() {
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
                <Code style={{ color: '#00ff99' }} />
                <h3 style={{ margin: 0, fontSize: '1.2rem', letterSpacing: '1px' }}>02 DEV WORKSPACE</h3>
            </div>
            <p style={{ margin: '0 0 15px 0', fontSize: '0.9rem', color: '#ccc' }}>
                Tracking active memory replication and transactional code additions in real-time.
            </p>
            <div style={{ background: '#111', padding: '10px', borderRadius: '8px', fontSize: '0.8rem' }}>
                <div style={{ color: '#00ff99' }}>$ npm run build:completed</div>
                <div style={{ color: '#888' }}>All bundles compiled in 1.4s</div>
            </div>
        </div>
    );
}