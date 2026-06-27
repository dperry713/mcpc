import { useState } from 'react';
import { LayoutDashboard, FileJson, PlayCircle, Store } from 'lucide-react';
import { GraphView } from './components/GraphView';
import { SpecEditor } from './components/SpecEditor';
import { ExecutionPanel } from './components/ExecutionPanel';
import { Marketplace } from './components/Marketplace';

function App() {
  const [activeTab, setActiveTab] = useState<'graph' | 'editor' | 'actions' | 'plugins'>('graph');

  return (
    <div className="h-screen w-screen flex flex-col bg-background text-foreground overflow-hidden font-sans">
      {/* Top Navigation Bar */}
      <header className="h-14 border-b border-border bg-card/80 backdrop-blur-md flex items-center px-6 shrink-0 z-50">
        <div className="flex items-center gap-3 mr-8">
          <div className="w-8 h-8 bg-foreground rounded-lg flex items-center justify-center text-background font-bold shadow-sm">
            M
          </div>
          <span className="font-semibold tracking-wide text-lg">MCPC</span>
        </div>
        
        <nav className="flex items-center gap-1">
          <TabButton 
            active={activeTab === 'graph'} 
            onClick={() => setActiveTab('graph')}
            icon={<LayoutDashboard className="w-4 h-4" />}
            label="Live Project"
          />
          <TabButton 
            active={activeTab === 'editor'} 
            onClick={() => setActiveTab('editor')}
            icon={<FileJson className="w-4 h-4" />}
            label="Spec Editor"
          />
          <TabButton 
            active={activeTab === 'actions'} 
            onClick={() => setActiveTab('actions')}
            icon={<PlayCircle className="w-4 h-4" />}
            label="Actions"
          />
          <TabButton 
            active={activeTab === 'plugins'} 
            onClick={() => setActiveTab('plugins')}
            icon={<Store className="w-4 h-4" />}
            label="Plugins"
          />
        </nav>
      </header>

      {/* Main Content Area */}
      <main className="flex-1 relative overflow-hidden bg-background p-6">
        <div className="h-full w-full max-w-7xl mx-auto relative z-10">
          {activeTab === 'graph' && <GraphView />}
          {activeTab === 'editor' && <SpecEditor />}
          {activeTab === 'actions' && <ExecutionPanel />}
          {activeTab === 'plugins' && <Marketplace />}
        </div>
      </main>
    </div>
  );
}

function TabButton({ active, onClick, icon, label }: { active: boolean, onClick: () => void, icon: React.ReactNode, label: string }) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 cursor-pointer ${
        active 
          ? 'bg-secondary text-foreground' 
          : 'text-muted-foreground hover:text-foreground hover:bg-secondary/50'
      }`}
    >
      {icon}
      {label}
    </button>
  );
}

export default App;
