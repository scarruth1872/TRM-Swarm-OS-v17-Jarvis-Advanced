import React from 'react';
import { ScrollText, FileText } from 'lucide-react';
export default function TechnicalWriterWidget() {
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
                <ScrollText style={{ color: '#e67e22' }} />
                <h3 style={{ margin: 0, fontSize: '1.2rem', letterSpacing: '1px' }}>10 KNOWLEDGE BASE</h3>
            </div>
            <p style={{ margin: '0 0 15px 0', fontSize: '0.9rem', color: '#ccc' }}>
                Quickstart documents, architectural blueprints and routing logs indexing.
            </p>
            <div style={{ color: '#e67e22', fontWeight: 'bold' }}>
                DOCS STATE: SYNCHRONIZED
            </div>
        </div>
    );
}