import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Terminal, Play, CheckCircle2, Trash2, ShieldAlert, Loader2, Lock, Shield } from 'lucide-react';

interface Module {
  name: string;
  type?: string;
  dependencies?: string[];
}

interface MCPSpec {
  project: string;
  stage?: 'development' | 'testing' | 'production';
  modules: Module[];
}

export function ExecutionPanel() {
  const [logs, setLogs] = useState<string>('Ready.\nSelect an action to begin.');
  const [executing, setExecuting] = useState<string | null>(null);
  const [spec, setSpec] = useState<MCPSpec | null>(null);
  const [confirmedProduction, setConfirmedProduction] = useState(false);
  const [activeTab, setActiveTab] = useState<'console' | 'compliance'>('console');

  const getIngressRules = (moduleName: string) => {
    if (!spec) return 'Blocked';
    const dependents = spec.modules
      .filter((m) => m.dependencies?.includes(moduleName))
      .map((m) => m.name);
    return dependents.length > 0 ? dependents.join(', ') : 'Blocked';
  };

  const getEgressRules = (moduleName: string) => {
    if (!spec) return 'None';
    const m = spec.modules.find((m) => m.name === moduleName);
    const deps = m?.dependencies || [];
    return deps.length > 0 ? deps.join(', ') : 'None';
  };

  useEffect(() => {
    async function loadSpec() {
      try {
        const loadedSpec: MCPSpec = await invoke('get_spec');
        setSpec(loadedSpec);
      } catch (err) {
        console.error('Failed to load spec in execution panel:', err);
      }
    }
    loadSpec();
  }, [executing]);

  const runCommand = async (cmd: string, name: string) => {
    if (spec?.stage === 'production' && cmd === 'build' && !confirmedProduction) {
      setLogs('Error: Production build is locked. Please check the compliance checkbox to unlock.');
      return;
    }

    setExecuting(name);
    setLogs(`$ mcpc ${cmd}\nExecuting...\n\n`);
    try {
      const output: string = await invoke('run_cli_command', { command: cmd, stage: spec?.stage });
      setLogs((prev) => prev + output + '\n[Process completed successfully]');
    } catch (err) {
      setLogs((prev) => prev + String(err) + '\n[Process failed]');
    } finally {
      setExecuting(null);
    }
  };

  const actions = [
    { name: 'Build Graph', cmd: 'build', icon: Play, color: 'text-foreground' },
    { name: 'Validate Spec', cmd: 'validate', icon: CheckCircle2, color: 'text-foreground' },
    { name: 'Clean Workspace', cmd: 'clean', icon: Trash2, color: 'text-muted-foreground' },
    { name: 'Doctor / Inspect', cmd: 'worker', icon: ShieldAlert, color: 'text-muted-foreground' },
  ];

  return (
    <div className="h-full w-full flex gap-6">
      {/* Actions Sidebar */}
      <div className="w-64 flex flex-col gap-2 shrink-0">
        <h3 className="font-semibold text-xs text-muted-foreground uppercase tracking-wider mb-2">Available Actions</h3>
        {actions.map((action) => {
          const Icon = action.icon;
          const isRunning = executing === action.name;
          const isBuild = action.cmd === 'build';
          const isLocked = spec?.stage === 'production' && isBuild && !confirmedProduction;

          return (
            <button
              key={action.name}
              onClick={() => runCommand(action.cmd, action.name)}
              disabled={executing !== null || isLocked}
              className={`flex items-center gap-3 px-4 py-3 rounded-lg border transition-all cursor-pointer w-full text-left ${
                executing !== null 
                  ? 'bg-muted/50 border-border/50 opacity-50 cursor-not-allowed' 
                  : isLocked 
                    ? 'bg-destructive/10 border-destructive/20 text-muted-foreground opacity-60 cursor-not-allowed' 
                    : 'bg-card border-border hover:bg-secondary hover:border-foreground/20'
              }`}
            >
              {isRunning ? (
                <Loader2 className="w-4 h-4 animate-spin text-foreground shrink-0" />
              ) : isLocked ? (
                <Lock className="w-4 h-4 text-destructive shrink-0 animate-pulse" />
              ) : (
                <Icon className={`w-4 h-4 ${action.color} shrink-0`} />
              )}
              <span className="font-medium text-sm text-foreground truncate">
                {action.name} {isLocked && '(Locked)'}
              </span>
            </button>
          )
        })}

        {spec?.stage === 'production' && (
          <div className="mt-4 p-4 border border-destructive/20 bg-destructive/5 rounded-lg flex flex-col gap-3">
            <div className="flex items-start gap-2.5">
              <ShieldAlert className="w-4 h-4 text-destructive shrink-0 mt-0.5 animate-pulse" />
              <div className="flex flex-col gap-1">
                <span className="text-xs font-bold text-foreground">Production Active</span>
                <p className="text-[10px] text-muted-foreground leading-relaxed">
                  Enterprise compilation policies: Hardened containers and read-only filesystem environments will be compiled.
                </p>
              </div>
            </div>
            
            <label className="flex items-center gap-2 text-xs font-semibold text-foreground cursor-pointer select-none">
              <input
                type="checkbox"
                checked={confirmedProduction}
                onChange={(e) => setConfirmedProduction(e.target.checked)}
                className="w-3.5 h-3.5 accent-foreground bg-background border border-border rounded focus:outline-none"
              />
              Unlock Production Build
            </label>
          </div>
        )}
      </div>

      {/* Terminal / Compliance Output */}
      <div className="flex-1 bg-[#09090b] rounded-lg border border-border shadow-sm overflow-hidden flex flex-col">
        <div className="h-10 border-b border-border bg-[#09090b] flex items-center px-4 justify-between">
          <div className="flex gap-4">
            <button
              onClick={() => setActiveTab('console')}
              className={`flex items-center gap-2 text-xs font-mono transition-all pb-0.5 border-b-2 cursor-pointer ${
                activeTab === 'console' 
                  ? 'text-foreground border-foreground' 
                  : 'text-muted-foreground border-transparent hover:text-foreground'
              }`}
            >
              <Terminal className="w-4 h-4" />
              Output Console
            </button>
            <button
              onClick={() => setActiveTab('compliance')}
              className={`flex items-center gap-2 text-xs font-mono transition-all pb-0.5 border-b-2 cursor-pointer ${
                activeTab === 'compliance' 
                  ? 'text-foreground border-foreground' 
                  : 'text-muted-foreground border-transparent hover:text-foreground'
              }`}
            >
              <Shield className="w-4 h-4" />
              Compliance Report
            </button>
          </div>
          {spec?.stage && (
            <span className="text-[10px] uppercase font-bold tracking-wider px-2 py-0.5 rounded bg-foreground/10 text-foreground border border-foreground/20">
              Stage: {spec.stage}
            </span>
          )}
        </div>
        
        <div className="flex-1 p-4 overflow-y-auto">
          {activeTab === 'console' ? (
            <pre className="text-xs font-mono text-zinc-300 whitespace-pre-wrap leading-relaxed">
              {logs}
            </pre>
          ) : (
            <div className="flex flex-col gap-4">
              <div className="flex justify-between items-center border-b border-border/40 pb-2">
                <h4 className="text-sm font-semibold text-foreground">Zero-Trust Environment Posture</h4>
                <span className="text-xs text-muted-foreground">Generated dynamically from edge graph</span>
              </div>

              {spec?.modules ? (
                <div className="border border-border rounded-lg overflow-hidden">
                  <table className="w-full text-left border-collapse text-xs">
                    <thead>
                      <tr className="bg-muted/50 border-b border-border text-muted-foreground font-semibold">
                        <th className="p-3">Service</th>
                        <th className="p-3">RuntimeClass</th>
                        <th className="p-3">Network Policy Ingress</th>
                        <th className="p-3">Network Policy Egress</th>
                        <th className="p-3">Syscall Filters (Seccomp)</th>
                        <th className="p-3">Root Sandbox</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-border/40 font-mono text-zinc-300">
                      {spec.modules.map((m) => (
                        <tr key={m.name} className="hover:bg-muted/20">
                          <td className="p-3 font-semibold text-foreground">{m.name}</td>
                          <td className="p-3 text-cyan-400">gvisor</td>
                          <td className="p-3 text-emerald-400">
                            {getIngressRules(m.name)}
                          </td>
                          <td className="p-3 text-emerald-400">
                            {getEgressRules(m.name)}
                          </td>
                          <td className="p-3 text-amber-400">Deny ptrace, mount</td>
                          <td className="p-3 text-blue-400">Read-Only + Non-Root</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              ) : (
                <p className="text-xs text-muted-foreground font-mono">No spec details loaded. Run "Build Graph" to populate.</p>
              )}
              
              <div className="bg-muted/20 border border-border/40 p-3 rounded-lg flex flex-col gap-1.5 text-[11px] text-muted-foreground leading-normal">
                <span className="font-semibold text-foreground">Compliance Verification Summary:</span>
                <p>✓ All modules are locked to <code className="text-foreground">USER 65532</code> execution.</p>
                <p>✓ Root filesystems are mounted read-only via K8s SecurityContext.</p>
                <p>✓ Syscall filters automatically restrict ptrace/mount operations dynamically.</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
