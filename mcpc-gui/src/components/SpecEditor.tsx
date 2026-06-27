import { useState, useEffect } from 'react';
import Editor from '@monaco-editor/react';
import { invoke } from '@tauri-apps/api/core';
import { CheckCircle2, Save, Loader2, AlertTriangle } from 'lucide-react';

export function SpecEditor() {
  const [content, setContent] = useState<string>('');
  const [original, setOriginal] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function load() {
      try {
        const spec = await invoke('get_spec');
        const json = JSON.stringify(spec, null, 2);
        setContent(json);
        setOriginal(json);
      } catch (err) {
        setError(String(err));
      } finally {
        setLoading(false);
      }
    }
    load();
  }, []);

  const handleSave = async () => {
    try {
      setSaving(true);
      setError(null);
      const parsed = JSON.parse(content);
      await invoke('save_spec', { spec: parsed });
      setOriginal(content);
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  const hasChanges = content !== original;

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full w-full">
        <Loader2 className="w-8 h-8 animate-spin text-primary" />
      </div>
    );
  }

  return (
    <div className="h-full w-full bg-background relative rounded-xl border border-border flex flex-col overflow-hidden shadow-sm">
      {/* Action Bar */}
      <div className="h-14 border-b border-border bg-card flex items-center justify-between px-4 z-10">
        <div className="flex items-center gap-2">
          {error ? (
            <div className="flex items-center gap-2 text-destructive bg-destructive/10 px-3 py-1.5 rounded-md border border-destructive/20">
              <AlertTriangle className="w-4 h-4" />
              <span className="text-sm font-medium">{error}</span>
            </div>
          ) : (
            <div className="flex items-center gap-2 text-emerald-500 bg-emerald-500/10 px-3 py-1.5 rounded-md border border-emerald-500/20">
              <CheckCircle2 className="w-4 h-4" />
              <span className="text-sm font-medium">Valid JSON Spec</span>
            </div>
          )}
        </div>
        <button 
          onClick={handleSave}
          disabled={!hasChanges || saving}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all ${
            hasChanges 
              ? 'bg-foreground text-background hover:opacity-90 cursor-pointer' 
              : 'bg-muted text-muted-foreground cursor-not-allowed opacity-50'
          }`}
        >
          {saving ? <Loader2 className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
          Save & Validate
        </button>
      </div>

      {/* Editor Area */}
      <div className="flex-1 relative">
        <Editor
          height="100%"
          defaultLanguage="json"
          theme="vs-dark"
          value={content}
          onChange={(val) => setContent(val || '')}
          options={{
            minimap: { enabled: false },
            fontSize: 14,
            fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
            padding: { top: 16 },
            scrollBeyondLastLine: false,
            smoothScrolling: true,
            cursorBlinking: "smooth",
            cursorSmoothCaretAnimation: "on",
            formatOnPaste: true,
          }}
        />
      </div>
    </div>
  );
}
