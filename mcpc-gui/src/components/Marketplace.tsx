import { useState } from 'react';
import { Package, Download, Star, Cloud } from 'lucide-react';

const PLUGINS = [
  {
    id: 'rust-axum-api',
    name: 'Rust Axum API',
    description: 'High-performance HTTP API template built with Axum, Tokio, and Tracing.',
    author: 'mcpc-core',
    downloads: '12.4k',
    stars: 432,
    type: 'template',
    tags: ['rust', 'api', 'http']
  },
  {
    id: 'python-celery-worker',
    name: 'Python Celery Worker',
    description: 'Scalable background task processor using Celery and Redis.',
    author: 'mcpc-core',
    downloads: '8.1k',
    stars: 215,
    type: 'template',
    tags: ['python', 'worker', 'async']
  },
  {
    id: 'go-mcp-agent',
    name: 'Go MCP Agent',
    description: 'Lightweight sidecar agent for Model Context Protocol integrations.',
    author: 'community',
    downloads: '3.2k',
    stars: 120,
    type: 'agent',
    tags: ['go', 'mcp', 'agent']
  },
  {
    id: 'k8s-helm-generator',
    name: 'K8s Helm Generator',
    description: 'Advanced Kubernetes manifest generator with service mesh support.',
    author: 'mcpc-core',
    downloads: '24.5k',
    stars: 890,
    type: 'plugin',
    tags: ['kubernetes', 'helm', 'infra']
  }
];

export function Marketplace() {
  const [installing, setInstalling] = useState<string | null>(null);

  const handleInstall = (id: string) => {
    setInstalling(id);
    setTimeout(() => setInstalling(null), 1500); // Mock install
  };

  return (
    <div className="h-full w-full flex flex-col">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-xl font-bold tracking-tight text-foreground">Discover Plugins & Templates</h3>
        <div className="flex items-center gap-2 text-sm text-muted-foreground bg-card px-3 py-1.5 rounded-full border border-border">
          <Cloud className="w-4 h-4" />
          Connected to MCPC Registry
        </div>
      </div>

      <div className="grid grid-cols-2 lg:grid-cols-3 gap-4 overflow-y-auto pb-8">
        {PLUGINS.map((plugin) => (
          <div 
            key={plugin.id}
            className="flex flex-col bg-card border border-border rounded-xl p-5 hover:border-foreground/20 hover:bg-secondary/50 transition-all group shadow-sm"
          >
            <div className="flex justify-between items-start mb-3">
              <div className="w-10 h-10 rounded-lg bg-background border border-border flex items-center justify-center text-foreground group-hover:scale-105 transition-transform shadow-sm">
                <Package className="w-5 h-5" />
              </div>
              <span className={`text-[10px] uppercase tracking-wider px-2 py-1 rounded-full border bg-background border-border text-muted-foreground`}>
                {plugin.type}
              </span>
            </div>
            
            <h4 className="font-bold text-base mb-1 text-foreground">{plugin.name}</h4>
            <p className="text-sm text-muted-foreground line-clamp-2 mb-4 flex-1">
              {plugin.description}
            </p>
            
            <div className="flex items-center gap-2 mb-4">
              {plugin.tags.map(tag => (
                <span key={tag} className="text-[10px] bg-background border border-border text-muted-foreground px-2 py-0.5 rounded-md">
                  {tag}
                </span>
              ))}
            </div>

            <div className="flex items-center justify-between pt-4 border-t border-border">
              <div className="flex items-center gap-3 text-xs text-muted-foreground">
                <div className="flex items-center gap-1">
                  <Download className="w-3 h-3" /> {plugin.downloads}
                </div>
                <div className="flex items-center gap-1">
                  <Star className="w-3 h-3" /> {plugin.stars}
                </div>
              </div>
              
              <button 
                onClick={() => handleInstall(plugin.id)}
                disabled={installing !== null}
                className={`text-xs font-semibold px-3 py-1.5 rounded-lg transition-colors cursor-pointer border ${
                  installing === plugin.id 
                    ? 'bg-muted text-muted-foreground border-transparent' 
                    : 'bg-foreground text-background border-foreground hover:opacity-90'
                }`}
              >
                {installing === plugin.id ? 'Installing...' : 'Install'}
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
