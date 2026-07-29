import React from 'react';
import { GitBranch, Activity } from 'lucide-react';
export default function DevOpsEngineerWidget() {
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
                <GitBranch style={{ color: '#ffb700' }} />
                <h3 style={{ margin: 0, fontSize: '1.2rem', letterSpacing: '1px' }}>06 CLUSTER STATUS</h3>
            </div>
            <p style={{ margin: '0 0 15px 0', fontSize: '0.9rem', color: '#ccc' }}>
                Active resource deployments and self-healing metrics on standard Kubernetes.
            </p>
            <div style={{ color: '#ffb700', fontWeight: 'bold' }}>
                Kubernetes Pods: <span style={{ color: '#fff' }}>100% HEALTHY</span>
            </div>
        </div>
    );
}