// Terminal component using xterm.js

import { useEffect, useRef, useCallback } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import { sendSshInput, resizeTerminal } from '../hooks/useTauri';

interface TerminalViewProps {
  sessionId: string;
  onDisconnect?: () => void;
}

export function TerminalView({ sessionId, onDisconnect }: TerminalViewProps) {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const pollIntervalRef = useRef<number | null>(null);

  const pollOutput = useCallback(async () => {
    if (!xtermRef.current || !sessionId) return;

    try {
      const response = await sendSshInput(sessionId, '');
      if (response.data) {
        xtermRef.current.write(response.data);
      }
      if (response.eof) {
        xtermRef.current.write('\r\n[Connection closed]\r\n');
        if (pollIntervalRef.current) {
          clearInterval(pollIntervalRef.current);
        }
        onDisconnect?.();
      }
    } catch (error) {
      console.error('Error polling SSH output:', error);
    }
  }, [sessionId, onDisconnect]);

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

    // Handle input
    terminal.onData(async (data) => {
      try {
        const response = await sendSshInput(sessionId, data);
        if (response.data) {
          terminal.write(response.data);
        }
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

    // Start polling for output
    pollIntervalRef.current = window.setInterval(pollOutput, 50);

    return () => {
      window.removeEventListener('resize', handleResize);
      if (pollIntervalRef.current) {
        clearInterval(pollIntervalRef.current);
      }
      terminal.dispose();
    };
  }, [sessionId, pollOutput]);

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

