export function SftpView() {
  return (
    <div className="flex-1 flex flex-col min-w-0 bg-background-light dark:bg-background-dark h-full">
      {/* Top Header & Connection Bar */}
      <header className="h-16 border-b dark:border-border-dark flex items-center justify-between px-6 bg-white dark:bg-panel-dark/50 backdrop-blur-sm shrink-0">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <span className="material-icons-round text-green-500 text-sm">circle</span>
            <h1 className="font-semibold text-slate-800 dark:text-white text-lg">Production Server</h1>
          </div>
          <span className="px-2 py-0.5 rounded text-xs font-medium bg-slate-100 dark:bg-slate-800 text-slate-500">192.168.1.55</span>
          <span className="text-xs text-slate-500 flex items-center gap-1">
            <span className="material-icons-round text-[14px]">bolt</span> 24ms
          </span>
        </div>
        <div className="flex items-center gap-3">
          <div className="relative">
            <span className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-slate-500">
              <span className="material-icons-round text-lg">search</span>
            </span>
            <input
              className="pl-9 pr-4 py-1.5 bg-slate-100 dark:bg-[#0c1017] border-none rounded-lg text-sm text-slate-700 dark:text-slate-200 focus:ring-1 focus:ring-primary w-64 placeholder-slate-500"
              placeholder="Find file..."
              type="text"
            />
          </div>
          <button className="p-2 text-slate-400 hover:text-white hover:bg-slate-800 rounded-lg transition-colors">
            <span className="material-icons-round">notifications</span>
          </button>
        </div>
      </header>

      {/* Action Toolbar */}
      <div className="h-14 border-b dark:border-border-dark flex items-center justify-between px-6 bg-white dark:bg-background-dark/50 shrink-0">
        <div className="flex items-center gap-1">
          <button className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-panel-dark rounded-lg transition-colors">
            <span className="material-icons-round text-lg text-primary">upload</span>
            Upload
          </button>
          <button className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-panel-dark rounded-lg transition-colors">
            <span className="material-icons-round text-lg text-primary">download</span>
            Download
          </button>
          <div className="w-px h-6 bg-slate-200 dark:bg-border-dark mx-2"></div>
          <button className="p-2 text-slate-500 hover:text-white hover:bg-slate-800 rounded-lg" title="New Folder">
            <span className="material-icons-round text-xl">create_new_folder</span>
          </button>
          <button className="p-2 text-slate-500 hover:text-white hover:bg-slate-800 rounded-lg" title="Rename">
            <span className="material-icons-round text-xl">drive_file_rename_outline</span>
          </button>
          <button className="p-2 text-slate-500 hover:text-red-400 hover:bg-slate-800 rounded-lg" title="Delete">
            <span className="material-icons-round text-xl">delete_outline</span>
          </button>
        </div>
        <div className="flex items-center gap-2">
          <button className="p-1.5 rounded bg-slate-200 dark:bg-primary text-white shadow-sm">
            <span className="material-icons-round text-lg">list</span>
          </button>
          <button className="p-1.5 rounded text-slate-500 hover:bg-slate-200 dark:hover:bg-panel-dark">
            <span className="material-icons-round text-lg">grid_view</span>
          </button>
        </div>
      </div>

      {/* File Manager Split Pane */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Pane (Local) */}
        <section className="flex-1 flex flex-col border-r dark:border-border-dark min-w-[300px]">
          {/* Breadcrumbs */}
          <div className="px-4 py-3 bg-slate-50 dark:bg-panel-dark/30 border-b dark:border-border-dark flex items-center gap-2 text-sm">
            <span className="material-icons-round text-slate-500">laptop_mac</span>
            <span className="text-slate-400">Local:</span>
            <div className="flex items-center gap-1 overflow-hidden text-slate-300">
              <span className="hover:underline cursor-pointer">~</span>
              <span>/</span>
              <span className="hover:underline cursor-pointer">Documents</span>
              <span>/</span>
              <span className="font-medium text-white">Projects</span>
            </div>
          </div>
          {/* Column Headers */}
          <div className="grid grid-cols-12 gap-4 px-4 py-2 border-b dark:border-border-dark bg-slate-50 dark:bg-background-dark text-xs font-semibold text-slate-500 uppercase tracking-wider">
            <div className="col-span-6">Name</div>
            <div className="col-span-3 text-right">Size</div>
            <div className="col-span-3 text-right">Date</div>
          </div>
          {/* File List */}
          <div className="flex-1 overflow-y-auto p-2 space-y-0.5">
            <FileItem icon="folder" iconColor="text-yellow-500" name="assets" date="Oct 24, 10:30" />
            <FileItem icon="description" iconColor="text-blue-400" name="index.html" size="24 KB" date="Today, 14:20" selected />
            <FileItem icon="css" iconColor="text-purple-400" name="style.css" size="12 KB" date="Yesterday" />
            <FileItem icon="javascript" iconColor="text-yellow-400" name="app.js" size="145 KB" date="Oct 22" />
          </div>
        </section>

        {/* Right Pane (Remote) */}
        <section className="flex-1 flex flex-col bg-slate-50/50 dark:bg-panel-dark/20 min-w-[300px] relative">
          {/* Breadcrumbs */}
          <div className="px-4 py-3 bg-slate-50 dark:bg-panel-dark/30 border-b dark:border-border-dark flex items-center gap-2 text-sm">
            <span className="material-icons-round text-primary">dns</span>
            <span className="text-slate-400">Remote:</span>
            <div className="flex items-center gap-1 overflow-hidden text-slate-300">
              <span className="hover:underline cursor-pointer">/</span>
              <span className="hover:underline cursor-pointer">var</span>
              <span>/</span>
              <span className="hover:underline cursor-pointer">www</span>
              <span>/</span>
              <span className="font-medium text-white">html</span>
            </div>
          </div>
          {/* Column Headers */}
          <div className="grid grid-cols-12 gap-4 px-4 py-2 border-b dark:border-border-dark bg-slate-50 dark:bg-background-dark text-xs font-semibold text-slate-500 uppercase tracking-wider">
            <div className="col-span-6">Name</div>
            <div className="col-span-2 text-right">Perms</div>
            <div className="col-span-2 text-right">Size</div>
            <div className="col-span-2 text-right">Owner</div>
          </div>
          {/* File List */}
          <div className="flex-1 overflow-y-auto p-2 space-y-0.5">
            <RemoteFileItem icon="folder" iconColor="text-yellow-500" name="images" perms="755" owner="root" />
            <RemoteFileItem icon="folder" iconColor="text-yellow-500" name="vendor" perms="755" owner="www-data" />
            <RemoteFileItem icon="settings" iconColor="text-slate-400" name=".htaccess" perms="644" size="4 KB" owner="root" />
            <RemoteFileItem icon="description" iconColor="text-gray-400" name="error.log" perms="644" size="2 MB" owner="syslog" />
            <RemoteFileItem icon="terminal" iconColor="text-green-400" name="deploy.sh" perms="755" size="1 KB" owner="root" />
          </div>
        </section>
      </div>

      {/* Bottom Panel (Transfer Queue) */}
      <div className="h-auto border-t dark:border-border-dark bg-white dark:bg-panel-dark z-10 shadow-[0_-5px_15px_rgba(0,0,0,0.3)] shrink-0">
        <div className="px-4 py-2 flex items-center justify-between cursor-pointer hover:bg-slate-50 dark:hover:bg-white/5">
          <div className="flex items-center gap-3">
            <span className="material-icons-round text-primary text-sm animate-pulse">sync</span>
            <span className="text-sm font-medium text-slate-800 dark:text-white">Active Transfers</span>
            <span className="bg-primary/20 text-primary text-xs px-2 py-0.5 rounded-full font-bold">3</span>
          </div>
          <div className="flex items-center gap-4 text-xs text-slate-500">
            <span>2.5 MB/s</span>
            <span className="material-icons-round">expand_more</span>
          </div>
        </div>
        <div className="border-t dark:border-border-dark bg-slate-50 dark:bg-[#0c1017] p-1">
          <TransferItem name="style.css" progress={45} icon="css" iconColor="text-purple-400" />
          <TransferItem name="app.js" progress={0} icon="javascript" iconColor="text-yellow-400" waiting />
        </div>
      </div>
    </div>
  );
}

interface FileItemProps {
  icon: string;
  iconColor: string;
  name: string;
  size?: string;
  date: string;
  selected?: boolean;
}

function FileItem({ icon, iconColor, name, size = "-", date, selected = false }: FileItemProps) {
  return (
    <div className={`grid grid-cols-12 gap-4 px-3 py-2 rounded items-center cursor-pointer group transition-colors ${selected ? 'bg-primary/20 border border-primary/20' : 'hover:bg-slate-200 dark:hover:bg-primary/10'}`}>
      <div className="col-span-6 flex items-center gap-3 overflow-hidden">
        <span className={`material-icons-round ${iconColor} text-xl`}>{icon}</span>
        <span className={`text-sm truncate group-hover:text-primary ${selected ? 'text-white font-medium' : 'text-slate-700 dark:text-slate-200'}`}>{name}</span>
      </div>
      <div className={`col-span-3 text-right text-xs font-mono ${selected ? 'text-slate-300' : 'text-slate-500'}`}>{size}</div>
      <div className={`col-span-3 text-right text-xs ${selected ? 'text-slate-300' : 'text-slate-500'}`}>{date}</div>
    </div>
  );
}

interface RemoteFileItemProps {
  icon: string;
  iconColor: string;
  name: string;
  perms: string;
  size?: string;
  owner: string;
}

function RemoteFileItem({ icon, iconColor, name, perms, size = "-", owner }: RemoteFileItemProps) {
  return (
    <div className="grid grid-cols-12 gap-4 px-3 py-2 rounded items-center hover:bg-slate-200 dark:hover:bg-primary/10 cursor-pointer group transition-colors">
      <div className="col-span-6 flex items-center gap-3 overflow-hidden">
        <span className={`material-icons-round ${iconColor} text-xl`}>{icon}</span>
        <span className="text-sm text-slate-700 dark:text-slate-200 truncate group-hover:text-primary">{name}</span>
      </div>
      <div className="col-span-2 text-right text-xs text-slate-500 font-mono">{perms}</div>
      <div className="col-span-2 text-right text-xs text-slate-500 font-mono">{size}</div>
      <div className="col-span-2 text-right text-xs text-slate-500">{owner}</div>
    </div>
  );
}

interface TransferItemProps {
  name: string;
  progress: number;
  icon: string;
  iconColor: string;
  waiting?: boolean;
}

function TransferItem({ name, progress, icon, iconColor, waiting = false }: TransferItemProps) {
  return (
    <div className="flex items-center gap-4 px-4 py-3 border-b dark:border-border-dark/50 last:border-0">
      <div className="w-8 h-8 rounded bg-panel-dark flex items-center justify-center shrink-0">
        <span className={`material-icons-round ${iconColor} text-lg`}>{icon}</span>
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex justify-between mb-1">
          <span className="text-xs font-medium text-slate-300 truncate">{name}</span>
          <span className="text-xs text-slate-500">{waiting ? 'Waiting...' : `${progress}%`}</span>
        </div>
        <div className="w-full bg-slate-700 rounded-full h-1.5">
          <div className={`${waiting ? 'bg-slate-600 w-0' : 'bg-primary'} h-1.5 rounded-full transition-all`} style={{ width: `${progress}%` }}></div>
        </div>
      </div>
      <button className="p-1 hover:text-white text-slate-500 rounded">
        <span className="material-icons-round text-lg">close</span>
      </button>
    </div>
  );
}
