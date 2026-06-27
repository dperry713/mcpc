import { Handle, Position } from '@xyflow/react';

export function CustomModuleNode({ data, isConnectable }: any) {
  const targetPos = data.dir === 'TB' ? Position.Top : Position.Left;
  const sourcePos = data.dir === 'TB' ? Position.Bottom : Position.Right;

  return (
    <div className="flex flex-col bg-card border border-border rounded-xl shadow-[0_1px_3px_0_rgb(0_0_0_/_0.1),_0_1px_2px_-1px_rgb(0_0_0_/_0.1)] overflow-hidden min-w-[160px] group hover:border-foreground/40 transition-colors cursor-grab active:cursor-grabbing">
      <Handle 
        type="target" 
        position={targetPos} 
        isConnectable={isConnectable} 
        className="w-2.5 h-2.5 !bg-muted-foreground border-none"
      />
      
      <div className="flex flex-col items-center justify-center p-4">
        <div className="font-bold text-sm mb-2 text-foreground">{data.name}</div>
        <div className="text-[10px] px-2 py-0.5 bg-foreground text-background rounded-full uppercase font-mono tracking-wider">
          {data.type || 'DEFAULT'}
        </div>
        {data.features && data.features.length > 0 && (
          <div className="mt-3 flex gap-1.5 flex-wrap justify-center">
            {data.features.map((f: string) => (
               <span key={f} className="w-1.5 h-1.5 rounded-full bg-blue-500" title={f} />
            ))}
          </div>
        )}
      </div>
      
      <Handle 
        type="source" 
        position={sourcePos} 
        isConnectable={isConnectable}
        className="w-2.5 h-2.5 !bg-foreground border-none"
      />
    </div>
  );
}
