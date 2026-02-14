// Terminal component using xterm.js with event-based output

import { useEffect, useRef } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { sendSshInput, resizeTerminal } from '../hooks/useTauri';

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
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  useEffect(() => {
    if (!terminalRef.current) return;

    // Create terminal
    const terminal = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'Consolas, "Courier New", monospace',
      theme: {
        background: '#1e1e1e',
        foreground: '#d4d4d4',
        cursor: '#d4d4d4',
        selectionBackground: '#264f78',
      },
    });

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

    // Initial resize
    setTimeout(handleResize, 100);

    // Listen for SSH output events from the backend
    const setupListener = async () => {
      unlistenRef.current = await listen<SshOutputEvent>('ssh-output', (event) => {
        // Only process events for this session
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
    };

    setupListener();

    return () => {
      window.removeEventListener('resize', handleResize);
      if (unlistenRef.current) {
        unlistenRef.current();
      }
      terminal.dispose();
    };
  }, [sessionId, onDisconnect]);

  // Handle container resize
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

  return (
    <div
      ref={terminalRef}
      style={{
        width: '100%',
        height: '100%',
        backgroundColor: '#1e1e1e',
      }}
    />
  );
}

