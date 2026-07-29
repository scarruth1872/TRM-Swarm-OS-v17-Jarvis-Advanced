import React from 'react';
import { Cpu, Activity } from 'lucide-react';
export default function ArchitectWidget() {
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
                <Cpu style={{ color: '#ff6600' }} />
                <h3 style={{ margin: 0, fontSize: '1.2rem', letterSpacing: '1px' }}>01 ARCHITECT PORTAL</h3>
            </div>
            <p style={{ margin: '0 0 15px 0', fontSize: '0.9rem', color: '#ccc' }}>
                Monitoring distributed neural node cluster topologies and path length safety bounds.
            </p>
            <div style={{ display: 'flex', gap: '20px', marginTop: '10px' }}>
                <div>
                    <div style={{ fontSize: '0.75rem', color: '#888' }}>COGNITIVE CORES</div>
                    <div style={{ fontSize: '1.5rem', fontWeight: 'bold', color: '#ff6600' }}>24 / 24</div>
                </div>
                <div>
                    <div style={{ fontSize: '0.75rem', color: '#888' }}>TOPOLOGY STATUS</div>
                    <div style={{ fontSize: '1.5rem', fontWeight: 'bold', color: '#00ff99' }}>SYNCHRONIZED</div>
                </div>
            </div>
        </div>
    );
}