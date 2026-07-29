import React from 'react';
import { Users, Rocket } from 'lucide-react';
export default function SwarmManagerWidget() {
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
                <Users style={{ color: '#ff5c5c' }} />
                <h3 style={{ margin: 0, fontSize: '1.2rem', letterSpacing: '1px' }}>09 SWARM MISSIONS</h3>
            </div>
            <p style={{ margin: '0 0 15px 0', fontSize: '0.9rem', color: '#ccc' }}>
                Milestone tracking and orchestrator routing dependencies metrics.
            </p>
            <div style={{ color: '#ff5c5c', fontWeight: 'bold' }}>
                ACTIVE DEPLOYMENTS: 12 / 12
            </div>
        </div>
    );
}