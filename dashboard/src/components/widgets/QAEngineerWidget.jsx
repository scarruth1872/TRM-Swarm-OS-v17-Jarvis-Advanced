import React from 'react';
import { Beaker, CheckCircle2 } from 'lucide-react';
export default function QAEngineerWidget() {
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
                <Beaker style={{ color: '#00ff99' }} />
                <h3 style={{ margin: 0, fontSize: '1.2rem', letterSpacing: '1px' }}>08 LATENCY PERCENTILES</h3>
            </div>
            <p style={{ margin: '0 0 15px 0', fontSize: '0.9rem', color: '#ccc' }}>
                Monitoring 95th & 99th latency percentiles across critical services.
            </p>
            <div style={{ display: 'flex', gap: '20px' }}>
                <div>
                    <div style={{ fontSize: '0.75rem', color: '#888' }}>P95 LATENCY</div>
                    <div style={{ fontSize: '1.3rem', color: '#00ff99', fontWeight: 'bold' }}>12ms</div>
                </div>
                <div>
                    <div style={{ fontSize: '0.75rem', color: '#888' }}>P99 LATENCY</div>
                    <div style={{ fontSize: '1.3rem', color: '#ffb700', fontWeight: 'bold' }}>24ms</div>
                </div>
            </div>
        </div>
    );
}