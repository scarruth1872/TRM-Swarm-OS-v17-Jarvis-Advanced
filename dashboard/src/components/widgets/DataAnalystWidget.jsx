import React from 'react';
import { Activity, Database } from 'lucide-react';
export default function DataAnalystWidget() {
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
                <Database style={{ color: '#00e676' }} />
                <h3 style={{ margin: 0, fontSize: '1.2rem', letterSpacing: '1px' }}>12 ANALYTICAL ENGINE</h3>
            </div>
            <p style={{ margin: '0 0 15px 0', fontSize: '0.9rem', color: '#ccc' }}>
                Root Mean Squared Error performance tracking on core ML models.
            </p>
            <div style={{ color: '#00e676', fontWeight: 'bold' }}>
                RMSE telemetry: 0.04
            </div>
        </div>
    );
}