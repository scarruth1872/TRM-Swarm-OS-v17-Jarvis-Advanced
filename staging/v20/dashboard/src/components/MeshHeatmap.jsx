/**
 * Mesh Heatmap Component
 * Phase 5: Vision - Telemetry Glassmorphism v2
 * 
 * Features:
 * - Real-time VRAM consumption visualization
 * - Task routing density heatmap
 * - Glassmorphism styling
 * - Animated data flows
 */

import React, { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import {
  Cpu, MemoryStick, Activity, TrendingUp, TrendingDown,
  Minus, Flame, Snowflake, Layers, Binary, GitFork, Gauge
} from 'lucide-react';
import axios from 'axios';

const API_BASE = 'http://127.0.0.1:8022';

// ─── VRAM Heat Cell ───────────────────────────────────────────────────────
const VRAMHeatCell = ({ node, index, maxValue }) => {
  const hasVram = node.vram_used != null && node.vram_total != null && node.vram_total > 0;
  const utilization = hasVram ? (node.vram_used / node.vram_total) * 100 : null;
  const intensity = hasVram ? Math.min(utilization / 100, 1) : 0;
  
  // Color gradient from blue (cold) to red (hot)
  const getHeatColor = (val) => {
    if (val < 0.3) return `rgba(51, 204, 255, ${0.3 + val})`; // Cold - Cyan
    if (val < 0.6) return `rgba(255, 204, 0, ${0.3 + val})`;  // Warm - Gold
    return `rgba(255, 51, 51, ${0.3 + val})`;                  // Hot - Red
  };

  return (
    <motion.div
      initial={{ scale: 0, opacity: 0 }}
      animate={{ scale: 1, opacity: 1 }}
      transition={{ delay: index * 0.03 }}
      className="heatmap-cell"
      style={{
        background: hasVram ? getHeatColor(intensity) : 'rgba(255, 255, 255, 0.05)',
        boxShadow: hasVram ? `0 0 ${20 * intensity}px ${getHeatColor(intensity)}` : 'none'
      }}
    >
      <div className="heatmap-cell-label">{node.name?.slice(0, 3)}</div>
      <div className="heatmap-cell-value" style={hasVram ? undefined : { fontSize: '8px' }}>{hasVram ? utilization.toFixed(0) + '%' : 'not measured'}</div>
    </motion.div>
  );
};

// ─── Task Routing Flow ────────────────────────────────────────────────────
const TaskRoutingFlow = ({ routes }) => {
  const maxRoutes = Math.max(...routes.map(r => r.count), 1);
  
  return (
    <div className="routing-flow-container">
      <svg className="routing-flow-svg" viewBox="0 0 400 200">
        {routes.map((route, idx) => {
          const intensity = route.count / maxRoutes;
          const y = 20 + (idx * 30) % 160;
          const x1 = 50;
          const x2 = 350;
          
          return (
            <g key={idx}>
              <motion.line
                initial={{ pathLength: 0 }}
                animate={{ pathLength: 1 }}
                transition={{ duration: 1, delay: idx * 0.1 }}
                x1={x1}
                y1={y}
                x2={x2}
                y2={y + Math.sin(idx) * 20}
                stroke={`rgba(255, 153, 0, ${0.3 + intensity * 0.7})`}
                strokeWidth={2 + intensity * 8}
                strokeLinecap="round"
              />
              <motion.circle
                initial={{ r: 0 }}
                animate={{ r: 4 + intensity * 6 }}
                cx={x2}
                cy={y + Math.sin(idx) * 20}
                fill="var(--lcars-orange)"
              />
            </g>
          );
        })}
      </svg>
    </div>
  );
};

// ─── Main Mesh Heatmap Component ──────────────────────────────────────────
const MeshHeatmap = () => {
  const [vramData, setVramData] = useState([]);
  const [routingData, setRoutingData] = useState([]);
  const [federationData, setFederationData] = useState({ nodes: [], federation_metrics: {} });
  const [isSyncing, setIsSyncing] = useState(false);
  const [syncResult, setSyncResult] = useState(null);
  const [systemMetrics, setSystemMetrics] = useState({
    totalVram: 0,
    usedVram: 0,
    avgUtilization: 0,
    taskRoutes: 0,
    hotNodes: 0,
    coldNodes: 0
  });
  const [isLive, setIsLive] = useState(true);

  const fetchMetrics = async () => {
    try {
      // Fetch mesh topology
      const meshRes = await axios.get(`${API_BASE}/mesh/topology`);
      const nodes = meshRes.data.nodes || [];
      
      // Fetch federation status (36 nodes)
      try {
        const fedRes = await axios.get(`${API_BASE}/federation/status`);
        if (fedRes.data && fedRes.data.nodes) {
          setFederationData(fedRes.data);
        }
      } catch (e) {
        console.warn("Federation fetch warning:", e);
      }

      // Fetch system resources
      const resRes = await axios.get(`${API_BASE}/system/resources`);
      const resources = resRes.data || {};
      
      // VRAM is measured on this host via nvidia-smi and served by /system/resources. The mesh
      // topology nodes carry no vram_* fields (the payload once asserted a 36-node fabric that is
      // not the real topology), so the heatmap reads the single real GPU instead.
      const gpuNode = (resources.total_vram_gb != null && resources.used_vram_gb != null)
        ? [{
            name: 'RTX 3070',
            node_id: 'gpu-rtx3070',
            vram_total: resources.total_vram_gb,
            vram_used: resources.used_vram_gb,
          }]
        : [];
      const vramNodes = gpuNode;
      
      setVramData(vramNodes);
      
      // Generate routing density data
      const connections = meshRes.data.connections || meshRes.data.links || [];
      const routingDensity = connections.slice(0, 10)
        .map((conn) => ({
          from: conn.from?.slice(0, 8),
          to: conn.to?.slice(0, 8),
          count: (typeof conn.count === 'number' ? conn.count : null),
        }))
        .filter((r) => r.count != null);
      setRoutingData(routingDensity);
      
      // Calculate system metrics (VRAM values are already in GB)
      const totalVram = vramNodes.reduce((sum, n) => sum + (n.vram_total || 0), 0);
      const usedVram = vramNodes.reduce((sum, n) => sum + (n.vram_used || 0), 0);
      const avgUtil = resources.gpu_util_pct != null ? resources.gpu_util_pct : (totalVram > 0 ? (usedVram / totalVram) * 100 : 0);
      
      const hasVramData = vramNodes.some(n => n.vram_used != null && n.vram_total != null && n.vram_total > 0);
      setSystemMetrics({
        totalVram: hasVramData ? totalVram.toFixed(1) : null,
        usedVram: hasVramData ? usedVram.toFixed(1) : null,
        avgUtilization: hasVramData ? avgUtil.toFixed(1) : null,
        taskRoutes: routingDensity.length > 0 ? routingDensity.length : null,
        hotNodes: hasVramData ? vramNodes.filter(n => n.vram_used != null && n.vram_total > 0 && n.vram_used / n.vram_total > 0.7).length : null,
        coldNodes: hasVramData ? vramNodes.filter(n => n.vram_used != null && n.vram_total > 0 && n.vram_used / n.vram_total < 0.3).length : null
      });
    } catch (err) {
      console.error("Metrics fetch error:", err);
    }
  };

  const handleSyncMesh = async () => {
    setIsSyncing(true);
    try {
      const res = await axios.post(`${API_BASE}/federation/sync?target_node_id=all`);
      const peers = res.data?.peers_synced?.length || 0;
      const mems = res.data?.memories_imported || 0;
      const phi = res.data?.stats?.voltage_harmony ?? res.data?.voltage_harmony ?? 'not measured';
      setSyncResult({
        success: true,
        text: `✓ SYNCED: ${peers} PEERS | +${mems} MEMORIES | Φ_V ${phi}`
      });
      await fetchMetrics();
    } catch (err) {
      setSyncResult({
        success: false,
        text: `✕ SYNC FAILED: ${err.message}`
      });
    } finally {
      setIsSyncing(false);
      setTimeout(() => setSyncResult(null), 6000);
    }
  };

  useEffect(() => {
    fetchMetrics();
    if (isLive) {
      const interval = setInterval(fetchMetrics, 2000);
      return () => clearInterval(interval);
    }
  }, [isLive]);

  return (
    <div className="mesh-heatmap-container">
      {/* ─── Header ─── */}
      <div className="heatmap-header">
        <div className="heatmap-title">
          <Layers size={24} className="text-cyan-400" />
          <span>MESH TELEMETRY MATRIX</span>
        </div>
        <div className="heatmap-controls" style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
          {syncResult && (
            <span style={{ 
              fontSize: '10px', 
              color: syncResult.success ? '#00ff41' : '#ff3333', 
              fontWeight: 800,
              letterSpacing: '1px',
              padding: '6px 10px',
              background: 'rgba(0,0,0,0.6)',
              border: `1px solid ${syncResult.success ? '#00ff41' : '#ff3333'}`,
              borderRadius: '4px'
            }}>
              {syncResult.text}
            </span>
          )}
          <button 
            onClick={handleSyncMesh}
            disabled={isSyncing}
            className="sync-mesh-btn"
            style={{
              padding: '8px 16px',
              background: isSyncing ? 'rgba(255, 153, 0, 0.2)' : 'rgba(0, 255, 65, 0.15)',
              border: isSyncing ? '1px solid #ff9900' : '1px solid #00ff41',
              color: isSyncing ? '#ff9900' : '#00ff41',
              fontSize: '10px',
              fontWeight: 800,
              letterSpacing: '2px',
              cursor: isSyncing ? 'wait' : 'pointer',
              borderRadius: '4px',
              transition: 'all 0.3s'
            }}
          >
            {isSyncing ? '⚡ SYNCING MESH...' : '⚡ SYNC TAILSCALE MESH'}
          </button>
          <button 
            onClick={() => setIsLive(!isLive)}
            className={`live-toggle ${isLive ? 'active' : ''}`}
          >
            {isLive ? '● LIVE' : '○ PAUSED'}
          </button>
        </div>
      </div>

      {/* ─── System Metrics Bar ─── */}
      <div className="metrics-bar">
        <div className="metric-item glass-card">
          <MemoryStick size={18} className="text-cyan-400" />
          <div className="metric-content">
            <span className="metric-label">VRAM TOTAL</span>
            <span className="metric-value">{systemMetrics.totalVram != null ? `${systemMetrics.totalVram} GB` : 'not measured'}</span>
          </div>
        </div>
        <div className="metric-item glass-card">
          <Activity size={18} className="text-orange-400" />
          <div className="metric-content">
            <span className="metric-label">VRAM USED</span>
            <span className="metric-value">{systemMetrics.usedVram != null ? `${systemMetrics.usedVram} GB` : 'not measured'}</span>
          </div>
        </div>
        <div className="metric-item glass-card">
          <Gauge size={18} className="text-purple-400" />
          <div className="metric-content">
            <span className="metric-label">AVG UTILIZATION</span>
            <span className="metric-value">{systemMetrics.avgUtilization != null ? `${systemMetrics.avgUtilization}%` : 'not measured'}</span>
          </div>
        </div>
        <div className="metric-item glass-card">
          <GitFork size={18} className="text-gold-400" />
          <div className="metric-content">
            <span className="metric-label">TASK ROUTES</span>
            <span className="metric-value">{systemMetrics.taskRoutes != null ? systemMetrics.taskRoutes : 'not measured'}</span>
          </div>
        </div>
        <div className="metric-item glass-card">
          <Flame size={18} className="text-red-400" />
          <div className="metric-content">
            <span className="metric-label">HOT NODES</span>
            <span className="metric-value">{systemMetrics.hotNodes != null ? systemMetrics.hotNodes : 'not measured'}</span>
          </div>
        </div>
        <div className="metric-item glass-card">
          <Snowflake size={18} className="text-blue-400" />
          <div className="metric-content">
            <span className="metric-label">COLD NODES</span>
            <span className="metric-value">{systemMetrics.coldNodes != null ? systemMetrics.coldNodes : 'not measured'}</span>
          </div>
        </div>
      </div>

      {/* ─── Main Grid ─── */}
      <div className="heatmap-grid">
        {/* VRAM Heatmap */}
        <div className="heatmap-section glass-card">
          <div className="section-header">
            <Cpu size={16} className="text-orange-400" />
            <span>VRAM CONSUMPTION HEATMAP</span>
          </div>
          <div className="heatmap-cells-grid">
            {vramData.length > 0 ? (
              vramData.map((node, idx) => (
                <VRAMHeatCell key={node.node_id || idx} node={node} index={idx} />
              ))
            ) : (
              <div className="no-data-message">NO MESH NODES DETECTED</div>
            )}
          </div>
          <div className="heatmap-legend">
            <div className="legend-item">
              <div className="legend-color cold"></div>
              <span>LOW (&lt;30%)</span>
            </div>
            <div className="legend-item">
              <div className="legend-color warm"></div>
              <span>MEDIUM (30-60%)</span>
            </div>
            <div className="legend-item">
              <div className="legend-color hot"></div>
              <span>HIGH (&gt;60%)</span>
            </div>
          </div>
        </div>

        {/* Task Routing Density */}
        <div className="heatmap-section glass-card">
          <div className="section-header">
            <Binary size={16} className="text-cyan-400" />
            <span>TASK ROUTING DENSITY</span>
          </div>
          <TaskRoutingFlow routes={routingData} />
          <div className="routing-stats">
            {routingData.slice(0, 5).map((route, idx) => (
              <div key={idx} className="routing-stat-item">
                <span className="route-path">{route.from} → {route.to}</span>
                <div className="route-bar-container">
                  <motion.div 
                    className="route-bar"
                    initial={{ width: 0 }}
                    animate={{ width: `${(route.count / 50) * 100}%` }}
                    transition={{ duration: 0.5, delay: idx * 0.1 }}
                  />
                </div>
                <span className="route-count">{route.count}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* ─── 36-Node Tailscale Federation Mesh Cluster Matrix ─── */}
      <div className="heatmap-section glass-card fed-matrix-section">
        <div className="section-header" style={{ display: 'flex', justifyContent: 'space-between', flexWrap: 'wrap', gap: '8px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Layers size={16} className="text-cyan-400" />
            <span>TAILSCALE FEDERATION MESH MATRIX ({federationData.total_nodes ?? 0} NODES)</span>
          </div>
          <div style={{ display: 'flex', gap: '16px', fontSize: '9px', fontWeight: 800, letterSpacing: '1px' }}>
            <span style={{ color: 'var(--lcars-gold)' }}>NODES: {federationData.total_nodes ?? federationData.connected_nodes ?? 'not measured'}</span>
            <span style={{ color: 'var(--lcars-orange)' }}>MESH AGENTS: {federationData.mesh_agents ?? 'not measured'}</span>
            <span style={{ color: '#00ff41' }}>HARMONIZED Φ_V: {federationData.stats?.voltage_harmony ?? 'not measured'}</span>
          </div>
        </div>

        {/* 1. Physical Machine Engines (5) */}
        <div className="fed-tier-group">
          <div className="fed-tier-label">⚡ PHYSICAL MACHINE ENGINES (5 NODES)</div>
          <div className="fed-cards-grid">
            {(federationData.nodes || []).slice(0, 5).map((node, idx) => (
              <motion.div 
                key={node.node_id || idx}
                className="fed-node-card glass-card"
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: idx * 0.05 }}
              >
                <div className="fed-card-top">
                  <span className="fed-node-badge">{node.node_id?.replace('node_', '').toUpperCase()}</span>
                  <span className={`fed-status-pill ${node.status?.includes('ONLINE') ? 'online' : 'degraded'}`}>
                    {node.status?.includes('ONLINE') ? 'ONLINE' : 'DEGRADED'}
                  </span>
                </div>
                <div className="fed-node-title">{node.name}</div>
                <div className="fed-node-meta">
                  <span>HOST: <strong style={{ color: '#33ccff' }}>{node.host}:{node.port}</strong></span>
                  <span>LATENCY: {node.latency_ms != null ? `${node.latency_ms} ms` : 'n/a'}</span>
                  <span>BACKEND: {node.backend ?? 'n/a'}</span>
                  <span>ANCHOR: <strong style={{ color: '#00ff41' }}>{node.monad_anchor ?? 'n/a'}</strong></span>
                </div>
              </motion.div>
            ))}
          </div>
        </div>

        {/* 2. Knowledge Shards (28) */}
        <div className="fed-tier-group" style={{ marginTop: '16px' }}>
          <div className="fed-tier-label">🧠 KNOWLEDGE SHARDS (28 VECTORS & GROUND TRUTH)</div>
          <div className="fed-cards-grid shards-grid">
            {(federationData.nodes || []).slice(5, 33).map((node, idx) => (
              <div key={node.node_id || idx} className="fed-node-card shard-card glass-card">
                <div className="fed-card-top">
                  <span className="fed-shard-id">{node.node_id?.replace('node_shard_', '').toUpperCase()}</span>
                  <span className="fed-phi-tag">Φ {node.phi_v}</span>
                </div>
                <div className="fed-shard-name">{node.name?.replace('SHARD-', '')}</div>
                <div className="fed-shard-host">{node.host}:{node.port}</div>
              </div>
            ))}
          </div>
        </div>

        {/* 3. Hardware Accelerators (3) */}
        <div className="fed-tier-group" style={{ marginTop: '16px' }}>
          <div className="fed-tier-label">🚀 HARDWARE ACCELERATION FABRICS (3 NODES)</div>
          <div className="fed-cards-grid">
            {(federationData.nodes || []).slice(33, 36).map((node, idx) => (
              <motion.div 
                key={node.node_id || idx}
                className="fed-node-card hw-card glass-card"
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ delay: idx * 0.05 }}
              >
                <div className="fed-card-top">
                  <span className="fed-node-badge hw-badge">{node.node_id?.replace('node_', '').toUpperCase()}</span>
                  <span className="fed-status-pill online">ACTIVE</span>
                </div>
                <div className="fed-node-title">{node.name}</div>
                <div className="fed-node-meta">
                  <span>HOST: <strong style={{ color: '#ff9900' }}>{node.host}:{node.port}</strong></span>
                  <span>LATENCY: {node.latency_ms != null ? `${node.latency_ms} ms` : 'n/a'}</span>
                  <span>ANCHOR: <strong style={{ color: '#00ff41' }}>{node.monad_anchor ?? 'n/a'}</strong></span>
                </div>
              </motion.div>
            ))}
          </div>
        </div>
      </div>

      {/* ─── Styles ─── */}
      <style>{`
        .fed-matrix-section {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .fed-tier-group {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .fed-tier-label {
          font-size: 9px;
          font-weight: 800;
          letter-spacing: 2px;
          color: var(--lcars-cyan);
          text-transform: uppercase;
        }

        .fed-cards-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
          gap: 10px;
        }

        .shards-grid {
          grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
          gap: 8px;
        }

        .fed-node-card {
          padding: 10px 12px;
          display: flex;
          flex-direction: column;
          gap: 6px;
          border-left: 3px solid var(--lcars-cyan);
        }

        .shard-card {
          padding: 8px 10px;
          border-left: 2px solid var(--lcars-gold);
        }

        .hw-card {
          border-left: 3px solid var(--lcars-orange);
        }

        .fed-card-top {
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .fed-node-badge {
          font-size: 8px;
          font-weight: 800;
          letter-spacing: 1px;
          color: var(--lcars-cyan);
        }

        .hw-badge {
          color: var(--lcars-orange);
        }

        .fed-shard-id {
          font-size: 8px;
          font-weight: 800;
          letter-spacing: 1px;
          color: var(--lcars-gold);
        }

        .fed-phi-tag {
          font-size: 8px;
          font-weight: 800;
          color: #00ff41;
        }

        .fed-status-pill {
          font-size: 8px;
          font-weight: 800;
          letter-spacing: 1px;
          padding: 2px 6px;
          border-radius: 3px;
        }

        .fed-status-pill.online {
          background: rgba(0, 255, 65, 0.15);
          color: #00ff41;
          border: 1px solid rgba(0, 255, 65, 0.3);
        }

        .fed-status-pill.degraded {
          background: rgba(255, 51, 51, 0.15);
          color: #ff3333;
          border: 1px solid rgba(255, 51, 51, 0.3);
        }

        .fed-node-title {
          font-size: 11px;
          font-weight: 700;
          color: #fff;
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }

        .fed-shard-name {
          font-size: 9px;
          font-weight: 600;
          color: rgba(255, 255, 255, 0.9);
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }

        .fed-shard-host {
          font-size: 8px;
          color: rgba(255, 255, 255, 0.4);
        }

        .fed-node-meta {
          display: flex;
          flex-direction: column;
          gap: 2px;
          font-size: 9px;
          color: rgba(255, 255, 255, 0.6);
        }

        .mesh-heatmap-container {
          display: flex;
          flex-direction: column;
          height: 100%;
          gap: 16px;
        }

        .heatmap-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px 16px;
          border-left: 4px solid var(--lcars-cyan);
          border-radius: 4px;
        }

        .heatmap-title {
          display: flex;
          align-items: center;
          gap: 12px;
          font-size: 14px;
          font-weight: 800;
          letter-spacing: 3px;
          color: var(--lcars-cyan);
          text-transform: uppercase;
        }

        .live-toggle {
          padding: 8px 16px;
          background: rgba(255, 255, 255, 0.05);
          border: 1px solid rgba(255, 255, 255, 0.1);
          color: #666;
          font-size: 10px;
          font-weight: 800;
          letter-spacing: 2px;
          cursor: pointer;
          transition: all 0.3s;
        }

        .live-toggle.active {
          background: rgba(0, 255, 65, 0.1);
          border-color: #00ff41;
          color: #00ff41;
        }

        .metrics-bar {
          display: grid;
          grid-template-columns: repeat(6, 1fr);
          gap: 12px;
        }

        .glass-card {
          background: rgba(255, 255, 255, 0.03);
          border: 1px solid rgba(255, 255, 255, 0.08);
          border-radius: 8px;
          backdrop-filter: blur(20px);
          -webkit-backdrop-filter: blur(20px);
        }

        .metric-item {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 12px 16px;
        }

        .metric-content {
          display: flex;
          flex-direction: column;
        }

        .metric-label {
          font-size: 8px;
          font-weight: 800;
          letter-spacing: 2px;
          color: rgba(255, 255, 255, 0.4);
          text-transform: uppercase;
        }

        .metric-value {
          font-size: 16px;
          font-weight: 700;
          color: #fff;
        }

        .heatmap-grid {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 16px;
          flex: 1;
        }

        .heatmap-section {
          padding: 16px;
          display: flex;
          flex-direction: column;
        }

        .section-header {
          display: flex;
          align-items: center;
          gap: 8px;
          font-size: 10px;
          font-weight: 800;
          letter-spacing: 2px;
          color: rgba(255, 255, 255, 0.6);
          text-transform: uppercase;
          margin-bottom: 12px;
          padding-bottom: 8px;
          border-bottom: 1px solid rgba(255, 255, 255, 0.05);
        }

        .heatmap-cells-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
          gap: 8px;
          flex: 1;
        }

        .heatmap-cell {
          aspect-ratio: 1;
          border-radius: 8px;
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          cursor: pointer;
          transition: transform 0.3s, box-shadow 0.3s;
        }

        .heatmap-cell:hover {
          transform: scale(1.1);
        }

        .heatmap-cell-label {
          font-size: 9px;
          font-weight: 800;
          color: rgba(0, 0, 0, 0.7);
          text-transform: uppercase;
        }

        .heatmap-cell-value {
          font-size: 14px;
          font-weight: 700;
          color: rgba(0, 0, 0, 0.9);
        }

        .heatmap-legend {
          display: flex;
          gap: 16px;
          margin-top: 12px;
          padding-top: 12px;
          border-top: 1px solid rgba(255, 255, 255, 0.05);
        }

        .legend-item {
          display: flex;
          align-items: center;
          gap: 6px;
          font-size: 9px;
          color: rgba(255, 255, 255, 0.5);
        }

        .legend-color {
          width: 12px;
          height: 12px;
          border-radius: 2px;
        }

        .legend-color.cold {
          background: rgba(51, 204, 255, 0.8);
        }

        .legend-color.warm {
          background: rgba(255, 204, 0, 0.8);
        }

        .legend-color.hot {
          background: rgba(255, 51, 51, 0.8);
        }

        .routing-flow-container {
          flex: 1;
          min-height: 150px;
        }

        .routing-flow-svg {
          width: 100%;
          height: 100%;
        }

        .routing-stats {
          margin-top: 12px;
          padding-top: 12px;
          border-top: 1px solid rgba(255, 255, 255, 0.05);
        }

        .routing-stat-item {
          display: grid;
          grid-template-columns: 120px 1fr 40px;
          align-items: center;
          gap: 8px;
          margin-bottom: 8px;
        }

        .route-path {
          font-size: 8px;
          font-family: 'Fira Code', monospace;
          color: rgba(255, 255, 255, 0.5);
        }

        .route-bar-container {
          height: 4px;
          background: rgba(255, 255, 255, 0.05);
          border-radius: 2px;
          overflow: hidden;
        }

        .route-bar {
          height: 100%;
          background: linear-gradient(90deg, var(--lcars-orange), var(--lcars-gold));
          border-radius: 2px;
        }

        .route-count {
          font-size: 10px;
          font-weight: 700;
          color: var(--lcars-orange);
          text-align: right;
        }

        .no-data-message {
          grid-column: 1 / -1;
          display: flex;
          align-items: center;
          justify-content: center;
          height: 100px;
          font-size: 12px;
          color: rgba(255, 255, 255, 0.2);
          letter-spacing: 4px;
          font-weight: 800;
        }

        .text-cyan-400 { color: var(--lcars-cyan); }
        .text-orange-400 { color: var(--lcars-orange); }
        .text-purple-400 { color: var(--lcars-purple); }
        .text-gold-400 { color: var(--lcars-gold); }
        .text-red-400 { color: #ff3333; }
        .text-blue-400 { color: #6699ff; }
      `}</style>
    </div>
  );
};

export default MeshHeatmap;