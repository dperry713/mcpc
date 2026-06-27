import { useEffect, useState, useCallback, useMemo } from 'react';
import {
  ReactFlow,
  Controls,
  Background,
  MiniMap,
  useNodesState,
  useEdgesState,
  MarkerType,
  Node,
  Edge,
  Connection,
  addEdge,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { invoke } from '@tauri-apps/api/core';
import { CheckCircle2, Loader2, Info, LayoutTemplate, Map as MapIcon, Plus, Save } from 'lucide-react';
import { CustomModuleNode } from './CustomModuleNode';

interface Module {
  name: string;
  type?: string;
  dependencies?: string[];
  features?: string[];
}

interface MCPSpec {
  project: string;
  modules: Module[];
}

export function GraphView() {
  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);
  const [loading, setLoading] = useState(true);
  
  // Editor & Spec state
  const [spec, setSpec] = useState<MCPSpec | null>(null);
  const [direction, setDirection] = useState<'TB' | 'LR'>('LR');
  const [showMiniMap, setShowMiniMap] = useState(true);
  const [selectedModuleName, setSelectedModuleName] = useState<string | null>(null);
  
  const selectedModule = spec?.modules.find(m => m.name === selectedModuleName) || null;

  const nodeTypes = useMemo(() => ({ customModule: CustomModuleNode }), []);

  const buildLayout = useCallback((modules: Module[], dir: 'TB' | 'LR') => {
    const newNodes: any[] = [];
    const newEdges: any[] = [];
    
    // Assign layers based on dependencies (simple auto-layout)
    const levels = new Map<string, number>();
    modules.forEach(mod => {
      let maxDepLevel = -1;
      (mod.dependencies || []).forEach(dep => {
        maxDepLevel = Math.max(maxDepLevel, levels.get(dep) || 0);
      });
      levels.set(mod.name, maxDepLevel + 1);
    });

    const levelCounts = new Map<number, number>();

    modules.forEach((mod) => {
      const level = levels.get(mod.name) || 0;
      const count = levelCounts.get(level) || 0;
      levelCounts.set(level, count + 1);

      const x = dir === 'LR' ? level * 280 : count * 220;
      const y = dir === 'LR' ? count * 150 : level * 180;

      newNodes.push({
        id: mod.name,
        type: 'customModule',
        position: { x, y },
        data: { 
          name: mod.name,
          type: mod.type,
          features: mod.features,
          dir
        }
      });

      (mod.dependencies || []).forEach(dep => {
        newEdges.push({
          id: `${dep}->${mod.name}`,
          source: dep,
          target: mod.name,
          animated: true,
          style: { stroke: 'hsl(var(--muted-foreground))', strokeWidth: 2 },
          markerEnd: { type: MarkerType.ArrowClosed, color: 'hsl(var(--muted-foreground))' },
        });
      });
    });

    return { newNodes, newEdges };
  }, []);

  useEffect(() => {
    async function loadSpec() {
      try {
        const loadedSpec: MCPSpec = await invoke('get_spec');
        setSpec(loadedSpec);
        const { newNodes, newEdges } = buildLayout(loadedSpec.modules, direction);
        setNodes(newNodes);
        setEdges(newEdges);
      } catch (err) {
        console.error('Failed to load spec:', err);
      } finally {
        setLoading(false);
      }
    }
    loadSpec();
  }, []);

  // Re-layout when direction changes (or spec updates externally)
  useEffect(() => {
    if (spec) {
      const { newNodes, newEdges } = buildLayout(spec.modules, direction);
      setNodes(newNodes);
      setEdges(newEdges);
    }
  }, [direction, buildLayout, spec]);

  const saveSpecToDisk = async (newSpec: MCPSpec) => {
    try {
      await invoke('save_spec', { spec: newSpec });
    } catch (e) {
      console.error('Failed to save spec:', e);
    }
  };

  const onConnect = useCallback((connection: Connection) => {
    if (!spec) return;
    
    // In our model, a connection from A -> B means B depends on A
    // connection.source is the dependency. connection.target is the dependent module.
    const newSpec = { ...spec };
    const targetMod = newSpec.modules.find(m => m.name === connection.target);
    
    if (targetMod && connection.source) {
      if (!targetMod.dependencies) targetMod.dependencies = [];
      if (!targetMod.dependencies.includes(connection.source)) {
        targetMod.dependencies.push(connection.source);
        setSpec(newSpec);
        saveSpecToDisk(newSpec);
        
        // Optimistically update React Flow edges
        setEdges((eds) => addEdge({
          ...connection,
          animated: true,
          style: { stroke: 'hsl(var(--muted-foreground))', strokeWidth: 2 },
          markerEnd: { type: MarkerType.ArrowClosed, color: 'hsl(var(--muted-foreground))' }
        }, eds));
      }
    }
  }, [spec, setEdges]);

  const onEdgesDelete = useCallback((edgesToDelete: Edge[]) => {
    if (!spec) return;
    const newSpec = { ...spec };
    let changed = false;

    edgesToDelete.forEach(edge => {
      const targetMod = newSpec.modules.find(m => m.name === edge.target);
      if (targetMod && targetMod.dependencies) {
        targetMod.dependencies = targetMod.dependencies.filter(d => d !== edge.source);
        changed = true;
      }
    });

    if (changed) {
      setSpec(newSpec);
      saveSpecToDisk(newSpec);
    }
  }, [spec]);

  const onNodeClick = (_: any, node: Node) => {
    setSelectedModuleName(node.id);
  };

  const handleModuleUpdate = (updates: Partial<Module>) => {
    if (!spec || !selectedModuleName) return;
    
    const newSpec = { ...spec };
    const modIndex = newSpec.modules.findIndex(m => m.name === selectedModuleName);
    if (modIndex !== -1) {
      newSpec.modules[modIndex] = { ...newSpec.modules[modIndex], ...updates };
      setSpec(newSpec);
      saveSpecToDisk(newSpec);
    }
  };

  const updateFeatures = (val: string) => {
    const features = val.split(',').map(s => s.trim()).filter(s => s.length > 0);
    handleModuleUpdate({ features });
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full w-full">
        <Loader2 className="w-8 h-8 animate-spin text-foreground" />
      </div>
    );
  }

  return (
    <div className="h-full w-full bg-background relative rounded-xl border border-border overflow-hidden flex">
      {/* Graph Area */}
      <div className="flex-1 relative">
        <div className="absolute top-4 left-4 z-10 flex gap-2">
          <div className="flex items-center gap-2 bg-card border border-border px-4 py-2 rounded-lg shadow-sm">
            <CheckCircle2 className="w-4 h-4 text-emerald-500" />
            <span className="text-sm font-medium">Valid Spec</span>
          </div>
        </div>
        
        {/* Customization Toolbar */}
        <div className="absolute top-4 right-4 z-10 flex gap-2">
          <button 
            onClick={() => setDirection(d => d === 'LR' ? 'TB' : 'LR')}
            className="flex items-center gap-2 bg-card border border-border hover:bg-secondary px-3 py-2 rounded-lg shadow-sm transition-colors text-xs font-medium cursor-pointer"
          >
            <LayoutTemplate className="w-4 h-4" />
            {direction === 'LR' ? 'Horizontal' : 'Vertical'}
          </button>
          <button 
            onClick={() => setShowMiniMap(s => !s)}
            className={`flex items-center gap-2 border px-3 py-2 rounded-lg shadow-sm transition-colors text-xs font-medium cursor-pointer ${
              showMiniMap ? 'bg-foreground text-background border-foreground' : 'bg-card border-border hover:bg-secondary'
            }`}
          >
            <MapIcon className="w-4 h-4" />
            Minimap
          </button>
        </div>

        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onEdgesDelete={onEdgesDelete}
          onNodeClick={onNodeClick}
          onPaneClick={() => setSelectedModuleName(null)}
          fitView
          className="dark"
          proOptions={{ hideAttribution: true }}
        >
          <Background color="hsl(var(--border))" gap={16} />
          <Controls className="!bg-card !border-border !fill-foreground" />
          {showMiniMap && <MiniMap nodeColor="hsl(var(--muted))" maskColor="rgba(0, 0, 0, 0.4)" style={{ backgroundColor: 'hsl(var(--card))' }} />}
        </ReactFlow>
      </div>

      {/* Editor Sidebar */}
      {selectedModule && (
        <div className="w-80 border-l border-border bg-card p-5 overflow-y-auto flex flex-col shadow-[-4px_0_15px_rgba(0,0,0,0.1)] z-20">
          <div className="flex items-center justify-between mb-6">
            <div className="flex items-center gap-2">
              <Info className="w-5 h-5 text-muted-foreground" />
              <h3 className="font-semibold text-base text-foreground">Edit Module</h3>
            </div>
            <div className="text-[10px] text-muted-foreground bg-secondary px-2 py-1 rounded">Auto-saving</div>
          </div>
          
          <div className="mb-5 flex flex-col gap-1">
            <label className="text-xs text-muted-foreground uppercase tracking-wider font-semibold">Name (ID)</label>
            <input 
              disabled
              value={selectedModule.name}
              className="font-mono text-sm bg-muted/50 text-muted-foreground border border-border px-3 py-2 rounded-md cursor-not-allowed"
            />
            <span className="text-[10px] text-muted-foreground">Rename coming soon.</span>
          </div>
          
          <div className="mb-5 flex flex-col gap-1">
            <label className="text-xs text-muted-foreground uppercase tracking-wider font-semibold">Type</label>
            <select
              value={selectedModule.type || 'unknown'}
              onChange={(e) => handleModuleUpdate({ type: e.target.value })}
              className="text-sm bg-background text-foreground border border-border px-3 py-2 rounded-md focus:outline-none focus:border-foreground/50 transition-colors"
            >
              <option value="plugin">plugin</option>
              <option value="agent">agent</option>
              <option value="tool">tool</option>
              <option value="app">app</option>
              <option value="unknown">unknown</option>
            </select>
          </div>
          
          <div className="mb-5 flex flex-col gap-1">
            <label className="text-xs text-muted-foreground uppercase tracking-wider font-semibold">Features (Comma Separated)</label>
            <input
              type="text"
              value={(selectedModule.features || []).join(', ')}
              onChange={(e) => updateFeatures(e.target.value)}
              placeholder="e.g. logging, auth"
              className="text-sm bg-background text-foreground border border-border px-3 py-2 rounded-md focus:outline-none focus:border-foreground/50 transition-colors"
            />
          </div>
          
          <div className="mb-5 flex flex-col gap-1">
            <label className="text-xs text-muted-foreground uppercase tracking-wider font-semibold mb-1">
              Dependencies ({selectedModule.dependencies?.length || 0})
            </label>
            {selectedModule.dependencies && selectedModule.dependencies.length > 0 ? (
              <div className="flex flex-wrap gap-2">
                {selectedModule.dependencies.map(dep => (
                  <span key={dep} className="text-xs bg-secondary border border-border px-2 py-1 rounded-md text-foreground flex items-center gap-1 group">
                    {dep}
                  </span>
                ))}
              </div>
            ) : (
              <div className="text-sm text-muted-foreground italic bg-background border border-dashed border-border p-3 rounded-md text-center">
                Drag a connection line from another node to add a dependency.
              </div>
            )}
            <span className="text-[10px] text-muted-foreground mt-1">To remove dependencies, select the connection line (edge) in the graph and press Backspace.</span>
          </div>
        </div>
      )}
    </div>
  );
}
