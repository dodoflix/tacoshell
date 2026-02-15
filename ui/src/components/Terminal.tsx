// Terminal component using xterm.js with event-based output

import { useEffect, useRef } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { sendSshInput, resizeTerminal } from '../hooks/useTauri';
import { useAppStore } from '../stores/appStore';

interface SshOutputEvent {
  session_id: string;
  data: string;
  eof: boolean;
}

interface TerminalViewProps {
  sessionId: string;
  onDisconnect?: () => void;
}

export function TerminalView({ sessionId, onDisconnect }: TerminalViewProps) {
  const { fontSize, fontFamily, servers } = useAppStore();
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  // Find server info for the session
  const session = useAppStore.getState().sessions.get(sessionId);
  const server = servers.find(s => s.id === session?.serverId);

  useEffect(() => {
    if (!terminalRef.current) return;

    // Create terminal
    const terminal = new Terminal({
      cursorBlink: true,
      fontSize: fontSize,
      fontFamily: fontFamily,
      theme: {
        background: '#0d1117', // Match terminal-bg
        foreground: '#d1d5db',
        cursor: '#135bec',
        selectionBackground: 'rgba(19, 91, 236, 0.4)',
      },
      allowProposedApi: true,
    });

    terminal.options.fontSize = fontSize;
    terminal.options.fontFamily = fontFamily;

    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.open(terminalRef.current);
    fitAddon.fit();

    xtermRef.current = terminal;
    fitAddonRef.current = fitAddon;

    // Handle input - send to backend
    terminal.onData(async (data) => {
      try {
        await sendSshInput(sessionId, data);
      } catch (error) {
        console.error('Error sending input:', error);
      }
    });

    // Handle resize
    const handleResize = () => {
      fitAddon.fit();
      resizeTerminal(sessionId, terminal.cols, terminal.rows).catch(console.error);
    };

    window.addEventListener('resize', handleResize);
    setTimeout(handleResize, 100);
    setTimeout(() => terminal.focus(), 150);

    let isMounted = true;
    const setupListener = async () => {
      const unlisten = await listen<SshOutputEvent>('ssh-output', (event) => {
        if (!isMounted) return;
        if (event.payload.session_id !== sessionId) return;

        if (event.payload.data && xtermRef.current) {
          xtermRef.current.write(event.payload.data);
        }

        if (event.payload.eof) {
          if (xtermRef.current) {
            xtermRef.current.write('\r\n[Connection closed]\r\n');
          }
          onDisconnect?.();
        }
      });

      if (isMounted) {
        unlistenRef.current = unlisten;
      } else {
        unlisten();
      }
    };

    setupListener();

    return () => {
      isMounted = false;
      window.removeEventListener('resize', handleResize);
      if (unlistenRef.current) {
        unlistenRef.current();
      }
      terminal.dispose();
    };
  }, [sessionId, onDisconnect]);

  useEffect(() => {
    const resizeObserver = new ResizeObserver(() => {
      if (fitAddonRef.current && xtermRef.current) {
        fitAddonRef.current.fit();
        resizeTerminal(sessionId, xtermRef.current.cols, xtermRef.current.rows).catch(console.error);
      }
    });

    if (terminalRef.current) {
      resizeObserver.observe(terminalRef.current);
    }

    return () => resizeObserver.disconnect();
  }, [sessionId]);

  // Update terminal options when settings change
  useEffect(() => {
    if (xtermRef.current) {
      xtermRef.current.options.fontSize = fontSize;
      xtermRef.current.options.fontFamily = fontFamily;
      fitAddonRef.current?.fit();
    }
  }, [fontSize, fontFamily]);

  return (
    <div className="flex h-full w-full overflow-hidden bg-background-dark">
      {/* Terminal Viewport */}
      <div className="flex-1 flex flex-col p-2 bg-background-dark overflow-hidden relative">
        <div
          className="w-full h-full bg-terminal-bg rounded-lg border border-white/5 shadow-inner p-4 font-mono overflow-hidden"
          onClick={() => xtermRef.current?.focus()}
        >
          <div ref={terminalRef} className="w-full h-full" />
        </div>

        {/* Connection Status Floating Indicator */}
        <div className="absolute bottom-6 right-8 bg-panel-dark/90 backdrop-blur border border-green-500/30 text-green-400 px-3 py-1.5 rounded-full text-xs font-mono flex items-center gap-2 shadow-lg">
          <span className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-green-500"></span>
          </span>
          Connected
        </div>
      </div>

      {/* Right Utility Panel */}
      <aside className="w-80 bg-panel-dark border-l border-white/5 flex flex-col flex-shrink-0 h-full overflow-y-auto">
        {/* Panel Header */}
        <div className="p-4 border-b border-white/5">
          <h2 className="text-sm font-semibold uppercase tracking-wider text-slate-500 mb-1">Host Information</h2>
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded bg-indigo-500/20 flex items-center justify-center text-indigo-400">
              <span className="material-icons-round">cloud_circle</span>
            </div>
            <div>
              <h3 className="font-bold text-white truncate w-48">{server?.name || 'Unknown Host'}</h3>
              <p className="text-xs text-slate-500">Ubuntu 22.04 LTS</p>
            </div>
          </div>
        </div>

        {/* System Stats */}
        <div className="p-4 border-b border-white/5 space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="bg-background-dark/50 p-2 rounded border border-white/5">
              <span className="text-xs text-slate-500 block">IP Address</span>
              <span className="text-sm font-mono text-slate-300 truncate block">{server?.host || '0.0.0.0'}</span>
            </div>
            <div className="bg-background-dark/50 p-2 rounded border border-white/5">
              <span className="text-xs text-slate-500 block">Uptime</span>
              <span className="text-sm font-mono text-slate-300">14d 2h 12m</span>
            </div>
          </div>
          {/* CPU Bar */}
          <div>
            <div className="flex justify-between text-xs mb-1">
              <span className="text-slate-400">CPU Load</span>
              <span className="text-green-400">12%</span>
            </div>
            <div className="w-full bg-white/10 rounded-full h-1.5 overflow-hidden">
              <div className="bg-green-500 h-1.5 rounded-full" style={{ width: '12%' }}></div>
            </div>
          </div>
          {/* RAM Bar */}
          <div>
            <div className="flex justify-between text-xs mb-1">
              <span className="text-slate-400">Memory (RAM)</span>
              <span className="text-yellow-400">64%</span>
            </div>
            <div className="w-full bg-white/10 rounded-full h-1.5 overflow-hidden">
              <div className="bg-yellow-500 h-1.5 rounded-full" style={{ width: '64%' }}></div>
            </div>
          </div>
        </div>

        {/* Snippets */}
        <div className="p-4 flex-1">
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-sm font-semibold uppercase tracking-wider text-slate-500">Snippets</h2>
            <button className="text-primary hover:text-primary-hover text-xs font-medium">View All</button>
          </div>
          <div className="space-y-2">
            {[
              { name: 'Restart Nginx', cmd: 'sudo systemctl restart nginx' },
              { name: 'Tail Access Log', cmd: 'tail -f /var/log/nginx/access.log' },
              { name: 'Docker Stats', cmd: 'docker stats --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}"' },
              { name: 'System Update', cmd: 'sudo apt update && sudo apt upgrade -y' },
            ].map((snippet) => (
              <div key={snippet.name} className="group bg-background-dark border border-white/5 hover:border-primary/50 rounded-lg p-3 transition-all cursor-pointer">
                <div className="flex justify-between items-start mb-1">
                  <span className="font-medium text-sm text-slate-200">{snippet.name}</span>
                  <span className="material-icons-round text-slate-500 hover:text-primary text-[16px]">play_circle</span>
                </div>
                <code className="block text-xs font-mono text-slate-500 truncate">{snippet.cmd}</code>
              </div>
            ))}
          </div>
        </div>

        {/* Port Forwarding Section */}
        <div className="p-4 border-t border-white/5 bg-black/20">
          <h2 className="text-sm font-semibold uppercase tracking-wider text-slate-500 mb-3">Active Tunnels</h2>
          <div className="flex items-center justify-between text-sm bg-background-dark/50 border border-white/5 rounded px-3 py-2">
            <div className="flex items-center gap-2">
              <span className="w-2 h-2 rounded-full bg-green-500"></span>
              <span className="text-slate-300 font-mono text-xs">L:8080 -{">"} R:80</span>
            </div>
            <button className="text-slate-500 hover:text-red-400">
              <span className="material-icons-round text-[16px]">close</span>
            </button>
          </div>
        </div>
      </aside>
    </div>
  );
}
