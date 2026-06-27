import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Terminal, Play, CheckCircle2, Trash2, ShieldAlert, Loader2 } from 'lucide-react';

export function ExecutionPanel() {
  const [logs, setLogs] = useState<string>('Ready.\nSelect an action to begin.');
  const [executing, setExecuting] = useState<string | null>(null);

  const runCommand = async (cmd: string, name: string) => {
    setExecuting(name);
    setLogs(`$ mcpc ${cmd}\nExecuting...\n\n`);
    try {
      const output: string = await invoke('run_cli_command', { command: cmd });
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
      <div className="w-64 flex flex-col gap-2">
        <h3 className="font-semibold text-xs text-muted-foreground uppercase tracking-wider mb-2">Available Actions</h3>
        {actions.map((action) => {
          const Icon = action.icon;
          const isRunning = executing === action.name;
          return (
            <button
              key={action.name}
              onClick={() => runCommand(action.cmd, action.name)}
              disabled={executing !== null}
              className={`flex items-center gap-3 px-4 py-3 rounded-lg border transition-all cursor-pointer ${
                executing !== null 
                  ? 'bg-muted/50 border-border/50 opacity-50 cursor-not-allowed' 
                  : 'bg-card border-border hover:bg-secondary hover:border-foreground/20'
              }`}
            >
              {isRunning ? (
                <Loader2 className="w-4 h-4 animate-spin text-foreground" />
              ) : (
                <Icon className={`w-4 h-4 ${action.color}`} />
              )}
              <span className="font-medium text-sm text-foreground">{action.name}</span>
            </button>
          )
        })}
      </div>

      {/* Terminal Output */}
      <div className="flex-1 bg-[#09090b] rounded-lg border border-border shadow-sm overflow-hidden flex flex-col">
        <div className="h-10 border-b border-border bg-[#09090b] flex items-center px-4 gap-2">
          <Terminal className="w-4 h-4 text-muted-foreground" />
          <span className="text-xs font-mono text-muted-foreground">Output Console</span>
        </div>
        <div className="flex-1 p-4 overflow-y-auto">
          <pre className="text-xs font-mono text-zinc-300 whitespace-pre-wrap leading-relaxed">
            {logs}
          </pre>
        </div>
      </div>
    </div>
  );
}
