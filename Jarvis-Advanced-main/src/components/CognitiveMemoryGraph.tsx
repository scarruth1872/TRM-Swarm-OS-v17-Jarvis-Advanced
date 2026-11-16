import React, { useEffect, useRef, useState, useMemo } from "react";
import * as d3 from "d3";
import { GitBranch, Plus, Search, Shield, Zap, X, RefreshCw, ZoomIn, ZoomOut, Maximize2, Sparkles } from "lucide-react";
import { KnowledgeNode, KnowledgeRelation } from "../types";

interface CognitiveMemoryGraphProps {
  nodes: KnowledgeNode[];
  relations: KnowledgeRelation[];
  onAddRelation?: (source: string, target: string, label: string) => Promise<boolean>;
  onRefresh?: () => void;
}

// Extends D3 simulation nodes to sustain force layouts
interface SimulationNode extends d3.SimulationNodeDatum {
  id: string;
  label: string;
  type: string;
  fx?: number | null;
  fy?: number | null;
}

interface SimulationLink extends d3.SimulationLinkDatum<SimulationNode> {
  source: string | SimulationNode;
  target: string | SimulationNode;
  label: string;
}

export default function CognitiveMemoryGraph({
  nodes = [],
  relations = [],
  onAddRelation,
  onRefresh
}: CognitiveMemoryGraphProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const svgRef = useRef<SVGSVGElement>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [selectedNode, setSelectedNode] = useState<SimulationNode | null>(null);
  const [activeTab, setActiveTab] = useState<"graph" | "list">("graph");
  const [relationshipFilter, setRelationshipFilter] = useState("");

  // Add customized relation form state
  const [newSource, setNewSource] = useState("");
  const [newTarget, setNewTarget] = useState("");
  const [newLabel, setNewLabel] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState("");
  const [submitSuccess, setSubmitSuccess] = useState(false);

  // Zoom management refs
  const zoomBehaviorRef = useRef<d3.ZoomBehavior<SVGSVGElement, unknown> | null>(null);

  // Parse nodes & links into D3 structures safely
  const { d3Nodes, d3Links } = useMemo(() => {
    // Collect all unique node IDs in relations to ensure we cover orphaned matches or auto-registered strings
    const uniqueNodeIds = new Set<string>();
    nodes.forEach(n => uniqueNodeIds.add(n.id));
    relations.forEach(r => {
      uniqueNodeIds.add(r.source);
      uniqueNodeIds.add(r.target);
    });

    const parsedNodes: SimulationNode[] = Array.from(uniqueNodeIds).map(id => {
      const match = nodes.find(n => n.id === id);
      return {
        id,
        label: match ? match.label : id,
        type: match ? match.type || "concept" : "concept"
      };
    });

    const parsedLinks: SimulationLink[] = relations.map(r => ({
      source: r.source,
      target: r.target,
      label: r.label
    })).filter(l => {
      // Keep link only if source and target can be mapped
      return uniqueNodeIds.has(typeof l.source === "string" ? l.source : "") && 
             uniqueNodeIds.has(typeof l.target === "string" ? l.target : "");
    });

    return { d3Nodes: parsedNodes, d3Links: parsedLinks };
  }, [nodes, relations]);

  // Determine node type colors
  const getNodeColor = (type: string, isHighlighted = false) => {
    if (isHighlighted) return "#38bdf8"; // cyan-400
    switch (type?.toLowerCase()) {
      case "person":
        return "#10b981"; // emerald-500
      case "system":
        return "#0ea5e9"; // sky-500
      case "object":
        return "#f43f5e"; // rose-500
      case "concept":
        return "#a855f7"; // purple-500
      case "philosophy":
        return "#eab308"; // yellow-500
      case "astrology":
      case "psychology":
      case "tarot":
        return "#6366f1"; // indigo-500
      default:
        return "#94a3b8"; // slate-400
    }
  };

  // Node category label helper
  const getNodeLabelSuffix = (type: string) => {
    switch (type?.toLowerCase()) {
      case "person": return "👤 Human";
      case "system": return "🧠 Core Core";
      case "object": return "📦 Target Item";
      case "concept": return "💡 Philosophy/Concept";
      default: return `⚡ ${type}`;
    }
  };

  // Render force directed layout on svgRef
  useEffect(() => {
    if (!svgRef.current || !containerRef.current || activeTab !== "graph") return;

    const width = containerRef.current.clientWidth || 400;
    const height = containerRef.current.clientHeight || 300;

    // Reset SVG
    const svg = d3.select(svgRef.current);
    svg.selectAll("*").remove();

    // Create central visual elements group for zoom/drag transformations
    const mainGroup = svg.append("g").attr("class", "graph-container");

    // Add marker defs for link heads
    const defs = svg.append("defs");
    defs.append("marker")
      .attr("id", "arrowhead")
      .attr("viewBox", "0 -5 10 10")
      .attr("refX", 22) // Avoid overlapping with circle node radius (10-15px)
      .attr("refY", 0)
      .attr("markerWidth", 6)
      .attr("markerHeight", 6)
      .attr("orient", "auto")
      .append("path")
      .attr("d", "M0,-5L10,0L0,5")
      .attr("fill", "#1e293b");

    // Configure zoom behavior
    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.25, 4])
      .on("zoom", (event) => {
        mainGroup.attr("transform", event.transform);
      });

    svg.call(zoom);
    zoomBehaviorRef.current = zoom;

    // Center standard frame
    svg.call(zoom.transform, d3.zoomIdentity.translate(width / 2, height / 2).scale(0.85));

    // Force simulation engine
    const simulation = d3.forceSimulation<SimulationNode>(d3Nodes)
      .force("link", d3.forceLink<SimulationNode, SimulationLink>(d3Links)
        .id(d => d.id)
        .distance(120)
      )
      .force("charge", d3.forceManyBody().strength(-150))
      .force("x", d3.forceX(0).strength(0.08))
      .force("y", d3.forceY(0).strength(0.08))
      .force("collide", d3.forceCollide(28));

    // Create links lines
    const link = mainGroup.append("g")
      .attr("class", "links")
      .selectAll<SVGGElement, SimulationLink>("g")
      .data(d3Links)
      .enter().append("g");

    const linkLine = link.append("line")
      .attr("stroke", "#1e293b")
      .attr("stroke-opacity", 0.5)
      .attr("stroke-width", 1.5)
      .attr("marker-end", "url(#arrowhead)");

    // Label on links
    const linkText = link.append("text")
      .attr("font-family", "monospace")
      .attr("font-size", "7px")
      .style("font-weight", "500")
      .attr("fill", "#475569")
      .attr("text-anchor", "middle")
      .attr("dy", -4)
      .text(d => (d as SimulationLink).label);

    // Create nodes container
    const node = mainGroup.append("g")
      .attr("class", "nodes")
      .selectAll<SVGGElement, SimulationNode>("g")
      .data(d3Nodes)
      .enter().append("g")
      .style("cursor", "move")
      .on("click", (event, d) => {
        const nodeDatum = d as SimulationNode;
        setSelectedNode(nodeDatum);
        highlightConnected(nodeDatum);
      });

    // Outer aura ring/halo on hover
    node.append("circle")
      .attr("r", 15)
      .attr("fill", "transparent")
      .attr("stroke", "transparent")
      .attr("stroke-width", 2)
      .attr("class", "node-halo")
      .style("transition", "stroke 0.2s, fill 0.2s");

    // Standard core circle
    const circles = node.append("circle")
      .attr("r", 9)
      .attr("fill", d => getNodeColor((d as SimulationNode).type))
      .attr("stroke", "#030712")
      .attr("stroke-width", 1.5)
      .attr("filter", "drop-shadow(0px 0px 4px rgba(0,0,0,0.6))");

    // Text descriptions
    const textLabels = node.append("text")
      .attr("dx", 12)
      .attr("dy", ".35em")
      .attr("font-family", "monospace")
      .attr("font-size", "9px")
      .style("font-weight", "bold")
      .attr("fill", "#cbd5e1")
      .text(d => (d as SimulationNode).label);

    // Add drag-pinning behaviors
    const dragHandler = d3.drag<SVGGElement, SimulationNode>()
      .on("start", (event, d) => {
        const nodeDatum = d as SimulationNode;
        if (!event.active) simulation.alphaTarget(0.2).restart();
        nodeDatum.fx = nodeDatum.x;
        nodeDatum.fy = nodeDatum.y;
      })
      .on("drag", (event, d) => {
        const nodeDatum = d as SimulationNode;
        nodeDatum.fx = event.x;
        nodeDatum.fy = event.y;
      })
      .on("end", (event, d) => {
        if (!event.active) simulation.alphaTarget(0);
      });

    node.call(dragHandler);

    // Release pinned nodes upon double clicking
    node.on("dblclick", (event, d) => {
      const nodeDatum = d as SimulationNode;
      nodeDatum.fx = null;
      nodeDatum.fy = null;
      d3.select(event.currentTarget).select("circle").attr("stroke", "#030712");
      if (!event.active) simulation.alphaTarget(0.1).restart();
    });

    // Update coordinates ticks
    simulation.on("tick", () => {
      linkLine
        .attr("x1", d => {
          const l = d as SimulationLink;
          return (l.source as SimulationNode).x || 0;
        })
        .attr("y1", d => {
          const l = d as SimulationLink;
          return (l.source as SimulationNode).y || 0;
        })
        .attr("x2", d => {
          const l = d as SimulationLink;
          return (l.target as SimulationNode).x || 0;
        })
        .attr("y2", d => {
          const l = d as SimulationLink;
          return (l.target as SimulationNode).y || 0;
        });

      linkText
        .attr("x", d => {
          const l = d as SimulationLink;
          const s = l.source as SimulationNode;
          const t = l.target as SimulationNode;
          return ((s.x || 0) + (t.x || 0)) / 2;
        })
        .attr("y", d => {
          const l = d as SimulationLink;
          const s = l.source as SimulationNode;
          const t = l.target as SimulationNode;
          return ((s.y || 0) + (t.y || 0)) / 2;
        });

      node.attr("transform", d => {
        const n = d as SimulationNode;
        return `translate(${n.x || 0}, ${n.y || 0})`;
      });
    });

    // Highlight helper to dim non-relational graph elements
    function highlightConnected(targetNode: SimulationNode) {
      // Find connected nodes
      const linkedNodeIds = new Set<string>();
      linkedNodeIds.add(targetNode.id);

      d3Links.forEach(l => {
        const sourceId = typeof l.source === "object" ? l.source.id : l.source;
        const targetId = typeof l.target === "object" ? l.target.id : l.target;

        if (sourceId === targetNode.id) linkedNodeIds.add(targetId);
        if (targetId === targetNode.id) linkedNodeIds.add(sourceId);
      });

      // Highlight links connected to clicked node
      linkLine.attr("stroke", l => {
        const lnk = l as SimulationLink;
        const sId = typeof lnk.source === "object" ? lnk.source.id : lnk.source;
        const tId = typeof lnk.target === "object" ? lnk.target.id : lnk.target;
        return (sId === targetNode.id || tId === targetNode.id) ? "#38bdf8" : "#111827";
      }).attr("stroke-opacity", l => {
        const lnk = l as SimulationLink;
        const sId = typeof lnk.source === "object" ? lnk.source.id : lnk.source;
        const tId = typeof lnk.target === "object" ? lnk.target.id : lnk.target;
        return (sId === targetNode.id || tId === targetNode.id) ? 0.9 : 0.15;
      }).attr("stroke-width", l => {
        const lnk = l as SimulationLink;
        const sId = typeof lnk.source === "object" ? lnk.source.id : lnk.source;
        const tId = typeof lnk.target === "object" ? lnk.target.id : lnk.target;
        return (sId === targetNode.id || tId === targetNode.id) ? 2.5 : 1.0;
      });

      // Halo ring animation and color shifts
      node.each(function(n) {
        const nNode = n as SimulationNode;
        const isLinked = linkedNodeIds.has(nNode.id);
        const element = d3.select(this);
        
        element.select(".node-halo")
          .attr("stroke", isLinked ? "#0ea5e9" : "transparent")
          .attr("stroke-opacity", isLinked ? 0.4 : 0)
          .attr("fill", isLinked && nNode.id !== targetNode.id ? "rgba(14,165,233,0.05)" : "transparent");

        element.select("circle:not(.node-halo)")
          .attr("fill", isLinked ? getNodeColor(nNode.type, true) : getNodeColor(nNode.type))
          .attr("stroke", nNode.id === targetNode.id ? "#38bdf8" : "#030712");

        element.select("text")
          .attr("fill", nNode.id === targetNode.id ? "#ffffff" : isLinked ? "#93c5fd" : "#334155");
      });
    }

    return () => {
      simulation.stop();
    };
  }, [d3Nodes, d3Links, activeTab]);

  // Zoom helpers
  const handleZoomIn = () => {
    if (!svgRef.current || !zoomBehaviorRef.current) return;
    d3.select(svgRef.current).transition().duration(250).call(zoomBehaviorRef.current.scaleBy, 1.25);
  };

  const handleZoomOut = () => {
    if (!svgRef.current || !zoomBehaviorRef.current) return;
    d3.select(svgRef.current).transition().duration(250).call(zoomBehaviorRef.current.scaleBy, 0.8);
  };

  const handleZoomReset = () => {
    if (!svgRef.current || !zoomBehaviorRef.current || !containerRef.current) return;
    const width = containerRef.current.clientWidth || 400;
    const height = containerRef.current.clientHeight || 300;
    d3.select(svgRef.current).transition().duration(300).call(
      zoomBehaviorRef.current.transform, 
      d3.zoomIdentity.translate(width / 2, height / 2).scale(0.85)
    );
  };

  // Submit adding connection to J.A.R.V.I.S. REST API
  const handleSubmitRelation = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newSource.trim() || !newTarget.trim() || !newLabel.trim()) {
      setSubmitError("Please populate all bridge values.");
      return;
    }

    setIsSubmitting(true);
    setSubmitError("");
    setSubmitSuccess(false);

    try {
      if (onAddRelation) {
        const success = await onAddRelation(newSource, newTarget, newLabel);
        if (success) {
          setSubmitSuccess(true);
          setNewSource("");
          setNewTarget("");
          setNewLabel("");
          if (onRefresh) onRefresh();
          setTimeout(() => setSubmitSuccess(false), 3000);
        } else {
          setSubmitError("Subsystem refused relationship synthesis.");
        }
      }
    } catch (err: any) {
      setSubmitError(err.message || "Synthesizer thread timeout.");
    } finally {
      setIsSubmitting(false);
    }
  };

  // Filter lists dynamically
  const filteredRelations = useMemo(() => {
    return relations.filter(r => {
      const srcTxt = r.source.toLowerCase();
      const tgtTxt = r.target.toLowerCase();
      const lblTxt = r.label.toLowerCase();
      const flt = relationshipFilter.toLowerCase();
      return srcTxt.includes(flt) || tgtTxt.includes(flt) || lblTxt.includes(flt);
    });
  }, [relations, relationshipFilter]);

  return (
    <div className="flex-1 flex flex-col min-h-0 text-[#cbd5e1] font-mono">
      
      {/* Sub-Tabs of our memory widget */}
      <div className="flex justify-between items-center text-[10px] uppercase border-b border-gray-900 pb-1.5 shrink-0 gap-2">
        <div className="flex items-center gap-1">
          <GitBranch className="w-3.5 h-3.5 text-blue-500 shrink-0" />
          <span className="font-bold text-gray-400">Interlinked Neural Net</span>
        </div>
        <div className="flex items-center gap-1 pointer-events-auto">
          <button
            onClick={() => setActiveTab("graph")}
            className={`px-2 py-0.5 rounded cursor-pointer transition-all ${
              activeTab === "graph" ? "bg-blue-900/40 text-blue-400 border border-blue-900/50" : "text-gray-500 hover:text-gray-300"
            }`}
          >
            Tactical Map
          </button>
          <button
            onClick={() => setActiveTab("list")}
            className={`px-2 py-0.5 rounded cursor-pointer transition-all ${
              activeTab === "list" ? "bg-blue-900/40 text-blue-400 border border-blue-900/50" : "text-gray-500 hover:text-gray-300"
            }`}
          >
            Flat Ledger ({relations.length})
          </button>
        </div>
      </div>

      {activeTab === "graph" ? (
        <div className="flex-1 flex flex-col md:flex-row min-h-0 relative my-2 gap-3">
          
          {/* Main Visualizer SVG */}
          <div 
            ref={containerRef} 
            className="flex-1 h-[210px] md:h-full min-h-[180px] bg-[#05070a]/90 rounded-lg border border-gray-900/70 relative overflow-hidden group"
          >
            {/* Action Overlay Controls */}
            <div className="absolute top-2 left-2 z-10 flex flex-col gap-1 opacity-75 group-hover:opacity-100 transition-opacity pointer-events-auto">
              <button 
                onClick={handleZoomIn} 
                className="p-1 bg-black/60 hover:bg-black text-gray-400 hover:text-white border border-gray-850 rounded hover:scale-105 transition-all"
                title="Siphon Grid Zoom"
              >
                <ZoomIn className="w-3 h-3" />
              </button>
              <button 
                onClick={handleZoomOut} 
                className="p-1 bg-black/60 hover:bg-black text-gray-400 hover:text-white border border-gray-850 rounded hover:scale-105 transition-all"
                title="Siphon Grid Out"
              >
                <ZoomOut className="w-3 h-3" />
              </button>
              <button 
                onClick={handleZoomReset} 
                className="p-1 bg-black/60 hover:bg-black text-gray-400 hover:text-white border border-gray-850 rounded hover:scale-105 transition-all"
                title="Recalibrate Focal Vector"
              >
                <Maximize2 className="w-3 h-3" />
              </button>
            </div>

            {/* Hint overlay */}
            <div className="absolute bottom-2 left-2 text-[8px] text-gray-600 uppercase pointer-events-none tracking-wider">
              DRAG TO CORRELATE :: DBL-CLICK TO UNPIN NEURON
            </div>

            {d3Nodes.length === 0 ? (
              <div className="absolute inset-0 flex items-center justify-center text-[10px] text-gray-600 text-center p-4">
                No active neural relationships stored. Introduce concepts via prompt input box to materialize nodes.
              </div>
            ) : (
              <svg 
                ref={svgRef} 
                className="w-full h-full pointer-events-auto"
                style={{ background: "radial-gradient(circle at center, rgba(3,7,18,0.4) 0%, rgba(1,3,6,0.9) 100%)" }}
              />
            )}
          </div>

          {/* Node Inspect HUD Column */}
          <div className="w-full md:w-[155px] shrink-0 bg-black/15 p-2 rounded border border-gray-850 flex flex-col justify-between gap-2 text-[10px] overflow-y-auto">
            <div>
              <div className="font-bold uppercase tracking-wider text-blue-500 border-b border-gray-900 pb-1 flex items-center gap-1 mb-1.5">
                <Sparkles className="w-3 h-3 text-cyan-400" />
                <span>Node Monitor</span>
              </div>

              {selectedNode ? (
                <div className="space-y-2 animate-fadeIn">
                  <div>
                    <span className="text-[8px] text-gray-500 uppercase block leading-none">Entity Identity</span>
                    <span className="text-gray-200 text-xs font-bold font-mono tracking-wide truncate block mt-0.5">{selectedNode.label}</span>
                  </div>
                  <div className="grid grid-cols-2 gap-1 bg-black/25 p-1 rounded border border-gray-900">
                    <div>
                      <span className="text-[7px] text-gray-550 uppercase block">Node ID</span>
                      <span className="text-cyan-400 truncate block text-[8px] mt-0.5">{selectedNode.id}</span>
                    </div>
                    <div>
                      <span className="text-[7px] text-gray-550 uppercase block">Category</span>
                      <span className="text-emerald-400 block text-[8px] truncate mt-0.5 font-bold uppercase">{selectedNode.type}</span>
                    </div>
                  </div>

                  <div className="border-t border-gray-900 pt-1.5">
                    <span className="text-[8px] text-gray-550 uppercase block mb-1">Adjacent Linkages</span>
                    <div className="space-y-1 max-h-[85px] overflow-y-auto pr-1">
                      {d3Links.filter(l => {
                        const sId = typeof l.source === "object" ? l.source.id : l.source;
                        const tId = typeof l.target === "object" ? l.target.id : l.target;
                        return sId === selectedNode.id || tId === selectedNode.id;
                      }).map((l, index) => {
                        const sLabel = typeof l.source === "object" ? l.source.label : l.source;
                        const tLabel = typeof l.target === "object" ? l.target.label : l.target;
                        const otherLabel = sLabel === selectedNode.label ? tLabel : sLabel;
                        return (
                          <div key={index} className="px-1 py-0.5 bg-gray-950/40 rounded border border-gray-900/60 text-[8px] text-gray-300 leading-tight">
                            <span className="text-sky-400 lowercase">{l.label}</span> <span className="text-gray-500">→</span> <span className="font-semibold text-gray-200">{otherLabel}</span>
                          </div>
                        );
                      })}
                    </div>
                  </div>
                </div>
              ) : (
                <div className="text-gray-600 italic py-5 text-center leading-normal">
                  STREAMS EMPTY.<br />SELECT A TACTICAL NEURON IN VECTOR SPACE TO PLOT PROJECTION.
                </div>
              )}
            </div>

            {/* Quick inject neural bridge launcher */}
            <div className="border-t border-gray-900 pt-2 pointer-events-auto">
              <span className="text-[8px] text-gray-400 block uppercase font-bold tracking-wider mb-1 flex items-center gap-1">
                <Plus className="w-2.5 h-2.5 text-blue-500" /> Synthesis Neu-Bridge
              </span>
              <form onSubmit={handleSubmitRelation} className="space-y-1 text-[8px]">
                <input 
                  type="text" 
                  placeholder="Source (e.g. Boss)" 
                  value={newSource}
                  onChange={(e) => setNewSource(e.target.value)}
                  className="w-full bg-black/35 py-1 px-1.5 focus:outline-none border border-gray-900 focus:border-blue-500/50 rounded text-gray-350"
                  required
                />
                <input 
                  type="text" 
                  placeholder="Predicate (e.g. likes)" 
                  value={newLabel}
                  onChange={(e) => setNewLabel(e.target.value)}
                  className="w-full bg-black/35 py-1 px-1.5 focus:outline-none border border-gray-900 focus:border-blue-500/50 rounded text-gray-350"
                  required
                />
                <input 
                  type="text" 
                  placeholder="Target (e.g. Pizza)" 
                  value={newTarget}
                  onChange={(e) => setNewTarget(e.target.value)}
                  className="w-full bg-black/35 py-1 px-1.5 focus:outline-none border border-gray-900 focus:border-blue-500/50 rounded text-gray-350"
                  required
                />
                {submitError && (
                  <p className="text-red-500 font-semibold">{submitError}</p>
                )}
                {submitSuccess && (
                  <p className="text-green-400 font-semibold uppercase animate-pulse">Bridge synthesized!</p>
                )}
                <button
                  type="submit"
                  disabled={isSubmitting}
                  className="w-full py-1 mt-0.5 bg-blue-900/30 hover:bg-blue-800/40 text-blue-400 border border-blue-900/50 rounded text-[8px] uppercase tracking-wider font-bold cursor-pointer disabled:opacity-45"
                >
                  {isSubmitting ? "SYNTHESIZING..." : "MUTATE GRAPH NODE"}
                </button>
              </form>
            </div>

          </div>

        </div>
      ) : (
        /* FLAT LIST LEDGER SUBTAB VIEW */
        <div className="flex-1 flex flex-col min-h-0 space-y-2 mt-2 pointer-events-auto">
          {/* List filtration searchbar */}
          <div className="relative flex items-center bg-black/30 rounded border border-gray-900 px-2 shrink-0">
            <Search className="w-3.5 h-3.5 text-gray-550 absolute left-2" />
            <input 
              type="text" 
              placeholder="Search cognitive records ledger..." 
              value={relationshipFilter}
              onChange={(e) => setRelationshipFilter(e.target.value)}
              className="w-full pl-6 pr-2 py-1 bg-transparent text-xs focus:outline-none placeholder:text-gray-600 text-gray-300"
            />
            {relationshipFilter && (
              <button onClick={() => setRelationshipFilter("")} className="text-gray-500 hover:text-white">
                <X className="w-3 h-3" />
              </button>
            )}
          </div>

          {/* Table Container */}
          <div className="flex-1 overflow-y-auto border border-gray-900/80 rounded bg-black/25">
            {filteredRelations.length === 0 ? (
              <div className="p-4 text-center text-gray-600 text-xs italic">
                {relationshipFilter ? "Filter matches empty. Alter target terms." : "Relations registry is barren."}
              </div>
            ) : (
              <table className="w-full text-left border-collapse text-[9px] font-mono">
                <thead>
                  <tr className="bg-[#0f172a]/40 border-b border-gray-900 text-gray-500 uppercase font-sans font-bold tracking-wider">
                    <th className="p-1 px-2.5">Origin Neuron</th>
                    <th className="p-1">Bridge Label</th>
                    <th className="p-1">Target Neuron</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-950">
                  {filteredRelations.map((rel, index) => (
                    <tr key={index} className="hover:bg-gray-900/20 text-gray-300 transition-colors">
                      <td className="p-1.5 px-2.5 font-bold text-gray-400">{rel.source}</td>
                      <td className="p-1.5 text-sky-450 italic font-medium">{rel.label}</td>
                      <td className="p-1.5 font-bold text-cyan-400">{rel.target}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        </div>
      )}

    </div>
  );
}
