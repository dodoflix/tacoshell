import { useState } from 'react';

export function KubernetesView() {
  const [expanded, setExpanded] = useState(true);

  return (
    <div className="flex-1 flex flex-col h-full overflow-hidden bg-background-light dark:bg-background-dark relative">
      {/* Header */}
      <header className="h-16 border-b border-slate-200 dark:border-slate-800 bg-white/50 dark:bg-surface-darker/50 backdrop-blur-sm flex items-center justify-between px-6 z-10 shrink-0">
        <div className="flex items-center gap-4">
          <h1 className="text-xl font-semibold text-slate-800 dark:text-white tracking-tight">Kubernetes Clusters</h1>
          <span className="px-2 py-0.5 rounded-full bg-primary/10 text-primary text-xs font-medium border border-primary/20">3 Active</span>
        </div>
        <div className="flex items-center gap-3">
          <div className="relative group">
            <span className="material-icons-round absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 text-lg group-focus-within:text-primary transition-colors">search</span>
            <input
              className="pl-10 pr-4 py-2 bg-slate-100 dark:bg-background-card border border-transparent dark:border-slate-700 focus:border-primary/50 focus:ring-2 focus:ring-primary/20 rounded-lg text-sm w-64 text-slate-700 dark:text-slate-200 placeholder-slate-400 dark:placeholder-slate-500 transition-all outline-none"
              placeholder="Search resources..."
              type="text"
            />
          </div>
          <button className="flex items-center gap-2 px-4 py-2 bg-primary hover:bg-primary-hover text-white rounded-lg text-sm font-medium transition-all shadow-lg shadow-primary/20 hover:shadow-primary/40">
            <span className="material-icons-round text-sm">add</span>
            Connect Cluster
          </button>
        </div>
      </header>

      {/* Content Body */}
      <div className="flex-1 overflow-y-auto p-6 space-y-4">
        {/* Cluster Item: Expanded */}
        <div className="bg-white dark:bg-background-card rounded-xl border border-primary/30 shadow-lg shadow-black/20 overflow-hidden">
          {/* Cluster Header */}
          <div
            onClick={() => setExpanded(!expanded)}
            className="p-4 flex items-center justify-between cursor-pointer bg-slate-50 dark:bg-background-sidebar/50 hover:bg-slate-100 dark:hover:bg-slate-800/50 transition-colors border-b border-slate-200 dark:border-slate-700"
          >
            <div className="flex items-center gap-4">
              <button className="p-1 rounded hover:bg-slate-200 dark:hover:bg-slate-700 text-slate-400 transition-colors">
                <span className="material-icons-round">{expanded ? 'expand_more' : 'chevron_right'}</span>
              </button>
              <div className="w-10 h-10 rounded bg-blue-500/10 flex items-center justify-center">
                <img alt="K8s Logo" className="w-6 h-6" src="https://lh3.googleusercontent.com/aida-public/AB6AXuAOftSAMLOXyVP11m4bthx4DZ7rjC8WerATQpWGrf5KsVykGirfgrw8dRSh6q0VqIUNNAts4AaOlSz5y7AuxFX32dlbwkIWn5kOM1DgR-zy2VPdLIyc6z_JctG1jnk7vhz2nzbw4kT3t7KUjfygnXTP0qbLNZysU7grVUFAMJzza8H4bdJ27e7EPnlBv8nwJjWnjeEX0bm1AQ3HeFjPq2vJ3sMSw62xigmBhBvxD-RN1NfSKLVEHgEmfFdA54LJczsTCuWssI5NpFxA" />
              </div>
              <div>
                <h3 className="font-semibold text-slate-800 dark:text-white">production-us-east-1</h3>
                <div className="flex items-center gap-2 text-xs text-slate-500">
                  <span className="font-mono">v1.27.4</span>
                  <span>•</span>
                  <span>12 Nodes</span>
                  <span>•</span>
                  <span className="text-slate-400">AWS EKS</span>
                </div>
              </div>
            </div>
            <div className="flex items-center gap-6">
              <div className="text-right">
                <div className="text-xs text-slate-500 uppercase font-medium tracking-wider mb-1">Status</div>
                <div className="flex items-center justify-end gap-2">
                  <span className="relative flex h-2.5 w-2.5">
                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                    <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500"></span>
                  </span>
                  <span className="text-sm font-medium text-emerald-500">Healthy</span>
                </div>
              </div>
              <div className="text-right hidden md:block">
                <div className="text-xs text-slate-500 uppercase font-medium tracking-wider mb-1">CPU Usage</div>
                <div className="flex items-center gap-2">
                  <div className="w-24 h-1.5 bg-slate-200 dark:bg-slate-700 rounded-full overflow-hidden">
                    <div className="h-full bg-primary w-[45%] rounded-full"></div>
                  </div>
                  <span className="text-sm font-mono text-slate-300">45%</span>
                </div>
              </div>
              <button className="p-2 text-slate-400 hover:text-white hover:bg-slate-700 rounded transition-colors">
                <span className="material-icons-round text-xl">more_vert</span>
              </button>
            </div>
          </div>
          {/* Expanded Content Area */}
          {expanded && (
            <div className="border-t border-slate-200 dark:border-slate-700">
              <div className="flex items-center px-4 border-b border-slate-200 dark:border-slate-700 bg-slate-50/50 dark:bg-surface-darker/30 gap-6">
                <button className="py-3 px-1 text-sm font-medium border-b-2 border-primary text-primary">Workloads</button>
                <button className="py-3 px-1 text-sm font-medium border-b-2 border-transparent text-slate-500 hover:text-slate-300 transition-colors">Nodes</button>
                <button className="py-3 px-1 text-sm font-medium border-b-2 border-transparent text-slate-500 hover:text-slate-300 transition-colors">Services</button>
                <div className="ml-auto flex items-center gap-2 py-2">
                  <span className="text-xs text-slate-500">Namespace:</span>
                  <select className="bg-slate-200 dark:bg-slate-800 border-none text-xs rounded px-2 py-1 text-slate-700 dark:text-slate-300 focus:ring-1 focus:ring-primary outline-none cursor-pointer">
                    <option>default</option>
                    <option>kube-system</option>
                  </select>
                </div>
              </div>
              <div className="overflow-x-auto">
                <table className="w-full text-left border-collapse">
                  <thead>
                    <tr className="border-b border-slate-200 dark:border-slate-700/50 text-xs uppercase tracking-wider text-slate-500 font-medium">
                      <th className="px-6 py-3">Name</th>
                      <th className="px-6 py-3">Status</th>
                      <th className="px-6 py-3">Restarts</th>
                      <th className="px-6 py-3">Age</th>
                      <th className="px-6 py-3 text-right">Actions</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-200 dark:divide-slate-700/50 text-sm">
                    <PodRow name="frontend-app-deployment-7d8" status="Running" statusColor="text-emerald-500" restarts="0" age="2d 4h" />
                    <PodRow name="redis-master-0" status="Running" statusColor="text-emerald-500" restarts="1" age="14d" />
                    <PodRow name="payment-service-worker-x92" status="CrashLoopBackOff" statusColor="text-red-500" restarts="14" age="4m 12s" error />
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>

        {/* Collapsed Items */}
        <CollapsedCluster name="staging-eu-central" version="v1.26.1" nodes="3" provider="DigitalOcean" status="Warning" statusColor="bg-yellow-500" textColor="text-yellow-500" />
        <CollapsedCluster name="dev-local-minikube" version="v1.28.0" nodes="1" provider="Local" status="Offline" statusColor="bg-slate-500" textColor="text-slate-500" offline />
      </div>

      {/* Terminal Drawer */}
      <div className="absolute bottom-0 left-0 right-0 z-30 pointer-events-none">
        <div className="bg-[#151b2b] border-t border-slate-700 text-white flex flex-col shadow-[0_-4px_20px_rgba(0,0,0,0.3)] transform translate-y-[calc(100%-2.5rem)] hover:translate-y-0 transition-transform duration-300 h-64 pointer-events-auto">
          <div className="h-10 bg-background-card flex items-center px-4 justify-between cursor-ns-resize hover:bg-slate-800 transition-colors border-b border-slate-700">
            <div className="flex items-center gap-3">
              <span className="material-icons-round text-sm text-primary">terminal</span>
              <span className="text-xs font-mono text-slate-300">redis-master-0 (sh)</span>
            </div>
            <div className="flex items-center gap-2">
              <button className="p-1 hover:bg-slate-700 rounded"><span className="material-icons-round text-sm">open_in_full</span></button>
              <button className="p-1 hover:bg-slate-700 rounded"><span className="material-icons-round text-sm">close</span></button>
            </div>
          </div>
          <div className="flex-1 bg-black p-4 font-mono text-sm text-slate-300 overflow-y-auto">
            <div className="opacity-50 mb-2">Connected to redis-master-0...</div>
            <div><span className="text-emerald-500">root@redis-master-0:/data#</span> redis-cli ping</div>
            <div className="mb-2">PONG</div>
            <div><span className="text-emerald-500">root@redis-master-0:/data#</span> <span className="animate-pulse">_</span></div>
          </div>
        </div>
      </div>
    </div>
  );
}

function PodRow({ name, status, statusColor, restarts, age, error = false }: any) {
  return (
    <tr className={`group hover:bg-slate-50 dark:hover:bg-slate-800/40 transition-colors ${error ? 'bg-red-500/5' : ''}`}>
      <td className="px-6 py-3 font-medium text-slate-700 dark:text-slate-200 flex items-center gap-3">
        <span className={`material-icons-round ${error ? 'text-red-400' : 'text-blue-400'} text-lg`}>{error ? 'error_outline' : 'layers'}</span>
        {name}
      </td>
      <td className="px-6 py-3">
        <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${statusColor}/10 ${statusColor} border border-${statusColor}/20`}>
          {status}
        </span>
      </td>
      <td className="px-6 py-3 font-mono text-slate-500">{restarts}</td>
      <td className="px-6 py-3 text-slate-500">{age}</td>
      <td className="px-6 py-3 text-right">
        <div className="flex items-center justify-end gap-1 opacity-40 group-hover:opacity-100 transition-opacity">
          <button className="p-1.5 text-slate-400 hover:text-primary hover:bg-primary/10 rounded transition-colors"><span className="material-icons-round text-sm">terminal</span></button>
          <button className="p-1.5 text-slate-400 hover:text-primary hover:bg-primary/10 rounded transition-colors"><span className="material-icons-round text-sm">edit</span></button>
        </div>
      </td>
    </tr>
  );
}

function CollapsedCluster({ name, version, nodes, provider, status, statusColor, textColor, offline = false }: any) {
  return (
    <div className={`bg-white dark:bg-background-card rounded-xl border border-slate-200 dark:border-slate-700 shadow-sm hover:border-slate-300 dark:hover:border-slate-600 transition-colors ${offline ? 'opacity-75' : ''}`}>
      <div className="p-4 flex items-center justify-between cursor-pointer">
        <div className="flex items-center gap-4">
          <button className="p-1 rounded hover:bg-slate-200 dark:hover:bg-slate-700 text-slate-400 transition-colors">
            <span className="material-icons-round">chevron_right</span>
          </button>
          <div className="w-10 h-10 rounded bg-slate-100 dark:bg-slate-800 flex items-center justify-center">
             <span className="material-icons-round text-slate-500 text-xl">dns</span>
          </div>
          <div>
            <h3 className="font-medium text-slate-700 dark:text-slate-300">{name}</h3>
            <div className="flex items-center gap-2 text-xs text-slate-500">
              <span className="font-mono">{version}</span>
              <span>•</span>
              <span>{nodes} Node{nodes !== "1" ? 's' : ''}</span>
              <span>•</span>
              <span className="text-slate-400">{provider}</span>
            </div>
          </div>
        </div>
        <div className="flex items-center gap-6">
          <div className="text-right">
            <div className="flex items-center justify-end gap-2">
              <span className={`relative inline-flex rounded-full h-2 w-2 ${statusColor}`}></span>
              <span className={`text-sm font-medium ${textColor}`}>{status}</span>
            </div>
          </div>
          <button className="p-2 text-slate-400 hover:text-white hover:bg-slate-700 rounded transition-colors">
            <span className="material-icons-round text-xl">more_vert</span>
          </button>
        </div>
      </div>
    </div>
  );
}
