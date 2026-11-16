import React from 'react';
import { Shield, ShieldAlert } from 'lucide-react';
export default function SecurityAuditorWidget() {
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
                <Shield style={{ color: '#00ff99' }} />
                <h3 style={{ margin: 0, fontSize: '1.2rem', letterSpacing: '1px' }}>05 ZERO-TRUST AGENT</h3>
            </div>
            <p style={{ margin: '0 0 15px 0', fontSize: '0.9rem', color: '#ccc' }}>
                Aggregating real-time trust postures across active network replication channels.
            </p>
            <div style={{ color: '#00ff99', fontSize: '1.1rem', fontWeight: 'bold' }}>
                OVERALL TRUST SCORE: 99.8%
            </div>
        </div>
    );
}