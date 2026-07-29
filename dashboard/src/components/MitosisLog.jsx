import React, { useState, useEffect } from 'react';

/**
 * Phase 12: Biomimetic Mitosis Log
 * Visualizes cellular division events triggered by high stress in the Swarm.
 */
export default function MitosisLog() {
  const [mitosisEvents, setMitosisEvents] = useState([]);

  useEffect(() => {
    // Poll the Phase 12 endpoint to get biomimetic mitosis telemetry
    const fetchMitosis = async () => {
      try {
        const response = await fetch('http://localhost:8021/swarm/spatial/mitosis');
        const data = await response.json();
        setMitosisEvents(data.mitosis_events || []);
      } catch (err) {
        console.error("Failed to fetch mitosis telemetry", err);
      }
    };

    const interval = setInterval(fetchMitosis, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="mitosis-log-container bg-slate-900 text-emerald-400 p-6 rounded-lg shadow-xl mt-6">
      <h2 className="text-2xl font-bold mb-4 tracking-wider">Biomimetic Mitosis Log</h2>
      <div className="mitosis-stream h-64 overflow-y-auto font-mono text-sm">
        {mitosisEvents.length === 0 ? (
          <p className="text-slate-500 animate-pulse">All agents stable. No cellular division required...</p>
        ) : (
          mitosisEvents.map((event, idx) => (
            <div key={idx} className="event-row flex justify-between border-b border-emerald-900 py-3">
              <span className="text-emerald-700">[{new Date(event.timestamp * 1000).toLocaleTimeString()}]</span>
              <div className="flex flex-col ml-4">
                <span className="text-emerald-300 font-semibold text-lg">MITOSIS TRIGGERED</span>
                <span className="text-slate-400">Parent: <span className="text-rose-400">{event.parent_agent_id}</span></span>
                <span className="text-slate-400">Child Clone: <span className="text-teal-300">{event.child_agent_id}</span></span>
              </div>
              <div className="flex flex-col items-end">
                <span className="text-slate-500 text-xs uppercase">Stress Threshold Exceeded</span>
                <span className="text-rose-500 font-bold text-xl">{(event.stress_threshold_exceeded * 100).toFixed(1)}%</span>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
